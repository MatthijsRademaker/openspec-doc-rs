use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::root::Project;

const CHANGES_DIR: &str = "changes";
const ARCHIVE_DIR: &str = "archive";
const SPECS_DIR: &str = "specs";

/// A discovered directory, identified by its own directory name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedDir {
    pub name: String,
    pub path: PathBuf,
}

/// The directory structure of an OpenSpec project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scan {
    pub project: Project,
    pub active_changes: Vec<NamedDir>,
    pub archived_changes: Vec<NamedDir>,
    pub specs: Vec<NamedDir>,
}

/// Enumerate active changes, archived changes, and current specs of `project`.
pub fn scan(project: Project) -> Result<Scan, Error> {
    let changes_dir = project.openspec_dir.join(CHANGES_DIR);

    let mut active_changes = read_dirs(&changes_dir)?;
    active_changes.retain(|dir| dir.name != ARCHIVE_DIR);

    let archived_changes = read_dirs(&changes_dir.join(ARCHIVE_DIR))?;
    let specs = read_dirs(&project.openspec_dir.join(SPECS_DIR))?;

    Ok(Scan {
        project,
        active_changes,
        archived_changes,
        specs,
    })
}

/// Subdirectories of `dir`, sorted by name. A missing `dir` yields an empty
/// list; any other read failure propagates.
fn read_dirs(dir: &Path) -> Result<Vec<NamedDir>, Error> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(Error::io(dir, source)),
    };

    let mut dirs = Vec::new();
    for entry in entries {
        let path = entry.map_err(|source| Error::io(dir, source))?.path();

        // Follows symlinks, so a symlinked change directory still counts.
        let metadata = fs::metadata(&path).map_err(|source| Error::io(&path, source))?;
        if !metadata.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| Error::NonUtf8Path { path: path.clone() })?
            .to_owned();

        dirs.push(NamedDir { name, path });
    }

    dirs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(dirs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::root::project_at;
    use crate::test_fixture::project_fixture;

    fn names(dirs: &[NamedDir]) -> Vec<&str> {
        dirs.iter().map(|dir| dir.name.as_str()).collect()
    }

    fn scan_fixture(dirs: &[&str]) -> Scan {
        let fixture = project_fixture(dirs);
        let project = project_at(fixture.path()).expect("project root");
        scan(project).expect("scan")
    }

    #[test]
    fn active_changes_exclude_the_archive_directory() {
        let result = scan_fixture(&[
            "openspec/changes/add-b",
            "openspec/changes/add-a",
            "openspec/changes/archive/2026-01-01-add-old",
        ]);

        assert_eq!(names(&result.active_changes), ["add-a", "add-b"]);
    }

    #[test]
    fn archived_changes_are_discovered_separately() {
        let result = scan_fixture(&[
            "openspec/changes/add-a",
            "openspec/changes/archive/2026-01-02-add-second",
            "openspec/changes/archive/2026-01-01-add-first",
        ]);

        assert_eq!(
            names(&result.archived_changes),
            ["2026-01-01-add-first", "2026-01-02-add-second"]
        );
        assert_eq!(names(&result.active_changes), ["add-a"]);
    }

    #[test]
    fn specs_are_listed_by_directory_name() {
        let result = scan_fixture(&[
            "openspec/specs/project-scanner",
            "openspec/specs/cli-surface",
        ]);

        assert_eq!(names(&result.specs), ["cli-surface", "project-scanner"]);
    }

    #[test]
    fn missing_directories_yield_empty_lists() {
        let result = scan_fixture(&[]);

        assert!(result.active_changes.is_empty());
        assert!(result.archived_changes.is_empty());
        assert!(result.specs.is_empty());
    }

    #[test]
    fn empty_directories_yield_empty_lists() {
        let result = scan_fixture(&["openspec/changes/archive", "openspec/specs"]);

        assert!(result.active_changes.is_empty());
        assert!(result.archived_changes.is_empty());
        assert!(result.specs.is_empty());
    }

    #[test]
    fn files_are_not_reported_as_directories() {
        let fixture = project_fixture(&["openspec/changes/add-a"]);
        fs::write(fixture.path().join("openspec/changes/README.md"), "stray").unwrap();

        let project = project_at(fixture.path()).expect("project root");
        let result = scan(project).expect("scan");

        assert_eq!(names(&result.active_changes), ["add-a"]);
    }
}
