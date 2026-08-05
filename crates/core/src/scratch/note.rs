use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::session::check_session_id;

use super::{SCRATCH_DIR, SESSION_DIR};

/// Opens the marker comment left at a vacated session path. Invisible when the
/// note is rendered, so the redirect stays machine-readable without showing up
/// as prose.
const MOVED_PREFIX: &str = "<!-- openspec-doc:moved-to ";
const MOVED_SUFFIX: &str = " -->";

/// Opens the marker naming the change a session says its exploration became.
/// Same convention as the moved-to pointer, and it travels with the note when
/// the note is promoted, so a formalized exploration states what it became.
const CLAIM_PREFIX: &str = "<!-- openspec-doc:claims ";
const CLAIM_SUFFIX: &str = " -->";

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

/// The record left at the vacated session-scoped path once a note has been
/// promoted, redirecting a reader to where the note now lives.
pub(super) fn moved_pointer(change: &str) -> String {
    let target = change_relative(change);

    format!("{MOVED_PREFIX}{target}{MOVED_SUFFIX}\n\nThis scratch note moved to `{target}`.\n")
}

/// The change `session_id`'s scratch note was promoted to, or `None` while the
/// note still lives at its session-scoped path.
///
/// This is the only association the project records between a session and a
/// change: a change created without an exploration behind it has no session to
/// route its review feedback back to.
pub fn promoted_to(root: &Path, session_id: &str) -> Result<Option<String>, Error> {
    let Some(contents) = read_session(root, session_id)? else {
        return Ok(None);
    };

    let Some(target) = contents
        .lines()
        .next()
        .and_then(|line| line.strip_prefix(MOVED_PREFIX))
        .and_then(|line| line.strip_suffix(MOVED_SUFFIX))
    else {
        return Ok(None);
    };

    Ok(target
        .strip_prefix(&format!("{SCRATCH_DIR}/"))
        .and_then(|name| name.strip_suffix(".md"))
        .map(str::to_owned))
}

/// The change `session_id` says its exploration became, or `None` when it has
/// claimed nothing.
///
/// Because the claim lives inside the note, a session with no note has no
/// claim — which is what makes "no note, nothing to promote" fall out of the
/// same read rather than needing a second check.
///
/// The last marker wins, so a note that somehow carries two claims resolves to
/// the one written most recently rather than to an arbitrary one.
pub fn claimed_change(root: &Path, session_id: &str) -> Result<Option<String>, Error> {
    let Some(contents) = read_session(root, session_id)? else {
        return Ok(None);
    };

    Ok(contents
        .lines()
        .rev()
        .find_map(claim_marker)
        .map(str::to_owned))
}

/// The change named by `line` when it is a claim marker.
fn claim_marker(line: &str) -> Option<&str> {
    line.strip_prefix(CLAIM_PREFIX)?.strip_suffix(CLAIM_SUFFIX)
}

/// Record that `session_id`'s exploration became change `change`, keeping the
/// exploration itself and replacing any claim already recorded.
///
/// The marker goes at the end so it never displaces the first line, which is
/// where the moved-to pointer is read from.
///
/// A session with no note is an error rather than a note created from nothing:
/// there is no exploration to promote, and an agent that claims before writing
/// its note should hear about it at that moment rather than at a turn boundary.
pub fn claim(root: &Path, session_id: &str, change: &str) -> Result<PathBuf, Error> {
    let path = session_path(root, session_id)?;

    let Some(contents) = read_session(root, session_id)? else {
        return Err(Error::MissingNote {
            session_id: session_id.to_owned(),
            change: change.to_owned(),
        });
    };

    let kept: Vec<&str> = contents
        .lines()
        .filter(|line| claim_marker(line).is_none())
        .collect();
    let body = kept.join("\n");
    let body = body.trim_end();

    let marker = format!("{CLAIM_PREFIX}{change}{CLAIM_SUFFIX}\n");
    let updated = if body.is_empty() {
        marker
    } else {
        format!("{body}\n\n{marker}")
    };
    fs::write(&path, updated).map_err(|source| Error::write(path.clone(), source))?;

    Ok(path)
}

