//! The append-only JSONL file holding one scope's comment stream, and the
//! operations that append to it or replay it.

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::error::Error;
use crate::session::check_session_id;

use super::anchor::{self, Anchor};
use super::artifact;
use super::record::{
    Comment, Edit, Event, Relocation, Reply, Status, StatusUpdate, Thread, new_id,
};
use super::{COMMENTS_DIR, SESSION_DIR};

/// The scope a comment sidecar is filed under: a session id while the artifact
/// is still explore-phase, its change name once promoted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeKey {
    Session(String),
    Change(String),
}

impl ScopeKey {
    /// The sidecar file for this scope, relative to the project root — the form
    /// a directive names it in, so the agent is pointed at a path it can read.
    ///
    /// Session-keyed sidecars sit one level deeper, so a session id can never
    /// collide with a change name — the same split the scratch notes use.
    pub fn relative(&self) -> Result<String, Error> {
        match self {
            Self::Session(session_id) => {
                check_session_id(session_id)?;
                Ok(format!("{COMMENTS_DIR}/{SESSION_DIR}/{session_id}.jsonl"))
            }
            // The name comes from a change directory, so it needs no guard.
            Self::Change(name) => Ok(format!("{COMMENTS_DIR}/{name}.jsonl")),
        }
    }

    /// The sidecar file for this scope under the project at `root`.
    pub fn path(&self, root: &Path) -> Result<PathBuf, Error> {
        Ok(root.join(self.relative()?))
    }
}

/// Anchor a comment to `selected_text` in `artifact_path` and record it in
/// `key`'s sidecar.
///
/// The artifact is read as it stands on disk, so an anchor is never created
/// against markdown the caller only believes is there.
pub fn add(
    root: &Path,
    key: &ScopeKey,
    artifact_path: &str,
    selected_text: &str,
    body: &str,
) -> Result<Comment, Error> {
    let markdown = artifact::read(root, artifact_path)?.ok_or_else(|| Error::MissingArtifact {
        artifact_path: artifact_path.to_owned(),
    })?;

    record_comment(
        root,
        key,
        Some(anchor::create(artifact_path, &markdown, selected_text)?),
        body,
    )
}

/// Record `body` against `key` itself, anchored to nothing.
///
/// Deliberately not routed through [`anchor::create`]: that function's contract
/// is that the selected text was found in the artifact, so calling it with an
/// empty selection would turn its error case into a formality. There is also no
/// artifact to read — the scope may hold none yet, which is exactly the state a
/// session is in before the agent has written its note.
pub fn add_unanchored(root: &Path, key: &ScopeKey, body: &str) -> Result<Comment, Error> {
    record_comment(root, key, None, body)
}

/// Replace the body of the comment `comment_id` in `key`'s sidecar.
///
/// The event is appended and the record that created the comment is left as
/// written, so the sidecar keeps the body the reviewer first wrote. Only a
/// comment can be edited: `comment_id` naming a reply is an unknown comment.
pub fn edit(root: &Path, key: &ScopeKey, comment_id: &str, body: &str) -> Result<Edit, Error> {
    require_comment(root, key, comment_id)?;

    let edit = Edit {
        id: new_id(),
        comment_id: comment_id.to_owned(),
        body: body.to_owned(),
        created_at: Utc::now().to_rfc3339(),
    };

    append(root, key, &Event::Edit { edit: edit.clone() })?;

    Ok(edit)
}

/// Append a comment event carrying `anchor`, whether or not there is one.
fn record_comment(
    root: &Path,
    key: &ScopeKey,
    anchor: Option<Anchor>,
    body: &str,
) -> Result<Comment, Error> {
    let comment = Comment {
        id: new_id(),
        anchor,
        body: body.to_owned(),
        created_at: Utc::now().to_rfc3339(),
    };

    append(
        root,
        key,
        &Event::Comment {
            comment: comment.clone(),
        },
    )?;

    Ok(comment)
}

