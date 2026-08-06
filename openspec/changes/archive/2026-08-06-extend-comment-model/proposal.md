## Why

Two things a reviewer wants to do have no representation in the comment model, and both are prerequisites of the dashboard rewrite rather than decorations on it.

**A comment must be able to have no anchor.** Today `anchor::create` refuses an empty selection, so every comment is attached to a span of an artifact. The reviewer's unanchored feedback therefore has nowhere to go except the `notes` field of a phase verdict — a third kind of text, neither anchored like a comment nor transported like one, that exists only because comments cannot be scoped to a whole document. `replace-dashboard-frontend` deletes that field and routes the same text through the comment sidecar, which means the sidecar has to accept it.

The exploration reached this by asking how a comment and a keep-exploring note actually differ. They differ in two ways: anchoring, and whether they fire a directive. The second is real and belongs to verdicts. The first is a gap in the comment model, not a difference in kind.

**A comment must be editable.** A reviewer who mistypes has no recourse but to add a second comment correcting the first, which leaves the agent two instructions and no way to tell which is current.

## What Changes

- A comment's anchor becomes optional. An unanchored comment is scoped to a session or a change rather than to a span, and is stored in the same sidecar as every other comment.
- An unanchored comment scoped to a specific artifact is **not** added. Either a comment points at text or it points at the scope; an artifact-wide comment is a third case with no demand behind it.
- Add an edit operation, recorded as an append-only edit event that the read folds like every other event. The original body stays in the file: this is an append-only log and a later record never rewrites an earlier one.
- Both operations reach the CLI, since the CLI is how the dashboard's behaviour is verified.
- Anchor resolution reports an unanchored comment as such rather than running it through the Exact/Fuzzy/Orphaned ladder, which has nothing to resolve against.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `anchored-comments`: a comment's anchor becomes optional; the operation set gains editing; the CLI subcommand requirement gains both.

## Impact

- `crates/core/src/comments/record.rs` — `Comment.anchor` becomes `Option<Anchor>`, and `Event` gains an edit variant. Existing sidecar records carry an `anchor` field and deserialize into the option unchanged, so the JSONL already on disk in this repository keeps reading.
- `crates/core/src/comments/sidecar.rs` — an add path that takes no selection, an `edit` writer, and the fold in `read`.
- `crates/core/src/comments/artifact.rs` — resolution for a comment with no anchor.
- `crates/cli/src/comment.rs` and `crates/cli/src/cli.rs` — the two operations.
- `crates/server/src/page/review.rs` — rendering a comment with no anchor, so the current dashboard does not panic on one before it is replaced.
- `openspec/specs/anchored-comments/spec.md`.

## Relationship to the other changes

This is `core` and CLI only, with no interface work beyond not breaking the existing page. It is split out of `replace-dashboard-frontend` because that change is large enough already and because these two operations are independently testable from a terminal.

**The overlap with `add-comment-thread-actions` is settled: that change was re-scoped to its model and CLI half and landed first.** So `Status` already holds `addressed`, `Event` already carries status updates for it, and the CLI group already has `address` and `reopen`. This change adds to that shape rather than racing it: the anchor becomes optional, `Event` gains an edit variant, and the CLI group gains `edit`. The MODIFIED CLI requirement in this change's delta is stated over the seven-operation version that change left behind, not the five-operation one it replaced.
