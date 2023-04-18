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
use dream_app::app::App;
use dream_editor::EditorEguiWgpu;
use dream_renderer::RendererWgpu;
use dream_window::window::Window;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Update function called every update loop which returns true when the application should close
fn update(
    app: &mut App,
    renderer: &mut RendererWgpu,
    editor: &mut EditorEguiWgpu,
    editor_raw_input: egui::RawInput,
    editor_pixels_per_point: f32,
) -> bool {
    // update component systems (scripts, physics, etc.)
    app.update();

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
    match editor.render_wgpu(&renderer, editor_raw_input, editor_pixels_per_point) {
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

    let app = Box::new(App::new());
    let window = Window::new();
    window.run(app, update).await;
}