/// Append a reply to the comment `comment_id` in `key`'s sidecar.
pub fn reply(root: &Path, key: &ScopeKey, comment_id: &str, body: &str) -> Result<Reply, Error> {
    require_comment(root, key, comment_id)?;

    let reply = Reply {
        id: new_id(),
        comment_id: comment_id.to_owned(),
        body: body.to_owned(),
        created_at: Utc::now().to_rfc3339(),
    };

    append(
        root,
        key,
        &Event::Reply {
            reply: reply.clone(),
        },
    )?;

    Ok(reply)
}

/// Move the comment `comment_id` in `key`'s sidecar to `to`.
///
/// Every transition is one appended event, including a move back to `Open`:
/// reopening is not a status of its own, it is the history recording that the
/// comment was elsewhere and came back.
pub fn set_status(
    root: &Path,
    key: &ScopeKey,
    comment_id: &str,
    to: Status,
) -> Result<StatusUpdate, Error> {
    require_comment(root, key, comment_id)?;

    let status = StatusUpdate {
        id: new_id(),
        comment_id: comment_id.to_owned(),
        status: to,
        created_at: Utc::now().to_rfc3339(),
    };

    append(
        root,
        key,
        &Event::Status {
            status: status.clone(),
        },
    )?;

    Ok(status)
}

/// Every comment in `key`'s sidecar with its current status and full history, in
/// the order the comments were made. A scope with no sidecar has no comments.
pub fn read(root: &Path, key: &ScopeKey) -> Result<Vec<Thread>, Error> {
    let path = key.path(root)?;
    let mut threads: Vec<Thread> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();

    for event in read_events(&path)? {
        match event {
            Event::Comment { comment } => {
                index.insert(comment.id.clone(), threads.len());
                threads.push(Thread {
                    comment,
                    status: Status::Open,
                    replies: Vec::new(),
                    status_history: Vec::new(),
                });
            }
            Event::Reply { reply } => {
                let thread = &mut threads[lookup(&index, &path, &reply.comment_id)?];
                thread.replies.push(reply);
            }
            Event::Edit { edit } => {
                let thread = &mut threads[lookup(&index, &path, &edit.comment_id)?];
                thread.comment.body = edit.body;
            }
            Event::Status { status } => {
                let thread = &mut threads[lookup(&index, &path, &status.comment_id)?];
                thread.status = status.status;
                thread.status_history.push(status);
            }
            Event::Relocate { relocate } => {
                // An unanchored comment names no artifact, so a relocation has
                // nothing to rewrite in it.
                for anchor in threads
                    .iter_mut()
                    .filter_map(|thread| thread.comment.anchor.as_mut())
                {
                    if anchor.artifact_path == relocate.from_artifact_path {
                        anchor.artifact_path = relocate.to_artifact_path.clone();
                    }
                }
            }
        }
    }

    Ok(threads)
}

/// How many of a scope's comments stand in each status.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StatusCounts {
    pub open: usize,
    pub addressed: usize,
    pub resolved: usize,
}

/// How `key`'s comments are distributed across the three statuses.
///
/// Counted from each comment's reconstructed status rather than from the status
/// events, so a comment that moved twice is counted once, where it ended up. A
/// scope with no sidecar has no comments and so counts zero everywhere.
pub fn counts(root: &Path, key: &ScopeKey) -> Result<StatusCounts, Error> {
    let mut counts = StatusCounts::default();

    for thread in read(root, key)? {
        let counter = match thread.status {
            Status::Open => &mut counts.open,
            Status::Addressed => &mut counts.addressed,
            Status::Resolved => &mut counts.resolved,
        };
        *counter += 1;
    }

    Ok(counts)
}

