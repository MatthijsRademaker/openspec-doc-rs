## Why

With only `open` and `resolved` in the status set, the review loop has no honest closure step: either the agent marks its own work resolved, or nothing ever advances. `addressed` is the missing state, and it is a prerequisite of everything downstream — `replace-dashboard-frontend` cannot show the agent's claim as distinct from the reviewer's acceptance without it, and `add-change-approval-gate` has no counts to gate on.

**This change was originally half an interface change and has been re-scoped to its model and CLI half.** The other half — per-comment reply, resolve and reopen controls, and rendered status counts — targeted `crates/server/src/page/review.rs` and `routes.rs`, both of which `replace-dashboard-frontend` deletes. Those requirements moved there rather than being dropped, because that change now owns how a reviewer acts on a comment thread. What is left here is the part that survives the rewrite untouched, and it is the part the rewrite depends on.

## What Changes

- Add `addressed` to the comment status set, sitting between `open` and `resolved`. Marking a comment addressed is a claim about work done, which an agent can legitimately make; resolving it is a judgement about whether the work is right, which only the reviewer makes.
- Add CLI operations for marking a comment addressed and reopening it, alongside the existing `add`/`list`/`reply`/`resolve`.
- Add a per-scope status count over open / addressed / resolved, as core data. Rendering it is `replace-dashboard-frontend`'s; `add-change-approval-gate` consumes it as its precondition.
- Point the verdict-to-directive reason templates at reply-and-mark-addressed rather than resolve, so the agent reports back into the thread instead of closing it.

Deliberately **not** in scope: anything rendered. The dashboard keeps creating comments and nothing else until the frontend is replaced, which means closing a comment stays a terminal operation in the meantime. That is a real gap for that window, and it is the lesser cost — the alternative is writing thread controls into files scheduled for deletion.

## Capabilities

### New Capabilities

None. This extends two capabilities that already exist.

### Modified Capabilities

- `anchored-comments`: the status set gains `addressed`; the CLI subcommand requirement gains the operations that set and clear it.

`dashboard-html-views` is no longer modified here. Its requirements moved to `replace-dashboard-frontend`.

## Impact

- `crates/core/src/comments/record.rs` — the `Status` enum, currently `Open | Resolved`.
- `crates/core/src/comments/sidecar.rs` — status events and the state reconstruction that folds them.
- `crates/cli/src/cli.rs` and `crates/cli/src/comment.rs` — the new subcommands.
- The reason-text templates introduced by `add-directive-verdict-loop` task 2.1.

Nothing in `crates/server/`. That is the point of the re-scope.

Both ordering constraints this change originally carried have since been satisfied and are removed: `dashboard-html-views` now exists in `openspec/specs/` (`add-dashboard-html-views` archived 2026-08-02), and `add-directive-verdict-loop` archived 2026-08-04, so the reason-template edit has its base.

**`replace-dashboard-frontend` depends on this.** Its task 0.2 blocks on `addressed` existing, and an interface that cannot distinguish the agent's claim from the reviewer's acceptance would misreport the one judgement the reviewer is there to make. Land this first; it is verifiable entirely from a terminal.
