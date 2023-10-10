use crossbeam_channel::unbounded;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use dream_app::app::App;
use dream_editor::editor::EditorEguiWgpu;

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

        let mut app = App::default();
        let mut renderer = dream_renderer::renderer::RendererWgpu::new(Some(&self.window)).await;
        let mut editor = EditorEguiWgpu::new(
            &app,
            &renderer,
            self.window.scale_factor() as f32,
            &self.event_loop,
        )
        .await;

        let sleep_millis: u64 = 16;
        let mut last_update_time = dream_time::time::now();
        self.event_loop.run(move |event, _, control_flow| {
            match event {
                Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                    if let Some(size) = rx.try_iter().last() {
                        self.window.set_inner_size(size);
                    }

                    let editor_raw_input = editor.egui_winit_state.take_egui_input(&self.window);
                    let editor_pixels_per_point = self.window.scale_factor() as f32;

                    let now = dream_time::time::now();
                    if now - last_update_time > sleep_millis as u128 {
                        app.update();
                        app.draw(&mut renderer);
                        last_update_time = dream_time::time::now();
                    }

                    // draw the scene (to texture)
                    match renderer.render() {
                        Ok(_) => {}
                        // reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            renderer.resize(None);
                            editor.handle_resize(&mut renderer);
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
                    match editor.render_wgpu(&renderer, editor_raw_input, editor_pixels_per_point) {
                        Ok(_) => {
                            renderer.set_camera_aspect_ratio(editor.get_renderer_aspect_ratio());
                        }
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            renderer.resize(None);
                            editor.handle_resize(&renderer);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                    }
                }

                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => app.process_mouse(delta.0, delta.1),

                Event::WindowEvent { event, .. } => {
                    if !editor.handle_event(&event) {
                        match event {
                            WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        virtual_keycode: Some(key),
                                        state,
                                        ..
                                    },
                                ..
                            } => app.process_keyboard(key, state),
                            WindowEvent::MouseWheel { delta, .. } => {
                                app.process_scroll(&delta);
                            }
                            WindowEvent::MouseInput {
                                button: MouseButton::Left,
                                state,
                                ..
                            } => {
                                app.process_mouse_input(state == ElementState::Pressed);
                            }
                            WindowEvent::Resized(physical_size) => {
                                renderer.resize(Some(physical_size));
                                editor.handle_resize(&renderer);
                            }
                            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                // new_inner_size is &mut so w have to dereference it twice
                                renderer.resize(Some(*new_inner_size));
                                editor.handle_resize(&renderer);
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
