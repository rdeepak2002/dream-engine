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
