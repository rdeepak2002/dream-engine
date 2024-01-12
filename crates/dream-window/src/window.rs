use crossbeam_channel::unbounded;
use winit::dpi::PhysicalSize;
use winit::keyboard::Key;
use winit::raw_window_handle::HasDisplayHandle;
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
        let event_loop = EventLoop::new().expect("Unable to create event loop");
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

            window.request_inner_size(winit::dpi::LogicalSize::new(
                win.inner_width().unwrap().as_f64().unwrap() as f32,
                win.inner_height().unwrap().as_f64().unwrap() as f32,
            ));

            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("dream-window-container")?;
                    let canvas = web_sys::Element::from(
                        window
                            .canvas()
                            .expect("Unable to acquire HTML canvas for winit"),
                    );
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

        let sleep_millis: u128 = 16;
        let mut last_update_time = dream_time::time::now();
        log::debug!("Starting event loop");
        self.event_loop
            .run(move |event, event_loop_window_target| {
                match event {
                    Event::NewEvents(StartCause) => match StartCause {
                        StartCause::ResumeTimeReached { .. } => {}
                        StartCause::WaitCancelled { .. } => {}
                        StartCause::Poll => {}
                        StartCause::Init => {}
                    },
                    Event::WindowEvent {
                        event,
                        window_id: _window_id,
                    } => {
                        editor.handle_event(&self.window, &event);
                        match event {
                            WindowEvent::ActivationTokenDone { .. } => {}
                            WindowEvent::Resized(physical_size) => {
                                log::debug!("Window event resized");
                                renderer.resize(Some(physical_size));
                                editor.handle_resize(&renderer);
                            }
                            WindowEvent::Moved(_) => {}
                            WindowEvent::CloseRequested => {
                                event_loop_window_target.exit();
                            }
                            WindowEvent::Destroyed => {}
                            WindowEvent::DroppedFile(_) => {}
                            WindowEvent::HoveredFile(_) => {}
                            WindowEvent::HoveredFileCancelled => {}
                            WindowEvent::Focused(_) => {}
                            WindowEvent::KeyboardInput {
                                device_id: _device_id,
                                event,
                                is_synthetic: _is_synthetic,
                            } => {
                                app.process_keyboard(event.logical_key, event.state);
                            }
                            WindowEvent::ModifiersChanged(_) => {}
                            WindowEvent::Ime(_) => {}
                            WindowEvent::CursorMoved { .. } => {}
                            WindowEvent::CursorEntered { .. } => {}
                            WindowEvent::CursorLeft { .. } => {}
                            WindowEvent::MouseWheel {
                                device_id: _device_id,
                                delta,
                                phase: _phase,
                            } => {
                                app.process_scroll(&delta);
                            }
                            WindowEvent::MouseInput {
                                device_id: _device_id,
                                state,
                                button,
                            } => match button {
                                MouseButton::Left => {
                                    app.process_mouse_left_input(state == ElementState::Pressed);
                                }
                                MouseButton::Right => {
                                    app.process_mouse_right_input(state == ElementState::Pressed);
                                }
                                MouseButton::Middle => {}
                                MouseButton::Back => {}
                                MouseButton::Forward => {}
                                MouseButton::Other(_) => {}
                            },
                            WindowEvent::TouchpadMagnify { .. } => {}
                            WindowEvent::SmartMagnify { .. } => {}
                            WindowEvent::TouchpadRotate { .. } => {}
                            WindowEvent::TouchpadPressure { .. } => {}
                            WindowEvent::AxisMotion { .. } => {}
                            WindowEvent::Touch(_) => {}
                            WindowEvent::ScaleFactorChanged {
                                scale_factor,
                                inner_size_writer: _inner_size_writer,
                            } => {
                                // TODO: handle scale factor change
                                log::warn!("TODO: handle scale factor change to {:?}", scale_factor);
                            }
                            WindowEvent::ThemeChanged(_) => {}
                            WindowEvent::Occluded(_) => {}
                            WindowEvent::RedrawRequested => {
                                if let Some(size) = rx.try_iter().last() {
                                    log::debug!("Received new window size");
                                    renderer.resize(Some(PhysicalSize {
                                        width: size.width as u32,
                                        height: size.height as u32,
                                    }));
                                    editor.handle_resize(&renderer);
                                }

                                // use 32 since bloom filter uses a mip chain of size 5 (2^5 = 32)
                                if self.window.inner_size().width < 32 || self.window.inner_size().height < 32 {
                                    log::warn!("Window size is too small in either height of width, skipping frame");
                                    return;
                                }

                                let editor_raw_input =
                                    editor.egui_winit_state.take_egui_input(&self.window);
                                let editor_pixels_per_point = self.window.scale_factor() as f32;

                                let now = dream_time::time::now();
                                if now - last_update_time > sleep_millis {
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
                                    }
                                    // ignore timeout
                                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                                }

                                // draw editor
                                match editor.render_wgpu(
                                    &renderer,
                                    editor_raw_input,
                                    editor_pixels_per_point,
                                ) {
                                    Ok(_) => {
                                        renderer.set_camera_aspect_ratio(
                                            editor.get_renderer_aspect_ratio(),
                                        );
                                    }
                                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                        log::warn!("Surface lost or outdated");
                                        renderer.resize(None);
                                        editor.handle_resize(&renderer);
                                    }
                                    Err(wgpu::SurfaceError::OutOfMemory) => {
                                        log::error!("Out of memory");
                                    }
                                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                                }
                            }
                        }
                    },
                    Event::DeviceEvent {
                        device_id: DeviceId,
                        event: DeviceEvent::MouseMotion { delta },
                    } => {
                        app.process_mouse(delta.0, delta.1);
                    }
                    Event::UserEvent(_) => {}
                    Event::Suspended => {}
                    Event::Resumed => {}
                    Event::AboutToWait => {
                        self.window.request_redraw();
                    }
                    Event::LoopExiting => {}
                    Event::MemoryWarning => {}
                    _ => {}
                }
            })
            .expect("Unable to run event loop");
    }
}
