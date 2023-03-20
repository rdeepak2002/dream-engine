use dream_ecs;
use dream_ecs::Transform;

pub struct App {
    #[allow(dead_code)]
    dt: f32,
    scene: dream_ecs::Scene,
}

impl App {
    pub fn new() -> Self {
        let dt: f32 = 0.0;
        let mut scene = dream_ecs::Scene::new();

        let e = scene.create_entity();
        println!("{}", e.to_string());
        e.add_transform(Transform::from(1., 1., 1.));
        println!("{}", e.to_string());

        Self { dt, scene }
    }

    pub fn update(&mut self) -> f32 {
        {
            // example execution of javascript code
            // let js_code = "7 * 8.1";
            // let mut context = boa_engine::Context::default();
            // match context.eval(js_code) {
            //     Ok(res) => {
            //         println!("{}", res.to_string(&mut context).unwrap());
            //         log::warn!("{}", res.to_string(&mut context).unwrap());
            //     }
            //     Err(e) => {
            //         // Pretty print the error
            //         eprintln!("Uncaught {}", e.display());
            //         log::error!("Uncaught {}", e.display());
            //     }
            // };
        }

        return 0.0;
    }
}
