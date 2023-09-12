use std::collections::HashMap;
use std::sync::{Mutex, Weak};

use rustpython_vm::convert::ToPyResult;
use rustpython_vm::{
    compiler, pyclass, pymodule, types::Constructor, Interpreter, PyObject, PyObjectRef, PyPayload,
    PyResult, VirtualMachine,
};

use dream_ecs::component::PythonScript;
use dream_ecs::entity::Entity;
use dream_ecs::scene::Scene;

use crate::system::System;

static SCENE: Mutex<Option<Weak<Mutex<Scene>>>> = Mutex::new(None);

pub struct PythonScriptComponentSystem {
    pub interpreter: Interpreter,
    pub entity_script: HashMap<u64, Option<PyObjectRef>>,
    pub script_cache: HashMap<String, String>,
}

impl Default for PythonScriptComponentSystem {
    fn default() -> Self {
        let interpreter = Interpreter::with_init(Default::default(), |vm| {
            vm.add_native_module("dream".to_owned(), Box::new(dream_py::make_module));
            vm.add_frozen(rustpython_vm::py_freeze!(dir = "src/pylib"));
        });
        Self {
            interpreter,
            entity_script: Default::default(),
            script_cache: Default::default(),
        }
    }
}

impl PythonScriptComponentSystem {
    pub(crate) async fn update(&mut self, dt: f32, scene: Weak<Mutex<Scene>>) {
        if SCENE.lock().unwrap().is_none() {
            *SCENE.lock().unwrap() = Some(scene.clone());
        }
        let python_entities = scene
            .upgrade()
            .expect("Unable to upgrade")
            .lock()
            .expect("Unable to lock")
            .get_entities_with_component::<dream_ecs::component::PythonScript>();
        for entity_id in python_entities {
            let mut script: String = include_str!("default-files/script.py").into();
            {
                let entity = Entity::from_handle(entity_id, scene.clone());
                if let Some(python_script_component) = entity.get_component::<PythonScript>() {
                    let resource_handle = python_script_component
                        .resource_handle
                        .as_ref()
                        .unwrap()
                        .upgrade();
                    let rh = resource_handle.unwrap();
                    let script_path = &rh.path;
                    let script_key = &rh.key;

                    if self.script_cache.contains_key(script_key) {
                        script = self
                            .script_cache
                            .get(script_key)
                            .expect("No cached script found")
                            .clone();
                    } else {
                        log::debug!("Caching script {}", script_path.to_str().unwrap());
                        let script_binary =
                            dream_fs::fs::read_binary(script_path.clone(), true).await;
                        if script_binary.is_ok() {
                            let script_binary = script_binary.unwrap().clone();
                            let script_str = String::from_utf8_lossy(&script_binary);
                            script = script_str.parse().unwrap();
                            self.script_cache.insert(script_key.clone(), script.clone());
                        }
                    }
                }
            }
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
                // TODO: only run all this if source code changed
                // step 1: compile python code and get name of class that is defined
                let code_obj = vm
                    .compile(
                        script.as_str(),
                        compiler::Mode::BlockExpr,
                        source_path.to_owned(),
                    )
                    .map_err(|err| vm.new_syntax_error(&err, None))
                    .unwrap();
                vm.run_code_obj(code_obj, scope)
                    .map(|entity_class_obj| {
                        let class_name_raw = entity_class_obj.get_attr("__name__", vm);
                        let class_name_py_str = class_name_raw.expect("No class name").str(vm);
                        let class_name = String::from(
                            class_name_py_str
                                .expect("Unable to convert class name to string")
                                .as_str(),
                        );
                        let scope = vm.new_scope_with_builtins();
                        let source_path;
                        cfg_if::cfg_if! {
                            if #[cfg(target_arch = "wasm32")] {
                                source_path = "<wasm>"
                            } else {
                                source_path = "<embedded>"
                            }
                        }
                        // step 2: run code that creates instance of class
                        let code_obj = vm
                            .compile(
                                &format!("{script}\n{class_name}({entity_id})"),
                                compiler::Mode::BlockExpr,
                                source_path.to_owned(),
                            )
                            .map_err(|err| vm.new_syntax_error(&err, None))
                            .unwrap();
                        vm.run_code_obj(code_obj, scope)
                            .map(|entity_script| {
                                // cuz this run code object only returns the class definition which we extract the class name from
                                self.entity_script
                                    .entry(entity_id)
                                    .or_insert(Some(entity_script));
                            })
                            .expect("Error running python code");
                        if let Ok(update) = self
                            .entity_script
                            .get(&entity_id)
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .get_attr("update", vm)
                        {
                            let args = vec![vm.ctx.new_float(dt as f64).into()];
                            let res = update.call(args, vm);
                            if let Err(..) = res {
                                let e = res.unwrap_err();
                                let py_err = e.get_arg(0).unwrap();
                                log::error!("{}", py_err.str(vm).unwrap());
                                let line_number = e.traceback().unwrap().lineno;
                                log::error!("line {}", line_number);
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
                    .expect("Error running python code");
            })
        }
    }
}

#[pymodule]
pub(crate) mod dream_py {
    use rustpython_vm::builtins::PyTypeRef;
    use rustpython_vm::TryFromBorrowedObject;

