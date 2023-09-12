use std::path::PathBuf;

use wasm_bindgen::prelude::*;

use crate::fs::{FileKind, ReadDir};

#[wasm_bindgen(module = "/js/dream-fs.js")]
extern "C" {
    fn readBinary(file_path: &str) -> JsValue;
    fn readDir(file_path: &str) -> JsValue;
    fn fileExists(file_path: &str) -> JsValue;
    fn writeAll(file_path: &str, content: js_sys::Uint8Array);
}

#[allow(dead_code)]
pub async fn read_binary_from_web_storage(file_path: &str) -> Vec<u8> {
    // {
    //     // Open my_db v1
    //     let mut db_req: OpenDbRequest = IdbDatabase::open_u32("my_db", 1).expect("Err");
    //     db_req.set_on_upgrade_needed(Some(|evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
    //         // Check if the object store exists; create it if it doesn't
    //         if let None = evt.db().object_store_names().find(|n| n == "my_store") {
    //             evt.db().create_object_store("my_store").expect("Err");
    //         }
    //         Ok(())
    //     }));
    //
    //     let db: IdbDatabase = db_req.into_future().await.expect("Err");
    //
    //     // Insert/overwrite a record
    //     let tx: IdbTransaction = db
    //         .transaction_on_one_with_mode("my_store", IdbTransactionMode::Readwrite)
    //         .expect("Err");
    //     let store: IdbObjectStore = tx.object_store("my_store").expect("Err");
    //     let value_to_put: JsValue = JsValue::from("bar");
    //     store.put_key_val_owned("foo", &value_to_put).expect("Err");
    //     tx.await.into_result().expect("Err");
    // }

    js_sys::Uint8Array::from(readBinary(file_path)).to_vec()
}

#[allow(dead_code)]
pub fn read_dir_from_web_storage(file_path: PathBuf) -> Vec<ReadDir> {
    let js_val = readDir(
        file_path
            .to_str()
            .expect("Unable to get string for file path"),
    );
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
pub fn exists(file_path: PathBuf) -> bool {
    js_sys::Boolean::from(fileExists(
        file_path
            .to_str()
            .expect("Unable to get string for file path"),
    ))
    .into()
}

#[allow(dead_code)]
pub fn write_all_to_web_storage(file_path: PathBuf, content: Vec<u8>) {
    let content = js_sys::Uint8Array::from(content.as_slice());
    writeAll(
        file_path
            .to_str()
            .expect("Unable to convert file path to a string"),
        content,
    );
}
