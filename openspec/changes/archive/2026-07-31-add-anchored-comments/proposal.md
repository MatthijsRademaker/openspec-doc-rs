## Why

Inline anchored commenting is the review-facing feature carried over most directly from the TS example, and it needs to work identically whether the underlying markdown is an explore-phase scratch note or a real OpenSpec artifact. Porting the anchor-resolution algorithm and comment sidecar as a standalone, artifact-agnostic capability now means the dashboard views and the directive loop can both consume it without re-deriving anchor logic.

## What Changes

- Ports the anchor-context creation and resolution algorithm (`createAnchorContext` / `resolveAnchorContext` from `openspec-doc-rs-example/src/core/review-anchors.ts`): exact offset match, then heading/before/after-text fuzzy resolution, then orphaned/missing.
- Ports the comment/reply/status-update record model and its JSONL append-only sidecar persistence (from `review-companion.ts`), generalized so `artifactPath` can be any markdown file — a scratch note or an OpenSpec change artifact — with no special-casing between them.
- Sidecar location follows the same scope key as its artifact: `.openspec-doc/comments/<name>.jsonl` post-promotion, `.openspec-doc/comments/_session/<session_id>.jsonl` pre-promotion — the same session/change namespacing the scratch notes use.
- Comment sidecar relocation in lockstep with the artifact's scope-key rename (session id to change name), preserving all events and appending a relocation event so the preserved anchors follow the renamed artifact.
- Fills the `comment` CLI placeholder subcommand from `add-cli-surface`: `add`, `list`, `reply`, `resolve`.

## Capabilities

### New Capabilities
- `anchored-comments`: anchor-context creation/resolution, the comment/reply/status sidecar model, scope-key relocation, and the `comment` CLI subcommand.

### Modified Capabilities
(none)

## Impact

New logic in the `core` crate (or a `comments` module within it). Fills the `comment` subcommand in the `cli` crate. Depends on `add-rust-foundation`.
