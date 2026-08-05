## Why

The dashboard can only create comments. Reply and status changes are CLI-only, so closing a piece of feedback means leaving the browser, running `comment list` to recover a UUID, and pasting it into `comment resolve` — in a tool whose whole premise is browser-based review. And with only `open` and `resolved` in the status set, the review loop has no honest closure step: either the agent marks its own work resolved, or nothing ever advances.

## What Changes

- Add `addressed` to the comment status set, sitting between `open` and `resolved`. Marking a comment addressed is a claim about work done, which an agent can legitimately make; resolving it is a judgement about whether the work is right, which only the reviewer makes.
- Add per-comment thread controls to session and change pages: reply, mark resolved, and reopen. These reuse the existing SSE refresh, so a status change in one tab lands in another with no reload.
- Add a CLI operation for marking a comment addressed, alongside the existing `add`/`list`/`reply`/`resolve`.
- Point the verdict-to-directive reason templates at reply-and-mark-addressed rather than resolve, so the agent reports back into the thread instead of closing it.
- Render per-scope status counts (open / addressed / resolved). This is the input `add-change-approval-gate` consumes as its precondition.

## Capabilities

### New Capabilities

None. This extends two capabilities that already exist.

### Modified Capabilities

- `anchored-comments`: the status set gains `addressed`; the CLI subcommand requirement gains the operation that sets it.
- `dashboard-html-views`: gains per-comment reply/resolve/reopen controls and per-scope status counts.

## Impact

- `crates/core/src/comments/record.rs` — the `Status` enum, currently `Open | Resolved`.
- `crates/core/src/comments/sidecar.rs` — status events and the state reconstruction that folds them.
- `crates/cli/src/cli.rs` and `crates/cli/src/comment.rs` — the new subcommand.
- `crates/server/src/page/review.rs` — thread controls and counts; `crates/server/src/routes.rs` — the POST routes behind them.
- The reason-text templates introduced by `add-directive-verdict-loop` task 2.1.

Ordering constraints, both real:

- The `dashboard-html-views` spec does not exist in `openspec/specs/` yet — it lives as a delta inside the unarchived `add-dashboard-html-views`. That change must be archived before this delta has a base to apply against.
- The reason-template edit assumes `add-directive-verdict-loop` groups 1–2 have landed. If they have not, that bullet drops out of scope and moves back into task 2.1 of that change.
