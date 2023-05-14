use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

pub struct ResourceManager {
    /// Map between guid and file path
    guid_to_filepath: HashMap<String, String>,
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
            // TODO: populate map with guid : file path
            for i in 0..read_dir_result.len() {
                let res = read_dir_result.get(i).unwrap();
                log::error!("read file from dir: {}", res.get_name());
                if res.is_dir() {
                    traversal_stack.push_front(res.get_path());
                }
            }
        }

        Self { guid_to_filepath }
    }
}
