//! Noticing at turn end that implementation has progressed on a change nobody
//! approved, and raising it through the pending-directive mechanism.
//!
//! This is after the fact by construction, and the wording everywhere says so. A
//! Stop hook's entire vocabulary is block-and-continue or allow-stop, so it
//! cannot prevent an action that has already happened — blocking makes the agent
//! do *more* work, which is the opposite of a veto. What is deliverable is a
//! report: the reviewer and the agent both learn the change went in unreviewed,
//! where the alternative is that nobody learns it.

use std::fs;
use std::io;
use std::path::PathBuf;

use crate::approval::{self, Approval};
use crate::error::Error;
use crate::root::Project;
use crate::scratch;

use super::directive;

/// Where the last unapproved-implementation report for each change is recorded,
/// so the same state is not reported at every turn boundary until it changes.
const REPORTED_DIR: &str = ".openspec-doc/approval";

/// What every directive opens with, so the agent can place it as a report from a
/// tool in its own project rather than as text of unknown origin.
const ATTRIBUTION: &str = "Review feedback from this project's openspec-doc dashboard";

/// Report, once per unapproved state, that `session_id`'s change has task
/// progress without a current approval. Returns the change reported on.
///
/// Nothing is reported for a session whose exploration was never promoted: a
/// change is the only thing that can be approved, and the promotion record is
/// the only thing that says which change this session's work belongs to.
pub fn report(project: &Project, session_id: &str) -> Result<Option<String>, Error> {
    let Some(change) = scratch::promoted_to(&project.root, session_id)? else {
        return Ok(None);
    };

    // Read before the approval state is evaluated, because an archived change
    // has no tasks file and no approval state to ask about either — and asking
    // would fail every turn boundary for the rest of the session.
    let completed = completed_tasks(project, &change)?;
    if completed == 0 {
        return Ok(None);
    }

    // A directive already waiting is one the next turn boundary will inject.
    // Writing over it would drop it, and this report is not marked either, so it
    // is made again at the boundary after that.
    if directive::load_pending(&project.root, session_id)?.is_some() {
        return Ok(None);
    }

    let approval = approval::state(project, &change)?;
    if approval.state.is_approved() {
        return Ok(None);
    }

    let signature = signature(&approval, completed);
    if read_reported(project, &change)?.as_deref() == Some(signature.as_str()) {
        return Ok(None);
    }

    directive::write_pending(&project.root, session_id, &reason(&change, &approval))?;
    write_reported(project, &change, &signature)?;

    Ok(Some(change))
}

/// The reason text for an unapproved change with work already done on it.
///
/// An attributed pointer like every other directive: it says where it came from
/// and names files and commands in the agent's own project, so every claim it
/// makes is one the agent can check. It embeds no artifact or comment content —
/// the state's own reason names artifacts that changed, which is a list of file
/// names rather than anything written in them.
pub fn reason(change: &str, approval: &Approval) -> String {
    format!(
        "{ATTRIBUTION}: tasks are ticked off on change `{change}` while its approval state is \
         {state} — {reason}. `openspec-doc approval state --change {change}` reports that state \
         and exits non-zero while the change is not cleared; the tasks are in \
         `openspec/changes/{change}/tasks.md` and the artifacts an approval would cover are \
         beside them. This is a report and not a veto: a turn-end hook runs after the work, so \
         nothing here has undone anything. Tell the owner where the implementation has got to, \
         and that it ran ahead of the review, rather than carrying on quietly. \
         `openspec-doc serve url` prints where they approve it.",
        state = approval.state,
        reason = approval.reason,
    )
}

/// What makes one unapproved state distinguishable from the next: which state it
/// is, the record it was decided from, and how much of the change has been
/// implemented.
///
/// Task progress is in it because further implementation on an unapproved change
/// is news; the record is in it because a withdrawal after an approval leaves the
/// state word and the task count unchanged while being a different fact.
fn signature(approval: &Approval, completed: usize) -> String {
    format!(
        "{}:{}:{completed}",
        approval.state.label(),
        approval.at.as_deref().unwrap_or("never")
    )
}

