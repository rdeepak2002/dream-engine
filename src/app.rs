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
use dream_ecs;
use dream_ecs::component::Transform;
use dream_ecs::component_system::ComponentSystem;
use dream_ecs::scene::Scene;

use crate::javascript_script_component_system::JavaScriptScriptComponentSystem;

pub struct App {
    pub dt: f32,
    pub scene: Option<Box<Scene>>,
    pub javascript_component_system: Option<Box<JavaScriptScriptComponentSystem>>,
}

impl App {
    pub fn new() -> Self {
        let dt: f32 = 0.0;
        let mut scene = Scene::new();

        let e = scene.create_entity();
        e.add_transform(Transform::from(dream_math::Vector3::from(1., 1., 1.)));

        let javascript_component_system = JavaScriptScriptComponentSystem::new();

        Self {
            dt,
            scene: Some(Box::new(scene)),
            javascript_component_system: Some(Box::new(javascript_component_system)),
        }
    }

    pub fn update(&mut self) -> f32 {
        if self.scene.is_some() {
            self.dt = 1.0 / 60.0;
            if self.javascript_component_system.is_some() {
                let jcs = self.javascript_component_system.as_mut().unwrap();
                jcs.update(self.dt, self.scene.as_mut().unwrap());
            }
            return self.dt;
        }
        return 0.0;
    }
}
