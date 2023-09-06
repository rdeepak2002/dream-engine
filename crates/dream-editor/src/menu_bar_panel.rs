use crate::editor::Panel;

#[derive(Default)]
pub struct MenuBarPanel {}

impl Panel for MenuBarPanel {
    fn draw(&mut self, egui_context: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(egui_context, |ui| {
            egui::menu::bar(ui, |ui| {
                let save_shortcut =
                    egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::S);

                if ui.input_mut(|i| i.consume_shortcut(&save_shortcut)) {
                    // TODO: allow saving
                    println!("TODO: save");
                    todo!();
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
                        todo!();
                    }
                });
            });
        });
    }
}
