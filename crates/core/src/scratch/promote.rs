use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::comments::{self, ScopeKey};
use crate::error::Error;
use crate::root::Project;

use super::{note, snapshot};

const OPENSPEC_BIN: &str = "openspec";

/// The outcome of the `openspec validate <name>` run that follows a promotion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validation {
    pub passed: bool,
    /// The validator's combined stdout and stderr, retained either way so a
    /// failure is available to the caller rather than discarded.
    pub output: String,
}

/// What a promotion check did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Promotion {
    /// Nothing to promote: this session had no prior snapshot to diff against,
    /// no new active change appeared since it, or the session has no scratch
    /// note to move.
    NotPromoted,
    /// Several active changes appeared at once, so which one formalized this
    /// session's exploration is unknowable. Promotion is skipped and the
    /// snapshot is left unadvanced, so the ambiguity keeps being reported
    /// instead of being silently forgotten.
    Ambiguous { candidates: Vec<String> },
    /// The session's scratch note now lives at the change-name key.
    Promoted {
        change: String,
        note_path: PathBuf,
        validation: Validation,
    },
}

/// Promote `session_id`'s scratch note when exactly one new active change has
/// appeared since this session's previous check, then validate that change.
///
/// The first call for a session only records its baseline snapshot. The
/// snapshot is taken whether or not a scratch note exists yet, so that a note
/// written mid-session still has a baseline to be promoted against.
pub fn check(project: &Project, session_id: &str) -> Result<Promotion, Error> {
    let current = snapshot::capture(project)?;

    let Some(previous) = snapshot::load(&project.root, session_id)? else {
        snapshot::store(&project.root, session_id, &current)?;
        return Ok(Promotion::NotPromoted);
    };

    let appeared: Vec<String> = current
        .iter()
        .filter(|name| !previous.contains(name))
        .cloned()
        .collect();

    if appeared.len() > 1 {
        return Ok(Promotion::Ambiguous {
            candidates: appeared,
        });
    }

    snapshot::store(&project.root, session_id, &current)?;

    let [change] = appeared.as_slice() else {
        return Ok(Promotion::NotPromoted);
    };

    let session_note = note::session_path(&project.root, session_id)?;
    if !session_note.exists() {
        return Ok(Promotion::NotPromoted);
    }

    let note_path = note::change_path(&project.root, change);
    fs::rename(&session_note, &note_path).map_err(|source| Error::Rename {
        from: session_note.clone(),
        to: note_path.clone(),
        source,
    })?;

    // Written only once the rename has landed, so a reader can never be sent to
    // a path that does not hold the note yet.
    let pointer = moved_pointer(change);
    fs::write(&session_note, pointer).map_err(|source| Error::write(session_note, source))?;

    // The note's comments are keyed by the same scope, so they move with it.
    comments::relocate(
        &project.root,
        &ScopeKey::Session(session_id.to_owned()),
        &ScopeKey::Change(change.clone()),
        &note::session_relative(session_id)?,
        &note::change_relative(change),
    )?;

    let validation = validate(project, change)?;

    Ok(Promotion::Promoted {
        change: change.clone(),
        note_path,
        validation,
    })
}

/// The record left at the vacated session-scoped path. The marker comment makes
/// the redirect machine-readable while staying invisible when rendered.
fn moved_pointer(change: &str) -> String {
    let target = note::change_relative(change);

    format!("<!-- openspec-doc:moved-to {target} -->\n\nThis scratch note moved to `{target}`.\n")
}

