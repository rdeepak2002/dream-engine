use std::env;

use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;

fn main() -> Result<()> {
    // rerun this script if something in examples/* changes.
    println!("cargo:rerun-if-changed=examples/*");

    // TODO: for web build copy examples folder rather than relying on shell script

    let out_dir = env::var("OUT_DIR")?;
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = Vec::new();
    paths_to_copy.push("examples/");
    copy_items(&paths_to_copy, out_dir, &copy_options)?;

    Ok(())
}
