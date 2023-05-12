use std::env;
use std::path::Path;

use anyhow::*;
use fs_extra::dir::CopyOptions;
use fs_extra::{copy_items, remove_items};

fn main() -> Result<()> {
    let example_folder = "examples";
    println!(
        "cargo:rerun-if-changed={}{}*",
        example_folder,
        std::path::MAIN_SEPARATOR
    );

    // delete old / previously-copied examples folder from the last time this script was run
    let old_example_folder = String::from(Path::new("web").join(example_folder).to_str().unwrap());
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
    let web_out_dir = Path::new("web");
    copy_items(&paths_to_copy, web_out_dir.to_str().unwrap(), &copy_options)?;

    Ok(())
}
