use std::sync::{Mutex, Weak};

use gc::{Finalize, Trace};
use rustpython_vm::builtins::PyIntRef;
use rustpython_vm::convert::{ToPyObject, ToPyResult};
use rustpython_vm::function::{FuncArgs, IntoPyNativeFunc, OptionalArg};
use rustpython_vm::protocol::PyNumber;
use rustpython_vm::{
    compiler, pyclass, pymodule, Interpreter, PyObject, PyObjectRef, PyPayload, PyResult,
    TryFromBorrowedObject, VirtualMachine,
};

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
            .get_entities_with_component::<dream_ecs::component::Transform>();
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
                        let handle = vm.ctx.new_int(entity_id).into();
                        let args = vec![handle];
                        let res = vm
                            .invoke(&update, args)
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
        fn get_transform(&self) -> Transform {
            let scene = SCENE.lock().unwrap().as_ref().unwrap().clone();
            let entity = Entity::from_handle(self.handle, scene);
            let transform: Option<dream_ecs::component::Transform> = entity.get_component();
            transform.expect("No transform component").into()
        }

        #[pymethod]
        fn set_position(&self, x: f32, y: f32, z: f32) {
            let position = Vector3 { x, y, z };
            let scene = SCENE.lock().unwrap().as_ref().unwrap().clone();
            let entity = Entity::from_handle(self.handle, scene);
            let transform: Option<dream_ecs::component::Transform> = entity.get_component();
            let mut transform = transform.unwrap();
            transform.position = position.into();
            entity.add_component(transform);
        }
    }

    impl TryFromBorrowedObject for PythonEntity {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> PyResult<Self> {
            let handle = obj.get_attr("handle", vm)?.try_into_value::<u64>(vm)?;
            Ok(PythonEntity { handle })
        }
    }

    #[pyattr]
    #[pyclass(module = "dream", name = "Transform")]
    #[derive(Debug, PyPayload)]
    struct Transform {
        position: Vector3,
    }

    #[pyclass]
    impl Transform {
        #[pygetset]
        fn position(&self) -> Vector3 {
            self.position
        }
    }

    impl Into<Transform> for dream_ecs::component::Transform {
        fn into(self) -> Transform {
            Transform {
                position: self.position.into(),
            }
        }
    }

    impl TryFromBorrowedObject for Transform {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> PyResult<Self> {
            let position = obj
                .get_attr("position", vm)?
                .try_into_value::<Vector3>(vm)?;
            Ok(Transform { position })
        }
    }

    #[pyattr]
    #[pyclass(module = "dream", name = "Vector3")]
    #[derive(Debug, Clone, Copy, PyPayload)]
    struct Vector3 {
        x: f32,
        y: f32,
        z: f32,
    }

    #[pyclass]
    impl Vector3 {
        #[pygetset]
        fn x(&self) -> f32 {
            self.x
        }

        #[pygetset]
        fn y(&self) -> f32 {
            self.y
        }

        #[pygetset]
        fn z(&self) -> f32 {
            self.z
        }
    }

    impl Into<Vector3> for dream_math::Vector3 {
        fn into(self) -> Vector3 {
            Vector3 {
                x: self.x,
                y: self.y,
                z: self.z,
            }
        }
    }

    impl Into<dream_math::Vector3> for Vector3 {
        fn into(self) -> dream_math::Vector3 {
            dream_math::Vector3 {
                x: self.x,
                y: self.y,
                z: self.z,
            }
        }
    }

    impl TryFromBorrowedObject for Vector3 {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> PyResult<Self> {
            let x = obj.get_attr("x", vm)?.try_into_value::<f32>(vm)?;
            let y = obj.get_attr("y", vm)?.try_into_value::<f32>(vm)?;
            let z = obj.get_attr("z", vm)?.try_into_value::<f32>(vm)?;
            Ok(Vector3 { x, y, z })
        }
    }
}
