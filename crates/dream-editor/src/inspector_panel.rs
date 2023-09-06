use std::sync::{Mutex, Weak};

use crossbeam_channel::Receiver;

use dream_ecs::component::{Tag, Transform};
use dream_ecs::entity::Entity;
use dream_ecs::scene::Scene;

use crate::editor::{EditorEvent, EditorEventType, Panel};

pub struct InspectorPanel {
    rx: Receiver<EditorEvent>,
    scene: Weak<Mutex<Scene>>,
    selected_entity_id: Option<u64>,
}

impl InspectorPanel {
    pub fn new(rx: Receiver<EditorEvent>, scene: Weak<Mutex<Scene>>) -> Self {
        Self {
            rx,
            scene,
            selected_entity_id: None,
        }
    }
}

impl Panel for InspectorPanel {
    fn draw(&mut self, egui_context: &egui::Context) {
        egui::SidePanel::right("inspector_panel")
            .resizable(false)
            .default_width(200.0)
            .max_width(400.0)
            .min_width(200.0)
            .show(egui_context, |ui| {
                if let Some(editor_event) = self.rx.try_iter().last() {
                    match editor_event.event_type {
                        EditorEventType::ShowEntityInInspector => {
                            let entity_id = editor_event.event_data;
                            self.selected_entity_id = Some(
                                entity_id
                                    .parse()
                                    .expect("Inspector did not receive a u64 for entity ID"),
                            );
                        }
                    }
                }

                egui::trace!(ui);

                if let Some(entity_id) = self.selected_entity_id {
                    let entity = Entity::from_handle(entity_id, self.scene.clone());

                    let tag_component: Option<Tag> = entity.get_component();
                    let transform_component: Option<Transform> = entity.get_component();

                    if let Some(tag_component) = tag_component {
                        // name entity name
                        ui.strong(tag_component.name);
                    }

                    if let Some(transform_component) = transform_component {
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
                                ui.label(transform_component.position.x.to_string());
                                ui.strong("y");
                                ui.label(transform_component.position.y.to_string());
                                ui.strong("z");
                                ui.label(transform_component.position.z.to_string());
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
                                ui.label("1.000");
                                ui.strong("y");
                                ui.label("1.000");
                                ui.strong("z");
                                ui.label("1.000");
                            });
                        });
                    }
                }
            });
    }
}
