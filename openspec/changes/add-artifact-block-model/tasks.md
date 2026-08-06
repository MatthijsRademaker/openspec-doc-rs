## 1. Blocks from markdown

- [ ] 1.1 Add `pulldown-cmark` to the workspace and a markdown module in `crates/server`
- [ ] 1.2 Decompose an artifact with `into_offset_iter()` into blocks: paragraphs, headings, list items, table rows, and lines within a code fence
- [ ] 1.3 Each block carries a stable id, its rendered HTML, its exact source text, and its byte range
- [ ] 1.4 Slice the source text from the file by the reported range. Do not reconstruct it from the event stream — that is the only place a mismatch with `markdown.find` can come from
- [ ] 1.5 Assert, for every block of every real artifact in this repository's `openspec/` tree, that `markdown.find(block.source)` succeeds. This is the property the whole frontend replacement rests on, so the sample is real documents rather than a hand-written fixture
- [ ] 1.6 Add targeted tests for nested lists, tables, fenced code, setext headings, and an HTML block

## 2. Markup in artifacts is text

- [ ] 2.1 Disable raw-HTML passthrough: escape `Event::Html` and `Event::InlineHtml` rather than emitting them
- [ ] 2.2 Add a test that `<script>alert(1)</script>` in an artifact reaches the caller as escaped text, not as markup

## 3. Anchoring the intended occurrence

- [ ] 3.1 Add `search_from: usize` to `anchor::create` and search from that offset, adding it back to the result
- [ ] 3.2 Thread the same parameter through `sidecar::add`
- [ ] 3.3 Update the CLI and server callers to pass `0`, which is what they mean: a string and no idea where in the file it is
- [ ] 3.4 Add a test with two identical list items under different headings, asserting a comment created against the second records the second's offset and the second's heading path
- [ ] 3.5 Add a test that `search_from: 0` reproduces today's behaviour exactly, so existing records are provably unaffected

## 4. Verify

- [ ] 4.1 `cargo test` across the workspace
- [ ] 4.2 Confirm no existing comment sidecar in this repository resolves differently before and after
