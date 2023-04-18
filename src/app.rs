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
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use once_cell::sync::Lazy;

use dream_ecs;
use dream_ecs::component::Transform;
use dream_ecs::component_system::ComponentSystem;
use dream_ecs::entity::Entity;
use dream_ecs::scene::get_current_scene;

use crate::javascript_script_component_system::JavaScriptScriptComponentSystem;
use crate::python_script_component_system::PythonScriptComponentSystem;

static APP: Lazy<RwLock<App>> = Lazy::new(|| RwLock::new(App::new()));

pub fn update_app() {
    APP.write().unwrap().update();
}

pub fn get_app_read_only() -> RwLockReadGuard<'static, App> {
    return APP.read().unwrap();
}

pub fn get_app() -> RwLockWriteGuard<'static, App> {
    return APP.write().unwrap();
}

pub struct App {
    should_init: bool,
    pub dt: f32,
    pub javascript_component_system: Option<Box<JavaScriptScriptComponentSystem>>,
    pub python_component_system: Option<Box<PythonScriptComponentSystem>>,
}

impl App {
    fn new() -> Self {
        Self {
            should_init: true,
            dt: 0.0,
            javascript_component_system: None,
            python_component_system: None,
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
        self.javascript_component_system = Some(Box::new(JavaScriptScriptComponentSystem::new()));
        self.python_component_system = Some(Box::new(PythonScriptComponentSystem::new()));
    }

    fn update(&mut self) -> f32 {
        if self.should_init {
            self.initialize();
            self.should_init = false;
        }
        self.dt = 1.0 / 60.0;
        if self.javascript_component_system.is_some() {
            let jcs = self.javascript_component_system.as_mut().unwrap();
            jcs.update(self.dt);
        }
        if self.python_component_system.is_some() {
            let pcs = self.python_component_system.as_mut().unwrap();
            pcs.update(self.dt);
        }
        return self.dt;
    }
}
