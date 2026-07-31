## 1. Companion Layout

- [x] 1.1 Add a dedicated wide companion page shell or route-level class without changing normal dashboard page width.
- [x] 1.2 Replace the sticky full-height companion side panel with a document-first layout and concise guidance section.
- [x] 1.3 Constrain rendered artifact content to a comfortable reading width inside the wider canvas.
- [x] 1.4 Remove full-height independently scrolling feedback panel styles from the primary companion flow.

## 2. Annotation Popovers

- [x] 2.1 Replace the permanent capture feedback form with hidden contextual composer markup for selected-text comments.
- [x] 2.2 Update selection handling so selecting artifact text shows a small comment affordance without focusing an editor or scrolling the page.
- [x] 2.3 Update right-click handling so it opens or shows the contextual comment action without creating feedback or forcing a scroll jump.
- [x] 2.4 Implement popover composer submission using the existing review feedback create API and sidecar payload fields.
- [x] 2.5 Add narrow-screen fallback behavior for the composer when popover positioning is not practical.

## 3. Existing Comment Markers and Threads

- [x] 3.1 Map loaded review feedback records to companion artifacts by artifact path and anchor state.
- [x] 3.2 Render unobtrusive markers for exact and fuzzy anchored comments near their resolved document context.
- [x] 3.3 Implement marker popovers that show comment body, selected text, anchor state, status, replies, and metadata.
- [x] 3.4 Wire reply, edit, and status controls in marker popovers to the existing review feedback thread APIs.
- [x] 3.5 Add hard-delete storage/API behavior that removes a comment and its thread events from sidecar feedback storage.
- [x] 3.6 Wire delete controls in marker popovers and comments overview with confirmation and refresh behavior.
- [x] 3.7 Ensure orphaned and missing comments remain visible even when no document marker can be placed.

## 4. Comments Overview

- [x] 4.1 Add a separate comments overview section below or near the review document content.
- [x] 4.2 Group overview feedback by open, addressed, and resolved status while preserving exact, fuzzy, orphaned, and missing anchor states.
- [x] 4.3 Add overview jump actions that scroll the primary page to document markers and briefly emphasize them.
- [x] 4.4 Keep existing comments inspectable from the overview when no document marker exists.

## 5. Tests and Validation

- [x] 5.1 Update viewer render tests to assert the document-first companion layout, absence of the permanent capture form, and presence of popover/overview containers.
- [x] 5.2 Add or update tests for marker and overview markup produced for loaded exact, fuzzy, orphaned, and missing comments where existing fixtures support it.
- [x] 5.3 Add regression coverage that selection/right-click JavaScript no longer calls `scrollIntoView` or focuses the comment editor before an explicit compose action.
- [x] 5.4 Add server and render regression coverage for hard deleting comments.
- [x] 5.5 Run `npm run build`, `npm run test`, and `openspec validate refine-review-companion-annotation-ux`.
