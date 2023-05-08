use egui::{RawInput, Widget};

pub struct EditorEguiWgpu {
    pub depth_texture_egui: dream_renderer::texture::Texture,
    pub renderer_aspect_ratio: f32,
    egui_wgpu_renderer: egui_wgpu::Renderer,
    egui_context: egui::Context,
    pub egui_winit_state: egui_winit::State,
    file_epaint_texture_id: egui::epaint::TextureId,
    play_icon_epaint_texture_id: egui::epaint::TextureId,
    directory_epaint_texture_id: egui::epaint::TextureId,
    render_output_epaint_texture_id: Option<egui::epaint::TextureId>,
}

pub fn generate_egui_wgpu_renderer(state: &dream_renderer::RendererWgpu) -> egui_wgpu::Renderer {
    egui_wgpu::Renderer::new(
        &state.device,
        state.surface_format,
        Some(dream_renderer::texture::Texture::DEPTH_FORMAT),
        1,
    )
}

pub fn generate_egui_wgpu_depth_texture(
    state: &dream_renderer::RendererWgpu,
) -> dream_renderer::texture::Texture {
    dream_renderer::texture::Texture::create_depth_texture(
        &state.device,
        &state.config,
        "depth_texture_egui",
    )
}

impl EditorEguiWgpu {
    pub async fn new(
        renderer: &dream_renderer::RendererWgpu,
        scale_factor: f32,
        event_loop: &winit::event_loop::EventLoop<()>,
    ) -> Self {
        let depth_texture_egui = generate_egui_wgpu_depth_texture(renderer);
        let mut egui_wgpu_renderer = generate_egui_wgpu_renderer(renderer);
        let mut egui_winit_state = egui_winit::State::new(&event_loop);
        egui_winit_state.set_pixels_per_point(scale_factor);
        let egui_winit_context = egui::Context::default();

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

        let play_icon_epaint_texture_id = egui_wgpu_renderer.register_native_texture(
            &renderer.device,
            &renderer.play_icon_texture.view,
            wgpu::FilterMode::Linear,
        );

        Self {
            renderer_aspect_ratio: 1.0,
            egui_wgpu_renderer,
            egui_context: egui_winit_context,
            egui_winit_state,
            file_epaint_texture_id,
            directory_epaint_texture_id,
            play_icon_epaint_texture_id,
            render_output_epaint_texture_id: None,
            depth_texture_egui,
        }
    }

