use std::path::Path;

pub fn display_path(path: &Path) -> String {
    path.display().to_string()
}
