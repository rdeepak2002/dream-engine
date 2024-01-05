use std::ops::RangeInclusive;
use std::sync::{Mutex, Weak};

use crossbeam_channel::Receiver;

use dream_ecs::component::{Bone, Light, MeshRenderer, PythonScript, Tag, Transform};
use dream_ecs::entity::Entity;
use dream_ecs::scene::Scene;
use dream_math::{degrees, pi, radians};

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
            .default_width(250.0)
            .max_width(250.0)
            .min_width(250.0)
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

                if let Some(entity_id) = self.selected_entity_id {
                    let entity = Entity::from_handle(entity_id, self.scene.clone());

                    let tag_component: Option<Tag> = entity.get_component();
                    let transform_component: Option<Transform> = entity.get_component();
                    let mesh_renderer_component: Option<MeshRenderer> = entity.get_component();
                    let python_script_component: Option<PythonScript> = entity.get_component();
                    let light_component: Option<Light> = entity.get_component();
                    let bone_component: Option<Bone> = entity.get_component();

                    if let Some(tag_component) = tag_component {
                        ui.strong(tag_component.name);
                    }

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            if let Some(mut transform_component) = transform_component {
                                egui::collapsing_header::CollapsingState::load_with_default_open(
                                    ui.ctx(),
                                    ui.make_persistent_id("TransformComponent"),
                                    true,
                                )
                                    .show_header(ui, |ui| {
                                        // ui.toggle_value(&mut self.selected, "Click to select/unselect");
                                        ui.strong("Transform");
                                    })
                                    .body(|ui| {
                                        ui.strong("Position");
                                        ui.strong("x");
                                        ui.add(
                                            egui::DragValue::new(&mut transform_component.position.x)
                                                .speed(0.1)
                                                .max_decimals(10),
                                        );
                                        ui.strong("y");
                                        ui.add(
                                            egui::DragValue::new(&mut transform_component.position.y)
                                                .speed(0.1)
                                                .max_decimals(10),
                                        );
                                        ui.strong("z");
                                        ui.add(
                                            egui::DragValue::new(&mut transform_component.position.z)
                                                .speed(0.1)
                                                .max_decimals(10),
                                        );

                                        let roll_pitch_yaw = transform_component.get_euler_angles();
                                        let mut roll = degrees(roll_pitch_yaw.0);
                                        let mut pitch = degrees(roll_pitch_yaw.1);
                                        let mut yaw = degrees(roll_pitch_yaw.2);
                                        ui.strong("Rotation");
                                        ui.strong("roll");
                                        ui.add(
                                            egui::DragValue::new(&mut roll)
                                                .speed(pi() / 10.0)
                                                .max_decimals(2),
                                        );
                                        ui.strong("pitch");
                                        ui.add(
                                            egui::DragValue::new(&mut pitch)
                                                .speed(pi() / 10.0)
                                                .max_decimals(2),
                                        );
                                        ui.strong("yaw");
                                        ui.add(
                                            egui::DragValue::new(&mut yaw)
                                                .speed(pi() / 10.0)
                                                .max_decimals(2),
                                        );
                                        roll = radians(roll);
                                        pitch = radians(pitch);
                                        yaw = radians(yaw);
                                        transform_component.set_euler_angles(roll, pitch, yaw);

                                        ui.strong("Scale");
                                        ui.strong("x");
                                        ui.add(
                                            egui::DragValue::new(&mut transform_component.scale.x)
                                                .speed(0.01)
                                                .max_decimals(5),
                                        );
                                        ui.strong("y");
                                        ui.add(
                                            egui::DragValue::new(&mut transform_component.scale.y)
                                                .speed(0.01)
                                                .max_decimals(5),
                                        );
                                        ui.strong("z");
                                        ui.add(
                                            egui::DragValue::new(&mut transform_component.scale.z)
                                                .speed(0.01)
                                                .max_decimals(5),
                                        );

                                        entity.add_component(transform_component);
                                        // ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                        //     ui.strong("x");
                                        //     ui.add(
                                        //         egui::DragValue::new(&mut transform_component.position.x)
                                        //             .speed(0.1)
                                        //             .max_decimals(3),
                                        //     );
                                        //     ui.strong("y");
                                        //     ui.add(
                                        //         egui::DragValue::new(&mut transform_component.position.y)
                                        //             .speed(0.1)
                                        //             .max_decimals(3),
                                        //     );
                                        //     ui.strong("z");
                                        //     ui.add(
                                        //         egui::DragValue::new(&mut transform_component.position.z)
                                        //             .speed(0.1)
                                        //             .max_decimals(3),
                                        //     );
                                        //     entity.add_component(transform_component);
                                        // });
                                    });
                            }

                            if let Some(mesh_renderer_component) = mesh_renderer_component {
                                egui::collapsing_header::CollapsingState::load_with_default_open(
                                    ui.ctx(),
                                    ui.make_persistent_id("MeshRendererComponent"),
                                    true,
                                )
                                    .show_header(ui, |ui| {
                                        ui.strong("Mesh Renderer");
                                    })
                                    .body(|ui| {
                                        ui.strong("Path");
                                        if let Some(resource_handle) = mesh_renderer_component.resource_handle {
                                            let path = resource_handle.upgrade().expect("Unable to upgrade resource handle for inspector for mesh renderer").path.clone();
                                            ui.label(path.to_str().expect("Unable to convert path to string"));
                                        } else {
                                            ui.label("None");
                                        }
                                        if let Some(mesh_idx) = mesh_renderer_component.mesh_idx {
                                            ui.strong("ID");
                                            ui.label(format!("{mesh_idx}"));
                                        }
                                    });
                            }

                            if let Some(python_script_component) = python_script_component {
                                egui::collapsing_header::CollapsingState::load_with_default_open(
                                    ui.ctx(),
                                    ui.make_persistent_id("PythonScriptComponent"),
                                    true,
                                )
                                    .show_header(ui, |ui| {
                                        ui.strong("Python Script");
                                    })
                                    .body(|ui| {
                                        ui.strong("Path");
                                        if let Some(resource_handle) = python_script_component.resource_handle {
                                            let path = resource_handle.upgrade().expect("Unable to upgrade resource handle for inspector for python script").path.clone();
                                            ui.label(path.to_str().expect("Unable to convert path to string"));
                                        } else {
                                            ui.label("None");
                                        }
                                    });
                            }

                            if let Some(mut light_component) = light_component {
                                egui::collapsing_header::CollapsingState::load_with_default_open(
                                    ui.ctx(),
                                    ui.make_persistent_id("LightComponent"),
                                    true,
                                )
                                    .show_header(ui, |ui| {
                                        ui.strong("Light");
                                    })
                                    .body(|ui| {
                                        ui.strong("Color");
                                        ui.strong("r");
                                        ui.add(
                                            egui::DragValue::new(&mut light_component.color.x)
                                                .speed(0.01)
                                                .max_decimals(5)
                                                .clamp_range(RangeInclusive::new(0.0, 100.0)),
                                        );
                                        ui.strong("g");
                                        ui.add(
                                            egui::DragValue::new(&mut light_component.color.y)
                                                .speed(0.01)
                                                .max_decimals(5)
                                                .clamp_range(RangeInclusive::new(0.0, 100.0))
                                        );
                                        ui.strong("b");
                                        ui.add(
                                            egui::DragValue::new(&mut light_component.color.z)
                                                .speed(0.01)
                                                .max_decimals(5)
                                                .clamp_range(RangeInclusive::new(0.0, 100.0))
                                        );
                                        ui.strong("radius");
                                        ui.add(
                                            egui::DragValue::new(&mut light_component.radius)
                                                .speed(1.0)
                                                .max_decimals(3)
                                                .clamp_range(RangeInclusive::new(0.0, 1000.0))
                                        );
                                        ui.strong("Direction");
                                        ui.strong("x");
                                        ui.add(
                                            egui::DragValue::new(&mut light_component.direction.x)
                                                .speed(0.01)
                                                .max_decimals(5)
                                                .clamp_range(RangeInclusive::new(-1.0, 1.0)),
                                        );
                                        ui.strong("y");
                                        ui.add(
                                            egui::DragValue::new(&mut light_component.direction.y)
                                                .speed(0.01)
                                                .max_decimals(5)
                                                .clamp_range(RangeInclusive::new(-1.0, 1.0))
                                        );
                                        ui.strong("z");
                                        ui.add(
                                            egui::DragValue::new(&mut light_component.direction.z)
                                                .speed(0.01)
                                                .max_decimals(5)
                                                .clamp_range(RangeInclusive::new(-1.0, 1.0))
                                        );

                                        entity.add_component(light_component);
                                    });
                            }

                            if let Some(bone_component) = bone_component {
                                egui::collapsing_header::CollapsingState::load_with_default_open(
                                    ui.ctx(),
                                    ui.make_persistent_id("BoneComponent"),
                                    true,
                                )
                                    .show_header(ui, |ui| {
                                        ui.strong("Bone");
                                    })
                                    .body(|ui| {
                                        ui.strong("Armature Root");
                                        ui.label(format!("{:?}", bone_component.is_root).as_str());
                                        ui.strong("Node ID");
                                        ui.label(format!("{:?}", bone_component.node_id).as_str());
                                        ui.strong("Bone ID");
                                        ui.label(format!("{:?}", bone_component.bone_id).as_str());
                                    });
                            }
                        });
                    });
                }
            });
    }
}
