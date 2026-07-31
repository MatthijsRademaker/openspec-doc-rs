## 1. Serve Configuration and Model State

- [x] 1.1 Add default live review agent analysis behavior plus an explicit opt-out serve option and document the enabled/default behavior.
- [x] 1.2 Extend viewer/server model state so rendered pages know whether review agent analysis is enabled, explicitly disabled, or unavailable.
- [x] 1.3 Update CLI/server startup output to report review agent analysis status and that initial analysis mode is non-mutating.

## 2. Review Bundle and Session Core

- [x] 2.1 Add types and helpers for review agent bundles containing change name, artifact snapshots, warnings, selected/open feedback threads, anchor states, and reviewer instruction.
- [x] 2.2 Validate browser session requests against safe active change names, active change artifacts, and selected comment ids.
- [x] 2.3 Persist lightweight bundle/session events under `.pi/openspec-review/` without modifying `openspec/` artifacts.
- [x] 2.4 Add recovery helpers that list recent sessions and completed response content for an active change.

## 3. Pi SDK Bridge and Server APIs

- [x] 3.1 Add a Pi SDK bridge that starts browser review analysis sessions by default unless review agent mode is explicitly disabled.
- [x] 3.2 Convert Pi SDK session events into normalized browser events for lifecycle, text deltas, tool activity, completion, abort, and errors.
- [x] 3.3 Add server endpoints for creating sessions, streaming events, listing recent sessions, sending follow-up messages, aborting running sessions, and retrying failed sessions where safe.
- [x] 3.4 Ensure the first implementation does not expose an apply endpoint and does not configure write-capable behavior for browser-triggered analysis.
- [x] 3.5 Return actionable unavailable errors when Pi SDK setup, credentials, or model availability prevent session startup.

## 4. Companion Browser UI

- [x] 4.1 Add Ask Pi controls to the active change companion page by default when review agent analysis is available.
- [x] 4.2 Add explicit disabled and unavailable review agent states that preserve existing comment creation and thread review behavior.
- [x] 4.3 Show the context that will be sent to Pi, including open or selected feedback, artifact snapshots, and warnings.
- [x] 4.4 Render a live in-browser transcript with streaming text, lifecycle state, completion state, failure messages, and safe retry actions.
- [x] 4.5 Add follow-up message UI for an existing review agent session, with clear behavior while a response is already streaming.
- [x] 4.6 Show recovered recent completed sessions after browser refresh.

## 5. Safety, Documentation, and Validation

- [x] 5.1 Add tests for default enabled mode, explicit opt-out mode, unavailable mode, active-change validation, archived/missing change rejection, and unsafe change rejection.
- [x] 5.2 Add tests for bundle creation scope, selected feedback filtering, persisted session recovery, retry behavior, and non-mutating behavior.
- [x] 5.3 Add viewer render/client tests for Ask Pi controls, disabled/unavailable states, transcript markup, retry actions, and follow-up UI.
- [x] 5.4 Add server/API tests using a stub Pi bridge so tests do not require live model credentials.
- [x] 5.5 Update README documentation for the review agent mode, safety boundaries, and relationship to existing sidecar feedback prompt workflow.
- [x] 5.6 Run `openspec validate add-browser-pi-review-sessions` and the relevant npm validation scripts.
