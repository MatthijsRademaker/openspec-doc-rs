## Why

The entire premise of this MVP — that a coding-agent's turn-end hook can veto the stop and re-inject a directive, autonomously, for both Claude Code and pi.dev — has not been proven in this codebase. The dashboard, comments, and scratch-note workflow are worthless if this doesn't actually work. This change proves the mechanism standalone, against a hand-authored directive file, before any dashboard exists to produce one.

## What Changes

- Implements the `hook` subcommand (filling the placeholder from `add-cli-surface`): `openspec-doc hook stop --agent <claude|pi>`.
- Reads the agent-specific Stop-hook payload from stdin and normalizes it into an internal, agent-agnostic event (agent kind, session id, transcript path, cwd).
- Defines a session-scoped pending-directive file format and lookup: if a pending directive exists for the session, emit that agent's "block and continue" response with the directive's reason text; otherwise emit "allow stop".
- Marks a consumed directive so it does not re-fire on a subsequent Stop event for the same session.
- Directive files in this change are hand-written test fixtures only — no dashboard or agent writes them yet. Change-name-scoped directives and the promotion mechanic belong to later changes (`add-scratch-note-workflow`, `add-directive-verdict-loop`).

## Capabilities

### New Capabilities
- `agent-hook-bridge`: normalized hook event ingestion, session-scoped directive lookup/consumption, and per-agent (Claude Code, pi.dev) block/allow decision emission.

### Modified Capabilities
(none)

## Impact

New `hook` module in the `core` crate (event model, per-agent wire formats, directive file) with the subcommand wiring in `cli`. Adds `serde`, `serde_json`, and `chrono` to the workspace. For manual verification in this repo: a local `.claude/settings.json` Stop hook, and — because pi.dev has no external-process hook — a thin project-local pi extension at `.pi/extensions/`, the only TypeScript in this repo. The `hook` placeholder error is removed from `cli`; no other existing code affected.
