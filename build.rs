use std::env;

use anyhow::*;
use fs_extra::dir::CopyOptions;
use fs_extra::{copy_items, remove_items};

fn main() -> Result<()> {
    let example_folder = "examples/";
    println!("cargo:rerun-if-changed={}*", example_folder);

    // remove old web build
    let old_example_folder = format!("web/{}", example_folder);
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
    let web_out_dir = std::path::Path::new("web");
    copy_items(&paths_to_copy, web_out_dir.to_str().unwrap(), &copy_options)?;

    Ok(())
}
