use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::comments::{self, ScopeKey};
use crate::error::Error;
use crate::root::Project;
use crate::scan::scan;

use super::note;

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
    /// Nothing to promote: the session claimed no change, the change it claimed
    /// is not an active change, it has no scratch note to move, or its note is
    /// the redirect a previous promotion left behind.
    NotPromoted,
    /// The session's scratch note now lives at the change-name key.
    Promoted {
        change: String,
        note_path: PathBuf,
        validation: Validation,
    },
}

/// Promote `session_id`'s scratch note onto the change that session claimed,
/// then validate that change.
///
/// Only a claim promotes. A change directory appearing says nothing about which
/// session created it, so nothing here looks at what appeared: an unclaimed
/// exploration stays at its session-scoped path, visible in the dashboard, and
/// the reviewer can claim it by hand. Losing an automatic move is the correct
/// failure; the alternative is renaming one session's exploration onto another
/// session's change.
pub fn check(project: &Project, session_id: &str) -> Result<Promotion, Error> {
    // First, before anything else: promotion leaves a redirect at the vacated
    // session path, and a redirect is not an exploration. Treating it as one
    // renames a 160-byte pointer onto a later change and destroys the note
    // already there.
    if note::promoted_to(&project.root, session_id)?.is_some() {
        return Ok(Promotion::NotPromoted);
    }

    // The claim lives in the note, so a session with no note has no claim and
    // this one read covers both refusals.
    let Some(change) = note::claimed_change(&project.root, session_id)? else {
        return Ok(Promotion::NotPromoted);
    };

    // A claimed name is agent-supplied, and this is what keeps it from reaching
    // `change_path`: only a name that matches an active change directory does.
    if !active_changes(project)?.contains(&change) {
        return Ok(Promotion::NotPromoted);
    }

    let session_note = note::session_path(&project.root, session_id)?;
    let note_path = note::change_path(&project.root, &change);
    fs::rename(&session_note, &note_path).map_err(|source| Error::Rename {
        from: session_note.clone(),
        to: note_path.clone(),
        source,
    })?;

    // Written only once the rename has landed, so a reader can never be sent to
    // a path that does not hold the note yet.
    let pointer = note::moved_pointer(&change);
    fs::write(&session_note, pointer).map_err(|source| Error::write(session_note, source))?;

    // The note's comments are keyed by the same scope, so they move with it.
    comments::relocate(
        &project.root,
        &ScopeKey::Session(session_id.to_owned()),
        &ScopeKey::Change(change.clone()),
        &note::session_relative(session_id)?,
        &note::change_relative(&change),
    )?;

    let validation = validate(project, &change)?;

    Ok(Promotion::Promoted {
        change,
        note_path,
        validation,
    })
}

