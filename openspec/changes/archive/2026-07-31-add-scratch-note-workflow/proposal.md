## Why

Explore-phase commenting needs a real markdown artifact to anchor against, and that artifact must exist before a formal OpenSpec change directory does — there is no `proposal.md` yet during pure exploration. This change defines that scratch note's location and format, keyed by session id, and the promotion mechanic that renames it (and folds in a validation check) to a change-name key once the agent formalizes the exploration into a proposal.

## What Changes

- Scratch note location/format: `.openspec-doc/scratch/_session/<session_id>.md`, a plain markdown file, while no change directory exists yet for that session.
- Promotion detection: when an active change directory newly appears under `openspec/changes/<name>/` for a session with a pending scratch note, treat it as that session's formalized proposal.
- Promotion action: rename `.openspec-doc/scratch/_session/<session_id>.md` to `.openspec-doc/scratch/<name>.md`, leaving a small "moved" pointer record at the old path so a still-open dashboard tab can redirect.
- Promotion is followed by running `openspec validate <name>` and recording the outcome (pass/fail plus output) as part of the promotion result — validation failure is surfaced, never silently discarded.
- Ambiguity guard: if more than one new active change appears between two checks for the same session, auto-promotion is skipped and an explicit ambiguity notice is recorded instead of guessing.
- Out of scope for this change: the dashboard UI, the comment engine, and wiring promotion into the live hook invocation path (that integration belongs to `add-directive-verdict-loop`). This change delivers promotion as a standalone, independently testable library operation.

## Capabilities

### New Capabilities
- `scratch-note-workflow`: session-scoped scratch note lifecycle, promotion detection, rename-on-promotion, and validate-on-promotion.

### Modified Capabilities
(none)

## Impact

New logic in the `core` crate (or a `scratch` module within it). No existing code affected. Depends on `add-rust-foundation` for project-root and active-change discovery.
