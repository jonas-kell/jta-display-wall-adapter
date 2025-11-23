use std::io::Write;
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub fn make_sure_folder_exists(path: &Path) -> Result<(), String> {
    if path.exists() {
        if path.is_dir() {
            set_perms(path);
            Ok(())
        } else {
            Err(format!(
                "Path '{}' exists but is not a directory",
                path.display()
            ))
        }
    } else {
        std::fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create directory '{}': {}", path.display(), e))?;
        set_perms(path);
        Ok(())
    }
}

pub fn read_image_files(folder_path: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    make_sure_folder_exists(folder_path)?;

    let mut image_files: Vec<(String, Vec<u8>)> = Vec::new();

    for entry in std::fs::read_dir(folder_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;

        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| String::from("Could not convert filename to string..."))?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                match ext.to_lowercase().as_str() {
                    "jpg" | "jpeg" | "png" => {
                        let data = std::fs::read(&path).map_err(|e| e.to_string())?;

                        image_files.push((name, data));
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(image_files)
}

pub fn create_file_if_not_there_and_write(path: &Path, content: &str) -> Result<(), String> {
    match std::fs::File::create(path) {
        Err(e) => Err(format!("Could not create file: {}", e.to_string())),
        Ok(mut file) => {
            set_perms(path);
            match write!(file, "{}", content) {
                Ok(()) => Ok(()),
                Err(e) => Err(format!("Could not write to file: {}", e.to_string())),
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub fn set_perms(path: &Path) {
    let perms = std::fs::Permissions::from_mode(0o777);
    let _ = std::fs::set_permissions(path, perms);
}

#[cfg(not(target_os = "linux"))]
pub fn set_perms(_path: &Path) {}
