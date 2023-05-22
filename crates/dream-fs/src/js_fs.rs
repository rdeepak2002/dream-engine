use std::path::PathBuf;

use wasm_bindgen::prelude::*;

use crate::fs::{FileKind, ReadDir};

#[wasm_bindgen(module = "/js/dream-fs.js")]
extern "C" {
    async fn readBinary(file_path: &str) -> JsValue;
    async fn readString(file_path: &str) -> JsValue;
    async fn readDir(file_path: &str) -> JsValue;
    async fn fileExists(file_path: &str) -> JsValue;
    async fn writeAll(file_path: &str, content: js_sys::Uint8Array);
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
pub async fn read_string_from_web_storage(file_path: &str) -> String {
    let js_val_async = readString(file_path).await;
    let promise = js_sys::Promise::resolve(&js_val_async);
    let result = wasm_bindgen_futures::JsFuture::from(promise).await;
    let js_val = result.unwrap();
    js_val
        .as_string()
        .expect("Unable to convert javascript value to string")
}

#[allow(dead_code)]
pub async fn read_dir_from_web_storage(file_path: PathBuf) -> Vec<ReadDir> {
    let js_val_async = readDir(
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

#[allow(dead_code)]
pub async fn exists(file_path: PathBuf) -> bool {
    let js_val_async = fileExists(
        file_path
            .to_str()
            .expect("Unable to get string for file path"),
    )
    .await;
    let promise = js_sys::Promise::resolve(&js_val_async);
    let result = wasm_bindgen_futures::JsFuture::from(promise).await;
    let js_val = result.unwrap();
    let data = js_sys::Boolean::from(js_val);
    data.into()
}

#[allow(dead_code)]
pub async fn write_all_to_web_storage(file_path: PathBuf, content: Vec<u8>) {
    let content = js_sys::Uint8Array::from(content.as_slice());
    writeAll(
        file_path
            .to_str()
            .expect("Unable to convert file path to a string"),
        content,
    )
    .await;
}
