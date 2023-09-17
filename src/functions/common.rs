use std::path::Path;

pub fn is_file_path(path: &str) -> bool {
    let path = path.trim();
    let path = Path::new(path);
    path.exists() && path.is_file()
}
