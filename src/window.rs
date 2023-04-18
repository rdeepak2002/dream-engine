use crossbeam_channel::unbounded;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use dream_editor::EditorEguiWgpu;
use dream_renderer::RendererWgpu;

use crate::app::App;

pub struct Window {
    pub window: winit::window::Window,
    pub event_loop: EventLoop<()>,
}

impl Window {
    pub fn new() -> Self {
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
                    let dst = doc.get_element_by_id("wasm-example")?;
                    let canvas = web_sys::Element::from(window.canvas());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        Self { window, event_loop }
    }

    pub async fn run(
        self,
        mut app: Box<App>,
        update_func: fn(
            &mut App,
            &mut RendererWgpu,
            &mut EditorEguiWgpu,
            &winit::window::Window,
        ) -> bool,
    ) {
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

        let mut renderer = dream_renderer::RendererWgpu::new(&self.window).await;
        let mut editor = dream_editor::EditorEguiWgpu::new(
            &renderer,
            self.window.scale_factor() as f32,
            &self.event_loop,
        )
        .await;

        // TODO: in render method allow us to call renderer.draw_mesh("dummy_guid", 0)
        // ^ cuz rn the code is always just drawing whatever has id "dummy_guid"
        // TODO: remove this
        renderer
            .store_model(Some("dummy_guid"), "cube.glb")
            .await
            .expect("Error loading model");

        self.event_loop.run(move |event, _, control_flow| {
            match event {
                Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                    if let Some(size) = rx.clone().try_iter().last() {
                        self.window.set_inner_size(size);
                    }

                    if update_func(app.as_mut(), &mut renderer, &mut editor, &self.window) {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                Event::WindowEvent { event, .. } => {
                    if !editor.handle_event(&event) {
                        match event {
                            WindowEvent::Resized(physical_size) => {
                                renderer.resize(physical_size);
                                editor.handle_resize(&renderer);
                            }
                            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                // new_inner_size is &mut so w have to dereference it twice
                                renderer.resize(*new_inner_size);
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
