use std::collections::HashMap;

pub struct ResourceManager {
    /// Map between guid and file path
    guid_to_filepath: HashMap<String, String>,
}

impl Default for ResourceManager {
    fn default() -> Self {
        // traverse all files in project folder and map guid's to file paths
        let guid_to_filepath = HashMap::default();
        let project_root = dream_fs::fs::get_fs_root();

        dream_fs::fs::read_dir(project_root);

        Self { guid_to_filepath }
    }
}
