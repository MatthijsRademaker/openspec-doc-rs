//! Whether a change is cleared for implementation, and the reviewer's acts that
//! clear it and take it back.
//!
//! Approval is something the reviewer *says*. It is never inferred, and in
//! particular it is not inferred from settled feedback: a change nobody looked
//! at has no open comments, so reading "nothing outstanding" as approval would
//! make every unreviewed change born approved.
//!
//! What an approval binds to is the artifact content it approved. The
//! fingerprint covers `proposal.md`, `design.md`, and every spec delta, and
//! deliberately not `tasks.md` — ticking a checkbox rewrites that file on
//! essentially every turn of implementation, so covering it would invalidate
//! every approval within seconds of work starting and train everyone to ignore
//! staleness. The cost is real and accepted: adding a *new task* to an approved
//! change does not invalidate its approval either.
//!
//! Staleness is computed on query rather than stored. A stored flag would need
//! something to set it, which is either a watcher racing the reader or a write
//! on every artifact edit.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::comments::{self, ScopeKey};
use crate::error::Error;
use crate::root::Project;
use crate::verdict::{self, Record, Verdict};

/// The change artifacts an approval covers, before its spec deltas.
///
/// `tasks.md` is absent on purpose; see the module documentation.
const REVIEWED_ARTIFACTS: [&str; 2] = ["proposal.md", "design.md"];

/// The reviewed content of one change, as a digest per artifact.
///
/// Per artifact rather than one digest over the set, because a stale approval
/// has to be able to say *what* changed under it, and a single digest can only
/// say that something did.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Fingerprint {
    artifacts: BTreeMap<String, String>,
}

impl Fingerprint {
    /// Which artifacts differ between the content this fingerprint recorded and
    /// `current`, named relative to the change directory.
    ///
    /// An artifact added or deleted since counts as changed: a change that grew
    /// a spec delta after approval is not the change that was approved.
    fn differences(&self, current: &Self) -> Vec<String> {
        let mut changed: Vec<String> = self
            .artifacts
            .iter()
            .filter(|(path, digest)| current.artifacts.get(*path) != Some(*digest))
            .map(|(path, _)| path.clone())
            .collect();

        changed.extend(
            current
                .artifacts
                .keys()
                .filter(|path| !self.artifacts.contains_key(*path))
                .cloned(),
        );

        changed.sort();
        changed.dedup();
        changed
    }
}

/// Where a change stands with the reviewer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Approved, and every artifact that approval covered still reads as it did.
    Approved,
    /// Approved once, but reviewed artifacts have changed under it since.
    Stale { changed: Vec<String> },
    /// Never approved, or the approval was withdrawn.
    NotApproved,
}

impl State {
    /// The one word this state is reported by, on the command line and in the
    /// dashboard alike.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Stale { .. } => "stale",
            Self::NotApproved => "not approved",
        }
    }

    /// Whether a change in this state is cleared for implementation. Only one of
    /// the three is, which is the whole point of there being three.
    pub fn is_approved(&self) -> bool {
        matches!(self, Self::Approved)
    }
}

impl fmt::Display for State {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

/// A change's approval state and why it is in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Approval {
    pub state: State,
    /// A sentence saying how the state was reached, for a reviewer reading the
    /// change page and an agent reading a precheck's output.
    pub reason: String,
    /// When the record the state was decided from was written, or `None` when no
    /// approval or withdrawal has ever been recorded for the change.
    pub at: Option<String>,
}

/// What a resolve-all-and-approve submission did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bulk {
    /// How many comments this submission moved to `resolved`. Comments already
    /// resolved are not counted, because nothing happened to them.
    pub resolved: usize,
    pub record: Record,
}

