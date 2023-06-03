use crossbeam_channel::unbounded;
// use dream_tasks::task_pool::get_task_pool;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

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
    pub async fn run(
        self,
        mut app: Box<dream_app::app::App>,
        update_func: fn(
            &mut dream_app::app::App,
            &mut dream_renderer::RendererWgpu,
            &mut dream_editor::EditorEguiWgpu,
            egui::RawInput,
            f32,
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

        // TODO: figure out why link is slow (just print stuff to see what stages are slow)
        // TODO: i think the repeated load bytes for default textures is the bottle neck

        // renderer
        //     .store_model(Some("link"), "link.glb")
        //     .await
        //     .expect("Error loading link model");

        renderer
            .store_model(Some("robot"), "robot.glb")
            .await
            .expect("unable to store robot.glb model");

        // renderer
        //     .store_model(Some("robot"), "scene.gltf")
        //     .await
        //     .expect("Error loading robot-gltf model");

        self.event_loop.run(move |event, _, control_flow| {
            match event {
                Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                    if let Some(size) = rx.clone().try_iter().last() {
                        self.window.set_inner_size(size);
                    }

                    let editor_raw_input = editor.egui_winit_state.take_egui_input(&self.window);
                    let editor_pixels_per_point = self.window.scale_factor() as f32;

                    if update_func(
                        app.as_mut(),
                        &mut renderer,
                        &mut editor,
                        editor_raw_input,
                        editor_pixels_per_point,
                    ) {
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