    use super::*;

    #[pyfunction]
    fn dream_entity(handle: u64, _vm: &VirtualMachine) -> PyResult<EntityInternal> {
        Ok(EntityInternal { handle })
    }

    #[pyfunction]
    fn dream_vec3(x: f64, y: f64, z: f64, _vm: &VirtualMachine) -> PyResult<Vector3Internal> {
        Ok(Vector3Internal { x, y, z })
    }

    #[pyattr]
    #[pyclass(module = "dream", name = "EntityInternal")]
    #[derive(Debug, PyPayload)]
    pub(crate) struct EntityInternal {
        pub(crate) handle: u64,
    }

    // impl Constructor for EntityInternal {
    //     type Args = FuncArgs;
    //
    //     fn py_new(cls: PyTypeRef, args: Self::Args, vm: &VirtualMachine) -> PyResult {
    //         let handle_arg = args.args.get(0);
    //         Ok(EntityInternal {
    //             handle: handle_arg
    //                 .unwrap()
    //                 .try_int(vm)
    //                 .unwrap()
    //                 .as_bigint()
    //                 .to_u64()
    //                 .unwrap(),
    //         }
    //         .to_pyobject(vm))
    //     }
    // }

    #[pyclass]
    impl EntityInternal {
        // TODO: this works... let's use this instead of the weird thing we are doing right now in dream_py.py
        // #[pyslot]
        // pub(crate) fn getattro(obj: &PyObject, name: &Py<PyStr>, vm: &VirtualMachine) -> PyResult {
        //     println!("object.__getattribute__({:?}, {:?})", obj, name);
        //     // obj.as_object().generic_getattr(name, vm)
        //     Ok(Vector3Internal {
        //         x: 0.,
        //         y: 0.,
        //         z: 0.,
        //     }
        //     .into_pyobject(vm))
        // }

        #[pymethod]
        fn get_handle(&self) -> PyResult<u64> {
            Ok(self.handle)
        }

        #[pymethod]
        fn get_transform(&self) -> PyResult<Transform> {
            let scene = SCENE.lock().unwrap().as_ref().unwrap().clone();
            let entity = dream_ecs::entity::Entity::from_handle(self.handle, scene);
            let transform: Option<dream_ecs::component::Transform> = entity.get_component();
            Ok(Transform::from(transform.expect("No transform component")))
        }

        #[inline]
        #[pymethod]
        fn get_position(&self) -> PyResult<Vector3Internal> {
            let scene = SCENE.lock().unwrap().as_ref().unwrap().clone();
            let entity = dream_ecs::entity::Entity::from_handle(self.handle, scene);
            let transform: Option<dream_ecs::component::Transform> = entity.get_component();
            let transform = transform.unwrap();
            Ok(Vector3Internal::from(transform.position))
        }

