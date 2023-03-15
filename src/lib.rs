/**********************************************************************************
 *  Dream is a software for developing real-time 3D experiences.
 *  Copyright (C) 2023 Deepak Ramalignam
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU Affero General Public License as published
 *  by the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU Affero General Public License for more details.
 *
 *  You should have received a copy of the GNU Affero General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 **********************************************************************************/

use dream_renderer;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        static mut WINDOW_WIDTH: u32 = 2000;
        static mut WINDOW_HEIGHT: u32 = 1500;
        static mut NEED_TO_RESIZE_WINDOW: bool = false;
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
        pub fn resize_window(width: u32, height: u32) {
            unsafe {
                WINDOW_WIDTH = width;
                WINDOW_HEIGHT = height;
                NEED_TO_RESIZE_WINDOW = true;
            }
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }

    {
        // example execution of javascript code
        // let js_code = "console.log('hello world');";
        let js_code = "7 * 8.1";
        let mut context = boa_engine::Context::default();
        match context.eval(js_code) {
            Ok(res) => {
                println!("{}", res.to_string(&mut context).unwrap());
                log::warn!("{}", res.to_string(&mut context).unwrap());
            }
            Err(e) => {
                // Pretty print the error
                eprintln!("Uncaught {}", e.display());
                log::error!("Uncaught {}", e.display());
            }
        };
    }

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
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        window.set_inner_size(winit::dpi::PhysicalSize::new(2000, 1500));

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

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = dream_renderer::State::new(window, &event_loop).await;
    event_loop.run(move |event, _, control_flow| {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe {
                if NEED_TO_RESIZE_WINDOW {
                    state
                        .window()
                        .set_inner_size(winit::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
                    state.resize(winit::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
                    NEED_TO_RESIZE_WINDOW = false;
                }
            }
        }
        match event {
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();

                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // We're ignoring timeouts
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }

                match state.render_egui() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // We're ignoring timeouts
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::WindowEvent { event, .. } => {
                let exclusive = state.egui_winit_state.on_event(&state.egui_context, &event);
                if !exclusive.consumed {
                    match event {
                        WindowEvent::Resized(physical_size) => {
                            state.resize(physical_size);
                        }
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            state.resize(*new_inner_size);
                        }
                        _ => (),
                    }
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}
