//! The phase verdict: a reviewer's decision about what should happen to a whole
//! scope next, recorded apart from any single comment's status.
//!
//! A verdict belongs to a session or a change, never to a comment, so it lives
//! in its own append-only sidecar rather than in the comment stream. The sidecar
//! is keyed the same way a comment sidecar is, and for the same reason: a
//! session id must never collide with a change name.

use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::comments::ScopeKey;
use crate::error::Error;
use crate::session::check_session_id;

/// The sidecar directory holding every scope's verdict stream.
const VERDICTS_DIR: &str = ".openspec-doc/verdicts";

/// Session-keyed sidecars sit one level deeper, so a session id can never
/// collide with a change name.
const SESSION_DIR: &str = "_session";

/// What a reviewer decided should happen to a scope next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verdict {
    /// Stay in the explore phase; the record's notes say what is still open.
    KeepExploring,
    /// The exploration is ready to be formalized into a change proposal.
    MoveToProposal,
    /// The change's open comments should be addressed.
    CommentResolution,
}

impl Verdict {
    /// Whether this verdict is a decision about an exploration rather than about
    /// a change that already exists.
    fn is_session_phase(self) -> bool {
        matches!(self, Self::KeepExploring | Self::MoveToProposal)
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::KeepExploring => "keep-exploring",
            Self::MoveToProposal => "move-to-proposal",
            Self::CommentResolution => "comment-resolution",
        };
        f.write_str(label)
    }
}

/// One phase verdict, as submitted. The file is append-only, so a scope's
/// verdict history is every record in it and its current verdict is the last.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub id: String,
    pub verdict: Verdict,
    /// The reviewer's open questions or resolutions. Empty for a verdict that
    /// carries no free text of its own.
    pub notes: String,
    pub created_at: String,
}

/// The verdict sidecar for `key` under the project at `root`.
pub fn path(root: &Path, key: &ScopeKey) -> Result<PathBuf, Error> {
    let dir = root.join(VERDICTS_DIR);

    match key {
        ScopeKey::Session(session_id) => {
            check_session_id(session_id)?;
            Ok(dir.join(SESSION_DIR).join(format!("{session_id}.jsonl")))
        }
        // The name comes from a change directory, so it needs no guard.
        ScopeKey::Change(name) => Ok(dir.join(format!("{name}.jsonl"))),
    }
}

/// Record `verdict` for `key`, with `notes` as the reviewer's free text.
///
/// A verdict that does not belong to the kind of scope it was submitted for is
/// refused, as is a keep-exploring verdict with nothing written in it: its whole
/// content is what is still open, so an empty one says nothing.
pub fn add(root: &Path, key: &ScopeKey, verdict: Verdict, notes: &str) -> Result<Record, Error> {
    let session_scoped = matches!(key, ScopeKey::Session(_));
    if session_scoped != verdict.is_session_phase() {
        return Err(Error::MisscopedVerdict {
            verdict,
            session_scoped,
        });
    }

    let notes = notes.trim();
    if verdict == Verdict::KeepExploring && notes.is_empty() {
        return Err(Error::EmptyVerdictNotes);
    }

    let record = Record {
        id: Uuid::new_v4().to_string(),
        verdict,
        notes: notes.to_owned(),
        created_at: Utc::now().to_rfc3339(),
    };

    append(root, key, &record)?;

    Ok(record)
}

/// Every verdict recorded for `key`, in the order they were submitted. A scope
/// with no sidecar has no verdicts.
pub fn read(root: &Path, key: &ScopeKey) -> Result<Vec<Record>, Error> {
    let path = path(root, key)?;

    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(Error::io(path, source)),
    };

    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            serde_json::from_str(line).map_err(|source| Error::VerdictSidecar {
                path: path.clone(),
                source,
            })
        })
        .collect()
}

