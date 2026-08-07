## Why

The scope pages are the dashboard. Everything the tool claims to be happens there, and two of the things wrong with them are defects rather than ugliness.

**An anchored comment is not visually anchored.** The whole premise of the tool is that a reviewer attaches a thought to an exact span. In the interface built to expose that, the anchor is a blockquote in a list far below the artifact, and finding the text it refers to means scrolling up and reading until you spot the fragment. A real change page measures 395 lines of HTML and 35KB: six artifacts fully expanded, then a hidden form, then the comments at HTML line 333 — roughly three hundred lines below the text they are anchored to. The server already computes what it needs — `anchor::resolve` returns a byte offset into the artifact — and spends it rendering the string `" at offset 42"`.

**A comment thread is a terminal operation from the browser.** Creating a comment is the only thread action the interface has ever supported. Closing one means leaving the browser, running `comment list` to recover a UUID, and pasting it into `comment resolve` — in a tool whose premise is browser-based review. `add-comment-thread-actions` built the core and CLI half and left the interface half for whichever change owned the pages.

Beyond those: the composer reveals itself hundreds of lines below where the selection was made, agent replies are visually identical to the reviewer's own words, and the standing verdict is at the bottom with nothing anywhere showing whether it was actually *delivered*.

`add-artifact-block-model` has produced blocks that can be rendered and anchored against. `add-vue-dashboard-foundation` has produced a toolchain, a theme and a shipping story, proven on the index. This change spends both on the pages they were built for.

## What Changes

- **The scope pages become Vue and the server becomes a JSON API for them.** `crates/server/src/page/` is deleted along with the form-post-and-redirect handling in `routes.rs`.
- **Markdown is rendered, not shown as source.** Artifacts arrive as blocks, each carrying its rendered HTML and the exact source it was sliced from. A comment on a block sends that source and the block's offset, so the anchor is exact by construction.
- **Comments render beside the block they anchor to,** collapsed behind expandable markers. The pooled list at the bottom is deleted.
- **Commenting is a hover gesture on a block.** Free-text selection within a block remains as a secondary path, refusing with its reason when the selection crosses inline markup.
- **Unanchored and orphaned comments share one surface,** with a count. They are both "comments with no block to sit beside", and a comment must not vanish at the moment its anchor breaks.
- **A comment thread can be acted on**: reply, resolve, reopen, selected by status, with the scope's open/addressed/resolved counts. No control marks a comment `addressed` — that asserts work an agent did. An `addressed` comment is shown as the agent's claim awaiting judgement, never as resolved.
- **A floating bar carries two controls**: a `+` opening a composer whose send submits the keep-exploring or comment-resolution verdict, and a primary control submitting the verdict that advances the work.
- **The verdict notes field is deleted.** The composer's text is recorded as an unanchored comment and the verdict becomes a pure send, so the directive templates must point at the comment sidecar instead of promising notes that will always be empty.
- **A header shows the standing verdict and whether it has been delivered,** read from `directives/_session/<id>.json`, which records it and which nothing has ever surfaced.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dashboard-html-views`: substantially rewritten. Artifact rendering, the comment surface, the commenting gesture, thread actions and the verdict controls all change. Its Purpose loses the "server-rendered HTML" clause that `add-vue-dashboard-foundation` narrowed to the scope pages.
- `dashboard-server`: the scope routes serve the interface plus a JSON endpoint; form-post handling is removed.

## Impact

- `crates/server/src/page/` — deleted. Roughly 670 lines including tests, minus `index.rs` which the foundation change already removed.
- `crates/server/src/routes.rs` — 881 lines; HTML routes and form handling replaced with JSON handlers.
- `web/` — the scope pages, the comment surface, the composer and the bar.
- `crates/core/src/hook/translate.rs` — the directive templates, which currently promise verdict notes.
- `openspec/specs/dashboard-html-views/spec.md` and `openspec/specs/dashboard-server/spec.md`.
- `docs/docs/` — anything describing the pages.

## Dependencies and ordering

- **`add-artifact-block-model` must land first.** This change renders blocks and anchors against them; without it there are none.
- **`add-vue-dashboard-foundation` must land first.** It provides the toolchain, the embedding, the theme, and the index that navigates into these pages.
- **`add-dashboard-development-harness` must land after the foundation and before this cutover.** It provides the pinned Bun workflow, Router boundary, local API proxy, frontend and embedded-browser gates, design reference, and repository-specific frontend guidance; this change consumes those contracts rather than recreating them.
- **`add-dashboard-development-harness` must land next.** It replaces the frontend package manager with pinned Bun, establishes Router, working local API proxying, executable frontend and browser gates, and the design and coding-harness guidance this interaction-heavy cutover depends on.
- **`extend-comment-model` has landed** (archived 2026-08-06). The `+` composer writes an unanchored comment, which the model can now represent.
- **`add-comment-thread-actions` has landed** (archived 2026-08-06). Its `addressed` status exists, which is what makes a promoted change page have something to highlight: after the agent writes a proposal the comments are still `open`, because the agent does not resolve them, deliberately and by three separate rules. Its interface half was deliberately left unbuilt and arrives here.
- **`add-change-approval-gate` carries a `dashboard-html-views` delta** targeting files this change deletes. It is last in priority and may never be built, so it is left alone rather than re-scoped speculatively. If it is ever started, re-scope it first. Its approve control becomes the primary slot of the floating bar on change pages.
- **Live artifact updates are not in this change.** `add-live-artifact-updates` follows immediately. Artifacts do not live-update today either, so shipping without it is not a regression — but it is the vision's headline claim and it should not sit unbuilt for long.

## What is given up, deliberately

**Forms stop working without JavaScript.** The composer is currently a real `<form method="post">` that posts and redirects whether or not the script ran.

**The reviewer stops seeing the file's own bytes.** `add-artifact-block-model` made anchoring correct by construction instead of correct by inspection; this is the change where the reviewer stops being able to check it by eye. The property test there is what stands in for that.
