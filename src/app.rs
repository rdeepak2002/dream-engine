pub struct App {
    dt: f32,
}

impl App {
    pub fn new() -> Self {
        let dt: f32 = 0.0;

        Self { dt }
    }

    pub fn update(&self) -> f32 {
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
