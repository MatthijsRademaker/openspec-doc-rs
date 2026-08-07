## Context

Two changes have already landed the parts of this rewrite that could be settled without an interface. `add-artifact-block-model` proved that markdown can be rendered without giving up anchoring, and that the block a reviewer comments on anchors to the occurrence they meant. `add-vue-dashboard-foundation` proved that a Vue application ships inside the binary, installs without Node, and does not go stale unnoticed.

`add-dashboard-development-harness` must land before this cutover. It changes no review behavior; it replaces npm with pinned Bun and establishes Router, local API proxying, frontend and embedded-browser gates, a review-workbench design reference, and repository-specific agent guidance. This design assumes those development contracts rather than recreating them here.

What is left is the part that is genuinely about the interface: what a reviewer sees, what they can do to a comment, and where the controls live. This is the cutover.

## Goals / Non-Goals

**Goals:**

- A comment is visible next to the text it is about.
- A comment thread can be replied to, resolved and reopened without leaving the browser.
- Reading a proposal in the dashboard is not worse than reading it in an editor.
- The reviewer can tell whether the verdict they submitted actually reached the agent.

**Non-Goals:**

- Not editing artifacts from the browser. The dashboard reads and annotates; the agent writes. That exclusion is in the vision and is unchanged.
- Not remote or multi-user. `127.0.0.1`, one reviewer.
- Not a diff view. Comments anchor to text in a live document, not to a frozen revision.
- Not live artifact updates. That is the immediate successor change, `add-live-artifact-updates`.

## Decisions

### One cutover, no coexistence

The foundation change could keep the old scope pages alongside the new index because the index has no comment surface. This change has no such escape. Keeping the server-rendered pages alongside the new ones means two comment surfaces writing to one sidecar, two composers to keep in agreement about what an anchor is, and two places for a status transition to be wrong. So `page/` is deleted in the same change that adds the JSON handlers, and form-post handling goes with it rather than being retained.

There is no worthwhile half-measure on the server side either. Hover gutters, inline expandable threads, an optimistic composer and status controls are the workload a component framework exists for; driving them from server-rendered fragments means reimplementing one in `page/script.rs`.

### Comments sit beside their block, and the ones that cannot still exist

"Comments beside their anchor only" would make a comment vanish at the exact moment its anchor broke, because an `Orphaned` or `Missing` anchor has no block to sit beside. That is the silent-failure shape this project refuses, and it would defeat the Exact/Fuzzy/Orphaned ladder's entire purpose — the ladder exists to *report* drift, not to absorb it.

Unanchored comments and orphaned comments are the same rendering problem — comments with no block — so they share one panel behind the `+`, with a count in the bar. Fuzzy anchors still resolve to a block, so they stay inline with a marker showing they moved.

### The bar has two controls, and one of them is not a verdict control

The exploration asked how a comment differs from keep-exploring notes. Two ways, and only one is real:

| | anchored to a span | fires a directive |
|---|---|---|
| Comment | yes | no |
| Keep-exploring notes | no | yes |

The second difference belongs to verdicts and stays. The first was a gap in the comment model, not a difference in kind, and `extend-comment-model` closed it.

So the notes field is deleted. The `+` composer writes an unanchored comment; the send fires the verdict; the directive points at the sidecar, which already holds anchored and unanchored feedback alike. One kind of thing written, one kind of thing pressed.

Two consequences:

- **The composer's text is optional.** Keep-exploring notes are `required` today. Once inline comments carry the substance, the common case is sending five anchored comments back with nothing to add, and demanding a sentence to allow it is friction for its own sake.
- **The directive templates must change in this change.** `translate::reason` currently says *"with notes on what is still open. Those notes are the last record in `{verdicts}`"*. That becomes false the moment notes are always empty, so the template edit is not a follow-up — it is the same unit of work, and the tests in `translate.rs` that assert the templates name the verdict sidecar for its notes move with it.

### The primary button says "Move to proposal", not "Approve"

`add-change-approval-gate` introduces an `approved` verdict with real preconditions: every comment resolved, bound to a fingerprint of the artifacts approved, stale once they change. Two buttons in the same product both labelled Approve — one meaning "formalize this exploration", one meaning "I accept this as ready to implement" — is a trap, and the second is the consequential one.

| page | left `( + )` | primary |
|---|---|---|
| session | comment, unanchored | Move to proposal |
| change | comment, unanchored | Approve — absent until `add-change-approval-gate` lands |

The change page gains a notes-carrying way to send comments back, which it does not have today: `comment-resolution` currently carries nothing beyond itself.

### `addressed` is a claim, and the interface must not let anyone else make it

`addressed` means the agent says it did the work. `resolved` means the reviewer accepts that it did. Those are different people making different statements, and collapsing them is the one thing this interface must not do.

So: no control anywhere marks a comment `addressed`, and an `addressed` comment is rendered as a report awaiting judgement — the agent's reply, with accept and reopen — never as a tick. A tick the agent set itself would be the tool lying about the single judgement the reviewer is present to make.

This also settles what the reviewer's and the agent's words look like: visually distinct, always. The current page renders them identically, which in a tool whose entire output is a conversation between two parties is not a styling gap.

### The reviewer must be able to tell whether a verdict landed

`directives/_session/<id>.json` records the queued directive and whether it has been injected. Nothing has ever surfaced it. So submitting a verdict is indistinguishable from submitting into nothing, and the failure mode the whole hook wiring exists to avoid — the agent never hearing the reviewer — is invisible from the only place the reviewer is looking.

The header shows the standing verdict, when it was submitted, and whether its directive has been delivered. Verdict history goes there too; it has no other home once the pooled bottom list is deleted.

## Risks / Trade-offs

- **This is the change where the reviewer stops seeing the file's bytes.** The certainty was traded in `add-artifact-block-model` and the property test is what stands in for it, but the moment it becomes visible is here.
- **Deleting `page/` is irreversible in the sense that matters:** if the Vue pages are wrong, there is no working page to fall back to. The mitigation is that `add-vue-dashboard-foundation` has already proven the shipping path on a real screen, so what is being risked here is component code and not distribution.
- **`add-change-approval-gate` still carries a `dashboard-html-views` delta against files this change deletes.** It is left alone deliberately — it is last in priority and may never be built, and re-scoping a change speculatively costs more than it saves. If it is ever started, that is its first task.
- **The change page's primary slot is empty until the approval gate lands.** A bar with one control on change pages and two on session pages is asymmetric; the alternative, a disabled control explaining itself, advertises a feature that may never exist.

## Migration Plan

The old and new scope pages do not coexist. `page/` is deleted in the same change that adds the JSON handlers.

`extend-comment-model` and `add-comment-thread-actions` have both landed, so the comment model can already express everything this interface offers: unanchored comments, replies, and the `addressed` status.

Existing comment records are unaffected. The anchoring contract is unchanged — an anchor is still created by finding the submitted text in the artifact's markdown — so every record written by the old interface resolves exactly as before.

## Open Questions

None outstanding. The two the superseded change carried are answered: repeated-block disambiguation was settled in `add-artifact-block-model`, and the change page's empty primary slot is decided above.
