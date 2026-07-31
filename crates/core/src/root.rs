use std::path::{Path, PathBuf};

use crate::error::Error;

const OPENSPEC_DIR: &str = "openspec";
const CONFIG_FILE: &str = "config.yaml";

/// An OpenSpec project root: a directory containing `openspec/config.yaml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub root: PathBuf,
    pub openspec_dir: PathBuf,
    pub config_path: PathBuf,
}

impl Project {
    fn at(root: PathBuf) -> Self {
        let openspec_dir = root.join(OPENSPEC_DIR);
        let config_path = openspec_dir.join(CONFIG_FILE);
        Self {
            root,
            openspec_dir,
            config_path,
        }
    }
}

/// Treat `root` as the project root without searching parent directories.
pub fn project_at(root: &Path) -> Result<Project, Error> {
    let project = Project::at(canonicalize(root)?);

    if !project.config_path.is_file() {
        return Err(Error::MissingConfig {
            config_path: project.config_path,
        });
    }

    Ok(project)
}

/// Walk up from `start` and return the first ancestor containing
/// `openspec/config.yaml`, stopping at the filesystem root.
pub fn find_project(start: &Path) -> Result<Project, Error> {
    let start = canonicalize(start)?;

    for candidate in start.ancestors() {
        let project = Project::at(candidate.to_path_buf());
        if project.config_path.is_file() {
            return Ok(project);
        }
    }

    Err(Error::NoProjectFound { start })
}

fn canonicalize(path: &Path) -> Result<PathBuf, Error> {
    path.canonicalize()
        .map_err(|source| Error::io(path, source))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixture::{empty_dir, project_fixture};

    #[test]
    fn explicit_root_is_used_without_walking_up() {
        let fixture = project_fixture(&["openspec/changes/nested-child"]);
        let nested = fixture.path().join("openspec/changes/nested-child");

        let project = project_at(fixture.path()).expect("explicit root");

        assert_eq!(project.root, fixture.path().canonicalize().unwrap());
        // A nested directory is not a project root even though an ancestor is.
        assert!(matches!(
            project_at(&nested),
            Err(Error::MissingConfig { .. })
        ));
    }

    #[test]
    fn walk_up_finds_ancestor_with_config() {
        let fixture = project_fixture(&["openspec/changes/some-change/specs/cap"]);
        let deep = fixture
            .path()
            .join("openspec/changes/some-change/specs/cap");

        let project = find_project(&deep).expect("walk-up discovery");

        assert_eq!(project.root, fixture.path().canonicalize().unwrap());
        assert_eq!(project.openspec_dir, project.root.join("openspec"));
        assert_eq!(
            project.config_path,
            project.openspec_dir.join("config.yaml")
        );
    }

    #[test]
    fn walk_up_without_config_reports_no_project() {
        let dir = empty_dir(&["nested/deeper"]);
        let start = dir.path().join("nested/deeper");

        let error = find_project(&start).expect_err("no project");

        assert!(matches!(error, Error::NoProjectFound { .. }));
    }

    #[test]
    fn missing_start_directory_is_an_io_error() {
        let dir = empty_dir(&[]);

        let error = find_project(&dir.path().join("does-not-exist")).expect_err("io error");

        assert!(matches!(error, Error::Io { .. }));
    }
}
