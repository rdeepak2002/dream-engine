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
    dt: f32,
    scene: Scene,
    javascript_component_system: JavaScriptScriptComponentSystem,
}

impl App {
    pub async fn new() -> Self {
        let dt: f32 = 0.0;
        let mut scene = Scene::new();

        let e = scene.create_entity();
        e.add_transform(Transform::from(dream_math::Vector3::from(1., 1., 1.)));

        let javascript_component_system = JavaScriptScriptComponentSystem::new();

        Self {
            dt,
            scene,
            javascript_component_system,
        }
    }

    pub fn update(&mut self) -> f32 {
        self.dt = 1.0 / 60.0;
        self.javascript_component_system
            .update(self.dt, &mut self.scene);
        return 0.0;
    }
}