/// The session's scratch note path, with the directory it lives in created so
/// the note can be written there.
///
/// The note file itself is deliberately not created. An agent asked to write an
/// exploration writes the file; Claude Code refuses to write over a file it has
/// not first read, so leaving an empty placeholder turns that first write into a
/// failure. An empty note is worth nothing to the reviewer either way, so a note
/// exists exactly when an exploration has been written down rather than when the
/// explore command was typed.
pub fn prepare_session(root: &Path, session_id: &str) -> Result<PathBuf, Error> {
    let path = session_path(root, session_id)?;

    let dir = root.join(SCRATCH_DIR).join(SESSION_DIR);
    fs::create_dir_all(&dir).map_err(|source| Error::write(dir, source))?;

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

    /// An empty placeholder would make the agent's first write fail, so the
    /// directory is readied and the note is left for the exploration to create.
    #[test]
    fn preparing_a_session_makes_the_directory_but_not_the_note() {
        let temp = TempDir::new().expect("temp dir");

        let path = prepare_session(temp.path(), "session-a").expect("prepare");

        assert!(
            path.parent().expect("parent").is_dir(),
            "the note has nowhere to be written"
        );
        assert!(!path.exists(), "an empty note was left behind");
    }

    #[test]
    fn an_existing_note_survives_untouched_while_no_change_directory_exists() {
        let temp = TempDir::new().expect("temp dir");
        let path = prepare_session(temp.path(), "session-a").expect("prepare");
        fs::write(&path, "# Exploration\n\nAn idea worth keeping.\n").expect("write note");

        // Repeated lifecycle calls, with no change directory anywhere in sight.
        prepare_session(temp.path(), "session-a").expect("second prepare");
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
    fn a_promoted_note_names_the_change_it_moved_to() {
        let temp = TempDir::new().expect("temp dir");
        prepare_session(temp.path(), "session-a").expect("prepare");
        fs::write(
            session_path(temp.path(), "session-a").expect("path"),
            moved_pointer("add-thing"),
        )
        .expect("write pointer");

        assert_eq!(
            promoted_to(temp.path(), "session-a").expect("promoted_to"),
            Some("add-thing".to_owned())
        );
    }

    #[test]
    fn a_note_still_holding_an_exploration_names_no_change() {
        let temp = TempDir::new().expect("temp dir");
        fs::write(
            prepare_session(temp.path(), "session-a").expect("prepare"),
            "# Exploration\n\nStill being explored.\n",
        )
        .expect("write note");

        assert_eq!(promoted_to(temp.path(), "session-a").expect("read"), None);
        assert_eq!(
            promoted_to(temp.path(), "session-b").expect("read"),
            None,
            "a session with no note at all has been promoted nowhere"
        );
    }

    #[test]
    fn a_claim_is_read_back_as_the_sessions_change() {
        let temp = TempDir::new().expect("temp dir");
        fs::write(
            prepare_session(temp.path(), "session-a").expect("prepare"),
            "# Exploration\n",
        )
        .expect("write note");

        claim(temp.path(), "session-a", "add-thing").expect("claim");

        assert_eq!(
            claimed_change(temp.path(), "session-a").expect("read claim"),
            Some("add-thing".to_owned())
        );
    }

    #[test]
    fn claiming_preserves_the_exploration() {
        let temp = TempDir::new().expect("temp dir");
        let exploration = "# Exploration\n\nAn idea worth keeping.\n";
        fs::write(
            prepare_session(temp.path(), "session-a").expect("prepare"),
            exploration,
        )
        .expect("write note");

        claim(temp.path(), "session-a", "add-thing").expect("claim");

        let contents = read_session(temp.path(), "session-a")
            .expect("read")
            .expect("note exists");
        assert!(
            contents.starts_with(exploration),
            "the exploration did not survive the claim: {contents:?}"
        );
    }

    #[test]
    fn re_claiming_replaces_rather_than_accumulates() {
        let temp = TempDir::new().expect("temp dir");
        fs::write(
            prepare_session(temp.path(), "session-a").expect("prepare"),
            "# Exploration\n",
        )
        .expect("write note");

        claim(temp.path(), "session-a", "add-thing").expect("first claim");
        claim(temp.path(), "session-a", "add-other").expect("second claim");

        assert_eq!(
            claimed_change(temp.path(), "session-a").expect("read claim"),
            Some("add-other".to_owned())
        );
        let contents = read_session(temp.path(), "session-a")
            .expect("read")
            .expect("note exists");
        assert!(
            !contents.contains("add-thing"),
            "the superseded claim is still in the note: {contents:?}"
        );
    }

    #[test]
    fn a_note_with_no_marker_claims_nothing() {
        let temp = TempDir::new().expect("temp dir");
        fs::write(
            prepare_session(temp.path(), "session-a").expect("prepare"),
            "# Exploration\n\nStill being explored.\n",
        )
        .expect("write note");

        assert_eq!(
            claimed_change(temp.path(), "session-a").expect("read claim"),
            None
        );
        assert_eq!(
            claimed_change(temp.path(), "session-b").expect("read claim"),
            None,
            "a session with no note at all has claimed nothing"
        );
    }

    /// A claim is a statement about an exploration, so there has to be one.
    #[test]
    fn claiming_without_a_note_fails_loudly() {
        let temp = TempDir::new().expect("temp dir");
        prepare_session(temp.path(), "session-a").expect("prepare");

        assert!(matches!(
            claim(temp.path(), "session-a", "add-thing"),
            Err(Error::MissingNote { .. })
        ));
    }

    #[test]
    fn session_ids_cannot_escape_the_scratch_directory() {
        let temp = TempDir::new().expect("temp dir");

        for session_id in ["", ".", "..", "../escape", "nested/id", r"nested\id"] {
            assert!(
                matches!(
                    prepare_session(temp.path(), session_id),
                    Err(Error::InvalidSessionId { .. })
                ),
                "accepted {session_id:?}"
            );
        }
    }
}
