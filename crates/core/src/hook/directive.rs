use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::session::check_session_id;

const DIRECTIVES_DIR: &str = ".openspec-doc/directives/_session";

/// A directive left for a session, injected back into the agent at its next
/// turn boundary. Written by hand in this change; by the dashboard later.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Directive {
    /// Whether this directive is still waiting to be injected.
    pub pending: bool,
    /// The text fed back to the agent when the turn end is blocked.
    pub reason: String,
    pub created_at: String,
    /// When the directive was injected; `None` while still pending.
    pub consumed_at: Option<String>,
}

/// The directory holding every session's directive file under `root`.
pub fn sessions_dir(root: &Path) -> PathBuf {
    root.join(DIRECTIVES_DIR)
}

/// Every session with a directive record under `root`, sorted by id. A session
/// whose directive was already consumed still counts: the record exists.
pub fn sessions(root: &Path) -> Result<Vec<String>, Error> {
    let dir = sessions_dir(root);

    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(Error::io(dir, source)),
    };

    let mut sessions = Vec::new();
    for entry in entries {
        let path = entry.map_err(|source| Error::io(&dir, source))?.path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }

        let session_id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| Error::NonUtf8Path { path: path.clone() })?;
        sessions.push(session_id.to_owned());
    }

    sessions.sort();
    Ok(sessions)
}

/// The directive file for `session_id` under the project at `root`.
pub fn path(root: &Path, session_id: &str) -> Result<PathBuf, Error> {
    check_session_id(session_id)?;

    Ok(root.join(DIRECTIVES_DIR).join(format!("{session_id}.json")))
}

/// The session's pending directive, or `None` when it has no directive file or
/// its directive was already consumed. A malformed directive file is an error.
pub fn load_pending(root: &Path, session_id: &str) -> Result<Option<Directive>, Error> {
    let path = path(root, session_id)?;

    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(Error::io(path, source)),
    };

    let directive: Directive =
        serde_json::from_str(&contents).map_err(|source| Error::Directive { path, source })?;

    Ok(directive.pending.then_some(directive))
}

/// Ensure `session_id` has a directive file, so the dashboard can discover the
/// session before any directive has been written for it.
///
/// Without this the discovery loop has no entry point: a session is listed
/// because it has a directive record, a directive comes from a verdict, and a
/// verdict is submitted from the session's page. The record left here is an
/// empty slot — never pending, so it is never injected, and never consumed, so
/// it does not read as an injection that already happened. A session that
/// already has a record keeps it untouched.
pub fn ensure_session(root: &Path, session_id: &str) -> Result<(), Error> {
    let path = path(root, session_id)?;
    if path.exists() {
        return Ok(());
    }

    let slot = Directive {
        pending: false,
        reason: String::new(),
        created_at: Utc::now().to_rfc3339(),
        consumed_at: None,
    };

    write(&path, &slot)
}

/// Write `reason` as `session_id`'s pending directive, replacing whatever
/// record was there before.
///
/// Callers must not use this to overwrite a directive that is still pending:
/// there is one directive per session, so writing over an uninjected one drops
/// it.
pub fn write_pending(root: &Path, session_id: &str, reason: &str) -> Result<Directive, Error> {
    let directive = Directive {
        pending: true,
        reason: reason.to_owned(),
        created_at: Utc::now().to_rfc3339(),
        consumed_at: None,
    };

    write(&path(root, session_id)?, &directive)?;

    Ok(directive)
}

/// Record `directive` as consumed so a later turn boundary in the same session
/// does not re-inject it. The file is kept as an audit trail, not deleted.
pub fn mark_consumed(root: &Path, session_id: &str, directive: &Directive) -> Result<(), Error> {
    let consumed = Directive {
        pending: false,
        consumed_at: Some(Utc::now().to_rfc3339()),
        ..directive.clone()
    };

    write(&path(root, session_id)?, &consumed)
}

