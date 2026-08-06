## 0. Before starting

- [ ] 0.1 Confirm `extend-comment-model` has landed — the `+` composer writes an unanchored comment and the model must be able to hold one
- [ ] 0.2 Confirm the `addressed` status exists, from a re-scoped or merged `add-comment-thread-actions`. Without it a promoted change page has no state meaning "the agent claims it handled this", and the highlight this change adds has nothing to show.
- [ ] 0.3 Re-scope `add-comment-thread-actions` and `add-change-approval-gate`, both of which carry `dashboard-html-views` deltas targeting `crates/server/src/page/review.rs` — a file this change deletes
- [ ] 0.4 Confirm the MVP directive fix has landed separately. This change edits the same templates; it must not be the thing that carries a blocking MVP repair.

## 1. Blocks from markdown

- [ ] 1.1 Add `pulldown-cmark` to the workspace and a markdown module in `crates/server`
- [ ] 1.2 Decompose an artifact into blocks with `into_offset_iter()`: paragraphs, headings, list items, table rows, and lines within a code fence
- [ ] 1.3 Each block carries a stable id, its rendered HTML, its exact source text sliced from the file, and its byte range
- [ ] 1.4 Slice the source text from the file by the reported range rather than reconstructing it from events. Reconstruction is where a mismatch with `markdown.find` would come from.
- [ ] 1.5 Add a test asserting, for each block of a realistic artifact, that `markdown.find(block.source)` succeeds. This is the property the whole anchoring design now rests on.
- [ ] 1.6 Add tests for nested lists, tables, fenced code, and an HTML block
- [ ] 1.7 Escape rendered HTML derived from artifact content. The markdown is agent-written and reaches the page; `pulldown-cmark`'s raw-HTML passthrough must be disabled.
- [ ] 1.8 Add a test that `<script>` in an artifact does not reach the client as markup

## 2. Repeated blocks

- [ ] 2.1 Decide, per `design.md`'s open question, whether to store the block's byte range in the anchor record or rely on before/after context to disambiguate two identical blocks
- [ ] 2.2 Add a test with two identical list items in one artifact, asserting a comment on the second does not resolve to the first
- [ ] 2.3 If the decision is to store the range, coordinate the schema change with `anchored-comments` rather than adding a field the spec does not describe

## 3. The JSON API

- [ ] 3.1 Replace the HTML page routes with JSON handlers: index, scope detail with blocks, review state, comment creation, comment status, verdict submission
- [ ] 3.2 Delete `crates/server/src/page/` and the form-post-and-redirect handling in `routes.rs`. Do not keep the old pages alongside the new ones — two comment surfaces against one sidecar is two things to keep correct.
- [ ] 3.3 Include, per comment, its anchor state and the id of the block it resolves to, so the client can place it without recomputing anything
- [ ] 3.4 Include the standing verdict and whether its directive has been delivered, read from `directives/_session/<id>.json`
- [ ] 3.5 Extend the SSE event to say what changed — artifacts, review state, or both — so the client refetches only what it needs
- [ ] 3.6 Add tests at the route level for each handler, including a scope with no artifacts

## 4. The Vue application

- [ ] 4.1 Create `web/` with Vite, Vue, Tailwind and shadcn-vue
- [ ] 4.2 Render an artifact's blocks with a hover gutter offering "comment here"
- [ ] 4.3 Render comments beside the block they resolve to, collapsed behind an expandable marker
- [ ] 4.4 Expand a marker to the comment, its replies, and its status, with the reviewer's words and the agent's replies visually distinct — that distinction is the product's whole point and the current page does not make it
- [ ] 4.5 Show an `addressed` comment as the agent's claim awaiting the reviewer's judgement, with accept and reopen actions. Never render it as resolved: resolving is the reviewer's act and a tick the agent set itself would be the tool lying about the one judgement the reviewer is there to make.
- [ ] 4.6 Keep free-text selection within a block as a secondary path to commenting
- [ ] 4.7 Surface the refusal reason when a selection crossing inline markup is rejected by the server

