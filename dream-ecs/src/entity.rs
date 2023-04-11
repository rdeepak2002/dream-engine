/**********************************************************************************
 *  Dream is a software for developing real-time 3D experiences.
 *  Copyright (C) 2023 Deepak Ramalingam
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU Affero General Public License as published
 *  by the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU Affero General Public License for more details.
 *
 *  You should have received a copy of the GNU Affero General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 **********************************************************************************/

use shipyard::{EntityId, Get};

use crate::component::{Hierarchy, Transform};
use crate::scene::{get_current_scene, get_current_scene_read_only, Scene};

pub struct Entity {
    pub handle: u64,
}

impl Entity {
    pub fn new(scene: &mut Scene) -> Self {
        let transform = Transform::new();
        let hierarchy_component = Hierarchy::new();
        let handle = scene.handle.add_entity((transform, hierarchy_component));
        scene.handle.add_component(handle, Transform::new());
        if handle.inner() == 0 {
            panic!("Entity internal ID cannot be 0");
        }
        Self {
            handle: handle.inner(),
        }
    }

    pub fn attach(&self, parent_entity_runtime_id: Option<u64>) {
        if parent_entity_runtime_id.is_some() {
            // define parent of child component for this entity
            let mut hierarchy_component: Hierarchy =
                self.get_component().expect("No hierarchy component");
            hierarchy_component.parent_runtime_id =
                parent_entity_runtime_id.expect("No parent runtime id");
            self.add_component(hierarchy_component);
            // add to child collection of parent
            let parent_shipyard_id =
                shipyard::EntityId::from_inner(parent_entity_runtime_id.unwrap()).unwrap();
            let parent_entity: Entity;
            // parent_entity = Entity::from_ptr(self.scene, parent_shipyard_id);
            parent_entity = Entity::from_handle(parent_shipyard_id.inner());
            let mut parent_hierarchy: Hierarchy = parent_entity.get_component().unwrap();
            if parent_hierarchy.first_child_runtime_id == 0 {
                parent_hierarchy.num_children += 1;
                parent_hierarchy.first_child_runtime_id = self.handle;
            } else {
                parent_hierarchy.num_children += 1;
                // TODO: we are adding a child that is not the first child, so update this by making it the 'previous' of the current child
                // basically just append to start of list for easiest implementation
                todo!()
            }
            parent_entity.add_component(parent_hierarchy);
        }
    }

    pub fn from_handle(handle: u64) -> Self {
        Self { handle }
    }

    pub fn to_string(&self) -> String {
        let trans: Option<Transform> = self.get_component();
        if trans.is_some() {
            format!("Entity({})", trans.unwrap().to_string())
        } else {
            format!("Entity()")
        }
    }

    pub fn get_runtime_id(&self) -> u64 {
        return self.handle;
    }

    pub fn add_component<C: shipyard::TupleAddComponent>(&self, component: C) {
        let mut scene = get_current_scene();
        scene
            .handle
            .add_component(EntityId::from_inner(self.handle).unwrap(), component);
    }

    pub fn get_component<T: shipyard::Component + Send + Sync + Clone>(&self) -> Option<T> {
        let mut comp_opt: Option<T> = None;
        let scene = get_current_scene_read_only();
        let system = |vm_pos: shipyard::ViewMut<T>| {
            let comp = vm_pos
                .get(EntityId::from_inner(self.handle).unwrap())
                .unwrap();
            comp_opt = Some(comp.clone());
        };
        scene.handle.run(system);
        return comp_opt;
    }

    pub fn has_component<T: shipyard::Component + Send + Sync + Clone>(&self) -> bool {
        let comp: Option<T> = self.get_component();
        return comp.is_some();
    }
}
