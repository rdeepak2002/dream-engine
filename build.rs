use std::env;

use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;

fn main() -> Result<()> {
    // copy examples folder to out directory for desktop build
    let example_folder = "examples/";
    println!("cargo:rerun-if-changed={}*", example_folder);
    let out_dir = env::var("OUT_DIR")?;
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let paths_to_copy = vec![example_folder];
    copy_items(&paths_to_copy, out_dir, &copy_options)?;
    Ok(())
}
