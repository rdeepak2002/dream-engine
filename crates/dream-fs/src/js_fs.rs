#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(module = "/js/dream-fs.js")]
extern "C" {
    async fn readFileFromStorage(file_path: &str) -> JsValue;
}

#[cfg(target_arch = "wasm32")]
pub async fn read_file_from_web_storage(file_path: &str) -> Vec<u8> {
    let js_val_async = readFileFromStorage(file_path).await;
    let promise = js_sys::Promise::resolve(&js_val_async);
    let result = wasm_bindgen_futures::JsFuture::from(promise).await;
    let js_val = result.unwrap();
    let data = js_sys::Uint8Array::from(js_val);
    let vec_data: Vec<u8> = data.to_vec();
    return vec_data;
}
