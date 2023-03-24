use boa_engine::class::{Class, ClassBuilder};
use boa_engine::{builtins::JsArgs, Context, JsResult, JsValue};
use boa_gc::{Finalize, Trace};
use shipyard::Get;

use crate::component::Transform;
use crate::scene::Scene;

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

#[derive(Debug, Trace, Finalize)]
pub struct EntityJS {
    name: String,
    age: u8,
}
