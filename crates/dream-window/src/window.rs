use std::sync::{Arc, Mutex};

use crossbeam_channel::unbounded;
use instant::Instant;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use dream_app::app::App;

pub struct Window {
    pub window: winit::window::Window,
    pub event_loop: EventLoop<()>,
}

impl Default for Window {
    fn default() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(winit::dpi::PhysicalSize::new(3000, 1750))
            .build(&event_loop)
            .unwrap();

        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
            } else {
                window.set_title("Dream Engine");
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let win = web_sys::window().unwrap();

            window.set_inner_size(winit::dpi::LogicalSize::new(
                win.inner_width().unwrap().as_f64().unwrap() as f32,
                win.inner_height().unwrap().as_f64().unwrap() as f32,
            ));

            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("dream-window-container")?;
                    let canvas = web_sys::Element::from(window.canvas());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        Self { window, event_loop }
    }
}

impl Window {
    pub async fn run(self) {
        // listen for screen resizing events for web build
        #[allow(unused_variables)]
        let (tx, rx) = unbounded::<winit::dpi::LogicalSize<f32>>();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            #[allow(unused_variables)]
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: web_sys::Event| {
                let win = web_sys::window().unwrap();
                let new_window_size = winit::dpi::LogicalSize::new(
                    win.inner_width().unwrap().as_f64().unwrap() as f32,
                    win.inner_height().unwrap().as_f64().unwrap() as f32,
                );
                tx.send(new_window_size).unwrap();
            }) as Box<dyn FnMut(_)>);
            let window = web_sys::window().unwrap();
            window
                .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }

        // let mut app = Arc::new(futures::lock::Mutex::new(App::new().await));
        let mut app = App::default();
        let renderer = Arc::new(Mutex::new(
            dream_renderer::RendererWgpu::default(&self.window).await,
        ));
        let mut editor = dream_editor::EditorEguiWgpu::new(
            &renderer,
            self.window.scale_factor() as f32,
            &self.event_loop,
        )
        .await;

        let sleep_millis: u64 = 16;
        let mut last_update_time = Instant::now();
        self.event_loop.run(move |event, _, control_flow| {
            match event {
                Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                    if let Some(size) = rx.clone().try_iter().last() {
                        self.window.set_inner_size(size);
                    }

                    let editor_raw_input = editor.egui_winit_state.take_egui_input(&self.window);
                    let editor_pixels_per_point = self.window.scale_factor() as f32;

                    let now = Instant::now();
                    if (now - last_update_time).as_millis() > sleep_millis as u128 {
                        app.update();
                        app.draw(&renderer);
                        last_update_time = Instant::now();
                    }

                    // draw the scene (to texture)
                    let mut ren = renderer.lock().unwrap();
                    let size = ren.size;
                    match ren.render() {
                        Ok(_) => {}
                        // reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            ren.resize(size);
                            editor.handle_resize(&mut ren);
                        }
                        // quit when system is out of memory
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("Quitting because system out of memory");
                            *control_flow = ControlFlow::Exit
                        }
                        // ignore timeout
                        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                    }

                    // draw editor
                    match editor.render_wgpu(&ren, editor_raw_input, editor_pixels_per_point) {
                        Ok(_) => {
                            ren.set_camera_aspect_ratio(editor.renderer_aspect_ratio);
                        }
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            ren.resize(size);
                            editor.handle_resize(&ren);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                    }
                }

                Event::WindowEvent { event, .. } => {
                    let mut ren2 = renderer.lock().unwrap();

                    if !editor.handle_event(&event) {
                        match event {
                            WindowEvent::Resized(physical_size) => {
                                ren2.resize(physical_size);
                                editor.handle_resize(&ren2);
                            }
                            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                // new_inner_size is &mut so w have to dereference it twice
                                ren2.resize(*new_inner_size);
                                editor.handle_resize(&ren2);
                            }
                            _ => (),
                        }
                    }
                }
                Event::MainEventsCleared => {
                    self.window.request_redraw();
                }
                _ => {}
            }
        });
    }
}
