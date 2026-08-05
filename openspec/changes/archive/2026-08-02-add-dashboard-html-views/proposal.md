## Why

The dashboard needs real, viewable pages before the review loop is usable by a human: rendering the scratch note or change artifacts, showing anchored comments, letting the reviewer select text and comment, and letting them submit a phase verdict. This change wires the placeholder routes from `add-dashboard-server` to real files and real interactions, without yet wiring any of it into the agent hook.

## What Changes

- Session-scoped page (`/sessions/<session_id>`): renders the scratch note markdown, its anchored comments, a select-to-comment UI, and phase-verdict controls — "keep exploring" with a free-text open-questions/resolutions box, or "move to proposal".
- Change-scoped page (`/changes/<name>`): renders `proposal.md`/`design.md`/`tasks.md`/spec deltas, their anchored comments, the same select-to-comment UI, and a comment-resolution verdict control ("send to agent: address open comments").
- Live updates: pages subscribe to the SSE endpoint from `add-dashboard-server` and update without a full reload when underlying files or comments change.
- Plain server-rendered HTML plus minimal vanilla JS for selection/comment interactions — no Vue, no client-side framework, no build step, per the MVP scope decision.
- A phase-verdict record type distinct from per-comment status: a session-level decision (keep-exploring-with-notes or move-to-proposal) that isn't attached to any single comment.
- Out of scope: wiring verdicts/resolved comments into the pending-directive file the hook bridge reads — that integration is `add-directive-verdict-loop`.

## Capabilities

### New Capabilities
- `dashboard-html-views`: server-rendered session and change pages, the select-to-comment UI, live SSE-driven updates, and the phase-verdict record.

### Modified Capabilities
(none)

## Impact

New page-rendering and static-asset code in the `server` crate, using `add-anchored-comments` for comment persistence and `add-dashboard-server`'s routing/SSE plumbing. No existing code affected.
