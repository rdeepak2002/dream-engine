use boa_engine::object::ObjectInitializer;
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
        let transform_entities = self.scene.transform_entities();
        for transform_entity in transform_entities {
            let t = transform_entity.get_transform().unwrap();
            let entity_js = EntityJS::new(t);
            let js_code = r#"
                    function update(entity) {
                        entity.transform.position.x = entity.transform.position.x + 1;
                        return entity;
                    }
                    "#;

            let mut context = boa_engine::Context::default();

            context
                .register_global_class::<EntityJS>()
                .expect("could not register class");

            match context.eval(js_code) {
                Ok(_res) => {
                    // script compiled
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
                            transform_entity.add_transform(Transform::from(position));

                            println!("x final: {}", x_js.as_number().unwrap());
                            log::warn!("x final: {}", x_js.as_number().unwrap());
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
        return 0.0;
    }
}
