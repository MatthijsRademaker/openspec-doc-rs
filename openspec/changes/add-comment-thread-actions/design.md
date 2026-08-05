## Context

`add-dashboard-html-views` shipped comment creation from the browser and nothing else. The rendered page has exactly three buttons: cancel, add comment, and the change-scoped "send to agent" verdict. Reply and resolve exist only as CLI subcommands, and `Status` in `crates/core/src/comments/record.rs` has two variants, `Open` and `Resolved`.

Two states cannot express the review loop the dashboard is for. When an agent acts on feedback, the only status it can set is the one that also means "the reviewer accepts this" — so either the agent closes its own thread, or the thread never moves and the next `comment-resolution` verdict re-points the agent at feedback it already handled. The predecessor implementation in `openspec-doc-rs-example` used three states (`open`, `addressed`, `resolved`) and exposed reply, status, edit, and delete from a thread popover; the Rust rewrite dropped both.

## Goals / Non-Goals

**Goals:**

- A reviewer can close a thread without leaving the browser.
- The agent has a status it can honestly set, distinct from the reviewer's acceptance.
- A scope reports how much feedback is outstanding, in a form `add-change-approval-gate` can gate on.
- Rejecting an agent's response reopens the existing thread rather than requiring a duplicate comment.

**Non-Goals:**

- Editing or deleting comments. The predecessor had both; neither is needed for the review loop, and deletion fights the append-only sidecar.
- In-text comment markers and popovers. The current pages render the markdown source verbatim with a comment list beside it; relocating comments into the document is a separate presentation change.
- Hard enforcement of who may set which status. See the trade-off below.

## Decisions

**`addressed` is a third `Status` variant, not a boolean beside the existing two.** The sidecar already appends status events and reconstructs current state by folding them, so a third variant costs one enum arm and one match in the fold. Modelling it as a separate "agent responded" flag would create two orthogonal state machines to reason about at the approval gate.

**Reopen is a status event to `open`, not a distinct `reopened` status.** The status history already records what came before, so a fourth variant would encode information the fold can already recover, and would double the cases every consumer has to match on.

**The dashboard offers resolve and reopen but not address; the CLI offers all three.** This is the split that makes the loop honest: `address` is a claim about work performed, which the agent makes from the CLI as part of acting on a directive; `resolve` is a judgement about that work, which the reviewer makes from the page. Spec'ing the absence of an address control keeps a later contributor from "completing" the UI by adding it.

**Counts are computed from reconstructed status, not tallied from events.** Folding is already the only correct way to read a sidecar; counting raw events would miscount every comment whose status changed.

**Thread controls are plain form posts, matching the composer.** The existing page is server-rendered HTML with roughly thirty lines of vanilla JS and no build step. Status controls are ordinary POSTs that redirect; the SSE subscription that already refetches the review fragment picks up the result in other tabs with no new client code.

## Risks / Trade-offs

- [Risk] Nothing stops an agent from calling `comment resolve` instead of `comment address`. Both are CLI operations and the agent has a shell. → Accepted, not mitigated. The separation is enforced by the reason-text templates and by the dashboard's control set, not by the CLI. Hard enforcement would require an actor concept — identity on every sidecar event — which is a much larger change than the loop currently justifies. Worth revisiting if an agent is observed resolving its own threads.
- [Risk] The `dashboard-html-views` delta has no base spec to apply against until `add-dashboard-html-views` is archived. → Ordering constraint, recorded in the proposal. This change cannot be archived before that one.
- [Risk] The reason-template bullet assumes `add-directive-verdict-loop` groups 1–2 have landed. → If they have not, that task drops and the decision moves back into task 2.1 of that change, where it is already recorded.
- [Trade-off] Adding `addressed` to the status set means an existing sidecar written by the current binary reads back fine, but a sidecar containing `addressed` cannot be read by an older binary. Per the project's no-backwards-compatibility rule this is not a migration concern; it is noted only so nobody mistakes it for one.
