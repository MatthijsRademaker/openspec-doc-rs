use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::session::check_session_id;

use super::{SCRATCH_DIR, SESSION_DIR};

/// The scratch note for `session_id`, relative to the project root — the form a
/// comment anchored to the note records it in.
pub fn session_relative(session_id: &str) -> Result<String, Error> {
    check_session_id(session_id)?;

    Ok(format!("{SCRATCH_DIR}/{SESSION_DIR}/{session_id}.md"))
}

/// The scratch note for the formalized change `name`, relative to the project
/// root. The name comes from a change directory, so it needs no separator guard
/// of its own.
pub fn change_relative(name: &str) -> String {
    format!("{SCRATCH_DIR}/{name}.md")
}

/// The scratch note for `session_id`, where an exploration's notes live for as
/// long as no change directory exists for that session.
pub fn session_path(root: &Path, session_id: &str) -> Result<PathBuf, Error> {
    Ok(root.join(session_relative(session_id)?))
}

/// The scratch note for the formalized change `name`, the path a session's note
/// takes on once promoted.
pub fn change_path(root: &Path, name: &str) -> PathBuf {
    root.join(change_relative(name))
}

/// The session's scratch note contents, or `None` when it has no note yet.
pub fn read_session(root: &Path, session_id: &str) -> Result<Option<String>, Error> {
    let path = session_path(root, session_id)?;

    match fs::read_to_string(&path) {
        Ok(contents) => Ok(Some(contents)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(Error::io(path, source)),
    }
}

/// The session's scratch note path, creating an empty note when none exists.
/// An existing note is left exactly as it was.
pub fn ensure_session(root: &Path, session_id: &str) -> Result<PathBuf, Error> {
    let path = session_path(root, session_id)?;
    if path.exists() {
        return Ok(path);
    }

    let dir = root.join(SCRATCH_DIR).join(SESSION_DIR);
    fs::create_dir_all(&dir).map_err(|source| Error::write(dir, source))?;
    fs::write(&path, "").map_err(|source| Error::write(&path, source))?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn a_session_note_lives_under_the_session_scoped_directory() {
        let temp = TempDir::new().expect("temp dir");

        let path = session_path(temp.path(), "session-a").expect("path");

        assert_eq!(
            path,
            temp.path()
                .join(".openspec-doc/scratch/_session/session-a.md")
        );
    }

    #[test]
    fn a_change_note_lives_directly_under_the_scratch_directory() {
        let temp = TempDir::new().expect("temp dir");

        assert_eq!(
            change_path(temp.path(), "add-thing"),
            temp.path().join(".openspec-doc/scratch/add-thing.md")
        );
    }

    #[test]
    fn a_missing_note_is_created_empty() {
        let temp = TempDir::new().expect("temp dir");

        let path = ensure_session(temp.path(), "session-a").expect("ensure");

        assert_eq!(fs::read_to_string(&path).expect("read back"), "");
    }

    #[test]
    fn an_existing_note_survives_untouched_while_no_change_directory_exists() {
        let temp = TempDir::new().expect("temp dir");
        let path = ensure_session(temp.path(), "session-a").expect("create");
        fs::write(&path, "# Exploration\n\nAn idea worth keeping.\n").expect("write note");

        // Repeated lifecycle calls, with no change directory anywhere in sight.
        ensure_session(temp.path(), "session-a").expect("second ensure");
        let contents = read_session(temp.path(), "session-a")
            .expect("read")
            .expect("note exists");

        assert_eq!(contents, "# Exploration\n\nAn idea worth keeping.\n");
        assert_eq!(
            path,
            session_path(temp.path(), "session-a").expect("path"),
            "the note stays at its session-scoped path"
        );
    }

    #[test]
    fn a_session_without_a_note_reads_as_none() {
        let temp = TempDir::new().expect("temp dir");

        assert_eq!(read_session(temp.path(), "session-a").expect("read"), None);
    }

    #[test]
    fn session_ids_cannot_escape_the_scratch_directory() {
        let temp = TempDir::new().expect("temp dir");

        for session_id in ["", ".", "..", "../escape", "nested/id", r"nested\id"] {
            assert!(
                matches!(
                    ensure_session(temp.path(), session_id),
                    Err(Error::InvalidSessionId { .. })
                ),
                "accepted {session_id:?}"
            );
        }
    }
}
