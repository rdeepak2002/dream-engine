use std::sync::{Mutex, Weak};

use dream_ecs::component::Transform;
use dream_ecs::entity::Entity;
use dream_ecs::scene::Scene;
use dream_math::{UnitQuaternion, UnitVector3, Vector3};

use crate::input::{
    get_mouse_move, get_mouse_scroll, is_mouse_left_pressed, is_mouse_right_pressed,
    is_renderer_panel_active,
};
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
        if is_renderer_panel_active() {
            for entity_id in scene_camera_entities {
                let entity = Entity::from_handle(entity_id, scene.clone());
                if let Some(mut transform) = entity.get_component::<Transform>() {
                    let mut rotation = transform.rotation;

                    let forward_vector =
                        rotation.transform_vector(&Vector3::<f32>::new(0.0, 0.0, -1.0));
                    let forward_vector_no_y =
                        Vector3::new(forward_vector.x, 0.0, forward_vector.z).normalize();

                    let up_vector = Vector3::new(0.0, 1.0, 0.0);
                    let up_vector_screen =
                        rotation.transform_vector(&Vector3::<f32>::new(0.0, -1.0, 0.0));

                    let right_vector = up_vector.cross(&forward_vector_no_y).normalize();

                    let mut delta_position = Vector3::<f32>::default();

                    // move camera using WASD
                    // if get_keyboard_state(VirtualKeyCode::W) == 1.0 {
                    //     delta_position += forward_vector_no_y;
                    // }
                    // if get_keyboard_state(VirtualKeyCode::S) == 1.0 {
                    //     delta_position -= forward_vector_no_y;
                    // }
                    // if get_keyboard_state(VirtualKeyCode::A) == 1.0 {
                    //     delta_position += right_vector;
                    // }
                    // if get_keyboard_state(VirtualKeyCode::D) == 1.0 {
                    //     delta_position -= right_vector;
                    // }
                    // if get_keyboard_state(VirtualKeyCode::Space) == 1.0 {
                    //     delta_position += up_vector;
                    // }
                    // if get_keyboard_state(VirtualKeyCode::LShift) == 1.0 {
                    //     delta_position -= up_vector;
                    // }

                    // move camera using scroll
                    delta_position -= forward_vector * get_mouse_scroll();
                    // if get_keyboard_state(VirtualKeyCode::F) == 1.0 {
                    //     delta_position += forward_vector;
                    // }
                    // if get_keyboard_state(VirtualKeyCode::B) == 1.0 {
                    //     delta_position -= forward_vector;
                    // }

                    // move camera using mouse drag
                    if is_mouse_left_pressed() {
                        let mouse_move = get_mouse_move();
                        delta_position += mouse_move.x * right_vector;
                        delta_position -= mouse_move.y * up_vector_screen;
                    }

                    // turn camera using mouse drag
                    if is_mouse_right_pressed() {
                        let mouse_move = get_mouse_move();

                        let x = UnitQuaternion::from_axis_angle(
                            &UnitVector3::new_normalize(up_vector),
                            mouse_move.x * 0.6 * dt,
                        );
                        let around_local_y_rot = x;

                        let y = UnitQuaternion::from_axis_angle(
                            &UnitVector3::new_normalize(right_vector),
                            -mouse_move.y * 0.6 * dt,
                        );
                        let around_local_x_rot = y;

                        transform.rotation =
                            around_local_x_rot * (around_local_y_rot * transform.rotation);
                    }

                    transform.position += delta_position * dt;
                    entity.add_component(transform);
                }
            }
        }
    }
}
