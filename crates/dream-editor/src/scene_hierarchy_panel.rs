use std::sync::{Mutex, Weak};

use crossbeam_channel::Sender;
use egui::{vec2, Sense, Ui};

use dream_ecs::component::Tag;
use dream_ecs::entity::Entity;
use dream_ecs::scene::Scene;
use dream_math::max;

use crate::editor::{EditorEvent, EditorEventType, Panel};

pub struct SceneHierarchyPanel {
    sx: Sender<EditorEvent>,
    scene: Weak<Mutex<Scene>>,
}

impl SceneHierarchyPanel {
    pub fn new(sx: Sender<EditorEvent>, scene: Weak<Mutex<Scene>>) -> Self {
        Self { sx, scene }
    }
}

impl Panel for SceneHierarchyPanel {
    fn draw(&mut self, egui_context: &egui::Context) {
        egui::SidePanel::left("scene_hierarchy")
            .resizable(false)
            .default_width(200.0)
            .max_width(200.0)
            .min_width(200.0)
            .show(egui_context, |ui| {
                let scene = self.scene.upgrade().unwrap();
                let scene = scene.lock().unwrap();
                let root_entity_id = scene.root_entity_runtime_id;
                drop(scene);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        if let Some(root_entity_id) = root_entity_id {
                            // self.draw_scene_hierarchy_entity(root_entity_id, ui);
                            let children =
                                Scene::get_children_for_entity(self.scene.clone(), root_entity_id);
                            for child in children {
                                self.draw_scene_hierarchy_entity(child, ui);
                            }
                        }
                    });
                });
            });
    }
}

impl SceneHierarchyPanel {
    fn draw_scene_hierarchy_entity(&self, entity_id: u64, ui: &mut Ui) {
        let id_str = format!("scene_panel_entity_{entity_id}");
        let mut collapsing_state = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id(id_str),
            false,
        );

        pub fn drop_down_icon(ui: &mut Ui, openness: f32, response: &egui::Response) {
            let visuals = ui.style().interact(response);

            let rect = response.rect;

            // draw a pointy triangle arrow
            let rect = egui::Rect::from_center_size(
                rect.center(),
                vec2(rect.width(), rect.height()) * 0.75,
            );
            let rect = rect.expand(visuals.expansion);
            let mut points = vec![rect.left_top(), rect.right_top(), rect.center_bottom()];
            use std::f32::consts::TAU;
            let rotation =
                egui::emath::Rot2::from_angle(egui::remap(openness, 0.0..=1.0, -TAU / 4.0..=0.0));
            for p in &mut points {
                *p = rect.center() + rotation * (*p - rect.center());
            }

            ui.painter().add(egui::Shape::convex_polygon(
                points,
                visuals.fg_stroke.color,
                egui::Stroke::NONE,
            ));
        }

        let children = Scene::get_children_for_entity(self.scene.clone(), entity_id);
        let header_res = ui.horizontal(|ui| {
            let default_indent = ui.spacing().indent;
            ui.spacing_mut().indent = 4.0;
            if !children.is_empty() {
                collapsing_state.show_toggle_button(ui, drop_down_icon);
            } else {
                let size = vec2(ui.spacing().indent, ui.spacing().icon_width);
                let (_id, _rect) = ui.allocate_space(size);
            }
            ui.spacing_mut().indent = default_indent;
            let entity = Entity::from_handle(entity_id, self.scene.clone());
            let name = entity.get_component::<Tag>().unwrap().name;
            {
                let available_width = ui.available_width();
                let label_response = ui.colored_label(egui::Color32::WHITE, name);
                let mut label_rect = label_response.rect;
                label_rect.set_width(max!(label_response.rect.width(), available_width));
                let response = ui.allocate_rect(label_rect, Sense::click());
                if response.clicked() {
                    self.sx
                        .send(EditorEvent {
                            event_type: EditorEventType::ShowEntityInInspector,
                            event_data: format!("{}", entity_id),
                        })
                        .expect("Unable to transmit show entity event");
                }
            }
        });

        collapsing_state.show_body_indented(&header_res.response, ui, |ui| {
            for child in children {
                self.draw_scene_hierarchy_entity(child, ui);
            }
        });
    }
}
