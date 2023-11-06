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
use std::sync::{Arc, Mutex, Weak};

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseScrollDelta, VirtualKeyCode};

use dream_ecs::component::{Bone, Light, LightType, MeshRenderer, SceneCamera, Transform};
use dream_ecs::entity::Entity;
use dream_ecs::scene::Scene;
use dream_math::{Matrix4, Quaternion, UnitQuaternion, Vector2, Vector3};
use dream_renderer::instance::Instance;
use dream_renderer::renderer::RendererWgpu;
use dream_resource::resource_manager::ResourceManager;
#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_rayon::init_thread_pool;

use crate::input::{
    get_mouse_move, get_mouse_scroll, set_keyboard_state, set_mouse_left_pressed, set_mouse_move,
    set_mouse_right_pressed, set_mouse_scroll,
};
use crate::python_script_component_system::PythonScriptComponentSystem;
use crate::scene_camera_component_system::SceneCameraComponentSystem;
use crate::system::System;

pub struct App {
    pub dt: f32,
    pub component_systems: Vec<Arc<Mutex<dyn System>>>,
    pub resource_manager: ResourceManager,
    pub scene: Arc<Mutex<Scene>>,
}

impl Default for App {
    fn default() -> App {
        let resource_manager = ResourceManager::default();
        let scene = Scene::create();

        // populate scene
        // let entity_handle = scene.lock().expect("Unable to lock scene").create_entity();
        let dummy_entity =
            Scene::create_entity(Arc::downgrade(&scene), Default::default(), None, None)
                .expect("Unable to create dummy entity");
        let _dummy_entity_child = Scene::create_entity(
            Arc::downgrade(&scene),
            Default::default(),
            Some(dummy_entity),
            None,
        )
        .expect("Unable to create dummy entity");
        // cube
        // {
        //     let cube_entity_handle =
        //         Scene::create_entity(Arc::downgrade(&scene), Some("Cube".into()), None, None)
        //             .expect("Unable to create cube entity");
        //     // add mesh renderer component
        //     MeshRenderer::add_to_entity(
        //         Arc::downgrade(&scene),
        //         cube_entity_handle,
        //         &resource_manager,
        //         "2dcd5e2e-714b-473a-bbdd-98771761cb37".into(),
        //         true,
        //         Default::default(),
        //     );
        //     Entity::from_handle(cube_entity_handle, Arc::downgrade(&scene)).add_component(
        //         Transform::new(
        //             Vector3::new(0., -1.1, 0.),
        //             Quaternion::identity(),
        //             Vector3::new(1., 1., 1.),
        //         ),
        //     );
        // }
        {
            let scene_camera_entity_handle = Scene::create_entity(
                Arc::downgrade(&scene),
                Some("Scene Camera".into()),
                None,
                Some(Transform::new(
                    Vector3::new(-1.33, 0.85, 1.228),
                    Quaternion::new(0.793, -0.0673, -0.603, -0.055),
                    Vector3::new(1.0, 1.0, 1.0),
                    // Vector3::new(0.7, 1.3, 4.4),
                    // Quaternion::new(0.981, -0.196, 0.0, 0.0),
                    // Vector3::new(1.0, 1.0, 1.0),
                )),
            )
            .expect("Unable to create scene camera entity");
            let e = Entity::from_handle(scene_camera_entity_handle, Arc::downgrade(&scene));
            e.add_component(SceneCamera::default());
        }
        {
            let directional_light_entity_handle =
                Scene::create_entity(Arc::downgrade(&scene), Some("Sun".into()), None, None)
                    .expect("Unable to create sun entity");
            let intensity = 5.0;
            Entity::from_handle(directional_light_entity_handle, Arc::downgrade(&scene))
                .add_component(Light::new(
                    LightType::DIRECTIONAL,
                    Vector3::new(
                        intensity * 255.0 / 255.0,
                        intensity * 255.0 / 255.0,
                        intensity * 251.0 / 255.0,
                    ),
                    4.0,
                    Vector3::new(-0.2, -0.4, -0.1),
                    true,
                ));
            // add cube for position visualization (initial shadow map debugging)
            // MeshRenderer::add_to_entity(
            //     Arc::downgrade(&scene),
            //     directional_light_entity_handle,
            //     &resource_manager,
            //     "2dcd5e2e-714b-473a-bbdd-98771761cb37".into(),
            //     true,
            //     Default::default(),
            // );
            Entity::from_handle(directional_light_entity_handle, Arc::downgrade(&scene))
                .add_component(Transform::new(
                    Vector3::new(4.3, 4.2, 2.3),
                    Quaternion::identity(),
                    Vector3::new(0.1, 0.1, 0.1),
                ));
        }
        {
            let cube_entity_handle =
                Scene::create_entity(Arc::downgrade(&scene), Some("2x Cube".into()), None, None)
                    .expect("Unable to create cube entity");
            // add mesh renderer component
            MeshRenderer::add_to_entity(
                Arc::downgrade(&scene),
                cube_entity_handle,
                &resource_manager,
                "2dcd5e2e-714b-473a-bbdd-98771761cb37".into(),
                true,
                Default::default(),
            );
            Entity::from_handle(cube_entity_handle, Arc::downgrade(&scene)).add_component(
                Transform::new(
                    Vector3::new(7.4, 2.1, 10.6),
                    Quaternion::new(1.0, 0.0, 0.6, 0.0),
                    Vector3::new(2.0, 2.0, 2.0),
                ),
            );
        }
        {
            let cube_entity_handle = Scene::create_entity(
                Arc::downgrade(&scene),
                Some("Floating Rectangle".into()),
                None,
                None,
            )
            .expect("Unable to create cube entity");
            // add mesh renderer component
            MeshRenderer::add_to_entity(
                Arc::downgrade(&scene),
                cube_entity_handle,
                &resource_manager,
                "2dcd5e2e-714b-473a-bbdd-98771761cb37".into(),
                true,
                Default::default(),
            );
            Entity::from_handle(cube_entity_handle, Arc::downgrade(&scene)).add_component(
                Transform::new(
                    Vector3::new(40.0, 10.0, 0.0),
                    Quaternion::new(1.0, 0.0, 0.0, 0.0),
                    Vector3::new(10.0, 1.0, 10.0),
                ),
            );
        }
        {
            let cube_entity_handle =
                Scene::create_entity(Arc::downgrade(&scene), Some("Floor".into()), None, None)
                    .expect("Unable to create point light entity");
            // add mesh renderer component
            MeshRenderer::add_to_entity(
                Arc::downgrade(&scene),
                cube_entity_handle,
                &resource_manager,
                "2dcd5e2e-714b-473a-bbdd-98771761cb37".into(),
                true,
                Default::default(),
            );
            Entity::from_handle(cube_entity_handle, Arc::downgrade(&scene)).add_component(
                Transform::new(
                    Vector3::new(0.0, 0.0, 0.0),
                    Quaternion::new(1.0, 0.0, 0.0, 0.0),
                    Vector3::new(100.0, 0.1, 100.0),
                ),
            );
        }
        // {
        //     let cube_entity_handle =
        //         Scene::create_entity(Arc::downgrade(&scene), Some("Cube".into()), None, None)
        //             .expect("Unable to create point light entity");
        //     // add mesh renderer component
        //     MeshRenderer::add_to_entity(
        //         Arc::downgrade(&scene),
        //         cube_entity_handle,
        //         &resource_manager,
        //         "2dcd5e2e-714b-473a-bbdd-98771761cb37".into(),
        //         true,
        //         Default::default(),
        //     );
        //     Entity::from_handle(cube_entity_handle, Arc::downgrade(&scene)).add_component(
        //         Transform::new(
        //             Vector3::new(0.0, 1.11, 0.0),
        //             Quaternion::new(1.0, 0.0, 0.0, 0.0),
        //             Vector3::new(1.0, 1.0, 1.0),
        //         ),
        //     );
        // }
        // {
        //     let cube_entity_handle = Scene::create_entity(
        //         Arc::downgrade(&scene),
        //         Some("Point Light".into()),
        //         None,
        //         None,
        //     )
        //     .expect("Unable to create point light entity");
        //     Entity::from_handle(cube_entity_handle, Arc::downgrade(&scene)).add_component(
        //         Light::new(
        //             LightType::POINT,
        //             Vector3::new(1.0, 0.76, 0.45),
        //             20.0,
        //             Vector3::new(1.0, 0.0, 0.0),
        //             false,
        //         ),
        //     );
        //     // add mesh renderer component
        //     MeshRenderer::add_to_entity(
        //         Arc::downgrade(&scene),
        //         cube_entity_handle,
        //         &resource_manager,
        //         "4d19dccc-e27a-4abb-9e34-e44d73d17997".into(),
        //         true,
        //         Default::default(),
        //     );
        //     Entity::from_handle(cube_entity_handle, Arc::downgrade(&scene)).add_component(
        //         Transform::new(
        //             Vector3::new(-0.6, -0.1, 1.3),
        //             Quaternion::new(1.0, -1.0, 0.0, 0.0),
        //             Vector3::new(0.15, 0.15, 0.15),
        //         ),
        //     );
        // }
        {
            let entity_handle =
                Scene::create_entity(Arc::downgrade(&scene), Some("Guts".into()), None, None)
                    .expect("Unable to create entity");
            // add mesh renderer component
            MeshRenderer::add_to_entity(
                Arc::downgrade(&scene),
                entity_handle,
                &resource_manager,
                "f358ffb3-b766-4839-a93f-30b81ff9c107".into(), // 1k: 7a71a1a6-a2ef-4e84-ad5d-4e3409d5ea87 ; 4k: f358ffb3-b766-4839-a93f-30b81ff9c107
                true,
                Default::default(),
            );
            Entity::from_handle(entity_handle, Arc::downgrade(&scene)).add_component(
                Transform::new(
                    Vector3::new(0.7, 0.195, 1.0),
                    Quaternion::new(1.0, 0.2, 8.5, 7.0),
                    Vector3::new(1.0, 1.0, 1.0),
                ),
            );
        }
        // pbr robot
        // {
        //     let entity_handle =
        //         Scene::create_entity(Arc::downgrade(&scene), Some("Robot".into()), None, None)
        //             .expect("Unable to create entity");
        //     // add mesh renderer component
        //     MeshRenderer::add_to_entity(
        //         Arc::downgrade(&scene),
        //         entity_handle,
        //         &resource_manager,
        //         "8efa6863-27d2-43ba-b814-ee8b60d12a9b".into(),
        //         true,
        //         Default::default(),
        //     );
        //     // add python script component
        //     PythonScript::add_to_entity(
        //         Arc::downgrade(&scene),
        //         entity_handle,
        //         &resource_manager,
        //         "c33a13c0-b9a9-4eef-b1b0-40ca8f41111a".into(),
        //     );
        // }

        // init component systems
        let component_systems = vec![
            Arc::new(Mutex::new(PythonScriptComponentSystem::default())) as Arc<Mutex<dyn System>>,
            Arc::new(Mutex::new(SceneCameraComponentSystem::default())) as Arc<Mutex<dyn System>>,
        ];

        Self {
            dt: 0.0,
            component_systems,
            resource_manager,
            scene,
        }
    }
}

