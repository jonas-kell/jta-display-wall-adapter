use std::io::Write;
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub fn make_sure_folder_exists(path: &Path) -> Result<(), String> {
    if path.exists() {
        if path.is_dir() {
            Ok(())
        } else {
            Err(format!(
                "Path '{}' exists but is not a directory",
                path.display()
            ))
        }
    } else {
        std::fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create directory '{}': {}", path.display(), e))
    }
}

pub fn create_file_if_not_there_and_write(path: &Path, content: &str) -> Result<(), String> {
    match std::fs::File::create(path) {
        Err(e) => Err(format!("Could not create file: {}", e.to_string())),
        Ok(mut file) => match write!(file, "{}", content) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Could not write to file: {}", e.to_string())),
        },
    }
}

#[cfg(target_os = "linux")]
pub fn set_perms(path: &Path) {
    let perms = std::fs::Permissions::from_mode(0o777);
    let _ = std::fs::set_permissions(path, perms);
}

#[cfg(not(target_os = "linux"))]
pub fn set_perms(_path: &Path) {}
