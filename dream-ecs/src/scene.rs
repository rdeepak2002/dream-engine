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

use shipyard::{IntoIter, IntoWithId};

use crate::component::Transform;
use crate::entity::Entity;

pub struct Scene {
    pub name: &'static str,
    pub handle: shipyard::World,
}

impl Scene {
    pub fn new() -> Self {
        let name = "scene";
        let handle = shipyard::World::new();
        return Self { name, handle };
    }

    pub fn create_entity(&mut self) -> Entity {
        let entity = Entity::new(self);
        return entity;
    }

    pub fn transform_entities(&mut self) -> Vec<Entity> {
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
            entity_vec.push(Entity::from(self, entity_id.clone()));
        }
        return entity_vec;
    }
}

impl Drop for Scene {
    /// Remove all entities from scene when scene is deleted (this prevents possible memory issues too)
    fn drop(&mut self) {
        self.handle
            .run(|mut all_storages: shipyard::AllStoragesViewMut| {
                let id = all_storages.add_entity(Transform::new());
                println!("Deleting entity {}", id.index());
                all_storages.delete_entity(id);
            });
    }
}