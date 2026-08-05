## Why

A change can go from proposal to implementation with nobody having said it is ready. There is no approval concept anywhere in this project — not in the live specs, and not in the predecessor implementation either, whose lifecycle phases are inferred from artifact presence and explicitly advisory. The review loop produces comments and verdicts that no one is required to act on before code gets written.

The obvious shortcut — treat "no open comments" as approval — is worse than nothing. A change nobody reviewed has no open comments, so every unreviewed change would be born approved. Approval has to be something a reviewer states, not something the absence of feedback implies.

## What Changes

- Add an `approved` verdict a reviewer submits from a change page, alongside the existing keep-exploring, move-to-proposal, and comment-resolution kinds.
- Refuse the approval when the change still has unresolved feedback. All comments must be `resolved`; `addressed` is not enough, because that is the agent's claim rather than the reviewer's acceptance.
- Bind every approval to a fingerprint of the artifacts it approved, and report an approval as stale once those artifacts change — the equivalent of dismissing a stale review when new commits land. Without this the agent can edit `proposal.md` after approval and the gate means nothing.
- Add an explicit un-approve, so a reviewer can withdraw approval without editing an artifact to force staleness.
- Report a change's approval state — approved, stale, or not approved, with the reason — from both the CLI and the change page.
- Detect at hook-stop time that implementation has progressed on a change with no current approval, and raise it as a directive through the existing pending-directive mechanism.

## Capabilities

### New Capabilities

- `change-approval`: the approval record and its verdict kind, the all-resolved precondition, the artifact fingerprint that makes an approval go stale, un-approval, the queryable approval state, and the stop-time detection of unapproved implementation.

### Modified Capabilities

- `dashboard-html-views`: gains an approve control on change pages, gains display of current approval state, and its phase-verdict requirement gains the new kind.

## Impact

- `crates/core/src/verdict.rs` — the verdict kind enum and the record, which gains the artifact fingerprint.
- New core module for fingerprinting a change's artifact set and evaluating approval state.
- `crates/cli/src/cli.rs` — a subcommand reporting approval state, suitable for the apply workflow to run as a precheck.
- `crates/server/src/page/scoped.rs` and `routes.rs` — the approve control and its route.
- `crates/core/src/hook/` — the stop-time detection.
- `.claude/skills/openspec-apply-change/SKILL.md` and the `opsx:apply` equivalent — a precheck step.

Ordering: depends on `add-comment-thread-actions` for the resolved-comment counts the precondition reads, and inherits its dependency on `add-dashboard-html-views` being archived.

**The enforcement here is a gate in the honest sense, not a lock.** A Stop hook runs at turn end and can only block-and-continue or allow-stop; it cannot prevent an action that has already happened. Hard prevention would need a pre-action hook, which Claude Code has and pi.dev — driven by a project-local extension rather than an external-process hook — does not. This change therefore delivers a precheck the apply workflow runs before starting, and after-the-fact detection that raises unapproved implementation as a directive. Anyone reading "gate" as "cannot physically proceed" will be disappointed, and that expectation should be corrected rather than designed around.