        #[inline]
        #[pymethod]
        fn set_position(&self, position: Vector3Internal) {
            let scene = SCENE.lock().unwrap().as_ref().unwrap().clone();
            let entity = dream_ecs::entity::Entity::from_handle(self.handle, scene);
            let transform: Option<dream_ecs::component::Transform> = entity.get_component();
            let mut transform = transform.unwrap();
            transform.position = dream_math::Vector3::from(position);
            entity.add_component(transform);
        }
    }

    impl TryFromBorrowedObject<'_> for EntityInternal {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> PyResult<Self> {
            let handle = obj.get_attr("handle", vm)?.try_into_value::<u64>(vm)?;
            Ok(EntityInternal { handle })
        }
    }

    #[pyattr]
    #[pyclass(module = "dream", name = "Transform")]
    #[derive(Debug, PyPayload)]
    struct Transform {
        position: Vector3Internal,
    }

    #[pyclass]
    impl Transform {
        #[pygetset]
        fn position(&self) -> PyResult<Vector3Internal> {
            Ok(self.position)
        }
    }

    impl From<dream_ecs::component::Transform> for Transform {
        fn from(transform: dream_ecs::component::Transform) -> Self {
            Transform {
                position: Vector3Internal::from(transform.position),
            }
        }
    }

    impl TryFromBorrowedObject<'_> for Transform {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> PyResult<Self> {
            let position = obj
                .get_attr("position", vm)?
                .try_into_value::<Vector3Internal>(vm)?;
            Ok(Transform { position })
        }
    }

    #[pyattr]
    #[pyclass(module = "dream", name = "Vector3Internal")]
    #[derive(Debug, Clone, Copy, PyPayload)]
    struct Vector3Internal {
        x: f64,
        y: f64,
        z: f64,
    }

    #[pyclass]
    impl Vector3Internal {
        #[pygetset]
        fn x(&self) -> f64 {
            self.x.clone()
        }

        #[pygetset]
        fn y(&self) -> f64 {
            self.y.clone()
        }

        #[pygetset]
        fn z(&self) -> f64 {
            self.z.clone()
        }
    }

    impl Constructor for Vector3Internal {
        type Args = (f64, f64, f64);

        fn py_new(_cls: PyTypeRef, args: Self::Args, vm: &VirtualMachine) -> PyResult {
            Self {
                x: args.0,
                y: args.1,
                z: args.2,
            }
            .to_pyresult(vm)
        }
    }

    impl From<Vector3Internal> for dream_math::Vector3 {
        fn from(vec3: Vector3Internal) -> Self {
            dream_math::Vector3 {
                x: vec3.x as f32,
                y: vec3.y as f32,
                z: vec3.z as f32,
            }
        }
    }

    impl From<dream_math::Vector3> for Vector3Internal {
        fn from(vec3: dream_math::Vector3) -> Self {
            Vector3Internal {
                x: vec3.x as f64,
                y: vec3.y as f64,
                z: vec3.z as f64,
            }
        }
    }

    impl TryFromBorrowedObject<'_> for Vector3Internal {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> PyResult<Self> {
            let x = obj
                .get_attr("x", vm)
                .expect("Unable to find x attribute")
                .try_float(vm)
                .expect("Unable to convert x to float")
                .to_f64();
            let y = obj
                .get_attr("y", vm)
                .expect("Unable to find y attribute")
                .try_float(vm)
                .expect("Unable to convert y to float")
                .to_f64();
            let z = obj
                .get_attr("z", vm)
                .expect("Unable to find z attribute")
                .try_float(vm)
                .expect("Unable to convert z to float")
                .to_f64();
            Ok(Vector3Internal { x, y, z })
        }
    }
}
