use dream_editor::EditorEguiWgpu;
use dream_renderer::RendererWgpu;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Dream is a software for developing real-time 3D experiences.
/// Copyright (C) 2023 Deepak Ramalignam
///
/// This program is free software: you can redistribute it and/or modify
/// it under the terms of the GNU Affero General Public License as published
/// by the Free Software Foundation, either version 3 of the License, or
/// (at your option) any later version.
///
/// This program is distributed in the hope that it will be useful,
/// but WITHOUT ANY WARRANTY; without even the implied warranty of
/// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
/// GNU Affero General Public License for more details.
///
/// You should have received a copy of the GNU Affero General Public License
/// along with this program.  If not, see <https://www.gnu.org/licenses/>.
use crate::app::update_app;
use crate::window::Window;

mod app;
mod entity_js;
mod javascript_script_component_system;
mod python_script_component_system;
mod window;

/// Update function called every update loop which returns true when the application should close
fn update(
    renderer: &mut RendererWgpu,
    editor: &mut EditorEguiWgpu,
    window: &winit::window::Window,
) -> bool {
    // update component systems (scripts, physics, etc.)
    update_app();

    // draw the scene (to texture)
    match renderer.render() {
        Ok(_) => {}
        // Reconfigure the surface if it's lost or outdated
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            renderer.resize(renderer.size);
            editor.handle_resize(&renderer);
        }
        // The system is out of memory, we should probably quit
        Err(wgpu::SurfaceError::OutOfMemory) => return true,
        // We're ignoring timeouts
        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
    }

    // draw editor
    match editor.render_wgpu(&renderer, &window) {
        Ok(_) => {}
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            renderer.resize(renderer.size);
            editor.handle_resize(&renderer);
        }
        Err(wgpu::SurfaceError::OutOfMemory) => return true,
        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
    }

    renderer.set_camera_aspect_ratio(editor.renderer_aspect_ratio);
    return false;
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

    let window = Window::new();
    window.run(update).await;
}
