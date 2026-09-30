use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn try_load_file(path: &Path) -> Option<String> {
    if let Ok(content) = fs::read_to_string(path) {
        Some(content)
    } else if let Ok(bytes) = fs::read(path) {
        Some(String::from_utf8_lossy(&bytes).to_string())
    } else {
        None
    }
}

pub(crate) fn get_startup_file() -> (String, Option<PathBuf>) {
    if let Some(arg) = std::env::args().nth(1) {
        let path = PathBuf::from(&arg);
        if path.is_file() {
            if let Some(content) = try_load_file(&path) {
                return (content, Some(path));
            }
        }
    }
    ("".to_owned(), None)
}
