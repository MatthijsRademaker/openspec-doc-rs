## Why

The review companion currently presents too much information in one page and makes comment creation feel detached from normal document review. Reviewers need a clearer artifact navigation model and a faster, more familiar way to comment directly on document content.

## What Changes

- Add a tabbed review interface for proposal, design, tasks, and spec delta stages in active changes.
- Add right-click comment creation on rendered artifact text so reviewers can comment without hunting for separate controls.
- Make existing comments editable in the browser with a word-doc-like inline editing flow while keeping all feedback in sidecar storage.
- Preserve existing selected-text anchoring, thread/status controls, and read-only OpenSpec artifact behavior.

## Capabilities

### New Capabilities

### Modified Capabilities
- `review-companion`: Adds tabbed stage navigation, context-menu comment creation, and editable inline sidecar comments.

## Impact

- Affected code: `src/viewer/render.ts`, review companion model helpers, server feedback endpoints, and tests.
- Affected behavior: browser review companion UX for active OpenSpec changes.
- Storage: sidecar feedback records gain update support for comment body edits; OpenSpec proposal/design/tasks/spec files remain unmodified by comments.
