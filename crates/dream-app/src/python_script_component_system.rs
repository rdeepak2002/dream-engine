use std::sync::{Mutex, Weak};

use rustpython_vm as vm;

use dream_ecs::component::Transform;
use dream_ecs::entity::Entity;
use dream_ecs::scene::Scene;

use crate::system::System;

pub struct PythonScriptComponentSystem {
    pub interpreter: rustpython_vm::Interpreter,
}

impl Default for PythonScriptComponentSystem {
    fn default() -> Self {
        Self {
            interpreter: vm::Interpreter::without_stdlib(Default::default()),
        }
    }
}

impl System for PythonScriptComponentSystem {
    fn update(&mut self, _dt: f32, scene: Weak<Mutex<Scene>>) {
        let transform_entities = scene
            .upgrade()
            .expect("Unable to upgrade")
            .lock()
            .expect("Unable to lock")
            .get_entities_with_component::<Transform>();
        // for entity_id in transform_entities {
        //     let _entity = Entity::from_handle(entity_id, scene);
        //     #[allow(clippy::needless_late_init)]
        //     self.interpreter.enter(|vm| {
        //         let scope = vm.new_scope_with_builtins();
        //         let source_path;
        //         cfg_if::cfg_if! {
        //             if #[cfg(target_arch = "wasm32")] {
        //                 source_path = "<wasm>"
        //             } else {
        //                 source_path = "<embedded>"
        //             }
        //         }
        //         let code_obj = vm
        //             .compile(r#"5"#, vm::compiler::Mode::Eval, source_path.to_owned())
        //             .map_err(|err| vm.new_syntax_error(&err))
        //             .unwrap();
        //         let py_obj_ref = vm
        //             .run_code_obj(code_obj, scope)
        //             .expect("Error running python code");
        //         let _res = py_obj_ref
        //             .try_int(vm)
        //             .expect("Error getting python result")
        //             .to_string();
        //     })
        // }
    }
}