## 5. The floating bar

- [ ] 5.1 Two controls: a `+` opening the composer, and the scope's primary verdict
- [ ] 5.2 The composer's textarea is optional; sending with it empty submits the verdict and records no comment
- [ ] 5.3 Sending with text records an unanchored comment and then submits the verdict
- [ ] 5.4 Label the session page's primary control "Move to proposal". Do not label it "Approve" — `add-change-approval-gate` defines an `approved` verdict with stricter meaning, and two Approves in one product is a trap.
- [ ] 5.5 Change pages: `+` submits `comment-resolution` with an optional comment; the primary slot stays empty until the approval gate lands
- [ ] 5.6 Show counts of unanchored and orphaned comments in the bar, opening the same panel

## 6. Unanchored and orphaned comments

- [ ] 6.1 Render unanchored and orphaned comments in one panel behind the `+`
- [ ] 6.2 Show an orphaned comment with its original quoted text and say plainly that its anchor was lost
- [ ] 6.3 Keep fuzzy-anchored comments inline against the block they resolved to, marked as moved
- [ ] 6.4 Add a test that an orphaned comment is reachable from the interface. "Beside the anchor only" must not mean a comment disappears when its anchor breaks.

## 7. Header and delivery state

- [ ] 7.1 Show the scope's title, its standing verdict, and when it was submitted
- [ ] 7.2 Show whether that verdict's directive has been delivered to the agent
- [ ] 7.3 Show verdict history, which has no home once the pooled bottom list is deleted

## 8. Live updates

- [ ] 8.1 Refetch artifacts as well as review state when the SSE event says artifacts changed. This is the defect: the watcher already fires and the client throws the information away.
- [ ] 8.2 Track whether a composer holds unsent text
- [ ] 8.3 Offer a refresh while unsent text exists; apply pending updates silently once it does not
- [ ] 8.4 Preserve scroll position across an artifact refresh
- [ ] 8.5 Verify by hand: open a session page, have an agent rewrite the note, and confirm the note updates. This cannot be settled by a test — it is the claim the product is sold on and it has never been observed working.

## 9. Verdict notes are deleted

- [ ] 9.1 Stop writing verdict notes from the dashboard
- [ ] 9.2 Update the directive templates in `crates/core/src/hook/translate.rs`, which currently promise notes that will always be empty
- [ ] 9.3 Decide whether `Record.notes` survives in core for CLI use or is removed outright, and record the reason
- [ ] 9.4 Update the tests in `translate.rs` that assert the templates name the verdict sidecar for its notes

## 10. Distribution

- [ ] 10.1 Add `rust-embed` and serve the built assets from the binary
- [ ] 10.2 Commit `dist/`
- [ ] 10.3 Add a CI check that rebuilds the frontend and fails if the committed assets differ. Without it, a stale `dist/` ships a frontend that does not match the source, which is the failure mode this distribution choice creates.
- [ ] 10.4 Confirm `cargo install --path .` produces a working dashboard on a machine with no Node

## 11. Specs and docs

- [ ] 11.1 Rewrite `dashboard-html-views`, including its Purpose — the "no framework, no build step" sentence is reversed by this change
- [ ] 11.2 Update `dashboard-server` for the JSON routes, the asset route, and the richer SSE event
- [ ] 11.3 Update `docs/docs/` wherever it describes the pages or the no-build-step property
- [ ] 11.4 Update `AGENTS.md` if it describes the dashboard's rendering

## 12. Verify against a real session

- [ ] 12.1 Comment on a block of a live exploration and confirm the anchor resolves exactly
- [ ] 12.2 Send a verdict with no composer text and confirm the directive reaches the agent and points at the comments
- [ ] 12.3 Rewrite an artifact under an open page with a composer holding text, and confirm the refresh is offered rather than applied
- [ ] 12.4 Rewrite an artifact under an open page with nothing unsent, and confirm it updates silently
- [ ] 12.5 Orphan a comment by rewriting the text it anchors to, and confirm it is still reachable