/// Replace the directive file at `path` with `directive`, creating the
/// directives directory the first time a session needs one.
fn write(path: &Path, directive: &Directive) -> Result<(), Error> {
    let dir = path.parent().expect("directive paths have a parent");
    fs::create_dir_all(dir).map_err(|source| Error::write(dir, source))?;

    let contents = serde_json::to_string_pretty(directive).map_err(|source| Error::Directive {
        path: path.to_owned(),
        source,
    })?;

    fs::write(path, contents).map_err(|source| Error::write(path, source))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// A project root whose directive file for `session_id` is `contents`.
    fn root_with_directive(session_id: &str, contents: &str) -> TempDir {
        let temp = TempDir::new().expect("temp dir");
        let dir = temp.path().join(DIRECTIVES_DIR);
        fs::create_dir_all(&dir).expect("create directives dir");
        fs::write(dir.join(format!("{session_id}.json")), contents).expect("write directive");
        temp
    }

    fn pending_json(reason: &str) -> String {
        format!(
            r#"{{"pending":true,"reason":"{reason}","createdAt":"2026-07-31T08:00:00Z","consumedAt":null}}"#
        )
    }

    #[test]
    fn a_pending_directive_is_read_with_its_reason() {
        let root = root_with_directive("session-a", &pending_json("Resolve open comments."));

        let directive = load_pending(root.path(), "session-a")
            .expect("load")
            .expect("pending directive");

        assert_eq!(directive.reason, "Resolve open comments.");
        assert_eq!(directive.created_at, "2026-07-31T08:00:00Z");
        assert_eq!(directive.consumed_at, None);
    }

    #[test]
    fn a_missing_directive_file_is_not_an_error() {
        let temp = TempDir::new().expect("temp dir");

        assert_eq!(load_pending(temp.path(), "session-a").expect("load"), None);
    }

    #[test]
    fn a_directive_is_consumed_exactly_once() {
        let root = root_with_directive("session-a", &pending_json("Resolve open comments."));

        let first = load_pending(root.path(), "session-a")
            .expect("first load")
            .expect("pending directive");
        mark_consumed(root.path(), "session-a", &first).expect("mark consumed");

        assert_eq!(
            load_pending(root.path(), "session-a").expect("second load"),
            None,
            "a consumed directive must not re-fire"
        );
    }

    #[test]
    fn consuming_preserves_the_record_and_stamps_the_injection() {
        let root = root_with_directive("session-a", &pending_json("Resolve open comments."));
        let directive = load_pending(root.path(), "session-a")
            .expect("load")
            .expect("pending directive");

        mark_consumed(root.path(), "session-a", &directive).expect("mark consumed");

        let path = path(root.path(), "session-a").expect("path");
        let stored: Directive =
            serde_json::from_str(&fs::read_to_string(&path).expect("read back")).expect("parse");
        assert!(!stored.pending);
        assert_eq!(stored.reason, directive.reason);
        assert_eq!(stored.created_at, directive.created_at);
        assert!(stored.consumed_at.is_some(), "injection is timestamped");
    }

    #[test]
    fn a_written_directive_is_pending_and_reads_back_with_its_reason() {
        let temp = TempDir::new().expect("temp dir");

        let written = write_pending(temp.path(), "session-a", "Open comments are waiting.")
            .expect("write directive");

        assert!(written.pending);
        assert_eq!(written.consumed_at, None);
        assert_eq!(
            load_pending(temp.path(), "session-a").expect("load"),
            Some(written)
        );
    }

    /// The dashboard discovers a session by its directive record, so a session
    /// that has never been sent one still needs the file to exist.
    #[test]
    fn registering_a_session_makes_it_discoverable_without_injecting_anything() {
        let temp = TempDir::new().expect("temp dir");

        ensure_session(temp.path(), "session-a").expect("register");

        assert_eq!(sessions(temp.path()).expect("sessions"), ["session-a"]);
        assert_eq!(
            load_pending(temp.path(), "session-a").expect("load"),
            None,
            "registration must not put a directive in front of the agent"
        );
        let stored: Directive = serde_json::from_str(
            &fs::read_to_string(path(temp.path(), "session-a").expect("path")).expect("read back"),
        )
        .expect("parse");
        assert!(
            stored.consumed_at.is_none() && stored.reason.is_empty(),
            "an empty slot must not read as a directive that was already injected: {stored:?}"
        );
    }

    #[test]
    fn registering_a_session_twice_leaves_the_first_record_alone() {
        let root = root_with_directive("session-a", &pending_json("Resolve open comments."));

        ensure_session(root.path(), "session-a").expect("register");

        assert_eq!(
            load_pending(root.path(), "session-a")
                .expect("load")
                .expect("pending directive")
                .reason,
            "Resolve open comments.",
            "a waiting directive must survive registration"
        );
    }

    #[test]
    fn a_malformed_directive_file_is_an_error() {
        let root = root_with_directive("session-a", "{ not json");

        let error = load_pending(root.path(), "session-a").expect_err("malformed directive");

        assert!(matches!(error, Error::Directive { .. }));
    }

    #[test]
    fn sessions_are_discovered_from_their_directive_files() {
        let root = root_with_directive("session-b", &pending_json("Later."));
        let dir = root.path().join(DIRECTIVES_DIR);
        fs::write(dir.join("session-a.json"), pending_json("Earlier.")).expect("write directive");
        fs::write(dir.join("notes.txt"), "not a directive").expect("write stray file");

        assert_eq!(
            sessions(root.path()).expect("sessions"),
            ["session-a", "session-b"]
        );
    }

    #[test]
    fn a_missing_directives_directory_yields_no_sessions() {
        let temp = TempDir::new().expect("temp dir");

        assert!(sessions(temp.path()).expect("sessions").is_empty());
    }

    #[test]
    fn session_ids_cannot_escape_the_directives_directory() {
        let temp = TempDir::new().expect("temp dir");

        for session_id in ["", ".", "..", "../escape", "nested/id", r"nested\id"] {
            assert!(
                matches!(
                    load_pending(temp.path(), session_id),
                    Err(Error::InvalidSessionId { .. })
                ),
                "accepted {session_id:?}"
            );
        }
    }
}
