use egui::Widget;
use egui_wgpu::Renderer;

use crate::editor::Panel;

pub struct RendererControlsPanel {
    play_icon_epaint_texture_id: egui::epaint::TextureId,
}

impl RendererControlsPanel {
    pub fn new(renderer: &dream_renderer::RendererWgpu, egui_wgpu_renderer: &mut Renderer) -> Self {
        let play_icon_epaint_texture_id = egui_wgpu_renderer.register_native_texture(
            &renderer.device,
            &renderer.play_icon_texture.view,
            wgpu::FilterMode::Linear,
        );

        Self {
            play_icon_epaint_texture_id,
        }
    }
}

impl Panel for RendererControlsPanel {
    fn draw(&mut self, egui_context: &egui::Context) {
        egui::TopBottomPanel::top("render-controls")
            .resizable(false)
            .default_height(25.0)
            .max_height(25.0)
            .min_height(25.0)
            .show(egui_context, |ui| {
                egui::trace!(ui);
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let btn = egui::ImageButton::new(
                        self.play_icon_epaint_texture_id,
                        egui::vec2(15.5, 15.5),
                    );
                    btn.ui(ui);
                });
            });
    }
}
