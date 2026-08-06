## Context

The dashboard was built to an explicit MVP constraint: server-rendered HTML, no framework, no build step. That constraint bought a self-contained binary and a page that is provably incapable of mis-anchoring, because it shows the artifact's bytes verbatim and a browser selection is therefore literally a substring of the file.

It also produced a page that is 395 lines and 35KB for one change, renders six documents as unstyled source, pools the comments three hundred lines below the text they annotate, and never refreshes the artifact it exists to display live. The constraint has been paid for and the interface is what was bought.

This change spends the constraint deliberately rather than eroding it. What replaces it has to keep the property that made anchoring trustworthy, and the block model below is the whole reason it can.

## Goals / Non-Goals

**Goals:**
- A comment is visible next to the text it is about.
- The artifact on screen matches the artifact on disk, without stealing the reviewer's half-typed sentence to achieve it.
- Reading a proposal in the dashboard is not worse than reading it in an editor.
- Anchoring keeps working exactly as it does now, with no new failure mode.
- The binary still installs with `cargo install` and no other toolchain.

**Non-Goals:**
- Not editing artifacts from the browser. The dashboard reads and annotates; the agent writes. That exclusion is in the vision and is unchanged.
- Not remote or multi-user. `127.0.0.1`, one reviewer.
- Not a diff view. Comments anchor to text in a live document, not to a frozen revision.
- Not restyling the docs site, which is a separate artifact on a separate toolchain.

## Decisions

### The block model, and why it dissolves the raw-versus-rendered problem

The existing design argues that rendering markdown would require mapping a DOM selection back onto source offsets. Reading `anchor::create` shows the contract is narrower than that:

```rust
let Some(start_offset) = markdown.find(selected_text) else { ... };
```

A substring search. The client sends `selection.toString()` and nothing else — no offsets, no ranges, no positions. So the requirement was never "the DOM must be the source". It is only "the string the client sends must appear verbatim in the source".

`pulldown-cmark`'s `into_offset_iter()` yields every event with its byte range into the source. So the server can decompose an artifact into blocks that carry both representations:

```
markdown source on disk
  │
  ├─ into_offset_iter() ──► block ranges
  │
  ▼
[ { id, source: "…exact bytes from the file…", range: [start,end], html: "<p>…</p>" }, … ]
  │
  ▼  JSON
Vue renders html; the gutter hover targets the block
  │
  ▼  comment on a block
POST { artifact_path, selected_text: block.source }
  │
  ▼
anchor::create runs markdown.find(block.source) ──► matches, by construction
```

The client never computes an offset and never sends one. It echoes back a string the server extracted from the file, so the find cannot fail for a block that still exists. **The entire core anchoring stack — `anchor.rs`, the Exact/Fuzzy/Orphaned ladder, the sidecar — is untouched by this change.**

Free-text selection inside a block stays available as a secondary gesture on the existing path. It can be refused when the selection crosses inline markup, because the rendered text of `**bold**` is `bold` and that is not in the source. That refusal already exists and already reports its reason.

What is genuinely lost: the page is no longer *provably* unable to mis-anchor, because the block's source is now something the server asserts rather than something the reviewer can see. It becomes correct by construction instead of correct by inspection. That is a real reduction in certainty and it is the price of the change.

### Blocks, not lines

The reviewer's request was hover-to-comment "based on lines". The gesture is right; the anchor must not be.

GitHub anchors comments to line numbers because a pull request diff is frozen. Nothing here is frozen — the premise of this tool is that an agent rewrites the artifact while the reviewer reads it. Insert one paragraph at the top and every line-anchored comment silently points at different text. Silently, which is the failure mode this project refuses everywhere else, and which `anchor.rs` was written to avoid: it stores before/after context and a heading path precisely so drift is reported rather than hidden.

A block is also the better hover target. In rendered markdown a paragraph is one element, not N source lines, so "the line" has no visual referent anyway. Blocks are paragraphs, list items, headings, table rows, and lines within a code fence.

### The bar has two controls, and one of them is not a verdict control

The exploration asked how a comment differs from keep-exploring notes. Two ways, and only one is real:

| | anchored to a span | fires a directive |
|---|---|---|
| Comment | yes | no |
| Keep-exploring notes | no | yes |

The second difference belongs to verdicts and stays. The first is a gap in the comment model, not a difference in kind — which `extend-comment-model` closes.

So the notes field is deleted. The `+` composer writes an unanchored comment; the send fires the verdict; the directive points at the sidecar, which already holds anchored and unanchored feedback alike. One kind of thing written, one kind of thing pressed.

Two consequences:

- **The composer's text is optional.** Keep-exploring notes are `required` today. Once inline comments carry the substance, the common case is sending five anchored comments back with nothing to add, and demanding a sentence to allow it is friction for its own sake.
- **The directive templates must change.** `translate::reason` currently says *"with notes on what is still open. Those notes are the last record in `{verdicts}`"*. That becomes false the moment notes are always empty. This is the same edit the roadmap lists as the MVP's first blocking item — pointing the move-to-proposal directive at the comment sidecar — and it should land on its own, before and independently of this change, rather than arriving as a side effect of a frontend rewrite scheduled for later.

