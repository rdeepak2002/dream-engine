#[cfg(target_arch = "wasm32")]
use std::io::{Read, Seek, SeekFrom};

use cfg_if::cfg_if;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_file_reader::WebSysFile;
#[cfg(target_arch = "wasm32")]
use web_sys::console;

#[cfg(target_arch = "wasm32")]
use crate::js_fs::read_file_from_web_storage;

pub mod js_fs;

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let data = read_file_from_web_storage(file_name).await;
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}