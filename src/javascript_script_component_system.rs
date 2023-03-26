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

use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;

use dream_ecs::component::Transform;
use dream_ecs::component_system::ComponentSystem;
use dream_ecs::scene::Scene;

pub struct JavaScriptScriptComponentSystem {}

impl JavaScriptScriptComponentSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl ComponentSystem for JavaScriptScriptComponentSystem {
    fn update(&mut self, dt: f32, scene: &mut Scene) {
        let transform_entities = scene.transform_entities();
        for entity in transform_entities {
            let js_code: String = include_str!("../res/script.js").into();

            let mut context = boa_engine::Context::default();

            match context.eval(js_code) {
                Ok(_res) => {
                    // script successfully compiled

                    // create javascript version of the scriptable entity
                    let js_entity = ObjectInitializer::new(&mut context).build();

                    if entity.has_transform() {
                        let transform = entity.get_transform().unwrap();

                        let js_position = ObjectInitializer::new(&mut context)
                            .property("x", transform.position.x, Attribute::all())
                            .property("y", transform.position.y, Attribute::all())
                            .property("z", transform.position.z, Attribute::all())
                            .build();

                        let js_transform = ObjectInitializer::new(&mut context)
                            .property("position", js_position, Attribute::all())
                            .build();

                        js_entity
                            .create_data_property("transform", js_transform, &mut context)
                            .expect("Unable to add transform entity to JS entity object");
                    }

                    context.register_global_property("entity", js_entity, Attribute::all());

                    // call update function defined in script
                    let js_code = format!("update(entity, {});", dt);

                    match context.eval(js_code) {
                        Ok(res) => {
                            let transform_js = res
                                .as_object()
                                .unwrap()
                                .get("transform", &mut context)
                                .unwrap();
                            let position_js = transform_js
                                .as_object()
                                .unwrap()
                                .get("position", &mut context)
                                .unwrap();
                            let x_js = position_js
                                .as_object()
                                .unwrap()
                                .get("x", &mut context)
                                .unwrap();
                            let y_js = position_js
                                .as_object()
                                .unwrap()
                                .get("y", &mut context)
                                .unwrap();
                            let z_js = position_js
                                .as_object()
                                .unwrap()
                                .get("z", &mut context)
                                .unwrap();

                            let x: f32 = x_js.as_number().unwrap() as f32;
                            let y: f32 = y_js.as_number().unwrap() as f32;
                            let z: f32 = z_js.as_number().unwrap() as f32;

                            // update entity present in shipyard ecs system
                            let position = dream_math::Vector3::from(x, y, z);
                            entity.add_transform(Transform::from(position));

                            // println!("x final: {}", x_js.as_number().unwrap());
                            // log::warn!("x final: {}", x_js.as_number().unwrap());
                        }
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
