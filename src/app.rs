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
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;
use shipyard::EntityId;

use dream_ecs;
use dream_ecs::component::Transform;
use dream_ecs::component_system::ComponentSystem;
use dream_ecs::entity::Entity;
use dream_ecs::scene::{Scene, SCENE};

use crate::javascript_script_component_system::JavaScriptScriptComponentSystem;

// 1.3.1

pub struct App {
    pub dt: f32,
    // pub scene: Option<Box<Scene>>,
    // pub scene: Option<RwLock<Box<Scene>>>,
    pub javascript_component_system: Option<Box<JavaScriptScriptComponentSystem>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            dt: 0.0,
            javascript_component_system: None,
        }
    }

    pub fn initialize(&mut self) {
        // init scene
        let mut e: Option<Entity> = None;
        {
            let mut scene = SCENE.write().unwrap();
            e = Some(scene.create_entity());
        }
        {
            e.unwrap()
                .add_transform(Transform::from(dream_math::Vector3::from(2., 1., 1.)));
        }
        // init component systems
        let javascript_component_system = JavaScriptScriptComponentSystem::new();
        self.javascript_component_system = Some(Box::new(javascript_component_system));
    }

    pub fn update(&mut self) -> f32 {
        self.dt = 1.0 / 60.0;
        if self.javascript_component_system.is_some() {
            let jcs = self.javascript_component_system.as_mut().unwrap();
            jcs.update(self.dt);
        }
        return self.dt;
        // if self.scene.is_some() {
        //     self.dt = 1.0 / 60.0;
        //     if self.javascript_component_system.is_some() {
        //         let jcs = self.javascript_component_system.as_mut().unwrap();
        //         jcs.update(self.dt, self.scene.as_mut().unwrap());
        //     }
        //     return self.dt;
        // }
        // return 0.0;
    }

    // pub fn get_entity_from_inner(&mut self, inner: u64) -> Option<Entity> {
    //     if self.scene.is_some() {
    //         let entity = Some(Entity::from_mut_ref(
    //             self.scene.as_mut().unwrap(),
    //             EntityId::from_inner(inner).expect("Unable to unwrap entity inner id"),
    //         ));
    //         return entity;
    //     }
    //     return None;
    // }
}