/// The active-change names visible in `project` right now.
fn active_changes(project: &Project) -> Result<Vec<String>, Error> {
    Ok(scan(project.clone())?
        .active_changes
        .into_iter()
        .map(|dir| dir.name)
        .collect())
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

    /// A project fixture with a scratch note for `session-a` holding `note`.
    fn explored_project(note: &str) -> (TempDir, Project) {
        let fixture = project_fixture(&[]);
        let project = project_at(fixture.path()).expect("project root");

        write_note(&project, "session-a", note);

        (fixture, project)
    }

    /// A scratch note for `session_id` holding `contents`.
    fn write_note(project: &Project, session_id: &str, contents: &str) {
        fs::write(
            note::prepare_session(&project.root, session_id).expect("prepare note"),
            contents,
        )
        .expect("write note");
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

    /// The session claims `name` the way `openspec-doc scratch claim` does.
    fn claim(project: &Project, session_id: &str, name: &str) {
        note::claim(&project.root, session_id, name).expect("claim");
    }

    #[test]
    fn a_claimed_change_promotes_the_note_to_its_name() {
        let (_fixture, project) = explored_project("# Exploration\n");
        write_valid_change(&project, "add-thing");
        claim(&project, "session-a", "add-thing");

        let promotion = check(&project, "session-a").expect("check");

        let Promotion::Promoted {
            change, note_path, ..
        } = promotion
        else {
            panic!("expected a promotion, got {promotion:?}");
        };
        assert_eq!(change, "add-thing");
        assert_eq!(note_path, note::change_path(&project.root, "add-thing"));
        assert!(
            read(&note_path).starts_with("# Exploration\n"),
            "the note's content moves intact"
        );
    }

    /// The claim marker travels with the note, so the formalized exploration
    /// states which change it became.
    #[test]
    fn the_promoted_note_records_what_it_came_from() {
        let (_fixture, project) = explored_project("# Exploration\n");
        write_valid_change(&project, "add-thing");
        claim(&project, "session-a", "add-thing");

        check(&project, "session-a").expect("check");

        let promoted = read(&note::change_path(&project.root, "add-thing"));
        assert!(
            promoted.contains("add-thing"),
            "the promoted note does not name its change: {promoted:?}"
        );
    }

    #[test]
    fn the_vacated_session_path_points_at_the_new_one() {
        let (_fixture, project) = explored_project("# Exploration\n");
        write_valid_change(&project, "add-thing");
        claim(&project, "session-a", "add-thing");

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
        claim(&project, "session-a", "add-thing");

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
        claim(&project, "session-a", "add-broken");

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
        claim(&project, "session-a", "add-thing");

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
    fn a_session_that_claimed_nothing_is_a_no_op() {
        let (_fixture, project) = explored_project("# Exploration\n");
        write_valid_change(&project, "add-thing");

        assert_eq!(
            check(&project, "session-a").expect("check"),
            Promotion::NotPromoted
        );
        assert_eq!(
            read(&note::session_path(&project.root, "session-a").expect("path")),
            "# Exploration\n",
            "the note stays where it is"
        );
        assert!(
            !note::change_path(&project.root, "add-thing").exists(),
            "an unclaimed change gets no note"
        );
    }

    /// The defect this change exists to fix. Creating a change from one session
    /// used to make every other session with a note eligible to be renamed onto
    /// it — a 29KB exploration was one turn boundary from landing on an
    /// unrelated change. Without a claim, neither session moves.
    #[test]
    fn a_change_from_one_session_does_not_promote_anothers_exploration() {
        let (_fixture, project) = explored_project("# Exploration A\n");
        write_note(&project, "session-b", "# Exploration B\n");
        write_valid_change(&project, "add-thing");

        assert_eq!(
            check(&project, "session-a").expect("check a"),
            Promotion::NotPromoted
        );
        assert_eq!(
            check(&project, "session-b").expect("check b"),
            Promotion::NotPromoted
        );
        assert_eq!(
            read(&note::session_path(&project.root, "session-a").expect("path")),
            "# Exploration A\n"
        );
        assert_eq!(
            read(&note::session_path(&project.root, "session-b").expect("path")),
            "# Exploration B\n"
        );
    }

    /// The other side of it: the session that did the work is promoted, and the
    /// idle one beside it is left alone.
    #[test]
    fn only_the_claiming_session_is_promoted() {
        let (_fixture, project) = explored_project("# Exploration A\n");
        write_note(&project, "session-b", "# Exploration B\n");
        write_valid_change(&project, "add-thing");
        claim(&project, "session-a", "add-thing");

        assert!(matches!(
            check(&project, "session-a").expect("check a"),
            Promotion::Promoted { .. }
        ));
        assert_eq!(
            check(&project, "session-b").expect("check b"),
            Promotion::NotPromoted
        );
        assert_eq!(
            read(&note::session_path(&project.root, "session-b").expect("path")),
            "# Exploration B\n",
            "the idle session's exploration was touched"
        );
    }

    #[test]
    fn a_claim_naming_no_active_change_is_a_no_op() {
        let (_fixture, project) = explored_project("# Exploration\n");
        claim(&project, "session-a", "add-imaginary");

        assert_eq!(
            check(&project, "session-a").expect("check"),
            Promotion::NotPromoted
        );
        assert!(
            !note::change_path(&project.root, "add-imaginary").exists(),
            "a claim on nothing created a note anyway"
        );
    }

    #[test]
    fn a_claim_naming_an_archived_change_is_a_no_op() {
        let (_fixture, project) = explored_project("# Exploration\n");
        let changes = project.openspec_dir.join("changes");
        fs::create_dir_all(changes.join("archive/2026-07-31-add-old")).expect("create archive");
        claim(&project, "session-a", "2026-07-31-add-old");

        assert_eq!(
            check(&project, "session-a").expect("check"),
            Promotion::NotPromoted
        );
    }

    #[test]
    fn a_session_without_a_note_is_never_promoted() {
        let fixture = project_fixture(&[]);
        let project = project_at(fixture.path()).expect("project root");
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

    /// The second defect: after promotion the session path holds a redirect, and
    /// a check that tests only for existence renames that 160-byte pointer onto
    /// the next change, destroying the note already there.
    #[test]
    fn an_already_promoted_session_is_not_promoted_again() {
        let (_fixture, project) = explored_project("# Exploration\n");
        write_valid_change(&project, "add-thing");
        claim(&project, "session-a", "add-thing");
        check(&project, "session-a").expect("first promotion");

        write_valid_change(&project, "add-other");
        claim(&project, "session-a", "add-other");
        let second = check(&project, "session-a").expect("second check");

        assert_eq!(second, Promotion::NotPromoted);
        let redirect = read(&note::session_path(&project.root, "session-a").expect("path"));
        assert!(
            redirect.contains(".openspec-doc/scratch/add-thing.md"),
            "the redirect no longer points at the promoted note: {redirect:?}"
        );
        assert!(
            !note::change_path(&project.root, "add-other").exists(),
            "the redirect was renamed onto the later change"
        );
        assert!(
            read(&note::change_path(&project.root, "add-thing")).contains("# Exploration"),
            "the promoted exploration was overwritten"
        );
    }
}
