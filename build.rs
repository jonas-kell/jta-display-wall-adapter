use images_cache_builder::{STORAGE_BYTES_ICONS_LOCATION, STORAGE_BYTES_LOCATION};
use std::env;

fn main() {
    // runtime vars to compile time
    println!("cargo:rustc-env=STORAGE_BYTES_LOCATION={STORAGE_BYTES_LOCATION}");
    println!("cargo:rustc-env=STORAGE_BYTES_ICONS_LOCATION={STORAGE_BYTES_ICONS_LOCATION}");
    println!("cargo:rustc-env=PROFILE={}", env::var("PROFILE").unwrap());
}
