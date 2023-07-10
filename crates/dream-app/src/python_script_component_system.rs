use rustpython_vm as vm;

use dream_ecs::entity::Entity;
use dream_ecs::scene::SCENE;

use crate::system::System;

pub struct PythonScriptComponentSystem {
    pub interpreter: rustpython_vm::Interpreter,
}

impl PythonScriptComponentSystem {
    pub fn new() -> Self {
        Self {
            interpreter: vm::Interpreter::without_stdlib(Default::default()),
        }
    }
}

impl System for PythonScriptComponentSystem {
    fn update(&mut self, _dt: f32) {
        let transform_entities: Vec<u64>;
        {
            let scene = SCENE.lock().unwrap();
            transform_entities = scene.transform_entities();
        }
        for entity_id in transform_entities {
            let _entity = Entity::from_handle(entity_id);
            #[allow(clippy::needless_late_init)]
            self.interpreter.enter(|vm| {
                let scope = vm.new_scope_with_builtins();
                let source_path;
                cfg_if::cfg_if! {
                    if #[cfg(target_arch = "wasm32")] {
                        source_path = "<wasm>"
                    } else {
                        source_path = "<embedded>"
                    }
                }
                let code_obj = vm
                    .compile(r#"5"#, vm::compiler::Mode::Eval, source_path.to_owned())
                    .map_err(|err| vm.new_syntax_error(&err))
                    .unwrap();
                let py_obj_ref = vm
                    .run_code_obj(code_obj, scope)
                    .expect("Error running python code");
                let _res = py_obj_ref
                    .try_int(vm)
                    .expect("Error getting python result")
                    .to_string();
            })
        }
    }
}
