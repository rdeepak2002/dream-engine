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

use cgmath::prelude::*;
use cgmath::{Matrix4, Quaternion, Vector3};

use dream_ecs::component::{MeshRenderer, PythonScript, Transform};
use dream_ecs::entity::Entity;
use dream_ecs::scene::Scene;
use dream_renderer::instance::Instance;
use dream_renderer::RendererWgpu;
use dream_resource::resource_manager::ResourceManager;
#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_rayon::init_thread_pool;

use crate::python_script_component_system::PythonScriptComponentSystem;
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
        {
            let cube_entity_handle =
                Scene::create_entity(Arc::downgrade(&scene), Some("Cube".into()), None, None)
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
        }
        {
            let entity_handle =
                Scene::create_entity(Arc::downgrade(&scene), Some("Robot".into()), None, None)
                    .expect("Unable to create entity");
            // add mesh renderer component
            MeshRenderer::add_to_entity(
                Arc::downgrade(&scene),
                entity_handle,
                &resource_manager,
                "8efa6863-27d2-43ba-b814-ee8b60d12a9b".into(), // berserk armor: "7a71a1a6-a2ef-4e84-ad5d-4e3409d5ea87"d
                true,
                Default::default(),
            );
            // add python script component
            PythonScript::add_to_entity(
                Arc::downgrade(&scene),
                entity_handle,
                &resource_manager,
                "c33a13c0-b9a9-4eef-b1b0-40ca8f41111a".into(),
            );
        }

        // init component systems
        let component_systems =
            vec![Arc::new(Mutex::new(PythonScriptComponentSystem::default()))
                as Arc<Mutex<dyn System>>];

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
            let root_entity = Entity::from_handle(root_entity_id, scene_weak_ref.clone());
            if let Some(transform) = root_entity.get_component::<Transform>() {
                mat = Matrix4::from_translation(Vector3::from(transform.position))
                    * Matrix4::from_nonuniform_scale(
                        transform.scale.x,
                        transform.scale.y,
                        transform.scale.z,
                    )
                    * Matrix4::from(Quaternion::from(transform.rotation));
            }
            let children_ids =
                Scene::get_children_for_entity(scene_weak_ref.clone(), root_entity_id);
            for child_id in children_ids {
                draw_entity_and_children(renderer, child_id, scene_weak_ref.clone(), mat);
            }
        }

        // draw and entity and its children
        fn draw_entity_and_children(
            renderer: &mut RendererWgpu,
            entity_id: u64,
            scene: Weak<Mutex<Scene>>,
            parent_mat: Matrix4<f32>,
        ) {
            let entity = Entity::from_handle(entity_id, scene.clone());
            let mut mat = Matrix4::identity();

            if let Some(transform) = entity.get_component::<Transform>() {
                // TODO: create cache of mat4 that is map of maps
                // so basically to invalidate caches for all children
                // all we have to do is remove the element from the map
                let position = Vector3::from(transform.position);
                let rotation = Quaternion::from(transform.rotation);
                let scale = Vector3::from(transform.scale);
                mat = Matrix4::from_translation(position)
                    * Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z)
                    * Matrix4::from(rotation);
                mat = parent_mat * mat;
                if let Some(mesh_renderer) = entity.get_component::<MeshRenderer>() {
                    if let Some(resource_handle) = mesh_renderer.resource_handle {
                        let upgraded_resource_handle = resource_handle
                            .upgrade()
                            .expect("Unable to upgrade resource handle");
                        let resource_key = &upgraded_resource_handle.key;

                        if renderer.is_model_stored(resource_key.as_str()) {
                            if let Some(mesh_idx) = mesh_renderer.mesh_idx {
                                renderer.draw_mesh(
                                    resource_key.as_str(),
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
            }

            let children_ids = Scene::get_children_for_entity(scene.clone(), entity_id);
            for child_id in children_ids {
                draw_entity_and_children(renderer, child_id, scene.clone(), mat);
            }
        }
    }
}
