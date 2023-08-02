use std::sync::{Mutex, Weak};

use gc::{Finalize, Trace};
use rustpython_vm::convert::{ToPyObject, ToPyResult};
use rustpython_vm::function::{FuncArgs, IntoPyNativeFunc};
use rustpython_vm::{
    compiler, pyclass, pymodule, Interpreter, PyObject, PyObjectRef, PyPayload, PyResult,
    TryFromBorrowedObject, VirtualMachine,
};

use dream_ecs::component::Transform;
use dream_ecs::entity::Entity;
use dream_ecs::scene::Scene;

use crate::system::System;

static SCENE: Mutex<Option<Weak<Mutex<Scene>>>> = Mutex::new(None);

pub struct PythonScriptComponentSystem {
    pub interpreter: Interpreter,
}

impl Default for PythonScriptComponentSystem {
    fn default() -> Self {
        let interpreter = Interpreter::with_init(Default::default(), |vm| {
            vm.add_native_module("dream".to_owned(), Box::new(dream::make_module));
        });
        Self { interpreter }
    }
}

impl System for PythonScriptComponentSystem {
    fn update(&mut self, _dt: f32, scene: Weak<Mutex<Scene>>) {
        if SCENE.lock().unwrap().is_none() {
            *SCENE.lock().unwrap() = Some(scene.clone());
        }

        let transform_entities = scene
            .upgrade()
            .expect("Unable to upgrade")
            .lock()
            .expect("Unable to lock")
            .get_entities_with_component::<Transform>();
        for entity_id in transform_entities {
            let script = include_str!("default-files/script.py");
            // let _entity = Entity::from_handle(entity_id, scene);
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
                    .compile(script, compiler::Mode::BlockExpr, source_path.to_owned())
                    .map_err(|err| vm.new_syntax_error(&err))
                    .unwrap();
                vm.run_code_obj(code_obj, scope)
                    .map(|value| {
                        let update = value.get_attr("update", vm).unwrap();
                        let res = vm
                            .invoke(&update, ())
                            .unwrap()
                            .try_int(vm)
                            .unwrap()
                            .to_string();
                        println!("{res}");
                        log::warn!("{res}");
                    })
                    .expect("Error running python code");
            })
        }
    }
}

#[pymodule]
mod dream {
    use rustpython_vm::{
        builtins::PyList, convert::ToPyObject, PyObjectRef, TryFromBorrowedObject,
    };

    use super::*;

    #[pyfunction]
    fn get_entity(handle: u64, _vm: &VirtualMachine) -> PyResult<PythonEntity> {
        Ok(PythonEntity { handle })
    }

    #[pyattr]
    #[pyclass(module = "dream", name = "PythonEntity")]
    #[derive(Debug, PyPayload)]
    struct PythonEntity {
        handle: u64,
    }

    #[pyclass]
    impl PythonEntity {
        #[pygetset]
        fn handle(&self) -> u64 {
            self.handle
        }

        #[pymethod]
        fn print_in_rust_from_python(&self) {
            println!("Calling a rust method from python");
        }

        #[pymethod]
        fn get_transform(&self) -> f32 {
            let scene = SCENE.lock().unwrap().as_ref().unwrap().clone();
            let entity = Entity::from_handle(self.handle, scene);
            let transform: Option<Transform> = entity.get_component();
            transform.expect("No transform component").position.x
        }
    }

    impl TryFromBorrowedObject for PythonEntity {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> PyResult<Self> {
            let handle = obj.get_attr("handle", vm)?.try_into_value::<u64>(vm)?;
            Ok(PythonEntity { handle })
        }
    }
}
