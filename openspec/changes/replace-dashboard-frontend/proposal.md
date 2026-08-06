## Why

The dashboard renders every artifact as raw markdown source in a `<pre>`, stacked in one column, with the comments pooled at the bottom. A real change page measures 395 lines of HTML and 35KB: six artifacts fully expanded, then a hidden form, then the comments at HTML line 333 — roughly three hundred lines below the text they are anchored to.

Two of the consequences are defects rather than ugliness.

**An anchored comment is not visually anchored.** The whole premise of the tool is that a reviewer attaches a thought to an exact span. In the interface built to expose that, the anchor is a blockquote in a list far below the artifact, and finding the text it refers to means scrolling up and reading until you spot the fragment. The server already computes what it needs — `anchor::resolve` returns a byte offset into the artifact — and spends it rendering the string `" at offset 42"`.

**The artifact never live-updates.** The vision's headline is that an exploration is readable in a browser *while it is happening*. `scope::target` deliberately watches the artifact tree, so the watcher fires when the agent rewrites its note and an SSE event reaches the browser. The client then refetches only the 2.6KB comments-and-verdicts fragment. The 32KB of markdown is never refetched. On a session page the scratch note *is* the only artifact, so the reviewer watches an exploration that does not move.

Beyond those: the composer reveals itself hundreds of lines below where the selection was made, agent replies are visually identical to the reviewer's own words, the standing verdict is at the bottom, nothing anywhere shows whether a submitted verdict was actually *delivered*, and there is no dark mode.

## What Changes

- **The frontend is replaced with Vue and shadcn-vue, built by Vite.** The server stops rendering HTML and becomes a JSON API. `crates/server/src/page/` largely deletes.
- **Markdown is rendered, not shown as source.** The server parses with `pulldown-cmark` and sends *blocks*: each carries its rendered HTML and the exact source text it came from. A comment created on a block sends that source text, which `anchor::create` then finds in the file by construction. The core anchoring stack is untouched.
- **Comments render beside the block they anchor to,** collapsed behind clickable expandable markers. The pooled list at the bottom is deleted.
- **Commenting is a hover gesture on a block,** not select-then-find-the-form. Free-text selection within a block remains as a secondary path.
- **A floating bar carries two controls.** A `+` opening a composer whose send submits the keep-exploring verdict, and a primary button submitting the verdict that moves the work forward.
- **The verdict notes field is deleted.** The composer's text is recorded as an unanchored comment, and the verdict is a pure send. This requires the directive templates to point at the comment sidecar instead of promising notes that will always be empty.
- **Unanchored and orphaned comments share one surface** behind the `+`, with a count in the bar. They are both "comments with no block to sit beside".
- **A header shows the standing verdict and whether it has been delivered,** read from `directives/_session/<id>.json`, which records it and which nothing has ever surfaced.
- **Live artifact updates**, with the refresh offered while a composer has unsent text and applied silently once it does not.
- **Dark mode**, via shadcn's theming rather than hand-written media queries.
- **Built assets are committed and embedded in the binary** with `rust-embed`, so `cargo install` needs no Node toolchain.

## Capabilities

### New Capabilities

None. This replaces how two existing capabilities are delivered.

### Modified Capabilities

- `dashboard-html-views`: substantially rewritten. Its Purpose currently reads "Pages are server-rendered HTML with the minimum vanilla JavaScript that selection and live updates require — no framework, no build step", and every part of that sentence is reversed. Artifact rendering, the comment surface, the commenting gesture, and the verdict controls all change.
- `dashboard-server`: the route table changes from HTML pages and form posts to a JSON API plus a static asset route; the SSE contract gains what changed so the client can decide whether to apply it.

## Impact

- `crates/server/src/page/` — ~670 lines including tests, most of it deleted.
- `crates/server/src/routes.rs` — 881 lines; HTML routes and form handling replaced with JSON handlers.
- A new markdown module in `server` using `pulldown-cmark`, a new workspace dependency.
- `rust-embed`, a new dependency, and a committed `dist/`.
- A new `web/` directory holding the Vue application, its `package.json`, and its Vite configuration.
- `crates/core/src/hook/translate.rs` — the directive templates, which currently promise verdict notes.
- `openspec/specs/dashboard-html-views/spec.md` and `openspec/specs/dashboard-server/spec.md`.
- `docs/docs/` — anything describing the pages or the no-build-step property.

## Dependencies and ordering

- **`extend-comment-model` must land first.** The `+` composer writes an unanchored comment, which the model cannot currently represent.
- **The `addressed` status must exist.** After the agent writes a proposal, comments are still `open` — the agent does not resolve them, deliberately and by three separate rules. Without `addressed` there is no state meaning "the agent claims it handled this", and the promoted change page has nothing to highlight. That status comes from `add-comment-thread-actions`, which must be re-scoped or merged before this starts.
- **`add-comment-thread-actions` is partly invalidated by this change.** Its per-comment reply/resolve/reopen controls target `crates/server/src/page/review.rs`, a file being deleted. Its core and CLI half is a prerequisite; its UI half is absorbed here.
- **`add-change-approval-gate` carries a `dashboard-html-views` delta** with the same problem, and its approve control becomes the primary slot of the floating bar on change pages.
- The `dashboard-html-views` spec does not yet exist in `openspec/specs/` — it lives as a delta in the unarchived `add-dashboard-html-views`. Both dependent changes inherit an ordering constraint on that; since this change rewrites the spec wholesale, that constraint is worth re-examining rather than inheriting.
- **Scheduled after MVP agreement.** None of this is on the MVP's completion criterion, which is that comments reach the agent before it writes the proposal.

## What is given up, deliberately

**The page stops being provably incapable of mis-anchoring.** Showing the file verbatim guaranteed that a selection was a substring of the source. The block model makes it correct by construction instead, which is a real step down in certainty for a large step up in readability.

**Forms stop working without JavaScript.** The composer is currently a real `<form method="post">` that posts and redirects whether or not the script ran.

**`cargo build` gains a committed build artifact.** The alternative — running `npm ci && npm run build` from `build.rs` — keeps the repository tidy and makes a Rust build depend on a working Node toolchain. Requiring Node to `cargo install` a local review tool is the worse tax.

One factual note, recorded once: the repository's only existing frontend toolchain is React, in the rspress docs site. Vue means two frameworks in one repository, and `shadcn-vue` is a community port that tracks behind the React original. They are separate artifacts and the cost is small; the choice is the owner's and is made.
