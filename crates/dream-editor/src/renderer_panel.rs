use egui_wgpu::Renderer;

use crate::editor::Panel;

pub struct RendererPanel {
    render_output_epaint_texture_id: Option<egui::epaint::TextureId>,
    aspect_ratio: f32,
}

impl RendererPanel {
    pub fn update_texture(
        &mut self,
        state: &dream_renderer::renderer::RendererWgpu,
        egui_wgpu_renderer: &mut Renderer,
    ) {
        // show final render
        if self.render_output_epaint_texture_id.is_some() {
            // free old texture to prevent memory leak
            egui_wgpu_renderer.free_texture(self.render_output_epaint_texture_id.as_ref().unwrap());
        }

        self.render_output_epaint_texture_id = Some(egui_wgpu_renderer.register_native_texture(
            &state.device,
            &state.frame_texture.view,
            wgpu::FilterMode::default(),
        ));

        // show deferred result
        // let i = 0;
        // if state.deferred_render_result_texture.is_some() {
        //     if self.render_output_epaint_texture_id.is_some() {
        //         // free old texture to prevent memory leak
        //         egui_wgpu_renderer
        //             .free_texture(self.render_output_epaint_texture_id.as_ref().unwrap());
        //     }
        //
        //     self.render_output_epaint_texture_id =
        //         Some(egui_wgpu_renderer.register_native_texture(
        //             &state.device,
        //             state.g_buffer_texture_views[i].as_ref().unwrap(),
        //             wgpu::FilterMode::default(),
        //         ));
        // }

        // show normal gbuffer
        // let i = 0;
        // if state.deferred_rendering_tech.g_buffer_texture_views[i].is_some() {
        //     if self.render_output_epaint_texture_id.is_some() {
        //         // free old texture to prevent memory leak
        //         egui_wgpu_renderer
        //             .free_texture(self.render_output_epaint_texture_id.as_ref().unwrap());
        //     }
        //
        //     self.render_output_epaint_texture_id = Some(
        //         egui_wgpu_renderer.register_native_texture(
        //             &state.device,
        //             &state.deferred_rendering_tech.g_buffer_texture_views[i]
        //                 .as_ref()
        //                 .unwrap()
        //                 .view,
        //             wgpu::FilterMode::default(),
        //         ),
        //     );
        // }

        // show albedo gbuffer
        // let i = 1;
        // if state.g_buffer_texture_views[i].is_some() {
        //     if self.render_output_epaint_texture_id.is_some() {
        //         // free old texture to prevent memory leak
        //         egui_wgpu_renderer
        //             .free_texture(self.render_output_epaint_texture_id.as_ref().unwrap());
        //     }
        //
        //     self.render_output_epaint_texture_id =
        //         Some(egui_wgpu_renderer.register_native_texture(
        //             &state.device,
        //             &state.g_buffer_texture_views[i].as_ref().unwrap().view,
        //             wgpu::FilterMode::default(),
        //         ));
        // }

        // show emissive gbuffer
        // let i = 2;
        // if state.g_buffer_texture_views[i].is_some() {
        //     if self.render_output_epaint_texture_id.is_some() {
        //         // free old texture to prevent memory leak
        //         egui_wgpu_renderer
        //             .free_texture(self.render_output_epaint_texture_id.as_ref().unwrap());
        //     }
        //
        //     self.render_output_epaint_texture_id =
        //         Some(egui_wgpu_renderer.register_native_texture(
        //             &state.device,
        //             state.g_buffer_texture_views[i].as_ref().unwrap(),
        //             wgpu::FilterMode::default(),
        //         ));
        // }

        // show ao + roughness + metallic gbuffer
        // let i = 3;
        // if state.g_buffer_texture_views[i].is_some() {
        //     if self.render_output_epaint_texture_id.is_some() {
        //         // free old texture to prevent memory leak
        //         egui_wgpu_renderer
        //             .free_texture(self.render_output_epaint_texture_id.as_ref().unwrap());
        //     }
        //
        //     self.render_output_epaint_texture_id =
        //         Some(egui_wgpu_renderer.register_native_texture(
        //             &state.device,
        //             state.g_buffer_texture_views[i].as_ref().unwrap(),
        //             wgpu::FilterMode::default(),
        //         ));
        // }
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }
}

impl Default for RendererPanel {
    fn default() -> Self {
        Self {
            render_output_epaint_texture_id: None,
            aspect_ratio: 1.0,
        }
    }
}

impl Panel for RendererPanel {
    fn draw(&mut self, egui_context: &egui::Context) {
        egui::CentralPanel::default().show(egui_context, |ui| {
            if self.render_output_epaint_texture_id.is_some() {
                let panel_size = ui.available_size();
                if panel_size.y != 0.0 {
                    let new_aspect_ratio = panel_size.x / panel_size.y;
                    if new_aspect_ratio > 0.0 {
                        self.aspect_ratio = new_aspect_ratio;
                    }
                    ui.image(self.render_output_epaint_texture_id.unwrap(), panel_size);
                }
            }
        });
    }
}
