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

use boa_engine::object::{JsObject, ObjectInitializer};
use boa_engine::property::Attribute;
use boa_engine::JsValue;
use epi::egui::emath::Numeric;

use dream_ecs::component::Transform;
use dream_ecs::component_system::ComponentSystem;
use dream_ecs::scene::Scene;

use crate::entity_js::{EntityJS, Vector3JS};

pub struct JavaScriptScriptComponentSystem {}

impl JavaScriptScriptComponentSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl ComponentSystem for JavaScriptScriptComponentSystem {
    fn update(&mut self, dt: f32, scene: &mut Box<Scene>) {
        let transform_entities = scene.transform_entities();
        for entity in transform_entities {
            let js_code: String = include_str!("../res/script.js").into();
            let mut context = boa_engine::Context::default();
            context.register_global_class::<EntityJS>().unwrap();
            context.register_global_class::<Vector3JS>().unwrap();

            // TODO: for persistence instead of saving context we should save a JS value? not sure
            // ^ this would involve storing the resulting compiled javascript class object which seems fine imo

            // evaluate all code
            match context.eval(js_code) {
                Ok(res) => {
                    let js_obj = res.as_object().expect("No object returned by script");
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
