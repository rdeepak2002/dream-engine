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

use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use once_cell::sync::Lazy;
use shipyard::{IntoIter, IntoWithId};

use crate::component::{Hierarchy, Transform};
use crate::entity::Entity;

static SCENE: Lazy<RwLock<Scene>> = Lazy::new(|| RwLock::new(Scene::new()));

pub fn get_current_scene_read_only() -> RwLockReadGuard<'static, Scene> {
    return SCENE.read().unwrap();
}

pub fn get_current_scene() -> RwLockWriteGuard<'static, Scene> {
    return SCENE.write().unwrap();
}

pub struct Scene {
    pub name: &'static str,
    pub handle: shipyard::World,
    pub root_entity_runtime_id: Option<u64>,
}

impl Scene {
    pub fn new() -> Self {
        let name = "scene";
        let handle = shipyard::World::new();
        return Self {
            name,
            handle,
            root_entity_runtime_id: None,
        };
    }

    pub fn create_entity(&mut self) -> Entity {
        let handle = self
            .handle
            .add_entity((Transform::new(), Hierarchy::new()))
            .inner();
        let entity = Entity::from_handle(handle);
        entity.attach_to_back_with_scene(self.root_entity_runtime_id, self);
        if self.root_entity_runtime_id.is_none() {
            self.root_entity_runtime_id = Some(entity.get_runtime_id());
        }
        return entity;
    }

    pub fn create_entity_with_parent(&mut self, parent_entity_runtime_id: u64) -> Entity {
        let handle = self
            .handle
            .add_entity((Transform::new(), Hierarchy::new()))
            .inner();
        let entity = Entity::from_handle(handle);
        entity.attach_to_back_with_scene(Some(parent_entity_runtime_id), self);
        return entity;
    }

    // TODO: if we use the below method, we first have to call remove
    // pub fn attach_entity_to_parent(&mut self, child_handle: u64, parent_handle: u64) {
    //     let child_entity = Entity::from_handle(child_handle);
    //     child_entity.attach_to_back_with_scene(Some(parent_handle), self);
    // }

    // TODO: generalize this
    pub fn transform_entities(&self) -> Vec<u64> {
        let mut entity_id_vec = Vec::new();
        self.handle
            .run(|vm_transform: shipyard::ViewMut<Transform>| {
                for t in vm_transform.iter().with_id() {
                    let entity_id = t.0;
                    entity_id_vec.push(entity_id);
                }
            });
        let mut entity_vec = Vec::new();
        for entity_id in &entity_id_vec {
            entity_vec.push(entity_id.inner());
        }
        return entity_vec;
    }
}

// impl Drop for Scene {
//     /// Remove all entities from scene when scene is deleted (this catches possible memory issues too since Entity struct has unsafe pointer reference)
//     fn drop(&mut self) {
//         self.handle
//             .run(|mut all_storages: shipyard::AllStoragesViewMut| {
//                 let id = all_storages.add_entity(Transform::new());
//                 println!("Deleting entity with runtime ID {}", id.inner());
//                 all_storages.delete_entity(id);
//             });
//     }
// }

#[cfg(test)]
mod tests {
    use crate::component::Hierarchy;
    use crate::scene::{get_current_scene, Scene};

    #[test]
    /// Test adding and removing entities and verifying the hierarchy is correct
    fn test_hierarchy_empty() {
        let scene = Scene::new();
        assert_eq!(scene.root_entity_runtime_id, None);
    }

    #[test]
    /// Test adding and removing entities and verifying the hierarchy is correct
    fn test_hierarchy_one_level() {
        let root_entity = get_current_scene().create_entity();
        let root_entity_child_1 = get_current_scene().create_entity();
        let root_entity_child_2 = get_current_scene().create_entity();
        let root_entity_child_3 = get_current_scene().create_entity();
        // check scene is referring to root entity as the root entity
        assert_eq!(
            get_current_scene().root_entity_runtime_id.unwrap(),
            root_entity.get_runtime_id()
        );
        // check hierarchy for root entity
        assert_eq!(
            root_entity.get_component::<Hierarchy>().unwrap(),
            Hierarchy {
                num_children: 3,
                parent_runtime_id: 0,
                first_child_runtime_id: root_entity_child_1.handle,
                prev_sibling_runtime_id: 0,
                next_sibling_runtime_id: 0,
            }
        );
        // check hierarchy for root_entity_child_1
        assert_eq!(
            root_entity_child_1.get_component::<Hierarchy>().unwrap(),
            Hierarchy {
                num_children: 0,
                parent_runtime_id: root_entity.handle,
                first_child_runtime_id: 0,
                prev_sibling_runtime_id: 0,
                next_sibling_runtime_id: root_entity_child_2.handle,
            }
        );
        // check hierarchy for root_entity_child_2
        assert_eq!(
            root_entity_child_2.get_component::<Hierarchy>().unwrap(),
            Hierarchy {
                num_children: 0,
                parent_runtime_id: root_entity.handle,
                first_child_runtime_id: 0,
                prev_sibling_runtime_id: root_entity_child_1.handle,
                next_sibling_runtime_id: root_entity_child_3.handle,
            }
        );
        // check hierarchy for root_entity_child_3
        assert_eq!(
            root_entity_child_3.get_component::<Hierarchy>().unwrap(),
            Hierarchy {
                num_children: 0,
                parent_runtime_id: root_entity.handle,
                first_child_runtime_id: 0,
                prev_sibling_runtime_id: root_entity_child_2.handle,
                next_sibling_runtime_id: 0,
            }
        );
    }

