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
use crate::scene::{Scene, SCENE};

pub struct Entity {
    // pub scene: *mut Scene,
    pub handle: EntityId, // TODO: make this a u64 for more generalization
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
        Self { handle }
    }

    pub fn attach(&self, parent_entity_runtime_id: Option<u64>) {
        if parent_entity_runtime_id.is_some() {
            // define parent of child component for this entity
            let mut hierarchy_component = self.get_hierarchy().expect("No hierarchy component");
            hierarchy_component.parent_runtime_id =
                parent_entity_runtime_id.expect("No parent runtime id");
            self.add_hierarchy(hierarchy_component);
            // add to child collection of parent
            let parent_shipyard_id =
                shipyard::EntityId::from_inner(parent_entity_runtime_id.unwrap()).unwrap();
            let parent_entity: Entity;
            // parent_entity = Entity::from_ptr(self.scene, parent_shipyard_id);
            parent_entity = Entity::from_handle(parent_shipyard_id.inner());
            let mut parent_hierarchy = parent_entity.get_hierarchy().unwrap();
            if parent_hierarchy.first_child_runtime_id == 0 {
                parent_hierarchy.num_children += 1;
                parent_hierarchy.first_child_runtime_id = self.handle.inner();
            } else {
                parent_hierarchy.num_children += 1;
                // TODO: we are adding a child that is not the first child, so update this by making it the 'previous' of the current child
                // basically just append to start of list for easiest implementation
                todo!()
            }
            parent_entity.add_hierarchy(parent_hierarchy);
        }
    }

    pub fn from_handle(handle: u64) -> Self {
        Self {
            handle: EntityId::from_inner(handle).expect("No entity with this id exists"),
        }
    }

    pub fn to_string(&self) -> String {
        let trans = self.get_transform();
        if trans.is_some() {
            format!("Entity({})", trans.unwrap().to_string())
        } else {
            format!("Entity()")
        }
    }

    pub fn get_runtime_id(&self) -> u64 {
        return self.handle.inner();
    }

    pub fn get_hierarchy(&self) -> Option<Hierarchy> {
        let mut comp_opt: Option<Hierarchy> = None;
        let scene = SCENE.read().unwrap();
        scene.handle.run(|vm_pos: shipyard::ViewMut<Hierarchy>| {
            let comp = vm_pos.get(self.handle).unwrap();
            comp_opt = Some(comp.clone());
        });
        return comp_opt;
    }

    pub fn add_hierarchy(&self, hierarchy: Hierarchy) {
        let mut scene = SCENE.write().unwrap();
        scene.handle.add_component(self.handle, hierarchy);
    }

    pub fn add_transform(&self, transform: Transform) {
        let mut scene = SCENE.write().unwrap();
        scene.handle.add_component(self.handle, transform);
    }

    pub fn get_transform(&self) -> Option<Transform> {
        let mut comp_opt: Option<Transform> = None;
        let scene = SCENE.read().unwrap();
        scene.handle.run(|vm_pos: shipyard::ViewMut<Transform>| {
            let comp = vm_pos.get(self.handle).unwrap();
            comp_opt = Some(comp.clone());
        });
        return comp_opt;
    }

    pub fn has_transform(&self) -> bool {
        self.get_transform().is_some()
    }
}
