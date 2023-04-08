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

use std::sync::RwLock;

use once_cell::sync::Lazy;
use shipyard::{IntoIter, IntoWithId};

use crate::component::Transform;
use crate::entity::Entity;

pub static SCENE: Lazy<RwLock<Scene>> = Lazy::new(|| RwLock::new(Scene::new()));

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
        let entity = Entity::new(self);
        entity.attach(self.root_entity_runtime_id);
        if self.root_entity_runtime_id.is_none() {
            self.root_entity_runtime_id = Some(entity.get_runtime_id());
        }
        return entity;
    }

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

impl Drop for Scene {
    /// Remove all entities from scene when scene is deleted (this catches possible memory issues too since Entity struct has unsafe pointer reference)
    fn drop(&mut self) {
        self.handle
            .run(|mut all_storages: shipyard::AllStoragesViewMut| {
                let id = all_storages.add_entity(Transform::new());
                println!("Deleting entity with runtime ID {}", id.inner());
                all_storages.delete_entity(id);
            });
    }
}

#[cfg(test)]
mod tests {
    use crate::scene::Scene;

    #[test]
    /// Test adding and removing entities and verifying the hierarchy is correct
    fn test_hierarchy_empty() {
        let scene = Scene::new();
        assert_eq!(scene.root_entity_runtime_id, None);
    }

    #[test]
    /// Test adding and removing entities and verifying the hierarchy is correct
    fn test_hierarchy_one_level() {
        let mut scene = Scene::new();
        let root_entity = scene.create_entity();
        let root_entity_child_1 = scene.create_entity();
        let root_entity_child_2 = scene.create_entity();
        let root_entity_child_3 = scene.create_entity();
        // ensure root entity is still considered root of scene
        // check scene is referring to root entity as the root
        assert_eq!(
            scene.root_entity_runtime_id.unwrap(),
            root_entity.get_runtime_id()
        );
        // check parent and siblings for root
        assert_eq!(root_entity.get_hierarchy().unwrap().parent_runtime_id, 0);
        assert_eq!(root_entity.get_hierarchy().unwrap().num_children, 3);
        assert_eq!(
            root_entity.get_hierarchy().unwrap().first_child_runtime_id,
            root_entity_child_1.get_runtime_id()
        );
        // TODO: check siblings
        // TODO: check num children
        // check parent and siblings for entity 1
        assert_eq!(
            root_entity_child_1
                .get_hierarchy()
                .unwrap()
                .parent_runtime_id,
            root_entity.get_runtime_id()
        );
        assert_eq!(
            root_entity_child_1
                .get_hierarchy()
                .unwrap()
                .first_child_runtime_id,
            0
        );
        assert_eq!(
            root_entity_child_1
                .get_hierarchy()
                .unwrap()
                .prev_sibling_runtime_id,
            0
        );
        assert_eq!(
            root_entity_child_1
                .get_hierarchy()
                .unwrap()
                .next_sibling_runtime_id,
            root_entity_child_2.get_runtime_id() // TODO: fix this so that when parent adds a second child it adds it to the linked list
        );
        // TODO: check siblings
        // TODO: check num children
        // check parent and siblings for entity 2
        assert_eq!(
            root_entity_child_3
                .get_hierarchy()
                .unwrap()
                .parent_runtime_id,
            root_entity.get_runtime_id()
        );
        // TODO: check siblings
        // TODO: check num children
        // check parent and siblings for entity 3
        assert_eq!(
            root_entity_child_2
                .get_hierarchy()
                .unwrap()
                .parent_runtime_id,
            root_entity.get_runtime_id()
        );
        // TODO: check siblings
        // TODO: check num children
    }

    // TODO: test removal of entities
}
