use std::collections::VecDeque;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::*;
use fs_extra::dir::CopyOptions;
use fs_extra::{copy_items, remove_items};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct FileIdentifier {
    filepath: String,
    #[serde(rename(serialize = "fileUrl", deserialize = "fileUrl"))]
    file_url: Option<String>,
}

impl FileIdentifier {
    pub fn new(filepath: String) -> Self {
        Self {
            filepath,
            file_url: None,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let example_folder_path_buf = Path::new("..").join("..").join("examples");
    let example_folder = example_folder_path_buf
        .to_str()
        .expect("Unable to genereate string for examples folder location");
    // println!(
    //     "cargo:rerun-if-changed={}{}*",
    //     example_folder,
    //     std::path::MAIN_SEPARATOR
    // );

    // generate JSON file keeping track of paths of all available files (used for web build at the moment)
    let example_project_name = "blank";
    let project_root = PathBuf::from(example_folder).join(example_project_name);
    let mut traversal_stack: VecDeque<PathBuf> = VecDeque::default();
    traversal_stack.push_front(project_root.clone());
    let mut files = Vec::new();
    while !traversal_stack.is_empty() {
        let cur_dir = traversal_stack.pop_front().expect("Traversal queue empty");
        let files_in_dir = std::fs::read_dir(cur_dir).unwrap();
        for dir_entry in files_in_dir {
            let entry = dir_entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                traversal_stack.push_front(entry.path().to_path_buf());
            } else {
                let entry_path = entry.path().clone().to_path_buf().clone();
                let stripped_path = entry_path
                    .strip_prefix(project_root.clone())
                    .expect("Unable to strip prefix of project file path")
                    .to_path_buf();
                let file_identifier =
                    FileIdentifier::new(String::from(stripped_path.to_str().unwrap()));
                files.push(file_identifier);
            }
        }
    }
    std::fs::write(
        project_root.join("files.json").to_str().unwrap(),
        serde_json::to_string_pretty(&files).unwrap(),
    )
    .unwrap();

    // delete old / previously-copied examples folder from the last time this script was run
    let old_example_folder = String::from(
        Path::new("..")
            .join("..")
            .join("web")
            .join("examples")
            .to_str()
            .unwrap(),
    );
    let paths_to_remove = vec![old_example_folder];
    remove_items(&paths_to_remove).expect("unable to remove paths");

    // paths to copy
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let paths_to_copy = vec![example_folder];

    // copy examples folder to out directory for desktop build
    let out_dir = env::var("OUT_DIR")?;
    copy_items(&paths_to_copy, out_dir, &copy_options)?;

    // copy examples folder to out directory for web build
    let web_out_dir = Path::new("..").join("..").join("web");
    copy_items(&paths_to_copy, web_out_dir.to_str().unwrap(), &copy_options)?;

    Ok(())
}
