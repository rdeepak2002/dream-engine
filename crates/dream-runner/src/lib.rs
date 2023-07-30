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
use dream_window::window::Window;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn run_main() {
    // setup logging (TODO: move logging logic to a new crate)
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }

    // set the root directory to be the project that is opened (by default this is blank example)
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let path = std::path::Path::new("examples").join("blank");
            dream_fs::fs::set_fs_root(path.to_str().unwrap());
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("examples")
                .join("blank");
            dream_fs::fs::set_fs_root(path.to_str().unwrap());
        }
    }

    // TODO: uncomment this
    let window = Window::default();
    window.run().await;

    // TODO: put this in test for memory leaks
    // use dream_editor::EditorEguiWgpu;
    // use dream_renderer::RendererWgpu;
    // for i in 0..1 {
    //     let mut app = App::default();
    //     let mut renderer = dream_renderer::RendererWgpu::default(None).await;
    //     app.update();
    //     app.draw(&mut renderer);
    //     println!("Completed loop {}", i);
    //     // std::thread::sleep(std::time::Duration::Duration::from_millis(16));
    // }
}
