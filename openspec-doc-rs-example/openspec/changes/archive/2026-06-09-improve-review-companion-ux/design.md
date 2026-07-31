## Context

The current review companion renders all active-change artifacts in one long page with a sticky side panel. Comment creation depends on selecting text and using a separate form, which is functional but feels unlike a document review workflow. The sidecar event log already supports comment creation, replies, and status updates; it does not yet support editing the original comment body.

## Goals / Non-Goals

**Goals:**
- Make the main review area easier to scan by grouping artifacts into tabs for proposal, design, tasks, and spec deltas.
- Let reviewers right-click selected artifact text to start a comment near their document context.
- Let reviewers edit existing comment bodies inline from the feedback list.
- Continue storing all comments, replies, statuses, and edits as sidecar events outside `openspec/` artifacts.

**Non-Goals:**
- No direct browser editing of proposal, design, tasks, or spec delta markdown files.
- No rich-text editor dependency or full collaborative editing model.
- No persistent visual text highlights beyond the existing anchor resolution metadata and jump-to-artifact behavior.

## Decisions

- **Use client-side tabs over separate routes.** The companion route will render all artifacts once and use accessible tab buttons to switch visible stage panels. This keeps the server route simple and avoids losing form/comment state during navigation.
- **Derive stage tabs from artifact names.** Existing `ViewerArtifact` records already identify Proposal, Design, Tasks, and Spec Delta artifacts. The renderer can group spec deltas under a single tab and show empty-state text for missing stages.
- **Use the native contextmenu event for right-click comments.** When the user right-clicks selected text inside a companion artifact, the browser prevents the default context menu, pre-fills the feedback form with anchor context, and focuses the comment field. This implements the requested right-click flow without adding a custom menu dependency.
- **Represent edits as append-only sidecar events.** Add an `edit` event type containing comment id, replacement body, author/source, and timestamp. While reading threads, apply the latest edit to the comment body and update `updatedAt`/`timestamp`. This preserves review history and avoids mutating existing JSONL records.
- **Inline edit in the feedback list.** Each feedback record gets an Edit button that toggles a small textarea form in place. Saving posts to a dedicated edit endpoint and reloads the feedback list.

## Risks / Trade-offs

- **Right-click selection behavior varies by browser** → Keep the existing selection-based form as a fallback and show clear helper text.
- **Client script complexity grows inside server-rendered HTML** → Keep the interaction code scoped to the companion page and covered by render/API tests.
- **Editable comments may obscure history** → Store edits append-only and expose updated timestamps now; detailed edit history UI can be added later if needed.