/// The current approval state of `change`.
///
/// A change that does not exist is an error rather than an unapproved change: a
/// precheck that answered "not approved" for a name nobody has ever created
/// would report a typo as a review problem.
pub fn state(project: &Project, change: &str) -> Result<Approval, Error> {
    let current = fingerprint(project, change)?;

    let Some(record) = standing_record(project, change)? else {
        return Ok(Approval {
            state: State::NotApproved,
            reason: "no approval has been recorded for this change".to_owned(),
            at: None,
        });
    };

    if record.verdict == Verdict::ApprovalWithdrawn {
        return Ok(Approval {
            state: State::NotApproved,
            reason: format!("approval was withdrawn on {}", record.created_at),
            at: Some(record.created_at),
        });
    }

    // Every approval this code records carries one, so a record without one was
    // written by hand into the sidecar. Reporting it as approved would be
    // reporting an approval nothing can tell apart from a stale one.
    let approved = record
        .fingerprint
        .ok_or_else(|| Error::FingerprintlessApproval {
            change: change.to_owned(),
        })?;

    let changed = approved.differences(&current);
    if changed.is_empty() {
        return Ok(Approval {
            state: State::Approved,
            reason: format!(
                "approved on {}, and no reviewed artifact has changed since",
                record.created_at
            ),
            at: Some(record.created_at),
        });
    }

    Ok(Approval {
        reason: format!(
            "approved on {}, but {} has changed since; the approval covers content that is no \
             longer on disk",
            record.created_at,
            changed.join(", ")
        ),
        state: State::Stale { changed },
        at: Some(record.created_at),
    })
}

/// Record an approval of `change`, refusing while any of its feedback is
/// outstanding.
///
/// `addressed` does not satisfy the precondition. It is the agent's claim that
/// work responding to a comment was done; resolving it is the reviewer's
/// judgement that the work is right, and letting the first stand in for the
/// second hands the gate back to the party being gated.
pub fn submit(project: &Project, change: &str) -> Result<Record, Error> {
    // Computed first, so a change that does not exist is refused before any
    // count is read against a sidecar nothing will ever render.
    let fingerprint = fingerprint(project, change)?;

    let key = ScopeKey::Change(change.to_owned());
    let counts = comments::counts(&project.root, &key)?;
    if counts.open > 0 || counts.addressed > 0 {
        return Err(Error::ApprovalBlocked {
            change: change.to_owned(),
            open: counts.open,
            addressed: counts.addressed,
        });
    }

    verdict::add_approval(&project.root, &key, fingerprint)
}

/// Withdraw the approval standing on `change`.
///
/// Appended, never removed: the sidecar is the one history this system keeps,
/// and a withdrawal that deleted the approval would leave no record that the
/// change was ever cleared.
pub fn withdraw(project: &Project, change: &str) -> Result<Record, Error> {
    // A stale approval is still an approval standing on the record, so it can be
    // withdrawn; a change with nothing to withdraw is refused rather than given
    // a record saying an approval that never existed no longer stands.
    let approval = state(project, change)?;
    if approval.state == State::NotApproved {
        return Err(Error::NothingToWithdraw {
            change: change.to_owned(),
            reason: approval.reason,
        });
    }

    verdict::add(
        &project.root,
        &ScopeKey::Change(change.to_owned()),
        Verdict::ApprovalWithdrawn,
        "",
    )
}

/// Resolve every comment on `change` that is not already resolved, then approve
/// it, as one submission.
///
/// Bulk resolution is not a weaker rule than resolving one at a time: resolution
/// is the reviewer's judgement whether they make it once or twenty times, and
/// the rule is about *who* resolves, not about how many clicks it takes. It does
/// sweep `open` comments the agent never replied to, which is the reviewer
/// withdrawing feedback rather than accepting work — included deliberately,
/// because a control that refused to sweep them would be unavailable in exactly
/// the case that motivates it.
///
/// The two halves are not atomic and this does not pretend they are. The
/// sidecars are append-only, so a failure after the resolutions land is reported
/// with both halves named rather than undone: compensating reopen records would
/// be indistinguishable from a reviewer reopening a thread, which corrupts the
/// history to hide a failure.
pub fn resolve_all_and_approve(project: &Project, change: &str) -> Result<Bulk, Error> {
    // Before anything is written: a name that is not a change must not leave
    // resolutions behind on a sidecar for a scope that does not exist.
    fingerprint(project, change)?;

    let resolved = comments::resolve_all(&project.root, &ScopeKey::Change(change.to_owned()))?;

    match submit(project, change) {
        Ok(record) => Ok(Bulk { resolved, record }),
        Err(source) => Err(Error::PartialApproval {
            change: change.to_owned(),
            resolved,
            source: Box::new(source),
        }),
    }
}

