use actix_web::{web, HttpResponse, Responder};
use include_dir::{include_dir, Dir};
use mime_guess::from_path;
use std::collections::HashMap;

const STATIC_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/web_client/dist");

pub fn cache_static_files() -> HashMap<String, &'static [u8]> {
    let mut file_map = HashMap::new();

    fn add_files_recursively(dir: &'static Dir, file_map: &mut HashMap<String, &'static [u8]>) {
        for file in dir.files() {
            if let Some(path) = file.path().to_str() {
                file_map.insert(String::from(path), file.contents());
            }
        }

        for subdir in dir.dirs() {
            add_files_recursively(subdir, file_map);
        }
    }

    // Start scanning from the root directory with an empty base path.
    add_files_recursively(&STATIC_FILES, &mut file_map);

    file_map
}

pub async fn static_handler(
    file_map: web::Data<HashMap<String, &'static [u8]>>,
    path: web::Path<String>,
) -> impl Responder {
    let path = path.into_inner();
    let file_map = file_map.get_ref();
    println!("Serving statically {}", path);

    // Try to resolve the requested path or default to `index.html`.
    let file_path = if path.is_empty() || path == "/" {
        "index.html"
    } else {
        path.as_str()
    };

    if let Some(content) = file_map.get(file_path) {
        let mime_type = from_path(file_path).first_or_octet_stream();
        HttpResponse::Ok()
            .content_type(mime_type.as_ref())
            .body(*content)
    } else {
        HttpResponse::NotFound().body("404 Not Found")
    }
}
