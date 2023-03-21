use shipyard::Get;

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

    pub fn from(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn to_string(&self) -> String {
        format!("Transform({}, {}, {})", self.x, self.y, self.z)
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
                println!("Deleting entity {}", id.index());
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
        let handle = scene.handle.add_entity(Transform::new());
        Self { scene, handle }
    }

    pub fn to_string(&self) -> String {
        let trans = self.get_transform();
        if trans.is_some() {
            format!("Entity({})", trans.unwrap().to_string())
        } else {
            format!("Entity()")
        }
    }

    pub fn add_transform(&self, transform: Transform) {
        // reason for unsafe: using raw pointer to scene is fine since removal of a scene should delete all entities from world
        unsafe {
            (*self.scene).handle.add_component(self.handle, transform);
        }
    }

    pub fn get_transform(&self) -> Option<Transform> {
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

    pub fn has_transform(&self) -> bool {
        self.get_transform().is_none()
    }
}
