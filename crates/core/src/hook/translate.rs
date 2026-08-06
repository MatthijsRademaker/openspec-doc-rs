//! Turning a reviewer's phase verdict into the pending directive the hook
//! bridge injects at the next turn boundary.
//!
//! A directive's reason text is a pointer, never an embed: it names the files
//! the reviewer's words are in and leaves reading them to the agent. That keeps
//! the injected text short and, more importantly, keeps it *checkable* — every
//! claim it makes is one the agent can verify against the repo it is working
//! in.
//!
//! That checkability is the point. A receiving agent is entitled to refuse an
//! instruction it cannot corroborate, and during `add-agent-hook-bridge`
//! verification one did exactly that: a hand-written directive demanding a
//! literal word back was delivered perfectly and then declined as suspected
//! prompt injection. So every template here opens by saying where it came from
//! and names paths inside the user's own project. None of them asks for
//! specific literal output, and none of them tells the agent to close a review
//! thread — see [`reason`].

use crate::comments::ScopeKey;
use crate::error::Error;
use crate::root::Project;
use crate::scratch;
use crate::verdict::{self, Record, Verdict};

use super::directive;

/// What every template opens with, so the agent can place the directive as
/// review feedback from a tool in its own project rather than as text of
/// unknown origin.
const ATTRIBUTION: &str = "Review feedback from this project's openspec-doc dashboard";

/// Translate the standing untranslated verdict for `session_id` into a pending
/// directive, and report which scope and verdict it came from.
///
/// Two scopes are candidates: the session's own, and — once its exploration has
/// been promoted — the change that exploration became. The most recently
/// submitted of the two wins, so feedback given after a promotion is not queued
/// behind explore-phase feedback that predates it.
pub fn translate(
    project: &Project,
    session_id: &str,
) -> Result<Option<(ScopeKey, Verdict)>, Error> {
    // A directive already waiting is one the next turn boundary will inject.
    // Writing over it would drop it, so the verdict simply stays untranslated
    // and is picked up at the boundary after that.
    if directive::load_pending(&project.root, session_id)?.is_some() {
        return Ok(None);
    }

    let mut keys = Vec::new();
    if let Some(change) = scratch::promoted_to(&project.root, session_id)? {
        keys.push(ScopeKey::Change(change));
    }
    keys.push(ScopeKey::Session(session_id.to_owned()));

    let Some((key, record)) = standing(project, keys)? else {
        return Ok(None);
    };

    // Written before the verdict is marked, so an interrupted translation
    // repeats itself rather than losing the feedback: an agent sent to the same
    // files twice has read them twice, which costs nothing.
    directive::write_pending(&project.root, session_id, &reason(&key, record.verdict)?)?;
    verdict::mark_translated(&project.root, &key, &record.id)?;

    Ok(Some((key, record.verdict)))
}

/// The most recently submitted untranslated verdict across `keys`. Ties go to
/// whichever key came first, which is why the caller orders the change scope
/// ahead of the session it was promoted from.
fn standing(project: &Project, keys: Vec<ScopeKey>) -> Result<Option<(ScopeKey, Record)>, Error> {
    let mut standing: Option<(ScopeKey, Record)> = None;

    for key in keys {
        let Some(record) = verdict::untranslated(&project.root, &key)? else {
            continue;
        };
        let supersedes = standing
            .as_ref()
            .is_none_or(|(_, current)| record.created_at > current.created_at);
        if supersedes {
            standing = Some((key, record));
        }
    }

    Ok(standing)
}

