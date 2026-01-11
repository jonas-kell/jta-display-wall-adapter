use images_core::images::{IconsStorage, ImagesStorage};
use std::{fs, path::PathBuf};

fn main() {
    // rerun only when these change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../images_core/");
    println!("cargo:rerun-if-changed=../assets/");

    println!("cargo:warning=Building image cache now!");
    let sizes_to_pre_cache = [(360, 120)];
    let storage = ImagesStorage::new_with_compile_data(&sizes_to_pre_cache);
    let sizes_to_pre_cache_icons = [(30, 20)];
    let storage_icons = IconsStorage::new_with_compile_data(&sizes_to_pre_cache_icons);
    println!("cargo:warning=Files loaded");
    let bytes = storage.to_bytes();
    let bytes_icons = storage_icons.to_bytes();
    println!("cargo:warning=Converted to bytes");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = PathBuf::from(out_dir.clone()).join("image_storage.bin");
    let dest_icons = PathBuf::from(out_dir).join("icon_storage.bin");
    fs::write(&dest, bytes).unwrap();
    fs::write(&dest_icons, bytes_icons).unwrap();
    println!("cargo:warning=Written for consumption");
}
