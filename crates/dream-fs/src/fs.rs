use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::*;
use cfg_if::cfg_if;

static FS_ROOT: Mutex<Option<String>> = Mutex::new(None);

#[derive(PartialEq)]
pub enum FileKind {
    FILE,
    DIRECTORY,
}

#[derive(PartialEq)]
pub struct ReadDir {
    file_name: String,
    file_path: PathBuf,
    file_type: FileKind,
}

impl ReadDir {
    pub fn new(file_name: String, file_path: PathBuf, file_type: FileKind) -> Self {
        Self {
            file_name,
            file_path,
            file_type,
        }
    }

    pub fn get_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    pub fn get_name(&self) -> String {
        self.file_name.clone()
    }

    pub fn is_dir(&self) -> bool {
        self.file_type == FileKind::DIRECTORY
    }
}

pub fn set_fs_root(fs_root: &str) {
    log::debug!("Setting root directory to {}", fs_root);
    *FS_ROOT.lock().unwrap() = Some(String::from(fs_root));
}

pub fn get_fs_root() -> PathBuf {
    let fs_root = FS_ROOT
        .lock()
        .unwrap()
        .clone()
        .expect("No file system root specified");
    std::path::PathBuf::from(fs_root)
}

pub async fn read_binary(file_path: PathBuf, absolute: bool) -> Result<Vec<u8>> {
    let path = if absolute {
        file_path
    } else {
        get_full_path(file_path)
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

pub fn read_dir(file_path: PathBuf) -> Result<Vec<ReadDir>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let files_in_directory = crate::js_fs::read_dir_from_web_storage(file_path);
        } else {
            let mut files_in_directory: Vec<ReadDir> = Vec::new();
            let paths = std::fs::read_dir(file_path.clone()).unwrap_or_else(|_| panic!("Unable to read file paths in folder {}", file_path.to_str().unwrap_or("none")));
            for path in paths {
                let dir_entry = path.unwrap();
                let file_name = String::from(dir_entry.file_name().to_str().unwrap());
                let is_dir = dir_entry.file_type().unwrap().is_dir();
                let path_buf = dir_entry.path().to_path_buf();
                let file_kind = if is_dir { FileKind::DIRECTORY } else { FileKind::FILE };
                let read_dir = ReadDir::new(file_name, path_buf, file_kind);
                files_in_directory.push(read_dir);
            }
        }
    }
    Ok(files_in_directory)
}

pub async fn write_binary(file_path: PathBuf, content: Vec<u8>) {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            crate::js_fs::write_all_to_web_storage(file_path, content);
        } else {
            use std::io::Write;
            let mut file = std::fs::File::create(file_path.clone()).unwrap_or_else(|_| panic!("Unable to create file {}", file_path.to_str().unwrap()));
            let c: &[u8] = &content;
            file.write_all(c).unwrap_or_else(|_| panic!("Unable to write all to file {}", file_path.to_str().unwrap()));
        }
    }
}

pub fn exists(file_path: PathBuf) -> bool {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            crate::js_fs::exists(file_path)
        } else {
            file_path.into_boxed_path().exists()
        }
    }
}

pub fn get_full_path(file_path: PathBuf) -> PathBuf {
    let fs_root = FS_ROOT.lock().unwrap().clone();
    match fs_root {
        Some(root_path) => std::path::Path::new(&root_path).join(file_path),
        None => file_path,
    }
}
