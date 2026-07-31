## 1. Sidecar Review Model

- [x] 1.1 Define anchored comment, thread reply, status, and digest types in the core review companion model.
- [x] 1.2 Implement sidecar storage read/write helpers for anchored comments and thread/status updates outside `openspec/`.
- [x] 1.3 Add validation for active change names, artifact references, selected text, and companion-mode write enablement.
- [x] 1.4 Add tests that verify browser feedback writes never modify files under `openspec/`.

## 2. Anchor Capture and Resolution

- [x] 2.1 Add server-side or shared utilities to create anchor context from artifact path, selected text, nearest heading/section, and surrounding text.
- [x] 2.2 Implement best-effort anchor resolution with `exact`, `fuzzy`, `orphaned`, and `missing` states.
- [x] 2.3 Add fixture tests for exact matches, moved text, changed surrounding context, missing text, and missing artifacts.

## 3. Companion Browser UI

- [x] 3.1 Add selected-text comment controls to active-change companion pages in default review companion mode.
- [x] 3.2 Render anchored comments in the companion panel with artifact, selected text, anchor state, status, and timestamps.
- [x] 3.3 Add interactions to jump from a comment to its resolved artifact location when available.
- [x] 3.4 Add thread reply and status update controls that write only to sidecar review storage.
- [x] 3.5 Ensure selected-text comment controls and write requests are unavailable when companion mode is explicitly disabled.

## 4. Pi Feedback Synthesis Workflow

- [x] 4.1 Add or extend a Pi-invokable workflow that reads open review companion comments for an active change.
- [x] 4.2 Generate a structured review digest grouped by theme, artifact, or affected requirement with source comment ids.
- [x] 4.3 Include proposed proposal/design/task/spec-delta updates when inferable and separate unresolved questions when not inferable.
- [x] 4.4 Require explicit user approval before applying synthesized changes to OpenSpec artifacts.
- [x] 4.5 Report clearly when no open review companion feedback exists for the active change.

## 5. Validation and Documentation

- [x] 5.1 Document the anchored comment sidecar format and safety boundary for companion mode.
- [x] 5.2 Add command/UI smoke tests for companion-mode default and disabled behavior.
- [x] 5.3 Run OpenSpec validation for `add-anchored-review-companion`.
- [x] 5.4 Run the relevant project tests once implementation exists.
- [x] 5.5 Open the browser by default to the resolved serve URL, with a `--no-open` opt-out.
