## Why

The current Pi-assisted review flow opens a read-only browser page, but reviewers still need to keep Pi/chat in a separate window to read guidance, record comments, and request iterations. A review companion mode should let users review artifacts and capture feedback in one browser experience while preserving Pi as the intentional iteration engine.

## What Changes

- Add an explicit review companion mode for active OpenSpec changes.
- Provide an in-browser companion panel alongside the proposal artifacts so users do not need two windows open to both review and read review guidance or existing feedback.
- Allow users to capture structured review comments from the browser without directly editing OpenSpec artifacts.
- Store review comments as sidecar feedback records outside `openspec/` artifacts.
- Add a Pi-facing workflow that can read sidecar feedback and route it back into Pi for intentional proposal/design/spec/task iteration.
- Keep default `openspec-doc serve` read-only; writable companion behavior must be opt-in and explicit.

## Capabilities

### New Capabilities
- `review-companion`: Covers the opt-in browser companion mode, sidecar review feedback records, and Pi-routed review iteration workflow.

### Modified Capabilities
- `browser-viewer`: Adds an integrated review companion layout so users can review artifacts and read/capture feedback without juggling a separate browser/Pi window.

## Impact

- Viewer UI gains an explicit companion mode route or toggle for active changes.
- Server layer may gain opt-in endpoints for writing sidecar feedback records outside `openspec/`.
- Pi prompts or workflow resources may gain commands for summarizing and applying review feedback.
- Tests must cover opt-in write behavior, default read-only behavior, sidecar storage, companion layout, and feedback routing instructions.
