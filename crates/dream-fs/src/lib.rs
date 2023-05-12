use std::sync::Mutex;

use anyhow::*;
use cfg_if::cfg_if;

pub mod js_fs;

static FS_ROOT: Mutex<Option<String>> = Mutex::new(None);

pub fn set_fs_root(fs_root: &str) {
    log::warn!("Setting root directory to {}", fs_root);
    *FS_ROOT.lock().unwrap() = Some(String::from(fs_root));
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        // TODO: use FS_ROOT for web build
        if #[cfg(target_arch = "wasm32")] {
            let data = crate::js_fs::read_file_from_web_storage(file_name).await;
        } else {
            let fs_root = FS_ROOT.lock().unwrap().clone();
            let path = match fs_root {
                Some(root_path) =>
                    std::path::Path::new(&root_path).join(file_name)
                ,
                None =>
                    std::path::Path::new(file_name).to_path_buf()
            };
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}
