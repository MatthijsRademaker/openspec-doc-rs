use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::root::Project;
use crate::scan::scan;
use crate::session::check_session_id;

use super::{SCRATCH_DIR, SESSION_DIR};

const SNAPSHOT_SUFFIX: &str = ".changes.json";

/// The active-change names visible in `project` right now.
pub(super) fn capture(project: &Project) -> Result<Vec<String>, Error> {
    let scan = scan(project.clone())?;

    Ok(scan
        .active_changes
        .into_iter()
        .map(|dir| dir.name)
        .collect())
}

/// The file holding the last snapshot taken for `session_id`.
fn path(root: &Path, session_id: &str) -> Result<PathBuf, Error> {
    check_session_id(session_id)?;

    Ok(root
        .join(SCRATCH_DIR)
        .join(SESSION_DIR)
        .join(format!("{session_id}{SNAPSHOT_SUFFIX}")))
}

/// The last snapshot taken for `session_id`, or `None` when the session has
/// never been checked. A malformed snapshot file is an error.
pub(super) fn load(root: &Path, session_id: &str) -> Result<Option<Vec<String>>, Error> {
    let path = path(root, session_id)?;

    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(Error::io(path, source)),
    };

    let names =
        serde_json::from_str(&contents).map_err(|source| Error::Snapshot { path, source })?;

    Ok(Some(names))
}

/// Replace the snapshot stored for `session_id` with `names`.
pub(super) fn store(root: &Path, session_id: &str, names: &[String]) -> Result<(), Error> {
    let path = path(root, session_id)?;

    let dir = root.join(SCRATCH_DIR).join(SESSION_DIR);
    fs::create_dir_all(&dir).map_err(|source| Error::write(dir, source))?;

    let contents = serde_json::to_string_pretty(names).map_err(|source| Error::Snapshot {
        path: path.clone(),
        source,
    })?;

    fs::write(&path, contents).map_err(|source| Error::write(path, source))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::root::project_at;
    use crate::test_fixture::project_fixture;

    #[test]
    fn a_capture_lists_active_change_names_only() {
        let fixture = project_fixture(&[
            "openspec/changes/add-b",
            "openspec/changes/add-a",
            "openspec/changes/archive/2026-01-01-add-old",
            "openspec/specs/some-cap",
        ]);
        let project = project_at(fixture.path()).expect("project root");

        assert_eq!(capture(&project).expect("capture"), ["add-a", "add-b"]);
    }

    #[test]
    fn a_stored_snapshot_round_trips() {
        let temp = project_fixture(&[]);
        let names = vec!["add-a".to_owned(), "add-b".to_owned()];

        store(temp.path(), "session-a", &names).expect("store");

        assert_eq!(load(temp.path(), "session-a").expect("load"), Some(names));
    }

    #[test]
    fn an_unchecked_session_has_no_snapshot() {
        let temp = project_fixture(&[]);

        assert_eq!(load(temp.path(), "session-a").expect("load"), None);
    }

    #[test]
    fn a_malformed_snapshot_file_is_an_error() {
        let temp = project_fixture(&[]);
        store(temp.path(), "session-a", &[]).expect("store");
        fs::write(path(temp.path(), "session-a").expect("path"), "{ not json").expect("corrupt");

        let error = load(temp.path(), "session-a").expect_err("malformed snapshot");

        assert!(matches!(error, Error::Snapshot { .. }));
    }
}
