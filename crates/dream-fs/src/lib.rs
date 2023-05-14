use std::sync::Mutex;

use anyhow::*;
use cfg_if::cfg_if;

pub mod js_fs;

static FS_ROOT: Mutex<Option<String>> = Mutex::new(None);

pub fn set_fs_root(fs_root: &str) {
    log::warn!("Setting root directory to {}", fs_root);
    *FS_ROOT.lock().unwrap() = Some(String::from(fs_root));
}

pub fn get_fs_root() -> std::path::PathBuf {
    let fs_root = FS_ROOT
        .lock()
        .unwrap()
        .clone()
        .expect("No file system root specified");
    std::path::PathBuf::from(fs_root)
}

pub async fn read_binary(file_path: std::path::PathBuf) -> Result<Vec<u8>> {
    let fs_root = FS_ROOT.lock().unwrap().clone();
    let path = match fs_root {
        Some(root_path) => std::path::Path::new(&root_path).join(file_path),
        None => file_path,
    };
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let data = crate::js_fs::read_file_from_web_storage(path.to_str().unwrap()).await;
        } else {
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}
