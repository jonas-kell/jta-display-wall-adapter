use images_core::images::ImagesStorage;
use std::{fs, path::PathBuf};

fn main() {
    // rerun only when these change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=images_core/");
    println!("cargo:rerun-if-changed=assets/");

    println!("cargo:warning=Building image cache now!");
    let sizes_to_pre_cache = [(360, 120)];
    let storage = ImagesStorage::new_with_compile_data(&sizes_to_pre_cache);
    println!("cargo:warning=Files loaded");
    let bytes = storage.to_bytes();
    println!("cargo:warning=Converted to bytes");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = PathBuf::from(out_dir).join("image_storage.bin");
    fs::write(&dest, bytes).unwrap();
    println!("cargo:warning=Written for consumption");
}
