use egui::load::SizedTexture;
use egui::Widget;
use egui_wgpu::Renderer;

use dream_renderer::image::Image;
use dream_renderer::texture;

use crate::editor::Panel;

pub struct RendererControlsPanel {
    play_icon_epaint_texture_id: egui::epaint::TextureId,
}

impl RendererControlsPanel {
    pub fn new(
        renderer: &dream_renderer::renderer::RendererWgpu,
        egui_wgpu_renderer: &mut Renderer,
    ) -> Self {
        let play_icon_texture_bytes = include_bytes!("icons/PlayIcon.png");
        let mut play_icon_image = Image::default();
        play_icon_image.load_from_bytes(play_icon_texture_bytes, "icons/PlayIcon.png", None);
        let rgba = play_icon_image.to_rgba8();
        let play_icon_texture = texture::Texture::new(
            &renderer.device,
            &renderer.queue,
            rgba.to_vec(),
            rgba.dimensions(),
            None,
            Some(wgpu::FilterMode::Nearest),
            None,
        )
        .expect("Unable to load play icon texture");

        let play_icon_epaint_texture_id = egui_wgpu_renderer.register_native_texture(
            &renderer.device,
            &play_icon_texture.view,
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
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let image = SizedTexture {
                        id: self.play_icon_epaint_texture_id,
                        size: egui::vec2(15.5, 15.5),
                    };
                    let btn = egui::ImageButton::new(image);
                    btn.ui(ui);
                });
            });
    }
}
