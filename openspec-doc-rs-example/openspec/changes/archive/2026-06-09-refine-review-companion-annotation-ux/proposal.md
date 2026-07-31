## Why

The current review companion still feels like a form-based side tool rather than a clean document review experience. Its narrow canvas, large sticky side panel, forced focus behavior, and nested scrolling make reading and navigation feel cramped and distracting.

## What Changes

- Replace the permanent comment creation form with intentional selected-text annotation controls.
- Show unobtrusive in-text or margin markers for anchored comments in rendered artifacts.
- Open comment creation and existing comment threads in popovers near the relevant document context.
- Stop automatically focusing the comment editor or scrolling users away from the selected text when they select or right-click content.
- Use a wider review companion canvas while keeping rendered Markdown line length comfortable for reading.
- Remove the dominant sticky companion side panel from the primary review flow.
- Move the comments overview into a separate, clean page section for review triage and navigation.
- Improve scrolling so the companion page uses one primary vertical scroll surface and avoids nested panel scrolling.
- Preserve the existing sidecar feedback storage model and review thread/status/edit/delete behaviors.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `review-companion`: Refine the browser review companion requirements for document-first annotation UX, in-text markers, popovers, wider layout, separate comments overview, and calmer scrolling behavior.

## Impact

- Affects the server-rendered companion route in `src/viewer/render.ts`.
- Affects browser-side companion JavaScript for text selection, comment creation, marker interaction, thread popovers, and overview navigation.
- May require enhancing anchor rendering so existing resolved comments are represented as document markers.
- Adds hard-delete behavior for browser review comments by removing the selected comment thread from sidecar feedback storage.
- Should not change sidecar feedback storage format unless marker rendering needs additional derived data.
- Should not introduce browser editing of OpenSpec proposal, design, tasks, or spec delta Markdown files.
