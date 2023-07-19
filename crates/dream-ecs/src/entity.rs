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

use std::sync::{Arc, Mutex, Weak};

use shipyard::{EntityId, Get};

use crate::component::{Hierarchy, Transform};
use crate::scene::Scene;

#[derive(Clone)]
pub struct Entity {
    pub handle: u64,
    pub scene: Weak<Mutex<Scene>>,
}

impl Entity {
    // pub(crate) fn attach_to_back_with_scene(
    //     &self,
    //     parent_entity_runtime_id: Option<u64>,
    //     scene: &mut Scene,
    // ) {
    //     if parent_entity_runtime_id.is_some() {
    //         let parent_entity = Entity::from_handle(parent_entity_runtime_id.unwrap());
    //         let mut parent_hierarchy: Hierarchy =
    //             parent_entity.get_component_with_scene(scene).unwrap();
    //         if parent_hierarchy.num_children == 0 {
    //             // when this is only child of parent
    //             // step 1: update parent
    //             {
    //                 parent_hierarchy.num_children += 1;
    //                 parent_hierarchy.first_child_runtime_id = self.handle;
    //                 parent_entity.add_component_with_scene::<Hierarchy>(parent_hierarchy, scene);
    //             }
    //             // step 2: update this
    //             {
    //                 let mut hierarchy_component: Hierarchy = self
    //                     .get_component_with_scene(scene)
    //                     .expect("No hierarchy component");
    //                 // update parent pointer of this hierarchy component
    //                 hierarchy_component.parent_runtime_id = parent_entity_runtime_id.unwrap();
    //                 self.add_component_with_scene(hierarchy_component, scene);
    //             }
    //         } else {
    //             // in case where parent already has children, we need to add this as a
    //             // sibling to the end of the linked list
    //             let last_sibling_id;
    //             // step 1: update last sibling
    //             {
    //                 let mut sibling_id = parent_hierarchy.first_child_runtime_id;
    //                 let mut sibling_entity = Entity::from_handle(sibling_id);
    //                 let mut sibling_hierarchy_component: Hierarchy = sibling_entity
    //                     .get_component_with_scene(scene)
    //                     .expect("Sibling does not have hierarchy component");
    //                 while sibling_hierarchy_component.next_sibling_runtime_id != 0 {
    //                     sibling_id = sibling_hierarchy_component.next_sibling_runtime_id;
    //                     sibling_entity = Entity::from_handle(sibling_id);
    //                     sibling_hierarchy_component = sibling_entity
    //                         .get_component_with_scene(scene)
    //                         .expect("Sibling does not have hierarchy component");
    //                 }
    //                 last_sibling_id = sibling_id;
    //                 // update previous pointer of sibling to this entity
    //                 sibling_hierarchy_component.next_sibling_runtime_id = self.handle;
    //                 sibling_entity.add_component_with_scene(sibling_hierarchy_component, scene);
    //             }
    //             // step 2: update parent
    //             {
    //                 // update count of parent hierarchy component
    //                 parent_hierarchy.num_children += 1;
    //                 parent_entity.add_component_with_scene::<Hierarchy>(parent_hierarchy, scene);
    //             }
    //             // step 3: update this
    //             {
    //                 let mut hierarchy_component: Hierarchy = self
    //                     .get_component_with_scene(scene)
    //                     .expect("No hierarchy component");
    //                 // update next pointer of this hierarchy component
    //                 hierarchy_component.prev_sibling_runtime_id = last_sibling_id;
    //                 // update parent pointer of this hierarchy component
    //                 hierarchy_component.parent_runtime_id = parent_entity_runtime_id.unwrap();
    //                 self.add_component_with_scene(hierarchy_component, scene);
    //             }
    //         }
    //     }
    // }

