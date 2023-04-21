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

use dream_ecs::component::Transform;
use dream_ecs::component_system::ComponentSystem;
use dream_ecs::entity::Entity;
use dream_ecs::scene::get_current_scene;
use dream_renderer::RendererWgpu;

use crate::javascript_script_component_system::JavaScriptScriptComponentSystem;
use crate::python_script_component_system::PythonScriptComponentSystem;

pub struct App {
    should_init: bool,
    pub dt: f32,
    pub component_systems: Vec<Box<dyn ComponentSystem>>,
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
        let mut e: Option<Entity> = None;
        {
            let mut scene = get_current_scene();
            e = Some(scene.create_entity());
        }
        {
            e.unwrap()
                .add_component(Transform::from(dream_math::Vector3::from(2., 1., 1.)));
        }
        // init component systems
        self.component_systems
            .push(Box::new(JavaScriptScriptComponentSystem::new()) as Box<dyn ComponentSystem>);
        self.component_systems
            .push(Box::new(PythonScriptComponentSystem::new()) as Box<dyn ComponentSystem>);
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
        // TODO: implement this (look at email for details)
        // todo!();
        renderer.draw_mesh("dummy_guid", 0); // TODO: also pass in transform matrix after testing this with other models
    }
}