/// The reason text for `verdict` recorded against `key`.
///
/// Each template is an attributed pointer: it says the feedback came from this
/// project's dashboard and names the in-project files holding it. What none of
/// them does is demand particular literal output, and the comment-resolution
/// template deliberately stops at asking the agent to reply on each thread —
/// closing a thread is the reviewer's acceptance of the work, and an agent that
/// closes its own threads has ended the review loop rather than completed it.
pub fn reason(key: &ScopeKey, verdict: Verdict) -> Result<String, Error> {
    let verdicts = verdict::relative(key)?;
    let comments = key.relative()?;

    match (key, verdict) {
        (ScopeKey::Session(session_id), Verdict::KeepExploring) => {
            let note = scratch::session_relative(session_id)?;
            Ok(format!(
                "{ATTRIBUTION}: the reviewer looked at this exploration and kept it in the \
                 explore phase, with notes on what is still open. Those notes are the last \
                 record in `{verdicts}`, the exploration itself is at `{note}`, and any \
                 anchored comments are in `{comments}` — \
                 `openspec-doc comment list --session {session_id}` prints them readably. \
                 Read those, then keep working on what they say is unsettled."
            ))
        }
        (ScopeKey::Session(session_id), Verdict::MoveToProposal) => {
            let note = scratch::session_relative(session_id)?;
            Ok(format!(
                "{ATTRIBUTION}: the reviewer judged this exploration ready to formalize. The \
                 exploration is at `{note}`, the verdict with any parting notes is the last \
                 record in `{verdicts}`, and the comments they anchored to the exploration are \
                 in `{comments}` — `openspec-doc comment list --session {session_id}` prints \
                 them readably. Turn what is in that note into an OpenSpec change under \
                 `openspec/changes/`, and account for every open comment in what you write: a \
                 passage the reviewer marked is one they want changed before it is formalized. \
                 Once that directory exists, run \
                 `openspec-doc scratch claim --session {session_id} --change <the change you \
                 created>` — that claim is what moves this exploration and its review \
                 comments onto the change; without it the note stays at its session path."
            ))
        }
        (ScopeKey::Change(name), Verdict::CommentResolution) => Ok(format!(
            "{ATTRIBUTION}: the reviewer sent the open comments on change `{name}` back for \
             work. They are in `{comments}` — `openspec-doc comment list --change {name}` \
             prints them with the artifacts they anchor to, all under \
             `openspec/changes/{name}/` — and the verdict is the last record in `{verdicts}`. \
             Work through each open comment, then say what you did on its thread with \
             `openspec-doc comment reply --change {name} --comment <id> --body <what changed>`. \
             Leave the thread status alone: the reviewer decides when a comment is settled."
        )),
        // `verdict::add` refuses a verdict that does not belong to the kind of
        // scope it was filed under, so a sidecar holding one is corrupt.
        (key, verdict) => Err(Error::MisscopedVerdict {
            verdict,
            session_scoped: matches!(key, ScopeKey::Session(_)),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;
    use crate::root::project_at;
    use crate::test_fixture::project_fixture;
    use crate::verdict::Verdict;

    const SESSION: &str = "session-a";
    const CHANGE: &str = "add-thing";

    /// Every template, paired with the scope it is written for.
    fn every_template() -> Vec<(ScopeKey, Verdict)> {
        vec![
            (session_key(), Verdict::KeepExploring),
            (session_key(), Verdict::MoveToProposal),
            (change_key(), Verdict::CommentResolution),
        ]
    }

    fn session_key() -> ScopeKey {
        ScopeKey::Session(SESSION.to_owned())
    }

    fn change_key() -> ScopeKey {
        ScopeKey::Change(CHANGE.to_owned())
    }

    fn explored_project() -> (TempDir, Project) {
        let fixture = project_fixture(&[]);
        let project = project_at(fixture.path()).expect("project root");
        fs::write(
            scratch::prepare_session(&project.root, SESSION).expect("prepare note"),
            "# Exploration\n",
        )
        .expect("write note");

        (fixture, project)
    }

    /// The reason text of the session's pending directive.
    fn pending_reason(project: &Project) -> String {
        directive::load_pending(&project.root, SESSION)
            .expect("load")
            .expect("pending directive")
            .reason
    }

    /// Leave the session's note the way a promotion to `CHANGE` leaves it,
    /// without running the validator a real promotion would.
    fn promote(project: &Project) {
        let target = scratch::change_relative(CHANGE);
        fs::write(
            scratch::session_path(&project.root, SESSION).expect("path"),
            format!("<!-- openspec-doc:moved-to {target} -->\n"),
        )
        .expect("write pointer");
    }

    #[test]
    fn a_keep_exploring_verdict_points_at_the_note_and_its_sidecars() {
        let (_fixture, project) = explored_project();
        verdict::add(
            &project.root,
            &session_key(),
            Verdict::KeepExploring,
            "The promotion trigger is still hand-waved.",
        )
        .expect("add verdict");

        let translated = translate(&project, SESSION).expect("translate");

        assert_eq!(translated, Some((session_key(), Verdict::KeepExploring)));
        let reason = pending_reason(&project);
        assert!(
            reason.contains(".openspec-doc/scratch/_session/session-a.md"),
            "{reason}"
        );
        assert!(
            reason.contains(".openspec-doc/verdicts/_session/session-a.jsonl"),
            "{reason}"
        );
        assert!(
            reason.contains(".openspec-doc/comments/_session/session-a.jsonl"),
            "{reason}"
        );
        assert!(
            !reason.contains("The promotion trigger is still hand-waved."),
            "the reviewer's notes are pointed at, not embedded: {reason}"
        );
    }

    #[test]
    fn a_move_to_proposal_verdict_points_at_the_note_and_the_changes_directory() {
        let (_fixture, project) = explored_project();
        verdict::add(&project.root, &session_key(), Verdict::MoveToProposal, "")
            .expect("add verdict");

        let translated = translate(&project, SESSION).expect("translate");

        assert_eq!(translated, Some((session_key(), Verdict::MoveToProposal)));
        let reason = pending_reason(&project);
        assert!(
            reason.contains(".openspec-doc/scratch/_session/session-a.md"),
            "{reason}"
        );
        assert!(reason.contains("openspec/changes/"), "{reason}");
    }

    /// The note and the verdict say what the reviewer concluded; the comment
    /// sidecar says which passages they wanted changed. Naming the first two and
    /// not the third sends the agent to formalize an exploration without telling
    /// it that anchored feedback on that exploration exists.
    #[test]
    fn a_move_to_proposal_verdict_points_at_the_reviewers_comments() {
        let reason = reason(&session_key(), Verdict::MoveToProposal).expect("reason");

        assert!(
            reason.contains(".openspec-doc/comments/_session/session-a.jsonl"),
            "{reason}"
        );
        assert!(
            reason.contains(&format!("openspec-doc comment list --session {SESSION}")),
            "{reason}"
        );
        assert!(
            reason.contains("open comment"),
            "the comments are named but not asked for: {reason}"
        );
    }

    /// Nothing else promotes the note, so the directive that asks for the change
    /// has to be the thing that asks for the claim, with the one argument the
    /// agent cannot look up already filled in.
    #[test]
    fn a_move_to_proposal_verdict_names_the_claim_command_for_this_session() {
        let reason = reason(&session_key(), Verdict::MoveToProposal).expect("reason");

        assert!(
            reason.contains(&format!(
                "openspec-doc scratch claim --session {SESSION} --change"
            )),
            "{reason}"
        );
    }

    #[test]
    fn a_comment_resolution_verdict_reaches_the_session_the_change_was_promoted_from() {
        let (_fixture, project) = explored_project();
        promote(&project);
        verdict::add(
            &project.root,
            &change_key(),
            Verdict::CommentResolution,
            "Two of these are still unanswered.",
        )
        .expect("add verdict");

        let translated = translate(&project, SESSION).expect("translate");

        assert_eq!(translated, Some((change_key(), Verdict::CommentResolution)));
        let reason = pending_reason(&project);
        assert!(
            reason.contains(".openspec-doc/comments/add-thing.jsonl"),
            "{reason}"
        );
        assert!(reason.contains("openspec-doc comment reply"), "{reason}");
        assert!(
            !reason.contains("Two of these are still unanswered."),
            "the reviewer's notes are pointed at, not embedded: {reason}"
        );
    }

    #[test]
    fn a_change_with_no_exploration_behind_it_has_no_session_to_reach() {
        let (_fixture, project) = explored_project();
        verdict::add(&project.root, &change_key(), Verdict::CommentResolution, "")
            .expect("add verdict");

        assert_eq!(
            translate(&project, SESSION).expect("translate"),
            None,
            "an unpromoted session must not pick up an unrelated change's verdict"
        );
    }

    #[test]
    fn a_translated_verdict_is_not_translated_again() {
        let (_fixture, project) = explored_project();
        verdict::add(
            &project.root,
            &session_key(),
            Verdict::KeepExploring,
            "Still open.",
        )
        .expect("add verdict");

        translate(&project, SESSION).expect("first translate");
        let first = directive::load_pending(&project.root, SESSION)
            .expect("load")
            .expect("pending directive");
        directive::mark_consumed(&project.root, SESSION, &first).expect("consume");

        assert_eq!(
            translate(&project, SESSION).expect("second translate"),
            None
        );
        assert_eq!(
            directive::load_pending(&project.root, SESSION).expect("load"),
            None,
            "no second directive is produced from the same verdict"
        );
    }

    #[test]
    fn a_verdict_submitted_after_the_last_one_was_translated_still_gets_through() {
        let (_fixture, project) = explored_project();
        verdict::add(
            &project.root,
            &session_key(),
            Verdict::KeepExploring,
            "First pass.",
        )
        .expect("first verdict");
        translate(&project, SESSION).expect("first translate");
        let first = directive::load_pending(&project.root, SESSION)
            .expect("load")
            .expect("pending directive");
        directive::mark_consumed(&project.root, SESSION, &first).expect("consume");

        verdict::add(&project.root, &session_key(), Verdict::MoveToProposal, "")
            .expect("second verdict");

        assert_eq!(
            translate(&project, SESSION).expect("second translate"),
            Some((session_key(), Verdict::MoveToProposal))
        );
    }

    #[test]
    fn a_directive_still_waiting_is_not_written_over() {
        let (_fixture, project) = explored_project();
        directive::write_pending(&project.root, SESSION, "Hand-written and still waiting.")
            .expect("write directive");
        verdict::add(
            &project.root,
            &session_key(),
            Verdict::KeepExploring,
            "Still open.",
        )
        .expect("add verdict");

        assert_eq!(translate(&project, SESSION).expect("translate"), None);
        assert_eq!(
            pending_reason(&project),
            "Hand-written and still waiting.",
            "the waiting directive survives"
        );
        assert!(
            verdict::untranslated(&project.root, &session_key())
                .expect("untranslated")
                .is_some(),
            "the verdict is left for the next turn boundary, not dropped"
        );
    }

    #[test]
    fn a_scope_with_no_verdict_produces_no_directive() {
        let (_fixture, project) = explored_project();

        assert_eq!(translate(&project, SESSION).expect("translate"), None);
        assert_eq!(
            directive::load_pending(&project.root, SESSION).expect("load"),
            None
        );
    }

    /// Feedback given on the change supersedes explore-phase feedback that
    /// predates the promotion, rather than queueing behind it.
    #[test]
    fn the_more_recent_of_two_untranslated_verdicts_wins() {
        let (_fixture, project) = explored_project();
        verdict::add(
            &project.root,
            &session_key(),
            Verdict::KeepExploring,
            "Written before the promotion.",
        )
        .expect("session verdict");
        promote(&project);
        verdict::add(&project.root, &change_key(), Verdict::CommentResolution, "")
            .expect("change verdict");

        assert_eq!(
            translate(&project, SESSION).expect("translate"),
            Some((change_key(), Verdict::CommentResolution))
        );
    }

    /// Every template has to be corroborable by the agent receiving it: it says
    /// where it came from and names files that agent can go and read.
    #[test]
    fn every_template_is_an_attributed_pointer() {
        for (key, verdict) in every_template() {
            let reason = reason(&key, verdict).expect("reason");

            assert!(
                reason.starts_with(ATTRIBUTION),
                "{verdict} does not say where it came from: {reason}"
            );
            assert!(
                reason.contains(".openspec-doc/"),
                "{verdict} names no in-project path to read: {reason}"
            );
        }
    }

    /// The refusal observed during `add-agent-hook-bridge` verification was of a
    /// directive demanding a literal word back. There is no mechanical test for
    /// "an agent will comply" — task 3.2 is the only thing that can check that —
    /// so this asserts the narrower thing that can be checked: no template is
    /// shaped like the one that was refused.
    #[test]
    fn no_template_demands_literal_output() {
        const INJECTION_SHAPED: [&str; 6] = [
            "exactly",
            "verbatim",
            "nothing else",
            "the single word",
            "respond with",
            "reply with",
        ];

        for (key, verdict) in every_template() {
            let reason = reason(&key, verdict).expect("reason").to_lowercase();

            for phrase in INJECTION_SHAPED {
                assert!(
                    !reason.contains(phrase),
                    "{verdict} demands literal output ({phrase:?}): {reason}"
                );
            }
        }
    }

    /// Closing a thread is the reviewer accepting the work, not the agent
    /// claiming to have done it. An agent that resolves its own comments has
    /// ended the review loop instead of completing a pass of it.
    #[test]
    fn no_template_tells_the_agent_to_resolve_a_comment() {
        for (key, verdict) in every_template() {
            let reason = reason(&key, verdict).expect("reason").to_lowercase();

            for phrase in ["resolv", "comment resolve", "close the thread", "mark done"] {
                assert!(
                    !reason.contains(phrase),
                    "{verdict} tells the agent to close a thread ({phrase:?}): {reason}"
                );
            }
        }
    }

    #[test]
    fn a_verdict_filed_under_the_wrong_kind_of_scope_is_an_error() {
        for (key, verdict) in [
            (session_key(), Verdict::CommentResolution),
            (change_key(), Verdict::KeepExploring),
            (change_key(), Verdict::MoveToProposal),
        ] {
            assert!(
                matches!(reason(&key, verdict), Err(Error::MisscopedVerdict { .. })),
                "produced a reason for {verdict} under {key:?}"
            );
        }
    }
}