/// Run `openspec validate <change>` from the project root.
fn validate(project: &Project, change: &str) -> Result<Validation, Error> {
    let output = Command::new(OPENSPEC_BIN)
        .arg("validate")
        .arg(change)
        .current_dir(&project.root)
        .output()
        .map_err(|source| Error::Validate {
            change: change.to_owned(),
            source,
        })?;

    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&output.stderr));

    Ok(Validation {
        passed: output.status.success(),
        output: combined,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tempfile::TempDir;

    use super::*;
    use crate::root::project_at;
    use crate::test_fixture::project_fixture;

    /// A project fixture with a scratch note for `session-a` holding `note`, and
    /// a baseline snapshot already recorded.
    fn explored_project(note: &str) -> (TempDir, Project) {
        let fixture = project_fixture(&[]);
        let project = project_at(fixture.path()).expect("project root");

        fs::write(
            note::ensure_session(&project.root, "session-a").expect("create note"),
            note,
        )
        .expect("write note");
        assert_eq!(
            check(&project, "session-a").expect("baseline check"),
            Promotion::NotPromoted,
            "the first check only records the baseline"
        );

        (fixture, project)
    }

    /// A change directory under `project` that `openspec validate` accepts.
    fn write_valid_change(project: &Project, name: &str) {
        let dir = project.openspec_dir.join("changes").join(name);
        fs::create_dir_all(dir.join("specs/some-cap")).expect("create change dir");
        fs::write(
            dir.join("proposal.md"),
            "## Why\nBecause.\n\n## What Changes\n- A thing.\n",
        )
        .expect("write proposal");
        fs::write(
            dir.join("specs/some-cap/spec.md"),
            "## ADDED Requirements\n\n### Requirement: A thing\nThe system SHALL do a thing.\n\n#### Scenario: It works\n- **WHEN** asked\n- **THEN** it does the thing\n",
        )
        .expect("write spec");
        fs::write(dir.join("tasks.md"), "- [ ] 1.1 Do the thing\n").expect("write tasks");
    }

    /// A change directory under `project` that `openspec validate` rejects: a
    /// proposal with no spec deltas at all.
    fn write_invalid_change(project: &Project, name: &str) {
        let dir = project.openspec_dir.join("changes").join(name);
        fs::create_dir_all(&dir).expect("create change dir");
        fs::write(dir.join("proposal.md"), "## Why\nBecause.\n").expect("write proposal");
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).expect("read file")
    }

    #[test]
    fn exactly_one_new_change_promotes_the_note_to_its_name() {
        let (_fixture, project) = explored_project("# Exploration\n");
        write_valid_change(&project, "add-thing");

        let promotion = check(&project, "session-a").expect("check");

        let Promotion::Promoted {
            change, note_path, ..
        } = promotion
        else {
            panic!("expected a promotion, got {promotion:?}");
        };
        assert_eq!(change, "add-thing");
        assert_eq!(note_path, note::change_path(&project.root, "add-thing"));
        assert_eq!(
            read(&note_path),
            "# Exploration\n",
            "the note's content moves intact"
        );
    }

    #[test]
    fn the_vacated_session_path_points_at_the_new_one() {
        let (_fixture, project) = explored_project("# Exploration\n");
        write_valid_change(&project, "add-thing");

        check(&project, "session-a").expect("check");

        let session_note = note::session_path(&project.root, "session-a").expect("path");
        let pointer = read(&session_note);
        assert!(
            pointer.contains(".openspec-doc/scratch/add-thing.md"),
            "pointer must identify the new path, got {pointer:?}"
        );
        assert!(
            !pointer.contains("# Exploration"),
            "the pointer is a redirect, not a copy of the note"
        );
    }

    #[test]
    fn a_passing_validation_is_recorded_with_its_output() {
        let (_fixture, project) = explored_project("# Exploration\n");
        write_valid_change(&project, "add-thing");

        let promotion = check(&project, "session-a").expect("check");

        let Promotion::Promoted { validation, .. } = promotion else {
            panic!("expected a promotion, got {promotion:?}");
        };
        assert!(
            validation.passed,
            "validation output: {}",
            validation.output
        );
        assert!(
            !validation.output.is_empty(),
            "the validator's output is retained"
        );
    }

    #[test]
    fn a_failing_validation_is_surfaced_rather_than_discarded() {
        let (_fixture, project) = explored_project("# Exploration\n");
        write_invalid_change(&project, "add-broken");

        let promotion = check(&project, "session-a").expect("check");

        let Promotion::Promoted {
            change,
            note_path,
            validation,
        } = promotion
        else {
            panic!("expected a promotion, got {promotion:?}");
        };
        assert_eq!(change, "add-broken");
        assert!(
            note_path.is_file(),
            "the rename still happens for an invalid change"
        );
        assert!(!validation.passed);
        assert!(
            !validation.output.is_empty(),
            "the failure output is retained"
        );
    }

    #[test]
    fn the_notes_comments_move_with_it_and_still_anchor_to_it() {
        let (_fixture, project) = explored_project("# Exploration\n\nAn idea worth keeping.\n");
        let session_scope = ScopeKey::Session("session-a".to_owned());
        let comment = comments::add(
            &project.root,
            &session_scope,
            &note::session_relative("session-a").expect("relative path"),
            "An idea worth keeping.",
            "Which part is worth keeping?",
        )
        .expect("add comment");
        write_valid_change(&project, "add-thing");

        check(&project, "session-a").expect("check");

        let change_scope = ScopeKey::Change("add-thing".to_owned());
        assert!(
            !session_scope.path(&project.root).expect("path").exists(),
            "the sidecar does not stay behind at the session key"
        );
        let threads = comments::read(&project.root, &change_scope).expect("read at the new key");
        let [thread] = threads.as_slice() else {
            panic!("expected one thread, got {}", threads.len());
        };
        assert_eq!(thread.comment.id, comment.id);
        assert_eq!(thread.comment.body, "Which part is worth keeping?");
        assert_eq!(
            thread.comment.anchor.artifact_path,
            note::change_relative("add-thing"),
            "the anchor follows the note to its new path"
        );
        assert_eq!(
            comments::resolve_anchor(&project.root, &thread.comment.anchor)
                .expect("resolve")
                .state,
            comments::AnchorState::Exact,
            "the comment still points at the text it was made against"
        );
    }

    #[test]
    fn no_new_change_is_a_no_op() {
        let (_fixture, project) = explored_project("# Exploration\n");

        assert_eq!(
            check(&project, "session-a").expect("check"),
            Promotion::NotPromoted
        );
        assert_eq!(
            read(&note::session_path(&project.root, "session-a").expect("path")),
            "# Exploration\n",
            "the note stays where it is"
        );
    }

    #[test]
    fn several_new_changes_trigger_the_ambiguity_guard() {
        let (_fixture, project) = explored_project("# Exploration\n");
        write_valid_change(&project, "add-thing");
        write_valid_change(&project, "add-other");

        let promotion = check(&project, "session-a").expect("check");

        assert_eq!(
            promotion,
            Promotion::Ambiguous {
                candidates: vec!["add-other".to_owned(), "add-thing".to_owned()],
            }
        );
        assert_eq!(
            read(&note::session_path(&project.root, "session-a").expect("path")),
            "# Exploration\n",
            "neither candidate is guessed at"
        );
    }

    #[test]
    fn an_unresolved_ambiguity_keeps_being_reported() {
        let (_fixture, project) = explored_project("# Exploration\n");
        write_valid_change(&project, "add-thing");
        write_valid_change(&project, "add-other");

        let first = check(&project, "session-a").expect("first check");
        let second = check(&project, "session-a").expect("second check");

        assert_eq!(first, second, "the snapshot must not swallow the ambiguity");
    }

    #[test]
    fn an_archived_change_does_not_look_new_afterwards() {
        let (_fixture, project) = explored_project("# Exploration\n");
        let changes = project.openspec_dir.join("changes");
        fs::create_dir_all(changes.join("add-unrelated")).expect("create change");

        check(&project, "session-a").expect("absorb the unrelated change");
        fs::create_dir_all(changes.join("archive")).expect("create archive");
        fs::rename(
            changes.join("add-unrelated"),
            changes.join("archive/2026-07-31-add-unrelated"),
        )
        .expect("archive it");

        assert_eq!(
            check(&project, "session-a").expect("check after archiving"),
            Promotion::NotPromoted
        );
    }

    #[test]
    fn a_session_without_a_note_is_never_promoted() {
        let fixture = project_fixture(&[]);
        let project = project_at(fixture.path()).expect("project root");
        check(&project, "session-a").expect("baseline check");
        write_valid_change(&project, "add-thing");

        assert_eq!(
            check(&project, "session-a").expect("check"),
            Promotion::NotPromoted
        );
        assert!(
            !note::change_path(&project.root, "add-thing").exists(),
            "no note means nothing to move"
        );
    }
}