/// Move `from`'s sidecar to `to`, recording that its artifact moved from
/// `from_artifact` to `to_artifact` so the relocated anchors still resolve.
///
/// A scope with no sidecar has nothing to relocate and is not an error.
pub fn relocate(
    root: &Path,
    from: &ScopeKey,
    to: &ScopeKey,
    from_artifact: &str,
    to_artifact: &str,
) -> Result<(), Error> {
    let from_path = from.path(root)?;
    if !from_path.exists() {
        return Ok(());
    }

    let to_path = to.path(root)?;
    if to_path.exists() {
        return Err(Error::SidecarExists { path: to_path });
    }

    // Checked before anything moves, so a bad path cannot leave the sidecar
    // relocated but missing the record of what its anchors now point at.
    artifact::check_path(from_artifact)?;
    artifact::check_path(to_artifact)?;

    fs::rename(&from_path, &to_path).map_err(|source| Error::Rename {
        from: from_path,
        to: to_path,
        source,
    })?;

    append(
        root,
        to,
        &Event::Relocate {
            relocate: Relocation {
                from_artifact_path: from_artifact.to_owned(),
                to_artifact_path: to_artifact.to_owned(),
                created_at: Utc::now().to_rfc3339(),
            },
        },
    )
}

/// Add one line to `key`'s sidecar, leaving every line already in it untouched.
fn append(root: &Path, key: &ScopeKey, event: &Event) -> Result<(), Error> {
    let path = key.path(root)?;

    let dir = path.parent().expect("sidecar paths have a parent");
    fs::create_dir_all(dir).map_err(|source| Error::write(dir, source))?;

    let line = serde_json::to_string(event).map_err(|source| Error::CommentSidecar {
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

/// Every event in the sidecar at `path`, in order. A missing file holds no
/// events; a line that is not a valid event makes the sidecar unreadable.
fn read_events(path: &Path) -> Result<Vec<Event>, Error> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(Error::io(path, source)),
    };

    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            serde_json::from_str(line).map_err(|source| Error::CommentSidecar {
                path: path.to_owned(),
                source,
            })
        })
        .collect()
}

/// The thread an event refers to. An event naming a comment that was never made
/// means the sidecar disagrees with itself, which is not silently skipped.
fn lookup(index: &HashMap<String, usize>, path: &Path, comment_id: &str) -> Result<usize, Error> {
    index
        .get(comment_id)
        .copied()
        .ok_or_else(|| Error::UnknownComment {
            path: path.to_owned(),
            comment_id: comment_id.to_owned(),
        })
}

