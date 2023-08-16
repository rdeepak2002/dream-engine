use std::collections::HashMap;
use std::sync::{Mutex, Weak};

use gc::{Finalize, Trace};
use rustpython_vm::builtins::{PyDictRef, PyIntRef};
use rustpython_vm::convert::{ToPyObject, ToPyResult};
use rustpython_vm::function::{ArgMapping, FuncArgs, OptionalArg};
use rustpython_vm::protocol::PyNumber;
use rustpython_vm::scope::Scope;
use rustpython_vm::{
    compiler, pyclass, pymodule,
    types::{Constructor, GetDescriptor, Unconstructible},
    AsObject, Interpreter, PyObject, PyObjectRef, PyPayload, PyResult, TryFromBorrowedObject,
    VirtualMachine,
};

use dream_ecs::scene::Scene;

use crate::system::System;

static SCENE: Mutex<Option<Weak<Mutex<Scene>>>> = Mutex::new(None);

pub struct PythonScriptComponentSystem {
    pub interpreter: Interpreter,
    pub entity_script: HashMap<u64, Option<PyObjectRef>>,
}

impl Default for PythonScriptComponentSystem {
    fn default() -> Self {
        let interpreter = Interpreter::with_init(Default::default(), |vm| {
            vm.add_native_module("dream_py".to_owned(), Box::new(dream_py::make_module));
        });
        Self {
            interpreter,
            entity_script: Default::default(),
        }
    }
}

impl System for PythonScriptComponentSystem {
    fn update(&mut self, dt: f32, scene: Weak<Mutex<Scene>>) {
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
                // TODO: only create code object on file saved or changed (get last updated property of meta data or something)
                let code_obj = vm
                    .compile(script, compiler::Mode::BlockExpr, source_path.to_owned())
                    .map_err(|err| vm.new_syntax_error(&err, None))
                    .unwrap();
                vm.run_code_obj(code_obj, scope)
                    .map(|value| {
                        self.entity_script.entry(entity_id).or_insert(Some(value));
                    })
                    .expect("Error running python code");
                let entity_script = self
                    .entity_script
                    .get(&entity_id)
                    .unwrap()
                    .as_ref()
                    .unwrap();

                if let Ok(update) = entity_script.get_attr("update", vm) {
                    let entity = dream_py::Entity { handle: entity_id }.to_pyobject(vm);
                    let args = vec![vm.ctx.new_float(dt as f64).into(), entity];
                    let res = vm.invoke(&update, args);
                    if res.is_err() {
                        log::error!("No python class exported");
                    }
                }

                // TODO: allow other python scripts to get variables that are defined
                // TODO: allow inspector to view attributes (have attributes map in script component)
                // for attribute in attributes {
                //     let attribute_name = attribute.0.to_string();
                //     // let attribute_value = attribute.1.to_pyresult(vm);
                //     println!("name: {attribute_name}");
                //     // let attribute_name = "x";
                //     let attribute_value = entity_script.get_attr(attribute_name, vm);
                //     // let x: f64 = attribute_value.unwrap().try_float(vm).unwrap().to_f64();
                //     // println!("Got variable from other object {}", x);
                // }

                // TODO: this attributes map exposes methods (such as update)
                // TODO: this can be useful for having one script call a method for another script?
                // let attributes = entity_script.class().get_attributes();
                // for attribute in attributes {
                //     let attribute_name = attribute.0.to_string();
                //     println!("name: {attribute_name}");
                //     let attribute_value = entity_script.get_attr(attribute_name, vm);
                // }
            })
        }
    }
}

#[pymodule]
pub(crate) mod dream_py {
    use rustpython_vm::{
        builtins::PyList, convert::ToPyObject, PyObjectRef, TryFromBorrowedObject,
    };

    use super::*;

    #[pyfunction]
    fn get_entity(handle: u64, _vm: &VirtualMachine) -> PyResult<Entity> {
        Ok(Entity { handle })
    }

    #[pyattr]
    #[pyclass(module = "dream_py", name = "Entity")]
    #[derive(Debug, PyPayload)]
    pub(crate) struct Entity {
        pub(crate) handle: u64,
    }

    #[pyclass]
    impl Entity {
        #[pygetset]
        fn handle(&self) -> u64 {
            self.handle
        }

        #[pymethod]
        fn get_transform(&self) -> Transform {
            let scene = SCENE.lock().unwrap().as_ref().unwrap().clone();
            let entity = dream_ecs::entity::Entity::from_handle(self.handle, scene);
            let transform: Option<dream_ecs::component::Transform> = entity.get_component();
            Transform::from(transform.expect("No transform component"))
        }

        #[pymethod]
        fn set_position(&self, x: f32, y: f32, z: f32) {
            let position = Vector3 { x, y, z };
            let scene = SCENE.lock().unwrap().as_ref().unwrap().clone();
            let entity = dream_ecs::entity::Entity::from_handle(self.handle, scene);
            let transform: Option<dream_ecs::component::Transform> = entity.get_component();
            let mut transform = transform.unwrap();
            transform.position = dream_math::Vector3::from(position);
            entity.add_component(transform);
        }
    }

    impl TryFromBorrowedObject<'_> for Entity {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> PyResult<Self> {
            let handle = obj.get_attr("handle", vm)?.try_into_value::<u64>(vm)?;
            Ok(Entity { handle })
        }
    }

    #[pyattr]
    #[pyclass(module = "dream_py", name = "Transform")]
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

    impl From<dream_ecs::component::Transform> for Transform {
        fn from(transform: dream_ecs::component::Transform) -> Self {
            Transform {
                position: Vector3::from(transform.position),
            }
        }
    }

    impl TryFromBorrowedObject<'_> for Transform {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> PyResult<Self> {
            let position = obj
                .get_attr("position", vm)?
                .try_into_value::<Vector3>(vm)?;
            Ok(Transform { position })
        }
    }

    #[pyattr]
    #[pyclass(module = "dream_py", name = "Vector3")]
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

    impl From<Vector3> for dream_math::Vector3 {
        fn from(vec3: Vector3) -> Self {
            dream_math::Vector3 {
                x: vec3.x,
                y: vec3.y,
                z: vec3.z,
            }
        }
    }

    impl From<dream_math::Vector3> for Vector3 {
        fn from(vec3: dream_math::Vector3) -> Self {
            Vector3 {
                x: vec3.x,
                y: vec3.y,
                z: vec3.z,
            }
        }
    }

    impl TryFromBorrowedObject<'_> for Vector3 {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> PyResult<Self> {
            let x = obj.get_attr("x", vm)?.try_into_value::<f32>(vm)?;
            let y = obj.get_attr("y", vm)?.try_into_value::<f32>(vm)?;
            let z = obj.get_attr("z", vm)?.try_into_value::<f32>(vm)?;
            Ok(Vector3 { x, y, z })
        }
    }
}
