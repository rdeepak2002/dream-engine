use egui_wgpu::Renderer;

use dream_renderer::image::Image;
use dream_renderer::texture;

use crate::editor::Panel;

pub struct AssetsPanel {
    file_epaint_texture_id: egui::epaint::TextureId,
    directory_epaint_texture_id: egui::epaint::TextureId,
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
                egui::trace!(ui);
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.style_mut().spacing.item_spacing = egui::vec2(20.0, 1.0);

                    {
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            ui.image(self.file_epaint_texture_id, egui::vec2(40.0, 40.0));
                            ui.strong("main.scene");
                        });
                    }

                    {
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            ui.image(self.directory_epaint_texture_id, egui::vec2(40.0, 40.0));
                            ui.strong("textures");
                        });
                    }
                });
            });
    }
}
