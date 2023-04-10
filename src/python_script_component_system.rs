#![allow(unused_mut)]

use rustpython_vm as vm;

pub fn test_python() {
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