impl App {
    pub fn update(&mut self) -> f32 {
        self.dt = 1.0 / 60.0;
        for i in 0..self.component_systems.len() {
            self.component_systems[i]
                .lock()
                .unwrap()
                .update(self.dt, Arc::downgrade(&self.scene));
        }
        // mouse dx and dy should be reset
        set_mouse_move(get_mouse_move() * 0.0);
        set_mouse_scroll(get_mouse_scroll() * 0.0);
        self.dt
    }

    pub async fn update_async(&mut self) {}

    pub fn draw(&mut self, renderer: &mut RendererWgpu) {
        renderer.clear();
        let scene_weak_ref = Arc::downgrade(&self.scene);
        let root_entity_id: Option<u64> = self
            .scene
            .lock()
            .expect("Unable to acquire lock on scene")
            .root_entity_runtime_id;
        // get children for root entity and render them
        if let Some(root_entity_id) = root_entity_id {
            let mut mat: Matrix4<f32> = Matrix4::identity();
            let mat_from_root_bone: Matrix4<f32> = Matrix4::identity();
            let root_entity = Entity::from_handle(root_entity_id, scene_weak_ref.clone());
            if let Some(transform) = root_entity.get_component::<Transform>() {
                mat = Matrix4::new_translation(&transform.position)
                    * Matrix4::new_nonuniform_scaling(&transform.scale)
                    * UnitQuaternion::from_quaternion(transform.rotation).to_homogeneous();
            }
            let children_ids =
                Scene::get_children_for_entity(scene_weak_ref.clone(), root_entity_id);
            for child_id in children_ids {
                draw_entity_and_children(
                    renderer,
                    child_id,
                    scene_weak_ref.clone(),
                    mat,
                    mat_from_root_bone,
                );
            }
        }

        // draw and entity and its children
        fn draw_entity_and_children(
            renderer: &mut RendererWgpu,
            entity_id: u64,
            scene: Weak<Mutex<Scene>>,
            parent_mat: Matrix4<f32>,
            mat_from_root_bone: Matrix4<f32>,
        ) {
            let entity = Entity::from_handle(entity_id, scene.clone());
            let mut mat = Matrix4::identity();
            let mut new_bone_mat = mat_from_root_bone;

            if let Some(transform) = entity.get_component::<Transform>() {
                // TODO: create cache of mat4 that is map of maps
                // so basically to invalidate caches for all children
                // all we have to do is remove the element from the map
                let position = transform.position;
                let rotation = transform.rotation;
                let scale = transform.scale;
                let model_mat = Matrix4::new_translation(&position)
                    * Matrix4::new_nonuniform_scaling(&scale)
                    * UnitQuaternion::from_quaternion(rotation).to_homogeneous();
                mat = parent_mat * model_mat;
                if let Some(_scene_camera_component) = entity.get_component::<SceneCamera>() {
                    renderer.set_camera(position.into(), rotation);
                }
                if let Some(light_component) = entity.get_component::<Light>() {
                    let position = Vector3::new(mat.m14, mat.m24, mat.m34);
                    renderer.draw_light(
                        light_component.light_type as u32,
                        position,
                        light_component.color,
                        light_component.radius,
                        light_component.direction,
                        light_component.cast_shadow,
                    );
                }
                if let Some(mesh_renderer) = entity.get_component::<MeshRenderer>() {
                    if let Some(resource_handle) = mesh_renderer.resource_handle {
                        let upgraded_resource_handle = resource_handle
                            .upgrade()
                            .expect("Unable to upgrade resource handle");
                        let resource_key = &upgraded_resource_handle.key;
                        let model_guid = resource_key;

                        if renderer.is_model_stored(resource_key.as_str()) {
                            if let Some(mesh_idx) = mesh_renderer.mesh_idx {
                                renderer.draw_mesh(
                                    model_guid.as_str(),
                                    mesh_idx as i32,
                                    Instance { mat },
                                );
                            }
                        } else {
                            let resource_path = &upgraded_resource_handle.path;
                            renderer
                                .store_model(
                                    Some(resource_key.as_str()),
                                    resource_path
                                        .to_str()
                                        .expect("Unable to convert resource path to a string"),
                                )
                                .expect("Unable to store model");
                        }
                    }
                }
                if let Some(bone_component) = entity.get_component::<Bone>() {
                    new_bone_mat *= model_mat;
                    let bone_mat: Matrix4<f32> = new_bone_mat * bone_component.inverse_bind_pose;
                    renderer.set_bone_transform(bone_component.bone_id, bone_mat);
                }
            }

            let children_ids = Scene::get_children_for_entity(scene.clone(), entity_id);
            for child_id in children_ids {
                draw_entity_and_children(renderer, child_id, scene.clone(), mat, new_bone_mat);
            }
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        set_keyboard_state(key, amount);
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        let dx = mouse_dx as f32;
        let dy = mouse_dy as f32;
        set_mouse_move(Vector2::new(dx, dy))
    }

    pub fn process_mouse_left_input(&mut self, is_pressed: bool) {
        set_mouse_left_pressed(is_pressed);
    }

    pub fn process_mouse_right_input(&mut self, is_pressed: bool) {
        set_mouse_right_pressed(is_pressed);
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        let scroll = -match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
        set_mouse_scroll(scroll);
    }
}
