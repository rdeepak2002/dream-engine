use std::ops::Deref;

use boa_engine::object::{NativeObject, ObjectInitializer};
use boa_engine::property::Attribute;
use shipyard::IntoIter;

use dream_ecs;
use dream_ecs::component::Transform;
use dream_ecs::entityjs::EntityJS;
use dream_ecs::scene::Scene;

pub struct App {
    dt: f32,
    #[allow(dead_code)]
    scene: Scene,
}

impl App {
    pub fn new() -> Self {
        let dt: f32 = 0.0;
        let mut scene = Scene::new();

        let e = scene.create_entity();
        e.add_transform(Transform::from(dream_math::Vector3::from(1., 1., 1.)));

        Self { dt, scene }
    }

    pub fn update(&mut self) -> f32 {
        self.dt = 1.0 / 60.0;
        {
            self.scene
                .handle
                .run(|mut vm_transform: shipyard::ViewMut<Transform>| {
                    for t in vm_transform.iter() {
                        let entity_js = EntityJS::new(t.clone());
                        // example of running script per entity
                        {
                            let js_code = r#"
                            function update(entity) {
                                entity.transform.position.x = 2;
                                // return entity.transform.position.x;
                                return entity;
                            }
                            "#;

                            let mut context = boa_engine::Context::default();

                            context
                                .register_global_class::<EntityJS>()
                                .expect("could not register class");

                            match context.eval(js_code) {
                                Ok(_res) => {
                                    // println!("{}", res.to_string(&mut context).unwrap());
                                    // log::warn!("{}", res.to_string(&mut context).unwrap());
                                }
                                Err(e) => {
                                    // Pretty print the error
                                    eprintln!("Uncaught (1) {}", e.display());
                                    log::error!("Uncaught (1) {}", e.display());
                                }
                            };

                            let position = ObjectInitializer::new(&mut context)
                                .property("x", entity_js.transform.position.x, Attribute::all())
                                .property("y", entity_js.transform.position.y, Attribute::all())
                                .property("z", entity_js.transform.position.z, Attribute::all())
                                .build();

                            let transform = ObjectInitializer::new(&mut context)
                                .property("position", position, Attribute::all())
                                .build();

                            let entity = ObjectInitializer::new(&mut context)
                                .property("transform", transform, Attribute::all())
                                .build();

                            context.register_global_property("entity", entity, Attribute::all());

                            let js_code = "update(entity);";

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
                                    println!("x final: {}", x_js.as_number().unwrap());
                                }
                                Err(e) => {
                                    // Pretty print the error
                                    eprintln!("Uncaught (2) {}", e.display());
                                    log::error!("Uncaught (2) {}", e.display());
                                }
                            };
                        }
                    }
                });
        }
        return 0.0;
    }
}
