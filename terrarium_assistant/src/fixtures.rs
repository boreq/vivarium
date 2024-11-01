use std::path::Path;
use std::path::PathBuf;

pub fn test_file_path(relative_path: &str) -> Box<Path> {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push(relative_path);
    d.into_boxed_path()
}
