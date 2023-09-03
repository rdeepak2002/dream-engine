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

use std::sync::{Mutex, Weak};

use anyhow::{anyhow, Result};
use shipyard::{IntoIter, IntoWithId};

use crate::component::{Hierarchy, Transform};
use crate::entity::Entity;

// pub(crate) static SCENE: Lazy<Mutex<Scene>> = Lazy::new(|| Mutex::new(Scene::default()));

pub struct Scene {
    pub name: &'static str,
    pub root_entity_runtime_id: Option<u64>,
    pub handle: shipyard::World,
}

impl Default for Scene {
    fn default() -> Scene {
        Self {
            name: "scene",
            handle: shipyard::World::new(),
            root_entity_runtime_id: None,
        }
    }
}

pub fn get_children_for_entity(scene: Weak<Mutex<Scene>>, entity_id: u64) -> Vec<u64> {
    let entity = Entity::from_handle(entity_id, scene.clone());
    let hierarchy_component: Option<Hierarchy> = entity.get_component();
    let mut result = Vec::new();
    if let Some(hierarchy_component) = hierarchy_component {
        let mut cur_entity_id = hierarchy_component.first_child_runtime_id;
        while let Some(cur_entity_id_unwrapped) = cur_entity_id {
            result.push(cur_entity_id_unwrapped);
            let entity = Entity::from_handle(cur_entity_id_unwrapped, scene.clone());
            let hierarchy_component: Option<Hierarchy> = entity.get_component();
            if let Some(hierarchy_component) = hierarchy_component {
                cur_entity_id = hierarchy_component.next_sibling_runtime_id;
            } else {
                cur_entity_id = None;
            }
        }
    }
    result
}

pub fn add_child_to_entity(scene: Weak<Mutex<Scene>>, child_entity_id: u64, parent_entity_id: u64) {
    let child_entity = Entity::from_handle(child_entity_id, scene.clone());
    let parent_entity = Entity::from_handle(parent_entity_id, scene.clone());

    if child_entity.has_component::<Hierarchy>() && parent_entity.has_component::<Hierarchy>() {
        let mut parent_hierarchy_component: Hierarchy = parent_entity.get_component().unwrap();
        let mut child_hierarchy_component: Hierarchy = child_entity.get_component().unwrap();

        if child_hierarchy_component.parent_runtime_id.is_some() {
            // TODO: if child already has parent, remove it from that parent (not a full remove cuz children need to move with it)
            // ^ might be best to create a general 'move' method where you move a child from one parent to a different parent
            todo!();
        }

        parent_hierarchy_component.num_children += 1;
        if parent_hierarchy_component.first_child_runtime_id.is_none() {
            // set child as first child of parent
            parent_hierarchy_component.first_child_runtime_id = Some(child_entity_id);
        } else {
            // insert entity to front of children list
            // set first child of parent to new child
            let former_first_child = parent_hierarchy_component.first_child_runtime_id;
            parent_hierarchy_component.first_child_runtime_id = Some(child_entity_id);
            // set child hierarchy component next to former first child
            child_hierarchy_component.next_sibling_runtime_id = former_first_child;
            // set former first child's previous to this child
            if let Some(former_first_child_entity_id) = former_first_child {
                let former_first_child_entity =
                    Entity::from_handle(former_first_child_entity_id, scene);
                if former_first_child_entity.has_component::<Hierarchy>() {
                    let mut former_first_child_hierarchy_component: Hierarchy =
                        former_first_child_entity.get_component().unwrap();
                    former_first_child_hierarchy_component.prev_sibling_runtime_id =
                        Some(child_entity_id);
                    former_first_child_entity.add_component(former_first_child_hierarchy_component);
                }
            }
        }
        child_hierarchy_component.parent_runtime_id = Some(parent_entity_id);
        parent_entity.add_component(parent_hierarchy_component);
        child_entity.add_component(child_hierarchy_component);
    }
}

