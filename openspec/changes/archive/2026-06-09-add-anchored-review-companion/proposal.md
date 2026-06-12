## Why

Review companion mode should support natural document review, not just detached feedback forms. Users need to select text in proposal artifacts, leave contextual comments, discuss them with the agent, and turn the accumulated feedback into intentional OpenSpec updates without allowing browser-side edits to mutate project artifacts directly.

## What Changes

- Open the browser to the resolved local viewer URL by default when serving.
- Add selected-text comment capture to review companion mode for rendered active-change artifacts.
- Store comment anchors with enough artifact, heading, selected text, and surrounding context to relocate or mark comments after artifacts change.
- Add comment thread and status behavior so user/agent back-and-forth can happen around specific review points.
- Add a Pi-routed synthesis workflow that compiles open comments into a structured review digest and proposed artifact changes.
- Preserve the safety boundary that browser feedback writes only to sidecar review storage; OpenSpec artifacts are changed only through explicit Pi/user-approved workflows.

## Capabilities

### New Capabilities

### Modified Capabilities
- `review-companion`: Add selected-text anchored comments, comment threads/status, and Pi-routed feedback synthesis for active-change review.

## Impact

- Browser companion UI for active-change review pages.
- Sidecar feedback storage schema under the documented review feedback location outside `openspec/`.
- Server routes or handlers that accept comment/thread/status writes in default companion mode and reject them when companion mode is explicitly disabled.
- Pi-invokable review feedback workflow that reads sidecar comments and produces actionable review digests/proposed OpenSpec artifact updates.
- Tests/fixtures for anchor storage, invalid write rejection, digest generation, and explicit read-only opt-out behavior.
