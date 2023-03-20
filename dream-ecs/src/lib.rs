use rapier3d::parry::transformation::utils::transform;
use shipyard::{Get, IntoIter};

#[derive(shipyard::Component, Debug, Clone)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

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
                println!("{}", id.index());
                all_storages.delete_entity(id);
            });
    }
}

pub struct Entity {
    pub scene: *mut Scene,
    pub handle: shipyard::EntityId,
}

impl Entity {
    pub fn new(scene: &mut Scene) -> Self {
        let handle = scene.handle.add_entity((Transform::new()));
        Self { scene, handle }
    }

    pub fn add_transform(self) {
        // reason for unsafe: using raw pointer to scene is fine since removal of a scene should delete all entities from world
        unsafe {
            (*self.scene)
                .handle
                .add_component(self.handle, Transform::new());
        }
    }

    pub fn get_transform(self) -> Option<Transform> {
        let mut transform_opt: Option<Transform> = None;
        // reason for unsafe: using raw pointer to scene is fine since removal of a scene should delete all entities from world
        unsafe {
            (*self.scene)
                .handle
                .run(|vm_pos: shipyard::ViewMut<Transform>| {
                    let transform = vm_pos.get(self.handle).unwrap();
                    transform_opt = Some(transform.clone());
                });
        }
        return transform_opt;
    }

    // pub fn set_transform(self) -> bool {}
}
