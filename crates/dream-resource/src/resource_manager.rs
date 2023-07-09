use std::collections::{HashMap, VecDeque};
use std::ops::Add;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ResourceHandle {
    pub key: String,
    pub path: PathBuf,
}

impl ResourceHandle {
    pub fn new(key: String, path: PathBuf) -> Self {
        Self { key, path }
    }
}

pub struct ResourceManager {
    /// Map between guid and file path
    guid_to_filepath: HashMap<String, Arc<ResourceHandle>>,
}

#[derive(Serialize, Deserialize)]
struct MetaData {
    guid: String,
}

impl Default for MetaData {
    fn default() -> Self {
        Self {
            guid: Uuid::new_v4().to_string(),
        }
    }
}

pub fn create_meta_file(file_path: PathBuf) {
    let new_meta_data = MetaData::default();
    let res =
        serde_yaml::to_string(&new_meta_data).expect("Unable to generate json file for new guid");
    let meta_file_path = format!("{}{}", file_path.to_str().unwrap(), ".meta");
    let meta_file_path = PathBuf::from(meta_file_path.clone());
    log::warn!("meta file path {}", meta_file_path.to_str().unwrap());
    dream_fs::fs::write_binary(meta_file_path, res.into_bytes().to_vec());
}

fn get_meta_data(file_path: PathBuf) -> MetaData {
    let meta_file_path = format!("{}{}", file_path.to_str().unwrap(), ".meta");
    let meta_file_path = PathBuf::from(meta_file_path.clone());
    let bytes = dream_fs::fs::read_binary(meta_file_path.clone(), true).unwrap_or_else(|_| {
        panic!(
            "Unable to retrieve bytes for {}",
            meta_file_path.to_str().unwrap()
        )
    });
    serde_yaml::from_slice(bytes.as_slice()).expect("Unable to get meta data")
}

impl ResourceManager {
    pub fn new() -> Self {
        let mut guid_to_filepath = HashMap::default();
        Self { guid_to_filepath }
        // // traverse all files in project folder and map guid's to file paths
        // let mut guid_to_filepath = HashMap::default();
        // let project_root = dream_fs::fs::get_fs_root();
        //
        // let mut traversal_stack: VecDeque<PathBuf> = VecDeque::default();
        // traversal_stack.push_front(project_root);
        //
        // while !traversal_stack.is_empty() {
        //     let cur_dir = traversal_stack.pop_front().expect("Traversal queue empty");
        //     let files_in_dir = dream_fs::fs::read_dir(cur_dir).await;
        //     let read_dir_result =
        //         files_in_dir.expect("Error reading directory for resource manager traversal");
        //     for i in 0..read_dir_result.len() {
        //         let res = read_dir_result.get(i).unwrap();
        //         // populate map with guid : file path for non-meta data files
        //         let file_name = res.get_name();
        //         let file_path = res.get_path();
        //         if !file_name.ends_with(".meta")
        //             && !file_name.starts_with('.')
        //             && file_name != "files.json"
        //         {
        //             // let meta_file_path = file_path.push(".meta");
        //             let meta_file_path =
        //                 PathBuf::from(String::from(file_path.to_str().unwrap()).add(".meta"));
        //             if !dream_fs::fs::exists(meta_file_path.clone()).await {
        //                 // create meta file if it does not exist
        //                 log::warn!(
        //                     "Creating metafile for path {}",
        //                     file_path.clone().to_str().unwrap_or("none")
        //                 );
        //                 println!(
        //                     "Creating metafile for path {}",
        //                     file_path.clone().to_str().unwrap_or("none")
        //                 );
        //                 create_meta_file(file_path.clone()).await;
        //             }
        //             // get the guid from the meta file
        //             let meta_data = get_meta_data(file_path.clone()).await;
        //             let guid = meta_data.guid;
        //             guid_to_filepath
        //                 .insert(guid.clone(), Arc::new(ResourceHandle::new(guid, file_path)));
        //         }
        //         // if a directory is found, push it onto the traversal stack, so we will look into it
        //         if res.is_dir() {
        //             traversal_stack.push_front(res.get_path());
        //         }
        //     }
        // }
        //
        // Self { guid_to_filepath }
    }

    pub fn init(&mut self) {
        // traverse all files in project folder and map guid's to file paths
        let mut guid_to_filepath: HashMap<String, Arc<ResourceHandle>> = HashMap::default();
        let project_root = dream_fs::fs::get_fs_root();

        let mut traversal_stack: VecDeque<PathBuf> = VecDeque::default();
        traversal_stack.push_front(project_root);

        while !traversal_stack.is_empty() {
            let cur_dir = traversal_stack.pop_front().expect("Traversal queue empty");
            let files_in_dir = dream_fs::fs::read_dir(cur_dir);
            let read_dir_result =
                files_in_dir.expect("Error reading directory for resource manager traversal");
            for i in 0..read_dir_result.len() {
                let res = read_dir_result.get(i).unwrap();
                // populate map with guid : file path for non-meta data files
                let file_name = res.get_name();
                let file_path = res.get_path();
                if !file_name.ends_with(".meta")
                    && !file_name.starts_with('.')
                    && file_name != "files.json"
                {
                    // let meta_file_path = file_path.push(".meta");
                    let meta_file_path =
                        PathBuf::from(String::from(file_path.to_str().unwrap()).add(".meta"));
                    if !dream_fs::fs::exists(meta_file_path.clone()) {
                        // create meta file if it does not exist
                        log::warn!(
                            "Creating metafile for path {}",
                            file_path.clone().to_str().unwrap_or("none")
                        );
                        println!(
                            "Creating metafile for path {}",
                            file_path.clone().to_str().unwrap_or("none")
                        );
                        create_meta_file(file_path.clone());
                    }
                    // get the guid from the meta file
                    let meta_data = get_meta_data(file_path.clone());
                    let guid = meta_data.guid;
                    guid_to_filepath
                        .insert(guid.clone(), Arc::new(ResourceHandle::new(guid, file_path)));
                }
                // if a directory is found, push it onto the traversal stack, so we will look into it
                if res.is_dir() {
                    traversal_stack.push_front(res.get_path());
                }
            }
        }

        self.guid_to_filepath = guid_to_filepath;
    }

    pub fn get_resource(&self, key: String) -> Option<&Arc<ResourceHandle>> {
        self.guid_to_filepath.get(key.as_str())
    }
}
