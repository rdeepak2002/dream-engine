use egui::load::SizedTexture;
use egui_wgpu::Renderer;

use dream_app::input::set_renderer_panel_active;

use crate::editor::Panel;

pub struct RendererPanel {
    render_output_epaint_texture_id: Option<egui::epaint::TextureId>,
    // debug_texture_0_id: Option<egui::epaint::TextureId>,
    // debug_texture_1_id: Option<egui::epaint::TextureId>,
    // debug_texture_2_id: Option<egui::epaint::TextureId>,
    // debug_texture_3_id: Option<egui::epaint::TextureId>,
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
            // TODO: try using update texture
            // egui_wgpu_renderer.update_texture(
            //     &state.device,
            //     &state.queue,
            //     *self.render_output_epaint_texture_id.as_ref().unwrap(),
            //     image_delta,
            // );
        }

        // self.render_output_epaint_texture_id = Some(egui_wgpu_renderer.register_native_texture(
        //     &state.device,
        //     &state.frame_texture.view,
        //     wgpu::FilterMode::default(),
        // ));

        // self.render_output_epaint_texture_id = Some(egui_wgpu_renderer.register_native_texture(
        //     &state.device,
        //     &state.bloom_tech.mask_texture.view,
        //     wgpu::FilterMode::default(),
        // ));

        // render HDR texture
        self.render_output_epaint_texture_id = Some(egui_wgpu_renderer.register_native_texture(
            &state.device,
            &state.hdr_tech.hdr_texture.view,
            wgpu::FilterMode::default(),
        ));

        // render bloom mask texture
        // self.render_output_epaint_texture_id = Some(egui_wgpu_renderer.register_native_texture(
        //     &state.device,
        //     &state.bloom_tech.mask_texture.view,
        //     wgpu::FilterMode::default(),
        // ));

        // depth texture of shadow
        // {
        //     if self.render_output_epaint_texture_id.is_some() {
        //         // free old texture to prevent memory leak
        //         egui_wgpu_renderer
        //             .free_texture(self.render_output_epaint_texture_id.as_ref().unwrap());
        //     }
        //
        //     self.render_output_epaint_texture_id =
        //         Some(egui_wgpu_renderer.register_native_texture(
        //             &state.device,
        //             &state.shadow_depth_texture_debugger.output_texture.view,
        //             wgpu::FilterMode::default(),
        //         ));
        // }

        // texture of shadow tech
        // {
        //     if self.debug_texture_0_id.is_some() {
        //         // free old texture to prevent memory leak
        //         egui_wgpu_renderer.free_texture(self.debug_texture_0_id.as_ref().unwrap());
        //     }
        //
        //     self.debug_texture_0_id = Some(egui_wgpu_renderer.register_native_texture(
        //         &state.device,
        //         &state.shadow_tech.frame_textures[0].view,
        //         wgpu::FilterMode::default(),
        //     ));
        // }

        // {
        //     if self.debug_texture_1_id.is_some() {
        //         // free old texture to prevent memory leak
        //         egui_wgpu_renderer.free_texture(self.debug_texture_1_id.as_ref().unwrap());
        //     }
        //
        //     self.debug_texture_1_id = Some(egui_wgpu_renderer.register_native_texture(
        //         &state.device,
        //         &state.shadow_tech.frame_textures[1].view,
        //         wgpu::FilterMode::default(),
        //     ));
        // }

        // {
        //     if self.debug_texture_2_id.is_some() {
        //         // free old texture to prevent memory leak
        //         egui_wgpu_renderer.free_texture(self.debug_texture_2_id.as_ref().unwrap());
        //     }
        //
        //     self.debug_texture_2_id = Some(egui_wgpu_renderer.register_native_texture(
        //         &state.device,
        //         &state.shadow_tech.frame_textures[2].view,
        //         wgpu::FilterMode::default(),
        //     ));
        // }

        // {
        //     if self.debug_texture_3_id.is_some() {
        //         // free old texture to prevent memory leak
        //         egui_wgpu_renderer.free_texture(self.debug_texture_3_id.as_ref().unwrap());
        //     }
        //
        //     self.debug_texture_3_id = Some(egui_wgpu_renderer.register_native_texture(
        //         &state.device,
        //         &state.shadow_tech.frame_textures[3].view,
        //         wgpu::FilterMode::default(),
        //     ));
        // }

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

        // show emissive gbuffer
        // let i = 2;
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
            // debug_texture_0_id: None,
            // debug_texture_1_id: None,
            // debug_texture_2_id: None,
            // debug_texture_3_id: None,
            aspect_ratio: 1.0,
        }
    }
}

impl Panel for RendererPanel {
    fn draw(&mut self, egui_context: &egui::Context) {
        egui::CentralPanel::default().show(egui_context, |ui| {
            if egui_context.is_pointer_over_area() {
                // TODO: why do we have to set this to false?
                set_renderer_panel_active(false);
            } else {
                set_renderer_panel_active(true);
            }
            if self.render_output_epaint_texture_id.is_some() {
                let panel_size = ui.available_size();
                if panel_size.y != 0.0 {
                    let new_aspect_ratio = panel_size.x / panel_size.y;
                    if new_aspect_ratio > 0.0 {
                        self.aspect_ratio = new_aspect_ratio;
                    }
                    ui.image(SizedTexture {
                        id: self.render_output_epaint_texture_id.unwrap(),
                        size: panel_size,
                    });
                    // egui::ScrollArea::vertical().show(ui, |ui| {
                    //     ui.image(SizedTexture {
                    //         id: self.render_output_epaint_texture_id.unwrap(),
                    //         size: panel_size / 1.5,
                    //     });
                    //     ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                    //         ui.image(SizedTexture {
                    //             id: self.debug_texture_0_id.unwrap(),
                    //             size: panel_size / 4.5,
                    //         });
                    //         ui.image(SizedTexture {
                    //             id: self.debug_texture_1_id.unwrap(),
                    //             size: panel_size / 4.5,
                    //         });
                    //         ui.image(SizedTexture {
                    //             id: self.debug_texture_2_id.unwrap(),
                    //             size: panel_size / 4.5,
                    //         });
                    //         ui.image(SizedTexture {
                    //             id: self.debug_texture_3_id.unwrap(),
                    //             size: panel_size / 4.5,
                    //         });
                    //     });
                    // });
                }
            }
        });
    }
}