/// Add one line to `key`'s sidecar, leaving every line already in it untouched.
fn append(root: &Path, key: &ScopeKey, record: &Record) -> Result<(), Error> {
    let path = path(root, key)?;

    let dir = path.parent().expect("sidecar paths have a parent");
    fs::create_dir_all(dir).map_err(|source| Error::write(dir, source))?;

    let line = serde_json::to_string(record).map_err(|source| Error::VerdictSidecar {
        path: path.clone(),
        source,
    })?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|source| Error::write(&path, source))?;

    writeln!(file, "{line}").map_err(|source| Error::write(path, source))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn session() -> ScopeKey {
        ScopeKey::Session("session-a".to_owned())
    }

    fn change() -> ScopeKey {
        ScopeKey::Change("add-thing".to_owned())
    }

    #[test]
    fn a_session_sidecar_sits_under_the_session_scoped_directory() {
        let temp = TempDir::new().expect("temp dir");

        assert_eq!(
            path(temp.path(), &session()).expect("path"),
            temp.path()
                .join(".openspec-doc/verdicts/_session/session-a.jsonl")
        );
        assert_eq!(
            path(temp.path(), &change()).expect("path"),
            temp.path().join(".openspec-doc/verdicts/add-thing.jsonl")
        );
    }

    #[test]
    fn session_ids_cannot_escape_the_verdicts_directory() {
        let temp = TempDir::new().expect("temp dir");

        for session_id in ["", ".", "..", "../escape", "nested/id", r"nested\id"] {
            assert!(
                matches!(
                    path(temp.path(), &ScopeKey::Session(session_id.to_owned())),
                    Err(Error::InvalidSessionId { .. })
                ),
                "accepted {session_id:?}"
            );
        }
    }

    #[test]
    fn a_keep_exploring_verdict_keeps_its_notes() {
        let temp = TempDir::new().expect("temp dir");

        let record = add(
            temp.path(),
            &session(),
            Verdict::KeepExploring,
            "  Still unclear how promotion is detected.  ",
        )
        .expect("add verdict");

        assert_eq!(record.verdict, Verdict::KeepExploring);
        assert_eq!(record.notes, "Still unclear how promotion is detected.");
        assert_eq!(
            read(temp.path(), &session()).expect("read"),
            [record],
            "the record reads back as it was written"
        );
    }

    #[test]
    fn a_move_to_proposal_verdict_needs_no_notes() {
        let temp = TempDir::new().expect("temp dir");

        let record =
            add(temp.path(), &session(), Verdict::MoveToProposal, "").expect("add verdict");

        assert_eq!(record.verdict, Verdict::MoveToProposal);
        assert!(record.notes.is_empty());
    }

    #[test]
    fn a_comment_resolution_verdict_is_recorded_for_a_change() {
        let temp = TempDir::new().expect("temp dir");

        let record =
            add(temp.path(), &change(), Verdict::CommentResolution, "").expect("add verdict");

        assert_eq!(record.verdict, Verdict::CommentResolution);
        assert_eq!(read(temp.path(), &change()).expect("read"), [record]);
    }

    #[test]
    fn a_keep_exploring_verdict_with_nothing_written_in_it_is_refused() {
        let temp = TempDir::new().expect("temp dir");

        for notes in ["", "   \n\t"] {
            assert!(
                matches!(
                    add(temp.path(), &session(), Verdict::KeepExploring, notes),
                    Err(Error::EmptyVerdictNotes)
                ),
                "accepted {notes:?}"
            );
        }
        assert!(
            !path(temp.path(), &session()).expect("path").exists(),
            "a refused verdict writes no sidecar"
        );
    }

    #[test]
    fn a_verdict_submitted_against_the_wrong_kind_of_scope_is_refused() {
        let temp = TempDir::new().expect("temp dir");

        for (key, verdict) in [
            (session(), Verdict::CommentResolution),
            (change(), Verdict::KeepExploring),
            (change(), Verdict::MoveToProposal),
        ] {
            assert!(
                matches!(
                    add(temp.path(), &key, verdict, "Notes."),
                    Err(Error::MisscopedVerdict { .. })
                ),
                "accepted {verdict} for {key:?}"
            );
        }
    }

    #[test]
    fn verdicts_are_appended_in_the_order_they_were_submitted() {
        let temp = TempDir::new().expect("temp dir");

        add(
            temp.path(),
            &session(),
            Verdict::KeepExploring,
            "One more pass.",
        )
        .expect("first verdict");
        add(temp.path(), &session(), Verdict::MoveToProposal, "").expect("second verdict");

        let verdicts: Vec<Verdict> = read(temp.path(), &session())
            .expect("read")
            .into_iter()
            .map(|record| record.verdict)
            .collect();

        assert_eq!(
            verdicts,
            [Verdict::KeepExploring, Verdict::MoveToProposal],
            "a later verdict does not overwrite the one it followed"
        );
    }

    #[test]
    fn a_scope_with_no_sidecar_has_no_verdicts() {
        let temp = TempDir::new().expect("temp dir");

        assert!(read(temp.path(), &change()).expect("read").is_empty());
    }

    #[test]
    fn a_malformed_verdict_line_is_an_error() {
        let temp = TempDir::new().expect("temp dir");
        add(temp.path(), &change(), Verdict::CommentResolution, "").expect("add verdict");
        let sidecar = path(temp.path(), &change()).expect("path");
        fs::write(&sidecar, "{ not json\n").expect("corrupt sidecar");

        let error = read(temp.path(), &change()).expect_err("malformed sidecar");

        assert!(matches!(error, Error::VerdictSidecar { .. }), "{error}");
    }
}
