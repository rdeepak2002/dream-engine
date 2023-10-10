use std::sync::{Mutex, Weak};

use winit::event::VirtualKeyCode;

use dream_ecs::component::Transform;
use dream_ecs::entity::Entity;
use dream_ecs::scene::Scene;
use dream_math::{UnitQuaternion, Vector3};

use crate::input::get_keyboard_state;
use crate::system::System;

#[derive(Default)]
pub struct SceneCameraComponentSystem {}

impl System for SceneCameraComponentSystem {
    fn update(&mut self, dt: f32, scene: Weak<Mutex<Scene>>) {
        let scene_camera_entities = scene
            .upgrade()
            .expect("Unable to upgrade")
            .lock()
            .expect("Unable to lock")
            .get_entities_with_component::<dream_ecs::component::SceneCamera>();
        for entity_id in scene_camera_entities {
            let entity = Entity::from_handle(entity_id, scene.clone());
            if let Some(mut transform) = entity.get_component::<Transform>() {
                let rotation = UnitQuaternion::from_quaternion(transform.rotation);
                let forward_vector =
                    rotation.transform_vector(&Vector3::<f32>::new(0.0, 0.0, -1.0));
                let forward_vector_no_y = Vector3::new(forward_vector.x, 0.0, forward_vector.z);
                let up_vector = Vector3::new(0.0, 1.0, 0.0);
                let right_vector = up_vector.cross(&forward_vector);

                let mut delta_position = Vector3::<f32>::default();
                if get_keyboard_state(VirtualKeyCode::W) == 1.0 {
                    delta_position += forward_vector_no_y;
                }
                if get_keyboard_state(VirtualKeyCode::S) == 1.0 {
                    delta_position -= forward_vector_no_y;
                }
                if get_keyboard_state(VirtualKeyCode::A) == 1.0 {
                    delta_position += right_vector;
                }
                if get_keyboard_state(VirtualKeyCode::D) == 1.0 {
                    delta_position -= right_vector;
                }
                if get_keyboard_state(VirtualKeyCode::Space) == 1.0 {
                    delta_position += up_vector;
                }
                if get_keyboard_state(VirtualKeyCode::LShift) == 1.0 {
                    delta_position -= up_vector;
                }
                transform.position += delta_position * dt;
                entity.add_component(transform);
            }
        }
    }
}
