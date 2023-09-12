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
use dream_window::window::Window;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn complete_task() {
    dream_tasks::task_pool::complete_task();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn set_multithreading_enabled(multithreading_enabled: bool) {
    dream_tasks::task_pool::set_multithreading(multithreading_enabled);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn start_worker_thread() {
    dream_tasks::task_pool::start_worker_thread();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn run_main() {
    // setup logging (TODO: move logging logic to a new crate)
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Debug).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }

    log::debug!("Running main application");

    // set the root directory to be the project that is opened (by default this is blank example)
    let example_project_name = "blank";
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let path = std::path::Path::new("examples").join(example_project_name);
            dream_fs::fs::set_fs_root(path.to_str().unwrap());
        } else {
            let examples_folder_possible_path = std::path::Path::new(env!("OUT_DIR"))
            .join("..").join("..").join("..").join("..").join("..").join("examples").join(example_project_name);
            if examples_folder_possible_path.exists() {
                // in dev mode try to use the examples folder present here
                dream_fs::fs::set_fs_root(examples_folder_possible_path.to_str().unwrap());
            } else {
                // otherwise in release mode use the examples folder present in the out directory
                println!("{}", examples_folder_possible_path.to_str().unwrap());
                let path = std::path::Path::new(env!("OUT_DIR"))
                    .join("examples")
                    .join(example_project_name);
                dream_fs::fs::set_fs_root(path.to_str().unwrap());
            }
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
