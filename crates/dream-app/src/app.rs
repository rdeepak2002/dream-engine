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
use std::sync::{Arc, Mutex};

use cgmath::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

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
        let dummy_entity = Scene::create_entity(
            Arc::downgrade(&scene),
            Default::default(),
            Default::default(),
        )
        .expect("Unable to create dummy entity");
        let _dummy_entity_child = Scene::create_entity(
            Arc::downgrade(&scene),
            Default::default(),
            Some(dummy_entity),
        )
        .expect("Unable to create dummy entity");
        let entity_handle = Scene::create_entity(
            Arc::downgrade(&scene),
            Default::default(),
            Default::default(),
        )
        .expect("Unable to create entity");
        Entity::from_handle(entity_handle, Arc::downgrade(&scene))
            .add_component(Transform::from(dream_math::Vector3::new(1.0, -4.8, -6.0)));
        // add mesh renderer component
        MeshRenderer::add_to_entity(
            Arc::downgrade(&scene),
            entity_handle,
            &resource_manager,
            "bbdd8f66-c1ad-4ef8-b128-20b6b91d8f13".into(),
            true,
        );
        // add python script component
        PythonScript::add_to_entity(
            Arc::downgrade(&scene),
            entity_handle,
            &resource_manager,
            "c33a13c0-b9a9-4eef-b1b0-40ca8f41111a".into(),
        );

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
        // TODO: traverse in tree fashion
        renderer.clear();
        let transform_entities = self
            .scene
            .lock()
            .expect("Unable to lock scene")
            .get_entities_with_component::<Transform>();
        for entity_id in transform_entities {
            if let Some(transform) = Entity::from_handle(entity_id, Arc::downgrade(&self.scene))
                .get_component::<Transform>()
            {
                if let Some(mesh_renderer) =
                    Entity::from_handle(entity_id, Arc::downgrade(&self.scene))
                        .get_component::<MeshRenderer>()
                {
                    if let Some(resource_handle) = mesh_renderer.resource_handle {
                        let upgraded_resource_handle = resource_handle
                            .upgrade()
                            .expect("Unable to upgrade resource handle");
                        let resource_key = &upgraded_resource_handle.key;

                        if renderer.is_model_stored(resource_key.as_str()) {
                            // TODO: instead of fixed indices, the sub entities should contain the index
                            for i in 0..2 {
                                renderer.draw_mesh(
                                    resource_key.as_str(),
                                    i,
                                    Instance {
                                        position: cgmath::Vector3::from(transform.position),
                                        rotation: cgmath::Quaternion::from_axis_angle(
                                            cgmath::Vector3::new(0., 0., 1.),
                                            cgmath::Deg(0.0),
                                        ) * cgmath::Quaternion::from_axis_angle(
                                            cgmath::Vector3::new(0., 1., 0.),
                                            cgmath::Deg(-0.0),
                                        ) * cgmath::Quaternion::from_axis_angle(
                                            cgmath::Vector3::new(1., 0., 0.),
                                            cgmath::Deg(-90.0),
                                        ),
                                        scale: cgmath::Vector3::new(0.025, 0.025, 0.025),
                                    },
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
        }
    }
}
