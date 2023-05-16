use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub struct ResourceManager {
    /// Map between guid and file path
    guid_to_filepath: HashMap<String, PathBuf>,
}

#[derive(Serialize, Deserialize)]
struct MetaData {
    guid: String,
}

pub async fn create_meta_file(file_path: PathBuf) {}

pub async fn get_meta_data(file_path: PathBuf) -> MetaData {
    let binary = dream_fs::fs::read_binary(file_path).await.unwrap();
    serde_yaml::from_slice::<MetaData>(&binary).expect("Unable to deserialize meta data file")
}

impl ResourceManager {
    pub async fn new() -> Self {
        // traverse all files in project folder and map guid's to file paths
        let mut guid_to_filepath = HashMap::default();
        let project_root = dream_fs::fs::get_fs_root();

        let mut traversal_stack: VecDeque<PathBuf> = VecDeque::default();
        traversal_stack.push_front(project_root);

        while !traversal_stack.is_empty() {
            let cur_dir = traversal_stack.pop_front().expect("Traversal queue empty");
            let files_in_dir = dream_fs::fs::read_dir(cur_dir).await;
            let read_dir_result =
                files_in_dir.expect("Error reading directory for resource manager traversal");
            for i in 0..read_dir_result.len() {
                let res = read_dir_result.get(i).unwrap();
                log::error!("read file from dir: {}", res.get_name());
                if res.is_dir() {
                    traversal_stack.push_front(res.get_path());
                } else {
                    // populate map with guid : file path for non-meta data files
                    let file_name = res.get_name();
                    let file_path = res.get_path();
                    if !file_name.ends_with(".meta") {
                        let meta_file_path = file_path.join(".meta");
                        if !dream_fs::fs::exists(meta_file_path).await {}
                        // get the guid from the meta file
                        let meta_data = get_meta_data(file_path.clone()).await;
                        let guid = meta_data.guid;
                        guid_to_filepath.insert(guid, file_path);
                    }
                }
            }
        }

        Self { guid_to_filepath }
    }
}