    // pub fn attach_to_front_with_scene(
    //     &self,
    //     parent_entity_runtime_id: Option<u64>,
    //     scene: &mut Scene,
    // ) {
    //     if parent_entity_runtime_id.is_some() {
    //         let parent_entity = Entity::from_handle(parent_entity_runtime_id.unwrap());
    //         let mut parent_hierarchy: Hierarchy =
    //             parent_entity.get_component_with_scene(scene).unwrap();
    //         if parent_hierarchy.num_children == 0 {
    //             // when this is only child of parent
    //             // step 1: update parent
    //             {
    //                 parent_hierarchy.num_children += 1;
    //                 parent_hierarchy.first_child_runtime_id = self.handle;
    //                 parent_entity.add_component_with_scene::<Hierarchy>(parent_hierarchy, scene);
    //             }
    //             // step 2: update this
    //             {
    //                 let mut hierarchy_component: Hierarchy = self
    //                     .get_component_with_scene(scene)
    //                     .expect("No hierarchy component");
    //                 // update parent pointer of this hierarchy component
    //                 hierarchy_component.parent_runtime_id = parent_entity_runtime_id.unwrap();
    //                 self.add_component_with_scene(hierarchy_component, scene);
    //             }
    //         } else {
    //             // in case where parent already has children, we need to add this as a
    //             // sibling to the beginning of the current child linked list
    //             let sibling_id = parent_hierarchy.first_child_runtime_id;
    //             // step 1: update sibling
    //             {
    //                 let sibling_entity = Entity::from_handle(sibling_id);
    //                 let mut sibling_hierarchy_component: Hierarchy = sibling_entity
    //                     .get_component_with_scene(scene)
    //                     .expect("Sibling does not have hierarchy component");
    //                 // update previous pointer of sibling to this entity
    //                 sibling_hierarchy_component.prev_sibling_runtime_id = self.handle;
    //                 sibling_entity.add_component_with_scene(sibling_hierarchy_component, scene);
    //             }
    //             // step 2: update parent
    //             {
    //                 // update first child pointer of parent hierarchy component
    //                 parent_hierarchy.first_child_runtime_id = self.handle;
    //                 // update count of parent hierarchy component
    //                 parent_hierarchy.num_children += 1;
    //                 parent_entity.add_component_with_scene::<Hierarchy>(parent_hierarchy, scene);
    //             }
    //             // step 3: update this
    //             {
    //                 let mut hierarchy_component: Hierarchy = self
    //                     .get_component_with_scene(scene)
    //                     .expect("No hierarchy component");
    //                 // update next pointer of this hierarchy component
    //                 hierarchy_component.next_sibling_runtime_id = sibling_id;
    //                 // update parent pointer of this hierarchy component
    //                 hierarchy_component.parent_runtime_id = parent_entity_runtime_id.unwrap();
    //                 self.add_component_with_scene(hierarchy_component, scene);
    //             }
    //         }
    //     }
    // }

    pub fn from_handle(handle: u64, scene: Weak<Mutex<Scene>>) -> Self {
        Self { handle, scene }
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
        let mut scene = self.scene.upgrade();
        scene
            .expect("Unable to upgrade scene smart pointer for getting component")
            .lock()
            .expect("Unable to get mutex lock")
            .handle
            .add_component(EntityId::from_inner(self.handle).unwrap(), component);
    }

    pub fn remove_component<T: shipyard::TupleRemove>(&self) {
        let mut scene = self.scene.upgrade();
        scene
            .expect("Unable to upgrade scene smart pointer for removing component")
            .lock()
            .expect("Unable to get mutex lock")
            .handle
            .remove::<T>(EntityId::from_inner(self.handle).unwrap());
    }

    pub fn get_component<T: shipyard::Component + Send + Sync + Clone>(&self) -> Option<T> {
        let mut comp_opt: Option<T> = None;
        let mut scene = self.scene.upgrade();
        let system = |vm_pos: shipyard::ViewMut<T>| {
            let comp = vm_pos
                .get(EntityId::from_inner(self.handle).unwrap())
                .unwrap();
            comp_opt = Some(comp.clone());
        };
        scene
            .expect("Unable to upgrade scene smart pointer for getting component")
            .lock()
            .expect("Unable to get mutex lock")
            .handle
            .run(system);
        comp_opt
    }

    pub fn has_component<T: shipyard::Component + Send + Sync + Clone>(&self) -> bool {
        let comp: Option<T> = self.get_component();
        return comp.is_some();
    }

    pub(crate) fn add_component_with_scene<T: shipyard::TupleAddComponent>(
        &self,
        component: T,
        scene: &mut Scene,
    ) {
        // let mut scene = get_current_scene();
        scene
            .handle
            .add_component(EntityId::from_inner(self.handle).unwrap(), component);
    }

    // pub(crate) fn remove_component_with_scene<T: shipyard::TupleRemove>(&self, scene: &mut Scene) {
    //     // let mut scene = get_current_scene();
    //     scene
    //         .handle
    //         .remove::<T>(EntityId::from_inner(self.handle).unwrap());
    // }

    pub(crate) fn get_component_with_scene<T: shipyard::Component + Send + Sync + Clone>(
        &self,
        scene: &Scene,
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

    // pub(crate) fn has_component_with_scene<T: shipyard::Component + Send + Sync + Clone>(
    //     &self,
    //     scene: &Scene,
    // ) -> bool {
    //     let comp: Option<T> = self.get_component_with_scene(scene);
    //     return comp.is_some();
    // }
}