/// The reviewed content of `change` as it stands on disk.
pub fn fingerprint(project: &Project, change: &str) -> Result<Fingerprint, Error> {
    let dir = change_dir(project, change);
    if !dir.is_dir() {
        return Err(Error::UnknownChange {
            change: change.to_owned(),
        });
    }

    let mut artifacts = BTreeMap::new();
    for name in REVIEWED_ARTIFACTS {
        if let Some(contents) = read_if_present(&dir.join(name))? {
            artifacts.insert(name.to_owned(), digest(&contents));
        }
    }

    for capability in spec_capabilities(&dir)? {
        let relative = format!("specs/{capability}/spec.md");
        if let Some(contents) = read_if_present(&dir.join(&relative))? {
            artifacts.insert(relative, digest(&contents));
        }
    }

    Ok(Fingerprint { artifacts })
}

/// The last approval or withdrawal recorded for `change`, the earlier ones being
/// history rather than the decision in force.
fn standing_record(project: &Project, change: &str) -> Result<Option<Record>, Error> {
    Ok(
        verdict::read(&project.root, &ScopeKey::Change(change.to_owned()))?
            .into_iter()
            .rfind(|record| {
                matches!(
                    record.verdict,
                    Verdict::Approved | Verdict::ApprovalWithdrawn
                )
            }),
    )
}

fn change_dir(project: &Project, change: &str) -> PathBuf {
    project.openspec_dir.join("changes").join(change)
}

/// The capability directories under a change's `specs/`, sorted. A change with
/// no spec deltas has none.
fn spec_capabilities(dir: &Path) -> Result<Vec<String>, Error> {
    let specs = dir.join("specs");
    let entries = match fs::read_dir(&specs) {
        Ok(entries) => entries,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(Error::io(specs, source)),
    };

    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| Error::io(&specs, source))?;
        if !entry.path().is_dir() {
            continue;
        }
        names.push(
            entry
                .file_name()
                .to_str()
                .ok_or_else(|| Error::NonUtf8Path { path: entry.path() })?
                .to_owned(),
        );
    }

    names.sort();
    Ok(names)
}

/// The contents of `path`, or `None` when the change does not have that artifact.
fn read_if_present(path: &Path) -> Result<Option<String>, Error> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(Some(contents)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(Error::io(path, source)),
    }
}

