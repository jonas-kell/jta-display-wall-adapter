use images_cache_builder::{STORAGE_BYTES_ICONS_LOCATION, STORAGE_BYTES_LOCATION};

fn main() {
    // rerun only when these change
    println!("cargo:rerun-if-changed=build.rs");

    // runtime vars to compile time
    println!("cargo:rustc-env=STORAGE_BYTES_LOCATION={STORAGE_BYTES_LOCATION}");
    println!("cargo:rustc-env=STORAGE_BYTES_ICONS_LOCATION={STORAGE_BYTES_ICONS_LOCATION}");
}
