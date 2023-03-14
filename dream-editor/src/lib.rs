use egui::Widget;

pub fn render_egui_editor_content(
    ctx: &egui::Context,
    render_output_epaint_texture_id: Option<egui::epaint::TextureId>,
    file_epaint_texture_id: egui::epaint::TextureId,
    directory_epaint_texture_id: egui::epaint::TextureId,
    play_icon_epaint_texture_id: egui::epaint::TextureId,
) -> f32 {
    // Draw the demo application.
    // self.demo_app.ui(&self.egui_context);

    egui::TopBottomPanel::top("menu_bar").show(&ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            let save_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::S);

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
        .show(&ctx, |ui| {
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
        .show(&ctx, |ui| {
            egui::trace!(ui);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(20.0, 1.0);

                {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.image(file_epaint_texture_id, egui::vec2(40.0, 40.0));
                        ui.strong("main.scene");
                    });
                }

                {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.image(directory_epaint_texture_id, egui::vec2(40.0, 40.0));
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
        .show(&ctx, |ui| {
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
        .show(&ctx, |ui| {
            egui::trace!(ui);
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let btn =
                    egui::ImageButton::new(play_icon_epaint_texture_id, egui::vec2(15.5, 15.5));
                btn.ui(ui);
            });
        });

    let mut aspect_ratio: f32 = 1.0;

    egui::CentralPanel::default().show(&ctx, |ui| {
        if render_output_epaint_texture_id.is_some() {
            let panel_size = ui.available_size();
            let new_aspect_ratio = panel_size.x / panel_size.y;
            if new_aspect_ratio > 0.0 {
                aspect_ratio = new_aspect_ratio;
            }
            ui.image(render_output_epaint_texture_id.unwrap(), panel_size);
        }
    });

    return aspect_ratio;
}