    #[test]
    /// Test adding and removing entities and verifying the hierarchy is correct
    fn test_hierarchy_three_levels() {
        let level_0 = get_current_scene().create_entity();
        let level_1_a = get_current_scene().create_entity_with_parent(level_0.handle);
        let level_2_a = get_current_scene().create_entity_with_parent(level_1_a.handle);
        let level_2_b = get_current_scene().create_entity_with_parent(level_1_a.handle);
        let level_2_c = get_current_scene().create_entity_with_parent(level_1_a.handle);
        let level_1_b = get_current_scene().create_entity_with_parent(level_0.handle);
        let level_2_d = get_current_scene().create_entity_with_parent(level_1_b.handle);
        // check scene is referring to root entity as the root entity
        assert_eq!(
            get_current_scene().root_entity_runtime_id.unwrap(),
            level_0.get_runtime_id()
        );
        // check hierarchy for root entity
        assert_eq!(
            level_0.get_component::<Hierarchy>().unwrap(),
            Hierarchy {
                num_children: 2,
                parent_runtime_id: 0,
                first_child_runtime_id: level_1_a.handle,
                prev_sibling_runtime_id: 0,
                next_sibling_runtime_id: 0,
            }
        );
        // check hierarchy for level_1_a entity
        assert_eq!(
            level_1_a.get_component::<Hierarchy>().unwrap(),
            Hierarchy {
                num_children: 3,
                parent_runtime_id: level_0.handle,
                first_child_runtime_id: level_2_a.handle,
                prev_sibling_runtime_id: 0,
                next_sibling_runtime_id: level_1_b.handle,
            }
        );
        // check hierarchy for level_1_b entity
        assert_eq!(
            level_1_b.get_component::<Hierarchy>().unwrap(),
            Hierarchy {
                num_children: 1,
                parent_runtime_id: level_0.handle,
                first_child_runtime_id: level_2_d.handle,
                prev_sibling_runtime_id: level_1_a.handle,
                next_sibling_runtime_id: 0,
            }
        );
        // check hierarchy for level_2_a entity
        assert_eq!(
            level_2_a.get_component::<Hierarchy>().unwrap(),
            Hierarchy {
                num_children: 0,
                parent_runtime_id: level_1_a.handle,
                first_child_runtime_id: 0,
                prev_sibling_runtime_id: 0,
                next_sibling_runtime_id: level_2_b.handle,
            }
        );
        // check hierarchy for level_2_b entity
        assert_eq!(
            level_2_b.get_component::<Hierarchy>().unwrap(),
            Hierarchy {
                num_children: 0,
                parent_runtime_id: level_1_a.handle,
                first_child_runtime_id: 0,
                prev_sibling_runtime_id: level_2_a.handle,
                next_sibling_runtime_id: level_2_c.handle,
            }
        );
        // check hierarchy for level_2_c entity
        assert_eq!(
            level_2_c.get_component::<Hierarchy>().unwrap(),
            Hierarchy {
                num_children: 0,
                parent_runtime_id: level_1_a.handle,
                first_child_runtime_id: 0,
                prev_sibling_runtime_id: level_2_b.handle,
                next_sibling_runtime_id: 0,
            }
        );
        // check hierarchy for level_2_d entity
        assert_eq!(
            level_2_d.get_component::<Hierarchy>().unwrap(),
            Hierarchy {
                num_children: 0,
                parent_runtime_id: level_1_b.handle,
                first_child_runtime_id: 0,
                prev_sibling_runtime_id: 0,
                next_sibling_runtime_id: 0,
            }
        );
    }

    // TODO: test removing entities
}
