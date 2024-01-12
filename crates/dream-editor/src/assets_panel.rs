use std::path::PathBuf;

use egui::load::SizedTexture;
use egui::Widget;
use egui_wgpu::Renderer;

use dream_fs::fs::ReadDir;
use dream_renderer::image::Image;
use dream_renderer::texture;

use crate::editor::Panel;

pub struct AssetsPanel {
    file_epaint_texture_id: egui::epaint::TextureId,
    directory_epaint_texture_id: egui::epaint::TextureId,
    current_directory: PathBuf,
    cached_directory_result: Option<Vec<ReadDir>>,
}

impl AssetsPanel {
    pub fn new(
        renderer: &dream_renderer::renderer::RendererWgpu,
        egui_wgpu_renderer: &mut Renderer,
    ) -> Self {
        let file_icon_texture_bytes = include_bytes!("icons/FileIcon.png");
        let mut file_icon_image = Image::default();
        file_icon_image.load_from_bytes(file_icon_texture_bytes, "icons/FileIcon.png", None);
        let rgba = file_icon_image.to_rgba8();
        let file_icon_texture = texture::Texture::new(
            &renderer.device,
            &renderer.queue,
            rgba.to_vec(),
            rgba.dimensions(),
            None,
            Some(wgpu::FilterMode::Nearest),
            None,
        )
        .expect("Unable to load file icon texture");

        let directory_icon_texture_bytes = include_bytes!("icons/DirectoryIcon.png");
        let mut directory_icon_image = Image::default();
        directory_icon_image.load_from_bytes(
            directory_icon_texture_bytes,
            "icons/DirectoryIcon.png",
            None,
        );
        let rgba = directory_icon_image.to_rgba8();
        let directory_icon_texture = texture::Texture::new(
            &renderer.device,
            &renderer.queue,
            rgba.to_vec(),
            rgba.dimensions(),
            None,
            Some(wgpu::FilterMode::Nearest),
            None,
        )
        .expect("Unable to load directory icon texture");

        let file_epaint_texture_id = egui_wgpu_renderer.register_native_texture(
            &renderer.device,
            &file_icon_texture.view,
            wgpu::FilterMode::Linear,
        );

        let directory_epaint_texture_id = egui_wgpu_renderer.register_native_texture(
            &renderer.device,
            &directory_icon_texture.view,
            wgpu::FilterMode::Linear,
        );

        Self {
            file_epaint_texture_id,
            directory_epaint_texture_id,
            current_directory: dream_fs::fs::get_fs_root(),
            cached_directory_result: None,
        }
    }
}

impl Panel for AssetsPanel {
    fn draw(&mut self, egui_context: &egui::Context) {
        egui::TopBottomPanel::bottom("assets")
            .resizable(false)
            .default_height(200.0)
            .max_height(200.0)
            .min_height(200.0)
            .show(egui_context, |ui| {
                // buttons to navigate back to root or any previous folder
                let root_dir = dream_fs::fs::get_fs_root();
                let cur_dir = self.current_directory.clone();
                let path_diff = pathdiff::diff_paths(cur_dir, root_dir.clone());
                if let Some(path_diff) = path_diff {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                        if ui
                            .add(egui::Button::new(
                                root_dir.file_name().unwrap().to_str().unwrap(),
                            ))
                            .clicked()
                        {
                            self.current_directory = root_dir.clone();
                            self.cached_directory_result = None;
                        }
                        if !path_diff.to_str().unwrap().is_empty() {
                            let mut tmp_path = root_dir.to_path_buf();
                            for path in path_diff.to_str().unwrap().split('/') {
                                if ui.add(egui::Button::new(path)).clicked() {
                                    tmp_path.push(path);
                                    self.current_directory = tmp_path.clone();
                                    self.cached_directory_result = None;
                                }
                            }
                        }
                    });
                }

                // grid
                let panel_width = ui.available_width();
                let size = 50.0;
                let extra_size = 25.0;
                let num_items_per_row = (panel_width / (size + 10.0)) as i32 as usize;
                if num_items_per_row == 0 {
                    return;
                }
                egui::Grid::new("AssetGrid")
                    .min_col_width(size + extra_size)
                    .max_col_width(size + extra_size)
                    .show(ui, |ui| {
                        if self.cached_directory_result.as_ref().is_none() {
                            self.cached_directory_result = Some(
                                dream_fs::fs::read_dir(self.current_directory.clone())
                                    .expect("Unable to read current directory"),
                            );
                            log::debug!("Reading directory info");
                        } else {
                            let mut idx = 0;
                            let mut reset_cached_directory_result = false;
                            for file in self.cached_directory_result.as_ref().unwrap() {
                                let excluded_files = vec![".DS_Store", "files.json"];
                                let file_name = file.get_name();
                                let is_excluded_file = excluded_files.contains(&&*file_name);
                                let is_meta_file =
                                    file.get_path().extension().unwrap_or("".as_ref()) == "meta";
                                if file.is_dir() || (!is_excluded_file && !is_meta_file) {
                                    if file.is_dir() {
                                        ui.with_layout(
                                            egui::Layout::top_down(egui::Align::LEFT),
                                            |ui| {
                                                let image = SizedTexture {
                                                    id: self.directory_epaint_texture_id,
                                                    size: egui::vec2(size, size),
                                                };
                                                if egui::ImageButton::new(image).ui(ui).clicked() {
                                                    self.current_directory = file.get_path();
                                                    reset_cached_directory_result = true;
                                                }
                                                ui.strong(file.get_name());
                                            },
                                        );
                                    } else {
                                        ui.with_layout(
                                            egui::Layout::top_down(egui::Align::LEFT),
                                            |ui| {
                                                let image = SizedTexture {
                                                    id: self.file_epaint_texture_id,
                                                    size: egui::vec2(size, size),
                                                };
                                                if egui::ImageButton::new(image).ui(ui).clicked() {
                                                    log::debug!("TODO: open this file");
                                                }
                                                ui.strong(file.get_name());
                                            },
                                        );
                                    }
                                    if (idx + 1) % num_items_per_row == 0 {
                                        ui.end_row();
                                    }
                                    idx += 1;
                                }
                            }
                            if reset_cached_directory_result {
                                self.cached_directory_result = None;
                            }
                        }
                    });
            });
    }
}