### The primary button says "Move to proposal", not "Approve"

`add-change-approval-gate` introduces an `approved` verdict with real preconditions: every comment resolved, bound to a fingerprint of the artifacts approved, stale once they change. Two buttons in the same product both labelled Approve — one meaning "formalize this exploration", one meaning "I accept this as ready to implement" — is a trap, and the second is the consequential one.

| page | left `( + )` | primary |
|---|---|---|
| session | comment, unanchored | Move to proposal |
| change | comment, unanchored | Approve — absent until `add-change-approval-gate` lands |

The change page gains a notes-carrying way to send comments back, which it does not have today: `comment-resolution` currently carries nothing beyond itself.

### Orphaned comments keep a home

"Comments beside their anchor only" would make a comment vanish at the exact moment its anchor breaks, because an `Orphaned` or `Missing` anchor has no block to sit beside. That is the silent-failure shape this project refuses, and it would defeat the Exact/Fuzzy/Orphaned ladder's entire purpose.

Unanchored comments and orphaned comments are the same rendering problem — comments with no block — so they share one panel behind the `+`, with a count in the bar. Fuzzy anchors still resolve to a block, so they stay inline with a marker showing they moved.

### Refresh: offer while dirty, apply when clean

The client tracks one thing: whether a composer is open with unsent text.

```
SSE event arrives
      │
      ├── composer has unsent text ──► show "this has changed — refresh"
      │                                  │
      │                                  └── on send or cancel ──► apply silently
      │
      └── nothing unsent ─────────────► apply silently
```

Nothing is ever yanked out from under someone mid-sentence, and nobody clicks a banner they do not care about. Rejected alternative: always swapping, which loses a half-written comment and is the reason the current client refetches only the review fragment.

### The server becomes a JSON API

There is no worthwhile half-measure. Hover gutters, inline expandable comment threads, an optimistic composer and dirty-aware live updates are the workload a component framework exists for, and driving them from server-rendered fragments means reimplementing a framework in `page/script.rs`.

Handlers return JSON; one route serves the embedded assets; the SSE stream stays and gains an indication of what changed so the client can decide whether the artifact or only the review state needs refetching. Form-post-and-redirect handling is deleted rather than kept alongside.

### Distribution: commit `dist/`, embed with `rust-embed`

| | `cargo install` needs Node | build output in git |
|---|---|---|
| commit `dist/` + `rust-embed` | no | yes |
| `npm ci && npm run build` in `build.rs` | yes | no |

Chosen: commit `dist/`. This is a local tool someone installs once and then forgets about; making that install depend on a working Node toolchain is a worse tax than diff noise on frontend commits. The cost is real — a stale `dist/` is a genuine failure mode — so a check that the committed assets match a fresh build belongs in CI, and a task covers it.

### Vue and shadcn-vue

Chosen by the owner. Recorded once for the record: the repository's only existing frontend toolchain is React, in the rspress docs site, and `shadcn-vue` is a community port that tracks behind the React original. The two frontends are separate artifacts with no shared code, so the cost is a second toolchain in the repository and nothing more.

## Risks / Trade-offs

- **A stale committed `dist/` ships a frontend that does not match the source.** Mitigated by a CI check that rebuilds and compares, which has to exist or the decision above is not safe.
- **`pulldown-cmark`'s block decomposition has edge cases** — nested lists, tables, HTML blocks, footnotes. A block whose source is not uniquely findable in the file makes `markdown.find` land on the wrong occurrence. The existing anchor already stores before/after context to disambiguate; the tasks cover feeding it.
- **Two comments on identical blocks.** Two list items reading `- [ ] Add tests` in the same file produce the same `selected_text`. `markdown.find` returns the first. This is a pre-existing property of the anchor model, but block-level commenting makes it much easier to hit, so the block's byte range must be used to disambiguate rather than discarded.
- **This is a large change with no MVP justification.** It is scheduled after MVP agreement for that reason, and nothing in it should be allowed to block the two directive-delivery items that are on the criterion's path.
- **A frontend rewrite invalidates unstarted UI work.** `add-comment-thread-actions` and `add-change-approval-gate` both carry `dashboard-html-views` deltas targeting files this change deletes. Both need re-scoping before this starts, not after.

## Migration Plan

The old and new frontends do not coexist. The server-rendered pages are deleted in the same change that adds the API, because keeping both means maintaining two comment surfaces against one sidecar.

`extend-comment-model` and the `addressed` status land first, both from the terminal and both verifiable there. This change then has a comment model that can express everything its interface offers.

## Open Questions

- Does the change page's primary slot stay empty until `add-change-approval-gate` lands, or does it show a disabled control that explains itself?
- Should the block-level anchor store the source byte range in the anchor record to disambiguate repeated blocks, or is before/after context sufficient? The second needs no schema change; the first is more direct.
- Is the docs site's React toolchain worth reusing for a shared component or theme, or do the two stay entirely separate? Recommendation: entirely separate.
