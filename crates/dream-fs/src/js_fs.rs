use std::path::PathBuf;

use wasm_bindgen::prelude::*;

use crate::fs::{FileKind, ReadDir};

#[wasm_bindgen(module = "/js/dream-fs.js")]
extern "C" {
    async fn readBinary(file_path: &str) -> JsValue;
    async fn readDir(file_path: &str) -> JsValue;
}

#[allow(dead_code)]
pub async fn read_binary_from_web_storage(file_path: &str) -> Vec<u8> {
    let js_val_async = readBinary(file_path).await;
    let promise = js_sys::Promise::resolve(&js_val_async);
    let result = wasm_bindgen_futures::JsFuture::from(promise).await;
    let js_val = result.unwrap();
    let data = js_sys::Uint8Array::from(js_val);
    data.to_vec()
}

#[allow(dead_code)]
pub async fn read_dir_from_web_storage(file_path: PathBuf) -> Vec<ReadDir> {
    let js_val_async = readBinary(
        file_path
            .to_str()
            .expect("Unable to get string for file path"),
    )
    .await;
    let promise = js_sys::Promise::resolve(&js_val_async);
    let result = wasm_bindgen_futures::JsFuture::from(promise).await;
    let js_val = result.unwrap();
    let data = js_sys::Array::from(&js_val);
    let mut vec_data: Vec<ReadDir> = Default::default();
    for i in 0..data.length() {
        let val: JsValue = data.at(i.try_into().unwrap());
        let val = js_sys::Array::from(&val);
        let file_name = val
            .at(0)
            .as_string()
            .expect("Unable to unwrap directory name as string");
        let is_dir = js_sys::Boolean::from(val.at(1)).value_of();
        let path_buf = file_path.join(PathBuf::from(file_name.clone()));
        let file_kind = if is_dir {
            FileKind::DIRECTORY
        } else {
            FileKind::FILE
        };
        let read_dir = ReadDir::new(file_name.clone(), path_buf, file_kind);
        vec_data.push(read_dir);
    }
    vec_data
}
