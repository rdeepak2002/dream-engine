use crate::Panel;

#[derive(Default)]
pub struct InspectorPanel {}

impl Panel for InspectorPanel {
    fn draw(&mut self, egui_context: &egui::Context) {
        egui::SidePanel::right("inspector_panel")
            .resizable(false)
            .default_width(200.0)
            .max_width(400.0)
            .min_width(200.0)
            .show(egui_context, |ui| {
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
    }
}
