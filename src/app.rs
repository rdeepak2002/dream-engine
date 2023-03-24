use dream_ecs;
use dream_ecs::component::Transform;
use dream_ecs::person::Person;
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
        e.add_transform(Transform::from(1., 1., 1.));

        Self { dt, scene }
    }

    pub fn update(&mut self) -> f32 {
        self.dt = 1.0 / 60.0;
        {
            // example execution of javascript code
            // let js_code = "7 * 8.1; let person = new Person("John", 28); person.say_hello();";
            let js_code = "let person = new Person('John', 29); person.say_hello(); 7 * 8.1";
            let mut context = boa_engine::Context::default();
            context
                .register_global_class::<Person>()
                .expect("could not register class");
            match context.eval(js_code) {
                Ok(res) => {
                    println!("{}", res.to_string(&mut context).unwrap());
                    log::warn!("{}", res.to_string(&mut context).unwrap());
                }
                Err(e) => {
                    // Pretty print the error
                    eprintln!("Uncaught {}", e.display());
                    log::error!("Uncaught {}", e.display());
                }
            };
        }
        return 0.0;
    }
}
