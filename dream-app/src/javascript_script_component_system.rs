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

use boa_engine::JsValue;
use epi::egui::emath::Numeric;

use dream_ecs::component_system::ComponentSystem;
use dream_ecs::entity::Entity;
use dream_ecs::scene::get_current_scene_read_only;

use crate::entity_js::{EntityJS, Vector3JS};

pub struct JavaScriptScriptComponentSystem {}

impl JavaScriptScriptComponentSystem {
    pub fn new() -> JavaScriptScriptComponentSystem {
        Self {}
    }
}

impl ComponentSystem for JavaScriptScriptComponentSystem {
    fn update(&mut self, dt: f32) {
        // let scene = SCENE.read().unwrap();
        let transform_entities: Vec<u64>;
        {
            // let scene = SCENE.read().unwrap();
            let scene = get_current_scene_read_only();
            transform_entities = scene.transform_entities().clone();
        }
        for entity_id in transform_entities {
            let entity = Entity::from_handle(entity_id);
            // TODO: read this using read bytes method defined in dream-fs
            let js_code: String = include_str!("../../res/script.js").into();
            let mut context = boa_engine::Context::default();

            // evaluate all code (expects a javascript class object to be returned at the end)
            match context.eval(js_code) {
                Ok(res) => {
                    // register global classes
                    context.register_global_class::<EntityJS>().unwrap();
                    context.register_global_class::<Vector3JS>().unwrap();
                    // get script class returned and call its update method
                    let js_obj = res.as_object().expect("No object returned by script");
                    // TODO: for persistence store this js object (probs better than storing context)
                    let js_obj_update_func = js_obj
                        .get("update", &mut context)
                        .expect("No update function found");
                    let js_obj_update_func_obj = js_obj_update_func
                        .as_object()
                        .expect("Unable to convert update to object");
                    let js_update_call = js_obj_update_func_obj.call(
                        &res,
                        &[
                            JsValue::Integer(entity.get_runtime_id() as i32),
                            JsValue::Rational(dt.to_f64()),
                        ],
                        &mut context,
                    );
                    match js_update_call {
                        Ok(_res) => {}
                        Err(e) => {
                            // Pretty print the error
                            eprintln!("Uncaught (2) {}", e.display());
                            log::error!("Uncaught (2) {}", e.display());
                        }
                    };
                }
                Err(e) => {
                    // script could not compile
                    eprintln!("Uncaught (1) {}", e.display());
                    log::error!("Uncaught (1) {}", e.display());
                }
            };
        }
    }
}
