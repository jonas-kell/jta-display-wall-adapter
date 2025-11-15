use jta_display_wall_adapter::build_data::ImagesStorage;
use std::{fs, path::PathBuf};

fn main() {
    // rerun only when these change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/");

    let storage = ImagesStorage::new_with_compile_data();
    let bytes = storage.to_bytes();

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = PathBuf::from(out_dir).join("image_storage.bin");

    fs::write(&dest, bytes).unwrap();
}