/// How many of `change`'s tasks are ticked off.
///
/// The checkbox state of `tasks.md` is the only implementation signal available
/// without reading the transcript or the git index. A change with no tasks file
/// has no progress to notice.
fn completed_tasks(project: &Project, change: &str) -> Result<usize, Error> {
    let path = project
        .openspec_dir
        .join("changes")
        .join(change)
        .join("tasks.md");

    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(source) => return Err(Error::io(path, source)),
    };

    Ok(contents
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            line.starts_with("- [x]") || line.starts_with("- [X]")
        })
        .count())
}

fn reported_path(project: &Project, change: &str) -> PathBuf {
    project
        .root
        .join(REPORTED_DIR)
        .join(format!("{change}.reported"))
}

fn read_reported(project: &Project, change: &str) -> Result<Option<String>, Error> {
    let path = reported_path(project, change);
    match fs::read_to_string(&path) {
        Ok(contents) => Ok(Some(contents)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(Error::io(path, source)),
    }
}

fn write_reported(project: &Project, change: &str, signature: &str) -> Result<(), Error> {
    let path = reported_path(project, change);
    let dir = path.parent().expect("the reported marker has a parent");
    fs::create_dir_all(dir).map_err(|source| Error::write(dir, source))?;

    fs::write(&path, signature).map_err(|source| Error::write(path, source))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approval::State;
    use crate::comments::ScopeKey;
    use crate::root::project_at;
    use crate::test_fixture::project_fixture;
    use crate::verdict;
    use tempfile::TempDir;

    const SESSION: &str = "session-a";
    const CHANGE: &str = "add-thing";

    /// A session promoted to a change with one task ticked off, and no approval.
    fn implementing_project() -> (TempDir, Project) {
        let fixture = project_fixture(&["openspec/changes/add-thing"]);
        let project = project_at(fixture.path()).expect("project root");
        let dir = project.openspec_dir.join("changes/add-thing");
        fs::write(dir.join("proposal.md"), "## Why\n\nBecause.\n").expect("write proposal");
        fs::write(dir.join("tasks.md"), "- [x] 1.1 Done\n- [ ] 1.2 Not yet\n")
            .expect("write tasks");

        fs::write(
            scratch::prepare_session(&project.root, SESSION).expect("prepare note"),
            format!(
                "<!-- openspec-doc:moved-to {} -->\n",
                scratch::change_relative(CHANGE)
            ),
        )
        .expect("write redirect");

        (fixture, project)
    }

    fn pending_reason(project: &Project) -> Option<String> {
        directive::load_pending(&project.root, SESSION)
            .expect("load")
            .map(|directive| directive.reason)
    }

    fn consume(project: &Project) {
        let directive = directive::load_pending(&project.root, SESSION)
            .expect("load")
            .expect("pending directive");
        directive::mark_consumed(&project.root, SESSION, &directive).expect("consume");
    }

    #[test]
    fn task_progress_without_approval_raises_a_directive() {
        let (_fixture, project) = implementing_project();

        assert_eq!(
            report(&project, SESSION).expect("report"),
            Some(CHANGE.to_owned())
        );
        let reason = pending_reason(&project).expect("a directive was written");
        assert!(reason.contains(CHANGE), "{reason}");
        assert!(
            reason.contains(&format!("openspec-doc approval state --change {CHANGE}")),
            "{reason}"
        );
    }

    #[test]
    fn a_current_approval_raises_nothing() {
        let (_fixture, project) = implementing_project();
        approval::submit(&project, CHANGE).expect("approve");

        assert_eq!(report(&project, SESSION).expect("report"), None);
        assert_eq!(pending_reason(&project), None);
    }

    #[test]
    fn a_stale_approval_raises_a_directive_naming_the_state() {
        let (_fixture, project) = implementing_project();
        approval::submit(&project, CHANGE).expect("approve");
        fs::write(
            project.openspec_dir.join("changes/add-thing/proposal.md"),
            "## Why\n\nA different reason.\n",
        )
        .expect("edit proposal");

        assert_eq!(
            report(&project, SESSION).expect("report"),
            Some(CHANGE.to_owned())
        );
        let reason = pending_reason(&project).expect("a directive was written");
        assert!(reason.contains("stale"), "{reason}");
        assert!(reason.contains("proposal.md"), "{reason}");
    }

    #[test]
    fn the_same_unapproved_state_is_not_reported_twice() {
        let (_fixture, project) = implementing_project();

        report(&project, SESSION).expect("first report");
        consume(&project);

        assert_eq!(report(&project, SESSION).expect("second report"), None);
        assert_eq!(
            pending_reason(&project),
            None,
            "no second directive for the same state"
        );
    }

    #[test]
    fn further_task_progress_is_reported_again() {
        let (_fixture, project) = implementing_project();
        report(&project, SESSION).expect("first report");
        consume(&project);

        fs::write(
            project.openspec_dir.join("changes/add-thing/tasks.md"),
            "- [x] 1.1 Done\n- [x] 1.2 Also done\n",
        )
        .expect("tick another task");

        assert_eq!(
            report(&project, SESSION).expect("second report"),
            Some(CHANGE.to_owned())
        );
    }

    /// The state word and the task count are unchanged by a withdrawal, so the
    /// record the state was decided from is what tells the two apart.
    #[test]
    fn a_withdrawal_after_an_approval_is_reported() {
        let (_fixture, project) = implementing_project();
        report(&project, SESSION).expect("first report");
        consume(&project);
        approval::submit(&project, CHANGE).expect("approve");
        approval::withdraw(&project, CHANGE).expect("withdraw");

        assert_eq!(
            report(&project, SESSION).expect("second report"),
            Some(CHANGE.to_owned())
        );
    }

    #[test]
    fn a_change_with_no_completed_tasks_raises_nothing() {
        let (_fixture, project) = implementing_project();
        fs::write(
            project.openspec_dir.join("changes/add-thing/tasks.md"),
            "- [ ] 1.1 Not yet\n",
        )
        .expect("untick");

        assert_eq!(report(&project, SESSION).expect("report"), None);
    }

    #[test]
    fn a_session_with_no_promoted_change_raises_nothing() {
        let fixture = project_fixture(&[]);
        let project = project_at(fixture.path()).expect("project root");
        fs::write(
            scratch::prepare_session(&project.root, SESSION).expect("prepare note"),
            "# Exploration\n",
        )
        .expect("write note");

        assert_eq!(report(&project, SESSION).expect("report"), None);
    }

    /// An archived change has no tasks file at the active path, and asking for
    /// its approval state would fail at every turn boundary for the rest of the
    /// session.
    #[test]
    fn a_change_that_is_no_longer_active_raises_nothing() {
        let (_fixture, project) = implementing_project();
        fs::remove_dir_all(project.openspec_dir.join("changes/add-thing"))
            .expect("archive the change");

        assert_eq!(report(&project, SESSION).expect("report"), None);
    }

    /// A verdict directive waiting to be injected is feedback the reviewer wrote.
    /// Writing over it would drop it for a report that can be made again.
    #[test]
    fn a_directive_still_waiting_is_not_written_over() {
        let (_fixture, project) = implementing_project();
        verdict::add(
            &project.root,
            &ScopeKey::Change(CHANGE.to_owned()),
            verdict::Verdict::CommentResolution,
            "",
        )
        .expect("add verdict");
        super::super::translate(&project, SESSION).expect("translate");

        assert_eq!(report(&project, SESSION).expect("report"), None);
        assert!(
            pending_reason(&project)
                .expect("the verdict directive")
                .contains("sent the open comments"),
            "the reviewer's own feedback survives"
        );
    }

    /// The directive says which files to read, never what is in them.
    #[test]
    fn the_directive_is_an_attributed_pointer() {
        let approval = Approval {
            state: State::NotApproved,
            reason: "no approval has been recorded for this change".to_owned(),
            at: None,
        };

        let reason = reason(CHANGE, &approval);

        assert!(reason.starts_with(ATTRIBUTION), "{reason}");
        assert!(
            reason.contains(&format!("openspec/changes/{CHANGE}/tasks.md")),
            "{reason}"
        );
        for phrase in ["exactly", "verbatim", "nothing else", "respond with"] {
            assert!(
                !reason.to_lowercase().contains(phrase),
                "the directive demands literal output ({phrase:?}): {reason}"
            );
        }
    }

    /// "Gate" implies prevention and this does not prevent. A reviewer or an
    /// agent who believes the tool stopped the work is worse off than one who
    /// knows it is reporting on it.
    #[test]
    fn the_directive_says_it_did_not_prevent_anything() {
        let approval = Approval {
            state: State::NotApproved,
            reason: "no approval has been recorded for this change".to_owned(),
            at: None,
        };

        let reason = reason(CHANGE, &approval);

        assert!(reason.contains("not a veto"), "{reason}");
    }
}
