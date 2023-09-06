use egui_wgpu::Renderer;

use crate::editor::Panel;

pub struct AssetsPanel {
    file_epaint_texture_id: egui::epaint::TextureId,
    directory_epaint_texture_id: egui::epaint::TextureId,
}

impl AssetsPanel {
    pub fn new(renderer: &dream_renderer::RendererWgpu, egui_wgpu_renderer: &mut Renderer) -> Self {
        let file_epaint_texture_id = egui_wgpu_renderer.register_native_texture(
            &renderer.device,
            &renderer.file_icon_texture.view,
            wgpu::FilterMode::Linear,
        );

        let directory_epaint_texture_id = egui_wgpu_renderer.register_native_texture(
            &renderer.device,
            &renderer.directory_icon_texture.view,
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