pub fn create_entity(scene: Weak<Mutex<Scene>>, parent_id: Option<u64>) -> Result<u64> {
    let scene_mutex = scene
        .upgrade()
        .ok_or_else(|| anyhow!("Unable to upgrade scene weak reference when creating entity"))?;
    let mut scene_mutex_lock = scene_mutex
        .lock()
        .map_err(|_| anyhow!("Unable to acquire scene mutex when creating entity"))
        .unwrap();
    // add root entity if it does not exist
    if scene_mutex_lock.root_entity_runtime_id.is_none() {
        let new_root_entity = scene_mutex_lock
            .handle
            .add_entity((Transform::default(), Hierarchy::default()))
            .inner();
        scene_mutex_lock.root_entity_runtime_id = Some(new_root_entity);
    }
    // create new entity and make it child of the root
    let new_entity_id = scene_mutex_lock
        .handle
        .add_entity((Transform::default(), Hierarchy::default()))
        .inner();
    let root_id = scene_mutex_lock.root_entity_runtime_id.unwrap();
    // drop mutex lock to allow other threads to modify scene
    drop(scene_mutex_lock);
    add_child_to_entity(scene, new_entity_id, parent_id.unwrap_or(root_id));
    Ok(new_entity_id)
}

impl Scene {
    pub fn get_entities_with_component<T: shipyard::Component + Send + Sync + Clone>(
        &self,
    ) -> Vec<u64> {
        let mut entity_id_vec = Vec::new();
        self.handle.run(|vm_transform: shipyard::ViewMut<T>| {
            for t in vm_transform.iter().with_id() {
                let entity_id = t.0;
                entity_id_vec.push(entity_id);
            }
        });
        let mut entity_vec = Vec::new();
        for entity_id in &entity_id_vec {
            entity_vec.push(entity_id.inner());
        }
        entity_vec
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::component::Hierarchy;
//     use crate::scene::Scene;
//
//     #[test]
//     /// Test adding and removing entities and verifying the hierarchy is correct
//     fn test_hierarchy_empty() {
//         let scene = Scene::new();
//         assert_eq!(scene.root_entity_runtime_id, None);
//     }
//
//     #[test]
//     /// Test adding and removing entities and verifying the hierarchy is correct
//     fn test_hierarchy_one_level() {
//         let root_entity = get_current_scene().create_entity();
//         let root_entity_child_1 = get_current_scene().create_entity();
//         let root_entity_child_2 = get_current_scene().create_entity();
//         let root_entity_child_3 = get_current_scene().create_entity();
//         // check scene is referring to root entity as the root entity
//         assert_eq!(
//             get_current_scene().root_entity_runtime_id.unwrap(),
//             root_entity.get_runtime_id()
//         );
//         // check hierarchy for root entity
//         assert_eq!(
//             root_entity.get_component::<Hierarchy>().unwrap(),
//             Hierarchy {
//                 num_children: 3,
//                 parent_runtime_id: 0,
//                 first_child_runtime_id: root_entity_child_1.handle,
//                 prev_sibling_runtime_id: 0,
//                 next_sibling_runtime_id: 0,
//             }
//         );
//         // check hierarchy for root_entity_child_1
//         assert_eq!(
//             root_entity_child_1.get_component::<Hierarchy>().unwrap(),
//             Hierarchy {
//                 num_children: 0,
//                 parent_runtime_id: root_entity.handle,
//                 first_child_runtime_id: 0,
//                 prev_sibling_runtime_id: 0,
//                 next_sibling_runtime_id: root_entity_child_2.handle,
//             }
//         );
//         // check hierarchy for root_entity_child_2
//         assert_eq!(
//             root_entity_child_2.get_component::<Hierarchy>().unwrap(),
//             Hierarchy {
//                 num_children: 0,
//                 parent_runtime_id: root_entity.handle,
//                 first_child_runtime_id: 0,
//                 prev_sibling_runtime_id: root_entity_child_1.handle,
//                 next_sibling_runtime_id: root_entity_child_3.handle,
//             }
//         );
//         // check hierarchy for root_entity_child_3
//         assert_eq!(
//             root_entity_child_3.get_component::<Hierarchy>().unwrap(),
//             Hierarchy {
//                 num_children: 0,
//                 parent_runtime_id: root_entity.handle,
//                 first_child_runtime_id: 0,
//                 prev_sibling_runtime_id: root_entity_child_2.handle,
//                 next_sibling_runtime_id: 0,
//             }
//         );
//     }
//
//     // #[test]
//     // Test adding and removing entities and verifying the hierarchy is correct
//     // fn test_hierarchy_three_levels() {
//     //     let level_0 = get_current_scene().create_entity();
//     //     let level_1_a = get_current_scene().create_entity_with_parent(level_0.handle);
//     //     let level_2_a = get_current_scene().create_entity_with_parent(level_1_a.handle);
//     //     let level_2_b = get_current_scene().create_entity_with_parent(level_1_a.handle);
//     //     let level_2_c = get_current_scene().create_entity_with_parent(level_1_a.handle);
//     //     let level_1_b = get_current_scene().create_entity_with_parent(level_0.handle);
//     //     let level_2_d = get_current_scene().create_entity_with_parent(level_1_b.handle);
//     //     // check scene is referring to root entity as the root entity
//     //     assert_eq!(
//     //         get_current_scene().root_entity_runtime_id.unwrap(),
//     //         level_0.get_runtime_id()
//     //     );
//     //     // check hierarchy for root entity
//     //     assert_eq!(
//     //         level_0.get_component::<Hierarchy>().unwrap(),
//     //         Hierarchy {
//     //             num_children: 2,
//     //             parent_runtime_id: 0,
//     //             first_child_runtime_id: level_1_a.handle,
//     //             prev_sibling_runtime_id: 0,
//     //             next_sibling_runtime_id: 0,
//     //         }
//     //     );
//     //     // check hierarchy for level_1_a entity
//     //     assert_eq!(
//     //         level_1_a.get_component::<Hierarchy>().unwrap(),
//     //         Hierarchy {
//     //             num_children: 3,
//     //             parent_runtime_id: level_0.handle,
//     //             first_child_runtime_id: level_2_a.handle,
//     //             prev_sibling_runtime_id: 0,
//     //             next_sibling_runtime_id: level_1_b.handle,
//     //         }
//     //     );
//     //     // check hierarchy for level_1_b entity
//     //     assert_eq!(
//     //         level_1_b.get_component::<Hierarchy>().unwrap(),
//     //         Hierarchy {
//     //             num_children: 1,
//     //             parent_runtime_id: level_0.handle,
//     //             first_child_runtime_id: level_2_d.handle,
//     //             prev_sibling_runtime_id: level_1_a.handle,
//     //             next_sibling_runtime_id: 0,
//     //         }
//     //     );
//     //     // check hierarchy for level_2_a entity
//     //     assert_eq!(
//     //         level_2_a.get_component::<Hierarchy>().unwrap(),
//     //         Hierarchy {
//     //             num_children: 0,
//     //             parent_runtime_id: level_1_a.handle,
//     //             first_child_runtime_id: 0,
//     //             prev_sibling_runtime_id: 0,
//     //             next_sibling_runtime_id: level_2_b.handle,
//     //         }
//     //     );
//     //     // check hierarchy for level_2_b entity
//     //     assert_eq!(
//     //         level_2_b.get_component::<Hierarchy>().unwrap(),
//     //         Hierarchy {
//     //             num_children: 0,
//     //             parent_runtime_id: level_1_a.handle,
//     //             first_child_runtime_id: 0,
//     //             prev_sibling_runtime_id: level_2_a.handle,
//     //             next_sibling_runtime_id: level_2_c.handle,
//     //         }
//     //     );
//     //     // check hierarchy for level_2_c entity
//     //     assert_eq!(
//     //         level_2_c.get_component::<Hierarchy>().unwrap(),
//     //         Hierarchy {
//     //             num_children: 0,
//     //             parent_runtime_id: level_1_a.handle,
//     //             first_child_runtime_id: 0,
//     //             prev_sibling_runtime_id: level_2_b.handle,
//     //             next_sibling_runtime_id: 0,
//     //         }
//     //     );
//     //     // check hierarchy for level_2_d entity
//     //     assert_eq!(
//     //         level_2_d.get_component::<Hierarchy>().unwrap(),
//     //         Hierarchy {
//     //             num_children: 0,
//     //             parent_runtime_id: level_1_b.handle,
//     //             first_child_runtime_id: 0,
//     //             prev_sibling_runtime_id: 0,
//     //             next_sibling_runtime_id: 0,
//     //         }
//     //     );
//     // }
//
//     // TODO: test removing entities
// }