/// Reject an operation against a comment that this scope does not hold, rather
/// than appending an event nothing will ever be able to attach to.
fn require_comment(root: &Path, key: &ScopeKey, comment_id: &str) -> Result<(), Error> {
    if read(root, key)?
        .iter()
        .any(|thread| thread.comment.id == comment_id)
    {
        return Ok(());
    }

    Err(Error::UnknownComment {
        path: key.path(root)?,
        comment_id: comment_id.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const ARTIFACT: &str = "openspec/changes/add-thing/proposal.md";
    const MARKDOWN: &str =
        "# Proposal\n\n## Why\n\nAlpha before. Selected sentence. Omega after.\n";
    const SELECTED: &str = "Selected sentence.";

    fn change() -> ScopeKey {
        ScopeKey::Change("add-thing".to_owned())
    }

    fn session() -> ScopeKey {
        ScopeKey::Session("session-a".to_owned())
    }

    /// A project root holding `ARTIFACT` with `MARKDOWN` in it.
    fn root_with_artifact() -> TempDir {
        let temp = TempDir::new().expect("temp dir");
        let path = temp.path().join(ARTIFACT);
        fs::create_dir_all(path.parent().expect("parent")).expect("create dirs");
        fs::write(path, MARKDOWN).expect("write artifact");
        temp
    }

    fn lines(path: &Path) -> Vec<String> {
        fs::read_to_string(path)
            .expect("read sidecar")
            .lines()
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn a_session_sidecar_sits_under_the_session_scoped_directory() {
        let temp = TempDir::new().expect("temp dir");

        assert_eq!(
            session().path(temp.path()).expect("path"),
            temp.path()
                .join(".openspec-doc/comments/_session/session-a.jsonl")
        );
        assert_eq!(
            change().path(temp.path()).expect("path"),
            temp.path().join(".openspec-doc/comments/add-thing.jsonl")
        );
    }

    #[test]
    fn session_ids_cannot_escape_the_comments_directory() {
        let temp = TempDir::new().expect("temp dir");

        for session_id in ["", ".", "..", "../escape", "nested/id", r"nested\id"] {
            assert!(
                matches!(
                    ScopeKey::Session(session_id.to_owned()).path(temp.path()),
                    Err(Error::InvalidSessionId { .. })
                ),
                "accepted {session_id:?}"
            );
        }
    }

    #[test]
    fn a_new_comment_is_anchored_and_appended() {
        let temp = root_with_artifact();

        let comment = add(
            temp.path(),
            &change(),
            ARTIFACT,
            SELECTED,
            "Needs a rationale.",
        )
        .expect("add comment");

        let anchor = comment.anchor.as_ref().expect("an anchored comment");
        assert_eq!(anchor.heading_path, ["Proposal", "Why"]);
        assert_eq!(anchor.artifact_path, ARTIFACT);
        assert_eq!(comment.body, "Needs a rationale.");
        assert_eq!(
            lines(&change().path(temp.path()).expect("path")).len(),
            1,
            "one comment is one line"
        );
    }

    /// The sidecars in this repository were written before the anchor became
    /// optional. Every one of their lines carries an `anchor` object, and they
    /// are real review history rather than a fixture.
    #[test]
    fn a_comment_line_written_with_an_anchor_still_reads_into_a_thread() {
        let temp = TempDir::new().expect("temp dir");
        let path = change().path(temp.path()).expect("path");
        fs::create_dir_all(path.parent().expect("parent")).expect("create dirs");
        fs::write(
            &path,
            r#"{"type":"comment","comment":{"id":"c1","anchor":{"artifactPath":"openspec/changes/add-thing/proposal.md","selectedText":"Selected sentence.","headingPath":["Proposal","Why"],"beforeText":"Alpha before. ","afterText":" Omega after.","startOffset":26,"endOffset":44},"body":"Needs a rationale.","createdAt":"2026-07-31T08:00:00Z"}}
"#,
        )
        .expect("write pre-existing sidecar");

        let threads = read(temp.path(), &change()).expect("read");

        let [thread] = threads.as_slice() else {
            panic!("expected one thread, got {}", threads.len());
        };
        let anchor = thread.comment.anchor.as_ref().expect("the anchor survived");
        assert_eq!(anchor.selected_text, SELECTED);
        assert_eq!(anchor.heading_path, ["Proposal", "Why"]);
        assert_eq!(thread.comment.body, "Needs a rationale.");
    }

    /// A session has no artifacts until the agent writes its note, and the
    /// reviewer's scope-level feedback must not have to wait for one.
    #[test]
    fn an_unanchored_comment_needs_no_artifact_on_disk() {
        let temp = TempDir::new().expect("temp dir");

        let comment =
            add_unanchored(temp.path(), &session(), "Where is the note?").expect("add unanchored");

        assert_eq!(comment.anchor, None);
        let threads = read(temp.path(), &session()).expect("read");
        let [thread] = threads.as_slice() else {
            panic!("expected one thread, got {}", threads.len());
        };
        assert_eq!(thread.comment, comment);
        assert_eq!(thread.status, Status::Open);
    }

    /// Relocation rewrites the artifact path in every anchor it finds. An
    /// unanchored comment has none, and must ride along rather than trip it.
    #[test]
    fn an_unanchored_comment_survives_its_sidecars_relocation() {
        let (temp, _) = root_with_session_comment();
        let unanchored = add_unanchored(temp.path(), &session(), "About the whole thing.")
            .expect("add unanchored");

        relocate(
            temp.path(),
            &session(),
            &change(),
            SESSION_NOTE,
            CHANGE_NOTE,
        )
        .expect("relocate");

        let threads = read(temp.path(), &change()).expect("read");
        assert_eq!(threads.len(), 2);
        assert_eq!(threads[1].comment, unanchored);
        assert_eq!(
            threads[0]
                .comment
                .anchor
                .as_ref()
                .expect("still anchored")
                .artifact_path,
            CHANGE_NOTE
        );
    }

    #[test]
    fn an_edited_comment_reads_back_with_its_latest_body() {
        let temp = root_with_artifact();
        let comment =
            add(temp.path(), &change(), ARTIFACT, SELECTED, "Wrong body.").expect("add comment");

        edit(temp.path(), &change(), &comment.id, "Corrected body.").expect("edit");
        edit(temp.path(), &change(), &comment.id, "Corrected again.").expect("second edit");

        let threads = read(temp.path(), &change()).expect("read");
        let [thread] = threads.as_slice() else {
            panic!("expected one thread, got {}", threads.len());
        };
        assert_eq!(thread.comment.body, "Corrected again.");
        assert_eq!(
            thread.comment.anchor, comment.anchor,
            "an edit changes the body and nothing else"
        );
    }

    #[test]
    fn an_edit_leaves_the_body_the_comment_was_created_with_in_the_file() {
        let temp = root_with_artifact();
        let path = change().path(temp.path()).expect("path");
        let comment =
            add(temp.path(), &change(), ARTIFACT, SELECTED, "Wrong body.").expect("add comment");
        let after_comment = lines(&path);

        edit(temp.path(), &change(), &comment.id, "Corrected body.").expect("edit");

        let after_edit = lines(&path);
        assert_eq!(after_edit.len(), 2, "an edit is one appended line");
        assert_eq!(
            after_edit[..1],
            after_comment[..],
            "the original comment line is left exactly as written"
        );
        assert!(
            after_edit[0].contains("Wrong body."),
            "the body first written is still on the record: {}",
            after_edit[0]
        );
    }

    #[test]
    fn editing_a_comment_this_scope_does_not_hold_is_an_error() {
        let temp = root_with_artifact();
        let comment =
            add(temp.path(), &change(), ARTIFACT, SELECTED, "Body.").expect("add comment");
        // A reply is not editable: its id is not a comment id.
        let reply = reply(temp.path(), &change(), &comment.id, "Done.").expect("reply");

        for unknown in ["not-a-comment-id", reply.id.as_str()] {
            let error =
                edit(temp.path(), &change(), unknown, "New body.").expect_err("unknown comment");

            assert!(
                matches!(&error, Error::UnknownComment { comment_id, .. } if comment_id == unknown),
                "{error}"
            );
        }
        assert_eq!(
            lines(&change().path(temp.path()).expect("path")).len(),
            2,
            "the rejected edits are not appended"
        );
    }

    #[test]
    fn a_comment_against_a_missing_artifact_is_an_error() {
        let temp = TempDir::new().expect("temp dir");

        let error =
            add(temp.path(), &change(), ARTIFACT, SELECTED, "Body.").expect_err("missing artifact");

        assert!(matches!(error, Error::MissingArtifact { .. }), "{error}");
        assert!(
            !change().path(temp.path()).expect("path").exists(),
            "a failed add writes no sidecar"
        );
    }

    #[test]
    fn replies_and_status_updates_are_appended_without_rewriting_prior_events() {
        let temp = root_with_artifact();
        let path = change().path(temp.path()).expect("path");
        let comment =
            add(temp.path(), &change(), ARTIFACT, SELECTED, "Why this?").expect("add comment");
        let after_comment = lines(&path);

        reply(temp.path(), &change(), &comment.id, "Because of X.").expect("reply");
        set_status(temp.path(), &change(), &comment.id, Status::Resolved).expect("resolve");

        let after_all = lines(&path);
        assert_eq!(after_all.len(), 3, "three events, three lines");
        assert_eq!(
            after_all[..1],
            after_comment[..],
            "the original comment line is left exactly as written"
        );
    }

    #[test]
    fn reading_back_reconstructs_status_and_thread_history() {
        let temp = root_with_artifact();
        let comment =
            add(temp.path(), &change(), ARTIFACT, SELECTED, "Why this?").expect("add comment");
        reply(temp.path(), &change(), &comment.id, "Because of X.").expect("reply");
        reply(temp.path(), &change(), &comment.id, "Understood.").expect("second reply");
        set_status(temp.path(), &change(), &comment.id, Status::Resolved).expect("resolve");

        let threads = read(temp.path(), &change()).expect("read");

        let [thread] = threads.as_slice() else {
            panic!("expected one thread, got {}", threads.len());
        };
        assert_eq!(thread.comment, comment);
        assert_eq!(thread.status, Status::Resolved);
        assert_eq!(
            thread
                .replies
                .iter()
                .map(|reply| reply.body.as_str())
                .collect::<Vec<_>>(),
            ["Because of X.", "Understood."],
            "replies keep the order they were appended in"
        );
        assert_eq!(thread.status_history.len(), 1);
    }

    #[test]
    fn a_scope_with_no_sidecar_has_no_comments() {
        let temp = TempDir::new().expect("temp dir");

        assert!(read(temp.path(), &change()).expect("read").is_empty());
    }

    #[test]
    fn comments_are_read_back_in_the_order_they_were_made() {
        let temp = root_with_artifact();
        let first = add(temp.path(), &change(), ARTIFACT, "Alpha before.", "First.")
            .expect("first comment");
        let second =
            add(temp.path(), &change(), ARTIFACT, "Omega after.", "Second.").expect("second");

        let ids: Vec<String> = read(temp.path(), &change())
            .expect("read")
            .into_iter()
            .map(|thread| thread.comment.id)
            .collect();

        assert_eq!(ids, [first.id, second.id]);
    }

    #[test]
    fn replying_to_a_comment_this_scope_does_not_hold_is_an_error() {
        let temp = root_with_artifact();
        add(temp.path(), &change(), ARTIFACT, SELECTED, "Why this?").expect("add comment");

        let error =
            reply(temp.path(), &change(), "not-a-comment-id", "Hello?").expect_err("unknown id");

        assert!(matches!(error, Error::UnknownComment { .. }), "{error}");
        assert_eq!(
            lines(&change().path(temp.path()).expect("path")).len(),
            1,
            "the rejected reply is not appended"
        );
    }

    #[test]
    fn a_status_change_to_a_comment_this_scope_does_not_hold_is_an_error() {
        let temp = root_with_artifact();
        add(temp.path(), &change(), ARTIFACT, SELECTED, "Why this?").expect("add comment");

        for to in [Status::Addressed, Status::Resolved, Status::Open] {
            let error =
                set_status(temp.path(), &change(), "not-a-comment-id", to).expect_err("unknown id");

            assert!(matches!(error, Error::UnknownComment { .. }), "{error}");
        }
        assert_eq!(
            lines(&change().path(temp.path()).expect("path")).len(),
            1,
            "no orphan status event is appended"
        );
    }

    /// The status an agent claims and the status the reviewer grants are
    /// separate steps, and the sidecar has to reconstruct both.
    #[test]
    fn a_comment_moves_from_open_through_addressed_to_resolved() {
        let temp = root_with_artifact();
        let comment =
            add(temp.path(), &change(), ARTIFACT, SELECTED, "Why this?").expect("add comment");
        let current = |temp: &TempDir| read(temp.path(), &change()).expect("read")[0].status;

        assert_eq!(current(&temp), Status::Open, "a new comment starts open");

        set_status(temp.path(), &change(), &comment.id, Status::Addressed).expect("address");
        assert_eq!(current(&temp), Status::Addressed);

        set_status(temp.path(), &change(), &comment.id, Status::Resolved).expect("resolve");
        assert_eq!(current(&temp), Status::Resolved);
    }

    /// A reviewer who rejects the response reopens the thread rather than
    /// filing a duplicate, from whichever status it had reached.
    #[test]
    fn a_comment_can_be_reopened_from_either_later_status() {
        for reached in [Status::Addressed, Status::Resolved] {
            let temp = root_with_artifact();
            let comment =
                add(temp.path(), &change(), ARTIFACT, SELECTED, "Why this?").expect("add comment");
            set_status(temp.path(), &change(), &comment.id, reached).expect("advance");

            set_status(temp.path(), &change(), &comment.id, Status::Open).expect("reopen");

            assert_eq!(
                read(temp.path(), &change()).expect("read")[0].status,
                Status::Open,
                "reopening from {reached}"
            );
        }
    }

    #[test]
    fn reopening_keeps_the_superseded_statuses_in_the_history() {
        let temp = root_with_artifact();
        let comment =
            add(temp.path(), &change(), ARTIFACT, SELECTED, "Why this?").expect("add comment");
        reply(temp.path(), &change(), &comment.id, "Done in X.").expect("reply");
        for to in [Status::Addressed, Status::Resolved, Status::Open] {
            set_status(temp.path(), &change(), &comment.id, to).expect("status");
        }

        let threads = read(temp.path(), &change()).expect("read");

        let [thread] = threads.as_slice() else {
            panic!("expected one thread, got {}", threads.len());
        };
        assert_eq!(
            thread
                .status_history
                .iter()
                .map(|update| update.status)
                .collect::<Vec<_>>(),
            [Status::Addressed, Status::Resolved, Status::Open],
            "the whole route the comment took is kept"
        );
        assert_eq!(thread.replies.len(), 1, "the thread survives a reopen");
    }

    #[test]
    fn counts_report_each_comment_once_under_where_it_ended_up() {
        let temp = root_with_artifact();
        // Two comments left open, one addressed, one resolved via addressed.
        for body in ["First.", "Second."] {
            add(temp.path(), &change(), ARTIFACT, SELECTED, body).expect("open comment");
        }
        let addressed =
            add(temp.path(), &change(), ARTIFACT, SELECTED, "Third.").expect("add comment");
        set_status(temp.path(), &change(), &addressed.id, Status::Addressed).expect("address");
        let resolved =
            add(temp.path(), &change(), ARTIFACT, SELECTED, "Fourth.").expect("add comment");
        set_status(temp.path(), &change(), &resolved.id, Status::Addressed).expect("address");
        set_status(temp.path(), &change(), &resolved.id, Status::Resolved).expect("resolve");

        assert_eq!(
            counts(temp.path(), &change()).expect("counts"),
            StatusCounts {
                open: 2,
                addressed: 1,
                resolved: 1,
            },
            "the superseded addressed event must not be counted twice"
        );
    }

    #[test]
    fn a_scope_with_no_sidecar_counts_zero_in_every_status() {
        let temp = TempDir::new().expect("temp dir");

        assert_eq!(
            counts(temp.path(), &change()).expect("counts"),
            StatusCounts::default()
        );
    }

    #[test]
    fn a_malformed_sidecar_line_is_an_error() {
        let temp = root_with_artifact();
        add(temp.path(), &change(), ARTIFACT, SELECTED, "Why this?").expect("add comment");
        let path = change().path(temp.path()).expect("path");
        fs::write(&path, format!("{}\n{{ not json\n", lines(&path).join("\n")))
            .expect("corrupt sidecar");

        let error = read(temp.path(), &change()).expect_err("malformed sidecar");

        assert!(matches!(error, Error::CommentSidecar { .. }), "{error}");
    }

    #[test]
    fn an_event_naming_a_comment_that_was_never_made_is_an_error() {
        let temp = root_with_artifact();
        let path = change().path(temp.path()).expect("path");
        fs::create_dir_all(path.parent().expect("parent")).expect("create dirs");
        fs::write(
            &path,
            r#"{"type":"reply","reply":{"id":"r1","commentId":"ghost","body":"Hello?","createdAt":"2026-07-31T08:00:00Z"}}
"#,
        )
        .expect("write dangling event");

        let error = read(temp.path(), &change()).expect_err("dangling event");

        assert!(matches!(error, Error::UnknownComment { .. }), "{error}");
    }

    /// The scratch-note paths a session's comments are anchored to before and
    /// after promotion.
    const SESSION_NOTE: &str = ".openspec-doc/scratch/_session/session-a.md";
    const CHANGE_NOTE: &str = ".openspec-doc/scratch/add-thing.md";

    /// A project root whose session-keyed sidecar holds one comment against the
    /// session's scratch note.
    fn root_with_session_comment() -> (TempDir, Comment) {
        let temp = TempDir::new().expect("temp dir");
        let note = temp.path().join(SESSION_NOTE);
        fs::create_dir_all(note.parent().expect("parent")).expect("create dirs");
        fs::write(note, MARKDOWN).expect("write note");

        let comment = add(
            temp.path(),
            &session(),
            SESSION_NOTE,
            SELECTED,
            "Worth keeping.",
        )
        .expect("add comment");

        (temp, comment)
    }

    #[test]
    fn a_relocated_sidecar_keeps_its_events_at_the_new_key() {
        let (temp, comment) = root_with_session_comment();
        reply(temp.path(), &session(), &comment.id, "Agreed.").expect("reply");

        relocate(
            temp.path(),
            &session(),
            &change(),
            SESSION_NOTE,
            CHANGE_NOTE,
        )
        .expect("relocate");

        assert!(
            !session().path(temp.path()).expect("path").exists(),
            "the old sidecar is gone"
        );
        let threads = read(temp.path(), &change()).expect("read at the new key");
        let [thread] = threads.as_slice() else {
            panic!("expected one thread, got {}", threads.len());
        };
        assert_eq!(thread.comment.id, comment.id);
        assert_eq!(thread.comment.body, "Worth keeping.");
        assert_eq!(thread.replies.len(), 1, "the reply survived");
    }

    #[test]
    fn a_relocated_comment_anchors_to_the_artifacts_new_path() {
        let (temp, _) = root_with_session_comment();
        // Promotion renames the note underneath the comment.
        fs::rename(
            temp.path().join(SESSION_NOTE),
            temp.path().join(CHANGE_NOTE),
        )
        .expect("rename note");

        relocate(
            temp.path(),
            &session(),
            &change(),
            SESSION_NOTE,
            CHANGE_NOTE,
        )
        .expect("relocate");

        let threads = read(temp.path(), &change()).expect("read");
        let anchor = threads[0].comment.anchor.as_ref().expect("still anchored");
        assert_eq!(anchor.artifact_path, CHANGE_NOTE);
        assert_eq!(
            artifact::resolve(temp.path(), Some(anchor))
                .expect("resolve")
                .state,
            anchor::AnchorState::Exact,
            "the comment still points at its text after promotion"
        );
    }

    #[test]
    fn relocating_a_scope_with_no_sidecar_is_a_no_op() {
        let temp = TempDir::new().expect("temp dir");

        relocate(
            temp.path(),
            &session(),
            &change(),
            SESSION_NOTE,
            CHANGE_NOTE,
        )
        .expect("relocate");

        assert!(!change().path(temp.path()).expect("path").exists());
    }

    #[test]
    fn relocating_onto_an_existing_sidecar_is_refused() {
        let (temp, _) = root_with_session_comment();
        let occupied = change().path(temp.path()).expect("path");
        fs::write(&occupied, "").expect("occupy the target");

        let error = relocate(
            temp.path(),
            &session(),
            &change(),
            SESSION_NOTE,
            CHANGE_NOTE,
        )
        .expect_err("occupied target");

        assert!(matches!(error, Error::SidecarExists { .. }), "{error}");
        assert!(
            session().path(temp.path()).expect("path").exists(),
            "the events stay where they are rather than being overwritten"
        );
    }
}
