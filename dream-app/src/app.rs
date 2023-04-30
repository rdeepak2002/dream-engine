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
use std::any::Any;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use cgmath::prelude::*;

use dream_ecs::component::Transform;
use dream_ecs::entity::Entity;
use dream_ecs::scene::{get_current_scene, get_current_scene_read_only};
use dream_renderer::RendererWgpu;

use crate::javascript_script_component_system::JavaScriptScriptComponentSystem;
use crate::python_script_component_system::PythonScriptComponentSystem;
use crate::system::System;

pub struct App {
    should_init: bool,
    pub dt: f32,
    pub component_systems: Vec<Box<dyn System>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_init: true,
            dt: 0.0,
            component_systems: Vec::new(),
        }
    }

    fn initialize(&mut self) {
        // init scene
        let mut e1: Option<Entity> = None;
        {
            let mut scene = get_current_scene();
            e1 = Some(scene.create_entity());
        }
        {
            e1.unwrap()
                .add_component(Transform::from(dream_math::Vector3::from(1.0, -4.8, -6.0)));
        }
        // let mut e1: Option<Entity> = None;
        // {
        //     let mut scene = get_current_scene();
        //     e1 = Some(scene.create_entity());
        // }
        // {
        //     e1.unwrap()
        //         .add_component(Transform::from(dream_math::Vector3::from(-1.5, -1.0, -5.)));
        // }
        // let mut e2: Option<Entity> = None;
        // {
        //     let mut scene = get_current_scene();
        //     e2 = Some(scene.create_entity());
        // }
        // {
        //     e2.unwrap()
        //         .add_component(Transform::from(dream_math::Vector3::from(1.0, -4.0, -5.)));
        // }
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
        return self.dt;
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
            for i in 0..18 {
                renderer.draw_mesh(
                    "link",
                    i,
                    dream_renderer::Instance {
                        position: cgmath::Vector3::from(entity_position),
                        rotation: cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::new(1., 0., 0.),
                            cgmath::Deg(0.0),
                        ),
                        scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
                    },
                );
            }
        }
    }
}
