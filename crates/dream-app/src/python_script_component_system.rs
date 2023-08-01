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

pub struct PythonScriptComponentSystem {
    pub interpreter: Interpreter,
}

impl Default for PythonScriptComponentSystem {
    fn default() -> Self {
        let interpreter = Interpreter::with_init(Default::default(), |vm| {
            vm.add_native_module(
                "rust_py_module".to_owned(),
                Box::new(rust_py_module::make_module),
            );
        });
        Self { interpreter }
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
                        let get_handle_func = value.get_attr("get_handle", vm).unwrap();
                        let res = vm
                            .invoke(&get_handle_func, ())
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
mod rust_py_module {
    use rustpython_vm::{
        builtins::PyList, convert::ToPyObject, PyObjectRef, TryFromBorrowedObject,
    };

    use super::*;

    #[pyfunction]
    fn rust_function(
        num: i32,
        s: String,
        python_person: PythonEntity,
        _vm: &VirtualMachine,
    ) -> PyResult<RustStruct> {
        println!(
            "Calling standalone rust function from python passing args:
            num: {},
            string: {},
            python_person.handle: {}",
            num, s, python_person.handle
        );
        Ok(RustStruct {
            numbers: NumVec(vec![1, 2, 3, 4]),
        })
    }

    #[derive(Debug, Clone)]
    struct NumVec(Vec<i32>);

    impl ToPyObject for NumVec {
        fn to_pyobject(self, vm: &VirtualMachine) -> PyObjectRef {
            let list = self.0.into_iter().map(|e| vm.new_pyobj(e)).collect();
            PyList::new_ref(list, vm.as_ref()).to_pyobject(vm)
        }
    }

    #[pyattr]
    #[pyclass(module = "rust_py_module", name = "RustStruct")]
    #[derive(Debug, PyPayload)]
    struct RustStruct {
        numbers: NumVec,
    }

    #[pyclass]
    impl RustStruct {
        #[pygetset]
        fn numbers(&self) -> NumVec {
            self.numbers.clone()
        }

        #[pymethod]
        fn print_in_rust_from_python(&self) {
            println!("Calling a rust method from python");
        }
    }

    struct PythonEntity {
        handle: u64,
    }

    impl TryFromBorrowedObject for PythonEntity {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> PyResult<Self> {
            let handle = obj.get_attr("handle", vm)?.try_into_value::<u64>(vm)?;
            Ok(PythonEntity { handle })
        }
    }
}
