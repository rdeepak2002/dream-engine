use std::collections::HashMap;

use log::warn;

pub struct ResourceManager {
    /// Map between guid and file path
    guid_to_filepath: HashMap<String, String>,
}

impl ResourceManager {
    pub async fn new() -> Self {
        // traverse all files in project folder and map guid's to file paths
        let guid_to_filepath = HashMap::default();
        let project_root = dream_fs::fs::get_fs_root();
        let files_in_dir = dream_fs::fs::read_dir(project_root).await;
        let read_dir_result = files_in_dir.unwrap();
        for res in read_dir_result {
            log::warn!("read file from dir: {}", res.get_name());
        }

        Self { guid_to_filepath }
    }
}
