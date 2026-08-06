## Context

`add-dashboard-html-views` shipped comment creation from the browser and nothing else. Reply and resolve exist only as CLI subcommands, and `Status` in `crates/core/src/comments/record.rs` has two variants, `Open` and `Resolved`.

Two states cannot express the review loop the dashboard is for. When an agent acts on feedback, the only status it can set is the one that also means "the reviewer accepts this" — so either the agent closes its own thread, or the thread never moves and the next `comment-resolution` verdict re-points the agent at feedback it already handled. The predecessor implementation in `openspec-doc-rs-example` used three states (`open`, `addressed`, `resolved`) and exposed reply, status, edit, and delete from a thread popover; the Rust rewrite dropped both.

The gap is observable rather than theoretical. `AGENTS.md` instructs agents not to resolve comments they were asked to address, which is a rule standing in for a missing state, and the MVP criterion run ended with two comments carrying agent replies and still marked `open` — correct behaviour with nothing to record it.

**This change was originally half an interface change.** Per-comment reply, resolve and reopen controls, and rendered status counts, targeted `crates/server/src/page/review.rs` and `routes.rs` — both deleted by `replace-dashboard-frontend`. Those requirements moved there; what remains here is the model, the CLI, and the directive templates. That half is untouched by the rewrite and is a prerequisite of it.

## Goals / Non-Goals

**Goals:**

- The agent has a status it can honestly set, distinct from the reviewer's acceptance.
- Rejecting an agent's response reopens the existing thread rather than requiring a duplicate comment.
- A scope reports how much feedback is outstanding, in a form `add-change-approval-gate` can gate on and the replacement frontend can render.
- The whole change is verifiable from a terminal.

**Non-Goals:**

- Anything rendered. Thread controls and visible counts belong to `replace-dashboard-frontend` now.
- Editing or deleting comments. The predecessor had both; neither is needed for the review loop, and deletion fights the append-only sidecar.
- Hard enforcement of who may set which status. See the trade-off below.

## Decisions

**`addressed` is a third `Status` variant, not a boolean beside the existing two.** The sidecar already appends status events and reconstructs current state by folding them, so a third variant costs one enum arm and one match in the fold. Modelling it as a separate "agent responded" flag would create two orthogonal state machines to reason about at the approval gate.

**Reopen is a status event to `open`, not a distinct `reopened` status.** The status history already records what came before, so a fourth variant would encode information the fold can already recover, and would double the cases every consumer has to match on.

**The CLI offers all three transitions; no interface may offer `address`.** This is the split that makes the loop honest: `address` is a claim about work performed, which the agent makes from the CLI as part of acting on a directive; `resolve` is a judgement about that work, which the reviewer makes. The prohibition is specified where the controls now live — `replace-dashboard-frontend` carries a requirement that no control moves a comment to `addressed` — so that a later contributor cannot "complete" the interface by adding one.

**Counts are computed from reconstructed status, not tallied from events.** Folding is already the only correct way to read a sidecar; counting raw events would miscount every comment whose status changed.

**Counts ship as core data with no consumer in this change.** Both consumers are downstream — the replacement frontend renders them, the approval gate blocks on them — so this is a function and its tests, exercised through `comment list` and nothing else. Deferring it until a caller exists would mean the caller's change carries a core-model addition instead, which is the wrong seam.

## Risks / Trade-offs

- [Risk] Nothing stops an agent from calling `comment resolve` instead of `comment address`. Both are CLI operations and the agent has a shell. → Accepted, not mitigated. The separation is enforced by the reason-text templates and by the interface's control set, not by the CLI. Hard enforcement would require an actor concept — identity on every sidecar event — which is a much larger change than the loop currently justifies. Worth revisiting if an agent is observed resolving its own threads.
- [Risk] Closing a comment stays a terminal operation until `replace-dashboard-frontend` lands, in a tool whose premise is browser-based review. → Accepted as the lesser cost. The alternative is writing thread controls into files already scheduled for deletion. It does mean this change improves the reviewer's position only indirectly, by making the state exist for the frontend to expose.
- [Trade-off] Adding `addressed` to the status set means an existing sidecar written by the current binary reads back fine, but a sidecar containing `addressed` cannot be read by an older binary. Per the project's no-backwards-compatibility rule this is not a migration concern; it is noted only so nobody mistakes it for one.

Two ordering constraints this change originally carried have since been satisfied and are removed: the `dashboard-html-views` delta needed `add-dashboard-html-views` archived, which happened on 2026-08-02 and is moot now that the delta has moved out; and the reason-template edit needed `add-directive-verdict-loop`, archived 2026-08-04.
