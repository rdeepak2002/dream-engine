use std::sync::Mutex;

use anyhow::*;
use cfg_if::cfg_if;

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
            let data = crate::js_fs::read_binary_from_web_storage(path.to_str().unwrap()).await;
        } else {
            let data = std::fs::read(path)?;
        }
    }
    Ok(data)
}

pub async fn read_dir(file_path: std::path::PathBuf) -> Result<Vec<std::path::PathBuf>> {
    let mut files_in_directory = Vec::new();

    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {

        } else {
            let paths = std::fs::read_dir(file_path).unwrap();
            for path in paths {
                let path_buf = path.unwrap().path().to_path_buf();
                files_in_directory.push(path_buf);
            }
        }
    }

    return Ok(files_in_directory);
}