/// A digest of one artifact's bytes: FNV-1a, 64 bits, hex.
///
/// Deliberately not a cryptographic hash. What this has to detect is an edit,
/// and nothing here defends against someone constructing a collision — they
/// could equally write the approval record themselves, since the sidecar is a
/// file in the repository. A dependency has to buy more than the code it
/// replaces, and a hash resisting forgery buys a property this design already
/// does not have.
fn digest(contents: &str) -> String {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = OFFSET;
    for byte in contents.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }

    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comments::Status;
    use crate::root::project_at;
    use crate::test_fixture::project_fixture;
    use tempfile::TempDir;

    const CHANGE: &str = "add-thing";

    /// A change with the three reviewed artifact kinds on disk and no review
    /// history: no comments, no verdicts, no approval.
    fn change_fixture() -> (TempDir, Project) {
        let fixture = project_fixture(&[
            "openspec/changes/add-thing/specs/some-cap",
            "openspec/changes/add-thing/specs/other-cap",
        ]);
        let dir = fixture.path().join("openspec/changes/add-thing");
        fs::write(dir.join("proposal.md"), "## Why\n\nBecause.\n").expect("write proposal");
        fs::write(dir.join("design.md"), "## Context\n\nAs follows.\n").expect("write design");
        fs::write(dir.join("tasks.md"), "- [ ] 1.1 Do the thing\n").expect("write tasks");
        fs::write(dir.join("specs/some-cap/spec.md"), "## ADDED\n").expect("write spec");
        fs::write(dir.join("specs/other-cap/spec.md"), "## MODIFIED\n").expect("write spec");

        let project = project_at(fixture.path()).expect("project root");
        (fixture, project)
    }

    fn key() -> ScopeKey {
        ScopeKey::Change(CHANGE.to_owned())
    }

    fn write(project: &Project, relative: &str, contents: &str) {
        fs::write(
            project
                .openspec_dir
                .join("changes")
                .join(CHANGE)
                .join(relative),
            contents,
        )
        .expect("write artifact");
    }

    fn comment(project: &Project, body: &str, status: Status) -> String {
        let comment = comments::add_unanchored(&project.root, &key(), body).expect("add comment");
        if status != Status::Open {
            comments::set_status(&project.root, &key(), &comment.id, status).expect("set status");
        }
        comment.id
    }

    fn state_of(project: &Project) -> Approval {
        state(project, CHANGE).expect("approval state")
    }

    #[test]
    fn the_fingerprint_covers_the_proposal_the_design_and_every_spec_delta() {
        let (_fixture, project) = change_fixture();

        let fingerprint = fingerprint(&project, CHANGE).expect("fingerprint");
        let covered: Vec<&str> = fingerprint.artifacts.keys().map(String::as_str).collect();

        assert_eq!(
            covered,
            [
                "design.md",
                "proposal.md",
                "specs/other-cap/spec.md",
                "specs/some-cap/spec.md"
            ]
        );
    }

    #[test]
    fn editing_a_reviewed_artifact_changes_the_fingerprint() {
        let (_fixture, project) = change_fixture();

        for (path, contents) in [
            ("proposal.md", "## Why\n\nBecause of something else.\n"),
            ("design.md", "## Context\n\nOtherwise.\n"),
            ("specs/some-cap/spec.md", "## ADDED\n\nMore.\n"),
        ] {
            let before = fingerprint(&project, CHANGE).expect("fingerprint");
            write(&project, path, contents);
            let after = fingerprint(&project, CHANGE).expect("fingerprint");

            assert_eq!(
                before.differences(&after),
                [path],
                "editing {path} did not move the fingerprint on its own"
            );
        }
    }

    /// The decision most likely to be got wrong by accident. Ticking checkboxes
    /// rewrites `tasks.md` on essentially every turn of implementation, so a
    /// fingerprint covering it would invalidate every approval within seconds of
    /// work starting.
    #[test]
    fn ticking_a_task_does_not_change_the_fingerprint() {
        let (_fixture, project) = change_fixture();
        let before = fingerprint(&project, CHANGE).expect("fingerprint");

        write(&project, "tasks.md", "- [x] 1.1 Do the thing\n");

        assert_eq!(
            before,
            fingerprint(&project, CHANGE).expect("fingerprint"),
            "tasks.md is not part of what the reviewer approved"
        );
    }

    /// A change that grew a spec delta after approval is not the change that was
    /// approved, so an added artifact counts as a difference.
    #[test]
    fn adding_a_spec_delta_changes_the_fingerprint() {
        let (_fixture, project) = change_fixture();
        let before = fingerprint(&project, CHANGE).expect("fingerprint");

        let added = project
            .openspec_dir
            .join("changes/add-thing/specs/late-cap");
        fs::create_dir_all(&added).expect("create capability dir");
        fs::write(added.join("spec.md"), "## ADDED\n").expect("write spec");

        assert_eq!(
            before.differences(&fingerprint(&project, CHANGE).expect("fingerprint")),
            ["specs/late-cap/spec.md"]
        );
    }

    #[test]
    fn a_change_with_no_comments_and_no_approval_record_is_not_approved() {
        let (_fixture, project) = change_fixture();

        let approval = state_of(&project);

        assert_eq!(approval.state, State::NotApproved);
        assert_eq!(approval.at, None);
        assert!(approval.reason.contains("no approval"), "{approval:?}");
    }

    #[test]
    fn a_change_with_every_artifact_present_is_still_not_approved() {
        let (_fixture, project) = change_fixture();

        assert_eq!(state_of(&project).state, State::NotApproved);
    }

    #[test]
    fn an_unknown_change_is_an_error_rather_than_an_unapproved_change() {
        let (_fixture, project) = change_fixture();

        assert!(
            matches!(
                state(&project, "add-nothing"),
                Err(Error::UnknownChange { .. })
            ),
            "a name nobody created must not be reported as a review problem"
        );
    }

    #[test]
    fn a_change_with_no_comments_can_be_approved() {
        let (_fixture, project) = change_fixture();

        let record = submit(&project, CHANGE).expect("approve");

        assert_eq!(record.verdict, Verdict::Approved);
        assert!(record.fingerprint.is_some());
        assert!(state_of(&project).state.is_approved());
    }

    #[test]
    fn a_change_whose_comments_are_all_resolved_can_be_approved() {
        let (_fixture, project) = change_fixture();
        comment(&project, "One thing.", Status::Resolved);

        submit(&project, CHANGE).expect("approve");

        assert!(state_of(&project).state.is_approved());
    }

    #[test]
    fn approval_is_refused_over_each_blocking_status() {
        for (status, open, addressed) in [(Status::Open, 1, 0), (Status::Addressed, 0, 1)] {
            let (_fixture, project) = change_fixture();
            comment(&project, "Look at this.", status);

            let error = submit(&project, CHANGE).expect_err("blocked");

            assert!(
                matches!(
                    error,
                    Error::ApprovalBlocked {
                        open: reported_open,
                        addressed: reported_addressed,
                        ..
                    } if reported_open == open && reported_addressed == addressed
                ),
                "{status} reported the wrong blocking counts: {error}"
            );
            assert_eq!(
                verdict::read(&project.root, &key()).expect("read"),
                [],
                "a refused approval must append nothing"
            );
            assert_eq!(state_of(&project).state, State::NotApproved);
        }
    }

    /// `addressed` is the agent's claim that work responding to a comment was
    /// done. Letting it satisfy the precondition hands the gate to the party
    /// being gated.
    #[test]
    fn addressed_comments_do_not_satisfy_the_precondition() {
        let (_fixture, project) = change_fixture();
        comment(&project, "Explain this.", Status::Addressed);

        let error = submit(&project, CHANGE).expect_err("blocked");

        assert!(
            error.to_string().contains("addressed"),
            "the refusal must say what is blocking it: {error}"
        );
    }

    #[test]
    fn editing_a_reviewed_artifact_makes_the_approval_stale() {
        let (_fixture, project) = change_fixture();
        submit(&project, CHANGE).expect("approve");

        write(&project, "proposal.md", "## Why\n\nA different reason.\n");

        let approval = state_of(&project);
        assert_eq!(
            approval.state,
            State::Stale {
                changed: vec!["proposal.md".to_owned()]
            }
        );
        assert!(approval.reason.contains("proposal.md"), "{approval:?}");
        assert!(!approval.state.is_approved());
    }

    #[test]
    fn ticking_a_task_leaves_the_approval_standing() {
        let (_fixture, project) = change_fixture();
        submit(&project, CHANGE).expect("approve");

        write(&project, "tasks.md", "- [x] 1.1 Do the thing\n");

        assert!(state_of(&project).state.is_approved());
    }

    #[test]
    fn re_approving_after_an_edit_clears_the_staleness() {
        let (_fixture, project) = change_fixture();
        submit(&project, CHANGE).expect("approve");
        write(&project, "design.md", "## Context\n\nRevised.\n");
        assert!(matches!(state_of(&project).state, State::Stale { .. }));

        submit(&project, CHANGE).expect("re-approve");

        assert!(state_of(&project).state.is_approved());
        assert_eq!(
            verdict::read(&project.root, &key()).expect("read").len(),
            2,
            "the superseded approval stays on the record"
        );
    }

    #[test]
    fn withdrawal_returns_the_change_to_not_approved_and_keeps_the_record() {
        let (_fixture, project) = change_fixture();
        let approved = submit(&project, CHANGE).expect("approve");

        withdraw(&project, CHANGE).expect("withdraw");

        let approval = state_of(&project);
        assert_eq!(approval.state, State::NotApproved);
        assert!(approval.reason.contains("withdrawn"), "{approval:?}");
        let history = verdict::read(&project.root, &key()).expect("read");
        assert_eq!(history[0], approved, "the approval is still on the record");
        assert_eq!(history[1].verdict, Verdict::ApprovalWithdrawn);
    }

    /// A stale approval is still an approval standing on the record, so it is
    /// withdrawable; a change with nothing to withdraw is refused rather than
    /// given a record saying an approval that never existed no longer stands.
    #[test]
    fn a_stale_approval_can_be_withdrawn_and_an_absent_one_cannot() {
        let (_fixture, project) = change_fixture();
        submit(&project, CHANGE).expect("approve");
        write(&project, "proposal.md", "## Why\n\nElse.\n");

        withdraw(&project, CHANGE).expect("withdraw a stale approval");

        assert!(
            matches!(
                withdraw(&project, CHANGE),
                Err(Error::NothingToWithdraw { .. })
            ),
            "withdrawing nothing must not be recorded as a withdrawal"
        );
    }

    /// An approval hand-written into the sidecar carries no fingerprint, and
    /// nothing can say whether what it approved is still on disk.
    #[test]
    fn an_approval_carrying_no_fingerprint_is_an_error() {
        let (_fixture, project) = change_fixture();
        let path = verdict::path(&project.root, &key()).expect("path");
        fs::create_dir_all(path.parent().expect("parent")).expect("create dir");
        fs::write(
            &path,
            "{\"id\":\"x\",\"verdict\":\"approved\",\"notes\":\"\",\"createdAt\":\"2026-09-02T00:00:00Z\"}\n",
        )
        .expect("write sidecar");

        assert!(
            matches!(
                state(&project, CHANGE),
                Err(Error::FingerprintlessApproval { .. })
            ),
            "a fingerprintless approval must not read as approved"
        );
    }

    #[test]
    fn a_plain_verdict_submission_cannot_record_an_approval() {
        let (_fixture, project) = change_fixture();

        assert!(matches!(
            verdict::add(&project.root, &key(), Verdict::Approved, ""),
            Err(Error::UnfingerprintedApproval)
        ));
    }

    #[test]
    fn resolve_all_and_approve_resolves_every_outstanding_comment_and_approves() {
        let (_fixture, project) = change_fixture();
        comment(&project, "Never replied to.", Status::Open);
        comment(&project, "Claimed done.", Status::Addressed);
        comment(&project, "Also open.", Status::Open);

        let bulk = resolve_all_and_approve(&project, CHANGE).expect("bulk approve");

        assert_eq!(bulk.resolved, 3);
        assert_eq!(
            comments::counts(&project.root, &key()).expect("counts"),
            comments::StatusCounts {
                open: 0,
                addressed: 0,
                resolved: 3
            }
        );
        assert!(state_of(&project).state.is_approved());
    }

    #[test]
    fn already_resolved_comments_get_no_further_status_record() {
        let (_fixture, project) = change_fixture();
        let settled = comment(&project, "Already accepted.", Status::Resolved);
        comment(&project, "Still open.", Status::Open);

        let bulk = resolve_all_and_approve(&project, CHANGE).expect("bulk approve");

        assert_eq!(bulk.resolved, 1, "only the outstanding comment moved");
        let history = comments::read(&project.root, &key()).expect("read");
        let resolved_thread = history
            .iter()
            .find(|thread| thread.comment.id == settled)
            .expect("the settled thread");
        assert_eq!(
            resolved_thread.status_history.len(),
            1,
            "no redundant record saying it stayed where it was"
        );
    }

    #[test]
    fn resolve_all_and_approve_with_nothing_outstanding_still_approves() {
        let (_fixture, project) = change_fixture();

        let bulk = resolve_all_and_approve(&project, CHANGE).expect("bulk approve");

        assert_eq!(bulk.resolved, 0);
        assert!(state_of(&project).state.is_approved());
    }

    /// The two halves are not atomic. When the approval fails after the
    /// resolutions land, both halves are named and neither is undone.
    #[test]
    fn a_failure_after_the_resolutions_reports_both_halves() {
        let (_fixture, project) = change_fixture();
        comment(&project, "Open.", Status::Open);
        // A directory where the verdict sidecar's file belongs: the resolutions
        // are written to their own sidecar and then the approval cannot be
        // appended, which is the shape of the failure this reports on.
        fs::create_dir_all(verdict::path(&project.root, &key()).expect("path"))
            .expect("block the sidecar");

        let error = resolve_all_and_approve(&project, CHANGE).expect_err("partial");

        assert!(
            matches!(error, Error::PartialApproval { resolved: 1, .. }),
            "{error}"
        );
        let message = error.to_string();
        assert!(message.contains("resolved 1"), "{message}");
        assert!(message.contains("not approved"), "{message}");
        assert_eq!(
            comments::counts(&project.root, &key())
                .expect("counts")
                .resolved,
            1,
            "the resolutions stand; nothing compensating was appended"
        );
    }

    #[test]
    fn a_bulk_submission_against_an_unknown_change_resolves_nothing() {
        let (_fixture, project) = change_fixture();

        assert!(matches!(
            resolve_all_and_approve(&project, "add-nothing"),
            Err(Error::UnknownChange { .. })
        ));
    }
}
