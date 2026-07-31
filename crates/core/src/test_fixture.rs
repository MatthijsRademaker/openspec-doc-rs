use std::fs;

use tempfile::TempDir;

/// A temp directory containing each of `dirs` (relative paths), with no
/// `openspec/config.yaml`.
pub fn empty_dir(dirs: &[&str]) -> TempDir {
    let temp = TempDir::new().expect("temp dir");
    for dir in dirs {
        fs::create_dir_all(temp.path().join(dir)).expect("create fixture dir");
    }
    temp
}

/// A temp directory that is a valid OpenSpec project root, containing each of
/// `dirs` (relative paths).
pub fn project_fixture(dirs: &[&str]) -> TempDir {
    let temp = empty_dir(dirs);
    fs::create_dir_all(temp.path().join("openspec")).expect("create openspec dir");
    fs::write(temp.path().join("openspec/config.yaml"), "").expect("write config");
    temp
}
