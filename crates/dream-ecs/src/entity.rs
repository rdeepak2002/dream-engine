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

use std::sync::{Mutex, Weak};

use shipyard::{EntityId, Get};

use crate::component::Transform;
use crate::scene::Scene;

#[derive(Clone)]
pub struct Entity {
    pub handle: u64,
    pub scene: Weak<Mutex<Scene>>,
}

impl Entity {
    pub fn from_handle(handle: u64, scene: Weak<Mutex<Scene>>) -> Self {
        Self { handle, scene }
    }

    pub fn get_runtime_id(&self) -> u64 {
        return self.handle;
    }

    pub fn add_component<T: shipyard::TupleAddComponent>(&self, component: T) {
        let scene = self.scene.upgrade();
        scene
            .expect("Unable to upgrade scene smart pointer for getting component")
            .lock()
            .expect("Unable to get mutex lock")
            .handle
            .add_component(EntityId::from_inner(self.handle).unwrap(), component);
    }

    pub fn remove_component<T: shipyard::TupleRemove>(&self) {
        let scene = self.scene.upgrade();
        scene
            .expect("Unable to upgrade scene smart pointer for removing component")
            .lock()
            .expect("Unable to get mutex lock")
            .handle
            .remove::<T>(EntityId::from_inner(self.handle).unwrap());
    }

    pub fn get_component<T: shipyard::Component + Send + Sync + Clone>(&self) -> Option<T> {
        let mut comp_opt: Option<T> = None;
        let scene = self.scene.upgrade();
        let system = |vm_pos: shipyard::ViewMut<T>| {
            let comp = vm_pos.get(EntityId::from_inner(self.handle).unwrap());
            if comp.is_ok() {
                comp_opt = Some(comp.unwrap().clone());
            } else {
                comp_opt = None;
            }
        };
        scene
            .expect("Unable to upgrade scene smart pointer for getting component")
            .lock()
            .expect("Unable to get mutex lock")
            .handle
            .run(system);
        comp_opt
    }

    pub fn has_component<T: shipyard::Component + Send + Sync + Clone>(&self) -> bool {
        let comp: Option<T> = self.get_component();
        comp.is_some()
    }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let trans: Option<Transform> = self.get_component();
        if trans.is_some() {
            write!(f, "Entity(Transform({}))", trans.unwrap())
        } else {
            write!(f, "Entity()")
        }
    }
}