    pub fn render_wgpu(
        &mut self,
        state: &dream_renderer::RendererWgpu,
        input: RawInput,
        pixels_per_point: f32,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = state.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // let input = self.egui_winit_state.take_egui_input(&window);
        self.egui_context.begin_frame(input);
        {
            if state.frame_texture_view.is_some() {
                self.render_output_epaint_texture_id =
                    Some(self.egui_wgpu_renderer.register_native_texture(
                        &state.device,
                        &state.frame_texture_view.as_ref().unwrap(),
                        wgpu::FilterMode::default(),
                    ));
            }

            self.renderer_aspect_ratio = self.render_egui_editor_content();
        }
        let egui_full_output = self.egui_context.end_frame();

        let egui_paint_jobs = self.egui_context.tessellate(egui_full_output.shapes);
        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("EGUI Render Encoder"),
            });

        {
            for (id, image_delta) in &egui_full_output.textures_delta.set {
                self.egui_wgpu_renderer.update_texture(
                    &state.device,
                    &state.queue,
                    *id,
                    image_delta,
                )
            }

            let egui_screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
                size_in_pixels: [state.config.width, state.config.height],
                pixels_per_point,
            };

            self.egui_wgpu_renderer.update_buffers(
                &state.device,
                &state.queue,
                &mut encoder,
                &egui_paint_jobs,
                &egui_screen_descriptor,
            );

            // draw editor
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("EGUI Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.10588235294,
                                g: 0.10588235294,
                                b: 0.10588235294,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture_egui.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                });

                self.egui_wgpu_renderer.render(
                    &mut render_pass,
                    &egui_paint_jobs,
                    &egui_screen_descriptor,
                );
            }

            state.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }

        for id in &egui_full_output.textures_delta.free {
            self.egui_wgpu_renderer.free_texture(id);
        }

        Ok(())
    }

    pub fn render_egui_editor_content(&mut self) -> f32 {
        egui::TopBottomPanel::top("menu_bar").show(&self.egui_context, |ui| {
            egui::menu::bar(ui, |ui| {
                let save_shortcut =
                    egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::S);

                if ui.input_mut(|i| i.consume_shortcut(&save_shortcut)) {
                    // TODO: allow saving
                    println!("TODO: save");
                }

                ui.menu_button("File", |ui| {
                    ui.set_min_width(100.0);
                    ui.style_mut().wrap = Some(false);

                    if ui
                        .add(
                            egui::Button::new("Save")
                                .shortcut_text(ui.ctx().format_shortcut(&save_shortcut)),
                        )
                        .clicked()
                    {
                        // TODO: allow saving
                        println!("TODO: save");
                        ui.close_menu();
                    }
                });
            });
        });

        egui::SidePanel::right("inspector_panel")
            .resizable(false)
            .default_width(200.0)
            .max_width(400.0)
            .min_width(200.0)
            .show(&self.egui_context, |ui| {
                egui::trace!(ui);

                // name entity name
                ui.strong("Entity 1");

                // sample tag component
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    ui.make_persistent_id("Tag"),
                    true,
                )
                .show_header(ui, |ui| {
                    // ui.toggle_value(&mut self.selected, "Click to select/unselect");
                    ui.strong("Tag");
                })
                .body(|ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        ui.label("Untagged");
                    });
                });

                // sample transform component
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    ui.make_persistent_id("Transform"),
                    true,
                )
                .show_header(ui, |ui| {
                    // ui.toggle_value(&mut self.selected, "Click to select/unselect");
                    ui.strong("Transform");
                })
                .body(|ui| {
                    ui.strong("Position");
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        ui.strong("x");
                        ui.label("0.000");
                        ui.strong("y");
                        ui.label("0.000");
                        ui.strong("z");
                        ui.label("0.000");
                    });
                    ui.strong("Rotation");
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        ui.strong("x");
                        ui.label("0.000");
                        ui.strong("y");
                        ui.label("0.000");
                        ui.strong("z");
                        ui.label("0.000");
                    });
                    ui.strong("Scale");
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        ui.strong("x");
                        ui.label("0.000");
                        ui.strong("y");
                        ui.label("0.000");
                        ui.strong("z");
                        ui.label("0.000");
                    });
                });
            });

        egui::TopBottomPanel::bottom("assets")
            .resizable(false)
            .default_height(200.0)
            .max_height(200.0)
            .min_height(200.0)
            .show(&self.egui_context, |ui| {
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

        egui::SidePanel::left("scene_hierarchy")
            .resizable(false)
            .default_width(200.0)
            .max_width(400.0)
            .min_width(200.0)
            .show(&self.egui_context, |ui| {
                egui::trace!(ui);

                // sample list entity 1
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    ui.make_persistent_id("Entity 1"),
                    false,
                )
                .show_header(ui, |ui| {
                    // ui.toggle_value(&mut self.selected, "Click to select/unselect");
                    ui.strong("Entity 1");
                })
                .body(|ui| {
                    // TODO: recursively call this
                    {
                        egui::collapsing_header::CollapsingState::load_with_default_open(
                            ui.ctx(),
                            ui.make_persistent_id("Entity 1 child"),
                            false,
                        )
                        .show_header(ui, |ui| {
                            // ui.toggle_value(&mut self.selected, "Click to select/unselect");
                            ui.strong("Entity 1 child");
                        })
                        .body(|_ui| {});
                    }
                });

                // sample list entity 2
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    ui.make_persistent_id("Entity 2"),
                    false,
                )
                .show_header(ui, |ui| {
                    // ui.toggle_value(&mut self.selected, "Click to select/unselect");
                    ui.strong("Entity 2");
                })
                .body(|ui| {
                    // TODO: recursively call this
                    {
                        egui::collapsing_header::CollapsingState::load_with_default_open(
                            ui.ctx(),
                            ui.make_persistent_id("Entity 2 child"),
                            false,
                        )
                        .show_header(ui, |ui| {
                            // ui.toggle_value(&mut self.selected, "Click to select/unselect");
                            ui.strong("Entity 2 child");
                        })
                        .body(|_ui| {});
                    }
                });

                // sample list entity 3
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    ui.make_persistent_id("Entity 3"),
                    false,
                )
                .show_header(ui, |ui| {
                    // ui.toggle_value(&mut self.selected, "Click to select/unselect");
                    ui.strong("Entity 3");
                })
                .body(|ui| {
                    // TODO: recursively call this
                    {
                        egui::collapsing_header::CollapsingState::load_with_default_open(
                            ui.ctx(),
                            ui.make_persistent_id("Entity 3 child"),
                            false,
                        )
                        .show_header(ui, |ui| {
                            // ui.toggle_value(&mut self.selected, "Click to select/unselect");
                            ui.strong("Entity 3 child");
                        })
                        .body(|_ui| {});
                    }
                });
            });

        egui::TopBottomPanel::top("render-controls")
            .resizable(false)
            .default_height(25.0)
            .max_height(25.0)
            .min_height(25.0)
            .show(&self.egui_context, |ui| {
                egui::trace!(ui);
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let btn = egui::ImageButton::new(
                        self.play_icon_epaint_texture_id,
                        egui::vec2(15.5, 15.5),
                    );
                    btn.ui(ui);
                });
            });

        let mut aspect_ratio: f32 = 1.0;

        egui::CentralPanel::default().show(&self.egui_context, |ui| {
            if self.render_output_epaint_texture_id.is_some() {
                let panel_size = ui.available_size();
                let new_aspect_ratio = panel_size.x / panel_size.y;
                if new_aspect_ratio > 0.0 {
                    aspect_ratio = new_aspect_ratio;
                }
                ui.image(self.render_output_epaint_texture_id.unwrap(), panel_size);
            }
        });

        return aspect_ratio;
    }

    pub fn handle_resize(&mut self, state: &dream_renderer::RendererWgpu) {
        self.depth_texture_egui = generate_egui_wgpu_depth_texture(state);
    }

    pub fn handle_event(&mut self, window_event: &winit::event::WindowEvent) -> bool {
        self.egui_winit_state
            .on_event(&self.egui_context, &window_event)
            .consumed
    }
}
