## Why

`openspec-doc` currently helps users inspect OpenSpec content and review active changes, but it does not guide users through the complete OpenSpec change lifecycle. Users still need to know when to explore, propose, review, apply, and archive, and how each phase connects to the browser viewer, Pi workflows, sidecar feedback, validation, and implementation tasks.

## What Changes

- Add lifecycle guidance that presents OpenSpec work as a staged flow: Explore → Propose → Review → Apply → Archive.
- Add browser lifecycle surfaces for active changes that summarize the current phase, recommended next actions, required artifacts, review state, validation status, and archive readiness.
- Add project-level lifecycle entry points so users can start from the dashboard when they have no active change, one active change, or multiple active changes.
- Add phase-specific guidance for how to use Pi, OpenSpec artifacts, browser review feedback, implementation tasks, validation, and archiving safely.
- Add non-mutating command guidance and copyable commands for `openspec`, Pi, and `openspec-doc` actions rather than performing destructive lifecycle transitions from the browser.
- Preserve approval-gated artifact mutation: browser guidance may recommend apply/archive actions, but it does not directly edit OpenSpec artifacts or archive changes unless a future explicit command adds that behavior.

## Capabilities

### New Capabilities
- `change-lifecycle-guidance`: Guides users through the OpenSpec change lifecycle across explore, propose, review, apply, and archive phases.

### Modified Capabilities
- `browser-viewer`: Adds project-level and change-level lifecycle navigation, status, and next-action guidance to the viewer.
- `review-companion`: Adds lifecycle-aware review guidance that connects review feedback to apply and archive readiness.

## Impact

- Affected code: viewer model/rendering, active change pages, dashboard sections, analyzer/status summarization, and tests/fixtures for active changes in different lifecycle states.
- Affected UX: users see lifecycle stage guidance and next actions in the browser alongside existing OpenSpec summaries and review companion pages.
- Affected commands: existing `serve`, `summary`, and `check` remain read-only; the proposal may add display of copyable lifecycle commands, but not browser-side execution of apply/archive actions.
- Dependencies: no new runtime dependency is expected for the initial guidance layer.
