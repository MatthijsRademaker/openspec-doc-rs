## 0. Before starting

- [ ] 0.1 Confirm `add-artifact-block-model` has landed. Without blocks there is nothing to render and nothing to anchor against
- [ ] 0.2 Confirm `add-vue-dashboard-foundation` has landed. It provides the toolchain, the embedding, the theme and the index that navigates into these pages
- [x] 0.3 `extend-comment-model` — archived 2026-08-06. Unanchored comments exist, so the `+` composer has a record to write
- [x] 0.4 `add-comment-thread-actions` — archived 2026-08-06. The `addressed` status exists and its interface half was deliberately left for this change
- [ ] 0.5 `add-change-approval-gate` carries a `dashboard-html-views` delta against files this change deletes. It is last in priority and may never be built, so it is left alone rather than re-scoped speculatively. If it is ever started, do this first
- [ ] 0.6 Confirm `add-dashboard-development-harness` has landed. It provides pinned Bun, Router, working Vite API proxying, frontend and embedded-browser gates, the review-workbench design reference, and repository-specific frontend skills and MCP

## 1. The JSON API

- [ ] 1.1 Add a scope detail endpoint returning the scope's artifacts as blocks, its comments with their resolved anchor states, and its verdict state
- [ ] 1.2 Include, per comment, its anchor state and the id of the block it resolves to, so the client places it without recomputing anything
- [ ] 1.3 Include the standing verdict and whether its directive has been delivered, read from `directives/_session/<id>.json`
- [ ] 1.4 Add JSON endpoints for comment creation, reply, status change, and verdict submission, recording through the same core writers the CLI uses
- [ ] 1.5 Pass the commented block's byte range as the anchor's `search_from`, so a repeated block anchors to the occurrence the reviewer pointed at. Passing `0` here would reintroduce the bug `add-artifact-block-model` fixed, with the parameter in place
- [ ] 1.6 Delete `crates/server/src/page/` and the form-post-and-redirect handling in `routes.rs`. Do not keep the old pages alongside the new ones — two comment surfaces against one sidecar is two things to keep correct
- [ ] 1.7 Serve the interface at `/sessions/<session_id>` and `/changes/<name>`, with 404 on an undiscovered scope from the data endpoint rather than an empty scope
- [ ] 1.8 Route-level tests for each handler, including a scope with no artifacts and a scope with no comments

## 2. Rendering the artifact

- [ ] 2.1 Render a scope's artifacts as their blocks' HTML
- [ ] 2.2 Offer a comment action on the block under the pointer
- [ ] 2.3 Keep free-text selection within a block as a secondary path to commenting
- [ ] 2.4 Surface the refusal reason when a selection crossing inline markup is rejected by the server. It already refuses and already reports why; the reason must reach the reviewer
- [ ] 2.5 Verify a comment created from a block resolves `exact`, and one created from a free-text selection still does

## 3. Comments beside their block

- [ ] 3.1 Render each comment against the block its anchor resolves to, collapsed behind an expandable marker
- [ ] 3.2 Expand a marker to the comment, its replies and its status
- [ ] 3.3 Render the reviewer's words and the agent's replies visually distinct. The current page renders them identically, which in a tool whose output is a conversation between two parties is not a styling gap
- [ ] 3.4 Keep fuzzy-anchored comments inline against the block they resolved to, marked as having moved
- [ ] 3.5 Delete the pooled comment list at the bottom of the page

## 4. Comments with no block

- [ ] 4.1 Render unanchored and orphaned comments in one panel behind the `+`
- [ ] 4.2 Show an orphaned comment with its original quoted text and say plainly that its anchor was lost
- [ ] 4.3 Show the count of them in the bar, opening the same panel
- [ ] 4.4 Test that an orphaned comment is reachable from the interface. "Beside the anchor only" must not mean a comment disappears when its anchor breaks

## 5. Thread actions

- [ ] 5.1 Offer reply, resolve and reopen per comment, selected by current status
- [ ] 5.2 Offer no control anywhere that marks a comment `addressed`. It asserts work an agent did, not a judgement the reviewer makes
- [ ] 5.3 Show an `addressed` comment as the agent's claim awaiting judgement, with accept and reopen. Never render it as resolved — a tick the agent set itself would be the tool lying about the one judgement the reviewer is there to make
- [ ] 5.4 Show the scope's open, addressed and resolved counts
- [ ] 5.5 Update threads and counts over the existing event channel rather than on reload
- [ ] 5.6 Tests: an open comment offers reply and resolve; a resolved one offers reopen and not resolve; no path in the interface can set `addressed`; a thread action in one tab appears in a second

## 6. The floating bar

- [ ] 6.1 Two controls: a `+` opening the composer, and the scope's primary verdict
- [ ] 6.2 The composer's textarea is optional; sending with it empty submits the verdict and records no comment
- [ ] 6.3 Sending with text records an unanchored comment and then submits the verdict
- [ ] 6.4 Label the session page's primary control "Move to proposal". Do not label it "Approve" — `add-change-approval-gate` defines an `approved` verdict with stricter meaning, and two Approves in one product is a trap
- [ ] 6.5 Change pages: `+` submits `comment-resolution` with an optional comment; the primary slot stays empty until the approval gate lands

## 7. Header and delivery state

- [ ] 7.1 Show the scope's title, its standing verdict, and when it was submitted
- [ ] 7.2 Show whether that verdict's directive has been delivered to the agent. It is recorded and has never been shown anywhere, so submitting a verdict is currently indistinguishable from submitting into nothing
- [ ] 7.3 Show verdict history, which has no home once the pooled bottom list is deleted

## 8. Verdict notes are deleted

- [ ] 8.1 Stop writing verdict notes from the dashboard
- [ ] 8.2 Update the directive templates in `crates/core/src/hook/translate.rs`, which currently promise notes that will always be empty, to point at the comment sidecar
- [ ] 8.3 Update the tests in `translate.rs` that assert the templates name the verdict sidecar for its notes
- [ ] 8.4 Decide whether `Record.notes` survives in core for CLI use or is removed outright, and record the reason

## 9. Specs and docs

- [ ] 9.1 Rewrite `dashboard-html-views` for the new pages, and finish the Purpose edit `add-vue-dashboard-foundation` started — the server-rendered clause it narrowed to the scope pages is now false for them too
- [ ] 9.2 Update `dashboard-server` for the scope routes' JSON endpoints and the removal of form-post handling
- [ ] 9.3 Update `docs/docs/` wherever it describes the pages
- [ ] 9.4 Update `AGENTS.md` if it describes the dashboard's rendering

## 10. Verify against a real session

- [ ] 10.1 Comment on a block of a live exploration and confirm the anchor resolves exactly
- [ ] 10.2 Comment on a block that appears twice in one artifact and confirm it lands on the one pointed at
- [ ] 10.3 Reply to a comment, resolve it, and reopen it, all without leaving the browser
- [ ] 10.4 Send a verdict with no composer text and confirm the directive reaches the agent and points at the comments
- [ ] 10.5 Orphan a comment by rewriting the text it anchors to, and confirm it is still reachable
- [ ] 10.6 Confirm the header reports a submitted verdict as undelivered until the agent's next turn boundary consumes it
