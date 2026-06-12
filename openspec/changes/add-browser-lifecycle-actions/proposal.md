## Why

Users can see lifecycle guidance in the browser, but still must leave the local viewer to run every lifecycle state transition. Because `openspec-doc serve` is a strictly local tool, it should optionally provide guarded browser buttons for full OpenSpec lifecycle actions while preserving explicit user confirmation and clear mutation boundaries.

## What Changes

- Add local-only browser lifecycle action endpoints behind the existing local dev server.
- Add UI buttons for Explore, Propose/continue, Review status, Apply handoff, validation, task state changes, and Archive actions where appropriate.
- Require browser-side confirmation for mutating or command-running actions and report command output/status inline.
- Keep actions allowlisted and scoped to discovered active changes and the target project root.
- Preserve non-hosted/local assumptions: no remote execution API, no SaaS mode, and no unauthenticated network exposure beyond the bound local server.

## Capabilities

### New Capabilities
- `browser-lifecycle-actions`: Covers local browser-triggered lifecycle actions, mutation guardrails, command execution results, and task state updates.

### Modified Capabilities
- `browser-viewer`: Adds action buttons and inline status for local lifecycle state changes on dashboard/change/review pages.
- `review-companion`: Adds lifecycle action controls while preserving sidecar review and Pi analysis behavior.

## Impact

- Affected code: local server routes, viewer rendering, lifecycle model/action metadata, task parsing/updating utilities, and tests.
- Affected APIs: new local HTTP endpoints under `/api/lifecycle-actions/...` for allowlisted browser actions.
- Affected UX: lifecycle panels will include buttons in addition to copyable commands, with confirmations and results.
- Security/safety: local-only command execution and file mutation require explicit confirmation, validate requested changes against the scanned model, and avoid arbitrary command execution.
