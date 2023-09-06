use std::sync::{Mutex, Weak};

use egui::Ui;

use dream_ecs::component::Tag;
use dream_ecs::entity::Entity;
use dream_ecs::scene::{get_children_for_entity, Scene};

use crate::Panel;

pub struct SceneHierarchyPanel {
    pub scene: Weak<Mutex<Scene>>,
}

impl SceneHierarchyPanel {
    pub fn new(scene: Weak<Mutex<Scene>>) -> Self {
        Self { scene }
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
                egui::trace!(ui);
                let scene = self.scene.upgrade().unwrap();
                let scene = scene.lock().unwrap();
                let root_entity_id = scene.root_entity_runtime_id;
                drop(scene);
                if let Some(root_entity_id) = root_entity_id {
                    let children = get_children_for_entity(self.scene.clone(), root_entity_id);
                    for child in children {
                        self.draw_scene_hierarchy_entity(child, ui);
                    }
                }
            });
    }
}

impl SceneHierarchyPanel {
    fn draw_scene_hierarchy_entity(&self, entity_id: u64, ui: &mut Ui) {
        let id_str = format!("scene_panel_entity_{entity_id}");
        let collapsing_state = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id(id_str.clone()),
            false,
        );
        // if collapsing_state.
        collapsing_state
            .show_header(ui, |ui| {
                let toggle_button = ui.toggle_value(&mut true, "Edit");
                if toggle_button.clicked() {
                    todo!();
                    // TODO: set a variable indicating that this entity is now selected and must be shown in inspector
                    // TODO: set a variable indicating that this entity is now selected and must be shown in inspector
                    // TODO: set a variable indicating that this entity is now selected and must be shown in inspector
                    // TODO: set a variable indicating that this entity is now selected and must be shown in inspector
                    // TODO: set a variable indicating that this entity is now selected and must be shown in inspector
                    // TODO: set a variable indicating that this entity is now selected and must be shown in inspector
                    // TODO: set a variable indicating that this entity is now selected and must be shown in inspector
                    // TODO: set a variable indicating that this entity is now selected and must be shown in inspector
                    // TODO: set a variable indicating that this entity is now selected and must be shown in inspector
                    // TODO: set a variable indicating that this entity is now selected and must be shown in inspector
                    // TODO: set a variable indicating that this entity is now selected and must be shown in inspector
                }
                let entity = Entity::from_handle(entity_id, self.scene.clone());
                if entity.has_component::<Tag>() {
                    let name = entity.get_component::<Tag>().unwrap().name;
                    ui.strong(name);
                } else {
                    ui.strong("Entity");
                }
            })
            .body(|ui| {
                let children = get_children_for_entity(self.scene.clone(), entity_id);
                for child in children {
                    self.draw_scene_hierarchy_entity(child, ui);
                }
            });
    }
}