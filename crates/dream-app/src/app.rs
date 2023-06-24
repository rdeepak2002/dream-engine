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
use std::sync::Arc;

use cgmath::prelude::*;

use dream_ecs::component::Transform;
use dream_ecs::entity::Entity;
use dream_ecs::scene::{get_current_scene, get_current_scene_read_only};
use dream_renderer::instance::Instance;
use dream_renderer::RendererWgpu;
use dream_resource::resource_manager::{ResourceHandle, ResourceManager};
use dream_tasks::task_pool::{get_task_pool, start_thread};

use crate::javascript_script_component_system::JavaScriptScriptComponentSystem;
use crate::python_script_component_system::PythonScriptComponentSystem;
use crate::system::System;

pub struct App {
    should_init: bool,
    pub dt: f32,
    pub component_systems: Vec<Box<dyn System>>,
    pub resource_manager: ResourceManager,
}

impl App {
    pub async fn new() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
            } else {
                start_thread(16);
            }
        }
        Self {
            should_init: true,
            dt: 0.0,
            component_systems: Vec::new(),
            resource_manager: ResourceManager::new().await,
        }
    }

    fn initialize(&mut self) {
        // init scene
        let e1;
        {
            let mut scene = get_current_scene();
            e1 = Some(scene.create_entity());
        }
        {
            e1.unwrap()
                .add_component(Transform::from(dream_math::Vector3::from(1.0, -4.8, -6.0)));
        }
        // init component systems
        self.component_systems
            .push(Box::new(JavaScriptScriptComponentSystem::new()) as Box<dyn System>);
        self.component_systems
            .push(Box::new(PythonScriptComponentSystem::new()) as Box<dyn System>);
    }

    pub fn update(&mut self) -> f32 {
        if self.should_init {
            self.initialize();
            self.should_init = false;
        }
        self.dt = 1.0 / 60.0;
        for i in 0..self.component_systems.len() {
            self.component_systems[i].update(self.dt);
        }
        self.dt
    }

    pub fn draw(&mut self, renderer: &mut RendererWgpu) {
        // TODO: traverse in tree fashion
        let transform_entities: Vec<u64>;
        {
            let scene = get_current_scene_read_only();
            transform_entities = scene.transform_entities().clone();
        }
        for entity_id in transform_entities {
            let entity = Entity::from_handle(entity_id);
            let entity_position = entity.get_component::<Transform>().unwrap().position;

            // let original_resource_handle = self
            //     .resource_manager
            //     .get_resource(String::from("8efa6863-27d2-43ba-b814-ee8b60d12a9b"));

            // if original_resource_handle.is_some() {
            //     println!(
            //         "strong count (a): {}",
            //         Arc::strong_count(original_resource_handle.unwrap())
            //     );
            //     let resource_handle = original_resource_handle.clone();
            //     println!(
            //         "strong count (b): {}",
            //         Arc::strong_count(original_resource_handle.unwrap())
            //     );
            // }

            for i in 0..2 {
                renderer.draw_mesh(
                    "robot",
                    i,
                    Instance {
                        position: cgmath::Vector3::from(entity_position),
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

            // for i in 0..18 {
            //     renderer.draw_mesh(
            //         "link",
            //         i,
            //         dream_renderer::Instance {
            //             position: cgmath::Vector3::from(entity_position),
            //             rotation: cgmath::Quaternion::from_axis_angle(
            //                 cgmath::Vector3::new(1., 0., 0.),
            //                 cgmath::Deg(0.0),
            //             ) * cgmath::Quaternion::from_axis_angle(
            //                 cgmath::Vector3::new(0., 1., 0.),
            //                 cgmath::Deg(0.0),
            //             ) * cgmath::Quaternion::from_axis_angle(
            //                 cgmath::Vector3::new(0., 0., 1.),
            //                 cgmath::Deg(0.0),
            //             ),
            //             scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
            //         },
            //     );
            // }

            // for i in 0..6 {
            //     renderer.draw_mesh(
            //         "ice_cube",
            //         i,
            //         dream_renderer::Instance {
            //             position: cgmath::Vector3::from(entity_position),
            //             rotation: cgmath::Quaternion::from_axis_angle(
            //                 cgmath::Vector3::new(1., 0., 0.),
            //                 cgmath::Deg(0.0),
            //             ) * cgmath::Quaternion::from_axis_angle(
            //                 cgmath::Vector3::new(0., 1., 0.),
            //                 cgmath::Deg(0.0),
            //             ) * cgmath::Quaternion::from_axis_angle(
            //                 cgmath::Vector3::new(0., 0., 1.),
            //                 cgmath::Deg(0.0),
            //             ),
            //             scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
            //         },
            //     );
            // }
        }
    }
}
