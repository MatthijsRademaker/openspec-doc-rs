## Why

The dashboard shows every artifact as its own source in a `<pre>`. That was not laziness: it makes the page provably incapable of mis-anchoring, because a browser selection is literally a substring of the file the server searches. Rendering the markdown appears to give that up, since a DOM selection would have to be mapped back onto source offsets.

It does not. `anchor::create` is a substring search:

```rust
let Some(start_offset) = markdown.find(selected_text) else { ... };
```

and the client sends `selection.toString()` and nothing else — no offsets, no ranges. The real requirement was never "the DOM must be the source". It is "the string the client sends must occur verbatim in the source".

`pulldown-cmark`'s `into_offset_iter()` yields every event with its byte range into the source, so the server can decompose an artifact into **blocks** that carry both representations at once: the rendered HTML to display, and the exact bytes it was sliced from to anchor against. A client that echoes back a block's source cannot fail the find, by construction.

This change builds that decomposition and nothing else. No rendering, no frontend, no route changes. It is the load-bearing technical risk of the whole frontend replacement — every interface decision downstream rests on the claim that `markdown.find(block.source)` always succeeds — and it is settleable entirely from `cargo test`, before a line of Vue exists.

## What Changes

- **`pulldown-cmark` becomes a workspace dependency** and a markdown module is added to `crates/server`.
- **An artifact is decomposed into blocks**: paragraphs, headings, list items, table rows, and lines within a code fence. Each block carries a stable id, its rendered HTML, the exact source text sliced from the file by the reported byte range, and that range.
- **The source text is sliced, never reconstructed from events.** Reconstruction is the only place a mismatch with `markdown.find` could come from.
- **Raw HTML passthrough is disabled.** The markdown is agent-written and reaches a browser; `<script>` in an artifact must arrive as text.
- **`anchor::create` and `sidecar::add` gain the offset to search from.** Today `create` finds the *first* occurrence of the selected text. Two list items reading `- [ ] Add tests` in one file produce the same source string, and the second one's comment silently anchors to the first. That is a pre-existing hazard that block-level commenting makes routine, and it cannot be fixed downstream: the block's byte range is known to the caller and discarded at the API boundary.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dashboard-server`: gains the artifact-decomposition requirement. Filed here rather than under `dashboard-html-views` deliberately — producing blocks is a server capability and displaying them is a rendering one, and conflating the two is what made the frontend replacement look like a single indivisible change.
- `anchored-comments`: `create` and `add` take the offset to search from, so a caller that knows where the text is says so. No schema change — `Anchor` already stores `start_offset`.

## Impact

- `Cargo.toml` — `pulldown-cmark` as a workspace dependency.
- A new markdown module in `crates/server`.
- `crates/core/src/comments/anchor.rs` and `crates/core/src/comments/sidecar.rs` — one parameter each.
- `crates/cli/src/comment.rs` and `crates/server/src/routes.rs` — callers pass `0`, which is what they mean today.

## Dependencies and ordering

- **Nothing blocks this.** It touches no interface, so it can land alongside `add-vue-dashboard-foundation` rather than before or after it.
- **`migrate-dashboard-review-to-vue` consumes it.** That change renders blocks and creates anchors from them; without this it has nothing to render.

## What is given up, deliberately

**The page stops being *provably* unable to mis-anchor.** Showing the file verbatim meant the reviewer could see that their selection was the file's own bytes. A block's source becomes something the server asserts instead — correct by construction rather than correct by inspection. That is a real reduction in certainty, and the property test in this change is what stands in for the reviewer's eyes.
