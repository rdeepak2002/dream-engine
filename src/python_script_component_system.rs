#![allow(unused_mut)]

use rustpython_vm as vm;

use dream_ecs::component_system::ComponentSystem;
use dream_ecs::entity::Entity;
use dream_ecs::scene::get_current_scene_read_only;

pub struct PythonScriptComponentSystem {}

impl PythonScriptComponentSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl ComponentSystem for PythonScriptComponentSystem {
    fn update(&mut self, dt: f32) {
        let transform_entities: Vec<u64>;
        {
            let scene = get_current_scene_read_only();
            transform_entities = scene.transform_entities().clone();
        }
        for entity_id in transform_entities {
            let entity = Entity::from_handle(entity_id);
            vm::Interpreter::without_stdlib(Default::default()).enter(|vm| {
                let scope = vm.new_scope_with_builtins();
                let mut source_path = "<embedded>";
                #[cfg(target_arch = "wasm32")]
                {
                    source_path = "<wasm>";
                }
                let code_obj = vm
                    .compile(r#"5"#, vm::compiler::Mode::Eval, source_path.to_owned())
                    .map_err(|err| vm.new_syntax_error(&err))
                    .unwrap();
                let py_obj_ref = vm
                    .run_code_obj(code_obj, scope)
                    .expect("Error running python code");
                let res = py_obj_ref
                    .try_int(vm)
                    .expect("Error getting python result")
                    .to_string();
                println!("Result from Python: {}", res);
                log::warn!("Result from Python: {}", res);
            })
        }
    }
}
