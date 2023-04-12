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

#[derive(Debug, Clone)]
pub struct Entity {
    pub handle: u64,
}

impl Entity {
    pub fn attach_with_scene(&self, parent_entity_runtime_id: Option<u64>, scene: &mut Scene) {
        if parent_entity_runtime_id.is_some() {
            let parent_shipyard_id =
                EntityId::from_inner(parent_entity_runtime_id.unwrap()).unwrap();
            let parent_entity: Entity;
            parent_entity = Entity::from_handle(parent_shipyard_id.inner());
            let mut parent_hierarchy: Hierarchy =
                parent_entity.get_component_with_scene(scene).unwrap();
            if parent_hierarchy.first_child_runtime_id == 0 {
                parent_hierarchy.num_children += 1;
                parent_hierarchy.first_child_runtime_id = self.handle;
            } else {
                parent_hierarchy.num_children += 1;
                // TODO: we are adding a child that is not the first child, so update this by making it the 'previous' of the current child
                // basically just append to start of list for easiest implementation
                // todo!()
            }
            parent_entity.add_component_with_scene::<Hierarchy>(parent_hierarchy, scene);
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

    pub fn add_component<T: shipyard::TupleAddComponent>(&self, component: T) {
        let mut scene = get_current_scene();
        scene
            .handle
            .add_component(EntityId::from_inner(self.handle).unwrap(), component);
    }

    pub fn remove_component<T: shipyard::TupleRemove>(&self) {
        let mut scene = get_current_scene();
        scene
            .handle
            .remove::<T>(EntityId::from_inner(self.handle).unwrap());
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

    pub fn add_component_with_scene<T: shipyard::TupleAddComponent>(
        &self,
        component: T,
        scene: &mut Scene,
    ) {
        // let mut scene = get_current_scene();
        scene
            .handle
            .add_component(EntityId::from_inner(self.handle).unwrap(), component);
    }

    pub fn remove_component_with_scene<T: shipyard::TupleRemove>(&self, scene: &mut Scene) {
        // let mut scene = get_current_scene();
        scene
            .handle
            .remove::<T>(EntityId::from_inner(self.handle).unwrap());
    }

    pub fn get_component_with_scene<T: shipyard::Component + Send + Sync + Clone>(
        &self,
        scene: &mut Scene,
    ) -> Option<T> {
        let mut comp_opt: Option<T> = None;
        let system = |vm_pos: shipyard::ViewMut<T>| {
            let comp = vm_pos
                .get(EntityId::from_inner(self.handle).unwrap())
                .unwrap();
            comp_opt = Some(comp.clone());
        };
        scene.handle.run(system);
        return comp_opt;
    }

    pub fn has_component_with_scene<T: shipyard::Component + Send + Sync + Clone>(
        &self,
        scene: &mut Scene,
    ) -> bool {
        let comp: Option<T> = self.get_component_with_scene(scene);
        return comp.is_some();
    }
}
