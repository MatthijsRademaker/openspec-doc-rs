## Why

Every prior change in this MVP proves one piece in isolation: the hook bridge blocks and reinjects against a hand-written directive, the dashboard writes phase-verdict records and comments against real files, and scratch-note promotion runs as a standalone library call. Nothing yet turns a dashboard-written verdict into a directive the hook bridge actually consumes, and nothing yet invokes promotion from within a live Stop event. This change closes both loops and is the end-to-end proof that the MVP's central premise — autonomous, hook-driven continuation — actually holds.

## What Changes

- Extends `openspec-doc hook stop` to run the scratch-note promotion check (`add-scratch-note-workflow`) for the session before directive lookup, recording the validate outcome when a promotion occurs.
- Adds verdict-to-directive translation: when an untranslated phase-verdict record exists (keep-exploring, move-to-proposal, or comment-resolution, from `add-dashboard-html-views`), translate it into the pending-directive file format `add-agent-hook-bridge` already consumes.
- Reason text is always a short pointer plus verdict — naming the relevant sidecar/artifact paths explicitly — never the literal comment or note content embedded inline, per the "pointer, not embed" principle.
- A translated verdict is marked consumed so it is not translated again on a later invocation.
- Includes an end-to-end manual verification against a real Claude Code session and a real pi.dev session as a required deliverable, not an afterthought.

## Capabilities

### New Capabilities
- `directive-verdict-loop`: promotion-on-hook-stop wiring, phase-verdict-to-directive translation, and verdict consumption.

### Modified Capabilities
(none)

## Impact

Extends the `hook stop` command in the `cli`/`server` crates, composing `add-agent-hook-bridge`, `add-scratch-note-workflow`, `add-anchored-comments`, and `add-dashboard-html-views`. No existing specs are modified — this is additive wiring.
