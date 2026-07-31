## Why

OpenSpec proposal review currently happens inside the chat or raw Markdown files, while `openspec-doc` already provides a browser surface for inspecting active changes. Starting a proposal review flow from Pi should make it easy to open the relevant browser page immediately, review the generated artifacts visually, and leave room for a future browser-to-Pi comment loop without changing the default read-only viewer behavior.

## What Changes

- Add a Pi-assisted review workflow entrypoint for opening or starting review of an active OpenSpec change.
- Start or reuse the local `openspec-doc` browser viewer when the review workflow begins.
- Direct the user to the active change page for the reviewed proposal.
- Keep normal `serve` behavior read-only and avoid browser-side writes in this first slice.
- Document the intended future "review companion" direction, where browser comments can be captured as sidecar feedback and routed back to Pi for iteration.

## Capabilities

### New Capabilities
- `pi-assisted-review-flow`: Pi integration behavior for starting an OpenSpec review session and connecting users to the browser viewer for an active change.

### Modified Capabilities
- `browser-viewer`: Adds review-oriented navigation for active changes while preserving read-only behavior by default.

## Impact

- Project Pi resources under `.pi/` may gain a prompt template and/or extension that starts the viewer and guides review.
- CLI/server code may need a reusable server startup path or stable URL behavior suitable for Pi integration.
- Browser viewer rendering may add review-oriented links or active-change detail affordances without adding write controls.
- Tests should cover the review URL/navigation behavior and ensure the viewer remains read-only in this slice.
