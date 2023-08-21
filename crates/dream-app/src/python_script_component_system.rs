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
    vm, AsObject, Interpreter, PyObject, PyObjectRef, PyPayload, PyResult, TryFromBorrowedObject,
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
            vm.add_native_module("dream".to_owned(), Box::new(dream_py::make_module));
            vm.add_frozen(rustpython_vm::py_freeze!(dir = "src/default-files"));
        });
        Self {
            interpreter,
            entity_script: Default::default(),
        }
    }
}

impl System for PythonScriptComponentSystem {
    fn update(&mut self, dt: f32, scene: Weak<Mutex<Scene>>) {
        // TODO: this method requires significant clean-up
        // general idea:
        // 1. compile code and get name of class
        // 2. compile modified version of code where a new line is added which instantiates the class

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
                        // TODO: no reason to store this in the map... instead store the actual object generate by the next run_code_object
                        // cuz this run code object only returns the class definition which we extract the class name from
                        self.entity_script.entry(entity_id).or_insert(Some(value));
                    })
                    .expect("Error running python code");
                let entity_script = self
                    .entity_script
                    .get(&entity_id)
                    .unwrap()
                    .as_ref()
                    .unwrap();
                let class_name_raw = entity_script.get_attr("__name__", vm);
                let x = class_name_raw.expect("No class name").str(vm);
                let class_name =
                    String::from(x.expect("Unable to convert class name to string").as_str());

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
                    .compile(
                        &format!("{script}\n{class_name}({entity_id})"),
                        compiler::Mode::BlockExpr,
                        source_path.to_owned(),
                    )
                    .map_err(|err| vm.new_syntax_error(&err, None))
                    .unwrap();

                vm.run_code_obj(code_obj, scope)
                    .map(|entity_script| {
                        if let Ok(update) = entity_script.get_attr("update", vm) {
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
                    })
                    .expect("Error running python code");

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
    use std::any::Any;

    use cgmath::num_traits::ToPrimitive;
    use rustpython_vm::builtins::{PyGenericAlias, PyStr, PyStrInterned, PyTypeRef};
    use rustpython_vm::protocol::PyNumberMethods;
    use rustpython_vm::types::AsNumber;
    use rustpython_vm::{
        builtins::PyList, convert::ToPyObject, Py, PyObjectRef, TryFromBorrowedObject,
    };

    use dream_math::Vector3;

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

    impl AsNumber for Vector3Internal {
        fn as_number() -> &'static PyNumberMethods {
            &PyNumberMethods {
                add: Some(|a, b, vm| {
                    let a = Vector3Internal::try_from_borrowed_object(vm, a).unwrap();
                    let b = Vector3Internal::try_from_borrowed_object(vm, b).unwrap();
                    let res: Self =
                        (dream_math::Vector3::from(a) + dream_math::Vector3::from(b)).into();
                    res.to_pyresult(vm)
                }),
                subtract: Some(|a, b, vm| {
                    let a = Vector3Internal::try_from_borrowed_object(vm, a).unwrap();
                    let b = Vector3Internal::try_from_borrowed_object(vm, b).unwrap();
                    let res: Self =
                        (dream_math::Vector3::from(a) - dream_math::Vector3::from(b)).into();
                    res.to_pyresult(vm)
                }),
                multiply: Some(|a, b, vm| {
                    // scenario where a is a scalar
                    if let a = a.try_float(vm) {
                        let a = a.unwrap().to_f64() as f32;
                        let b = Vector3Internal::try_from_borrowed_object(vm, b).unwrap();
                        let res: Self = (a * dream_math::Vector3::from(b)).into();
                        return res.to_pyresult(vm);
                    }

                    // scenario where b is a scalar
                    if let b = b.try_float(vm) {
                        let a = Vector3Internal::try_from_borrowed_object(vm, a).unwrap();
                        let b = b.unwrap().to_f64() as f32;
                        let res: Self = (dream_math::Vector3::from(a) * b).into();
                        return res.to_pyresult(vm);
                    }

                    // scenario where both a and b are vectors
                    let a = Vector3Internal::try_from_borrowed_object(vm, a).unwrap();
                    let b = Vector3Internal::try_from_borrowed_object(vm, b).unwrap();
                    let res: Self =
                        (dream_math::Vector3::from(a) * dream_math::Vector3::from(b)).into();
                    res.to_pyresult(vm)
                }),
                // power: Some(|a, b, c, vm| {...}),
                // negative: Some(|num, vm| (&PyInt::number_downcast(num).value).neg().to_pyresult(vm)),
                // positive: Some(|num, vm| Ok(PyInt::number_downcast_exact(num, vm).into())),
                // absolute: Some(|num, vm| PyInt::number_downcast(num).value.abs().to_pyresult(vm)),
                // invert: Some(|num, vm| (&PyInt::number_downcast(num).value).not().to_pyresult(vm)),
                // floor_divide: Some(|a, b, vm| PyInt::number_op(a, b, inner_floordiv, vm)),
                // true_divide: Some(|a, b, vm| PyInt::number_op(a, b, inner_truediv, vm)),
                ..PyNumberMethods::NOT_IMPLEMENTED
            }
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
