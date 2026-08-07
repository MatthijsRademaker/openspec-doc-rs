## 1. Blocks from markdown

- [x] 1.1 Add `pulldown-cmark` to the workspace and a markdown module in `crates/server`
- [x] 1.2 Decompose an artifact with `into_offset_iter()` into blocks: paragraphs, headings, list items, table rows, and lines within a code fence
- [x] 1.3 Each block carries a stable id, its rendered HTML, its exact source text, and its byte range
- [x] 1.4 Slice the source text from the file by the reported range. Do not reconstruct it from the event stream — that is the only place a mismatch with `markdown.find` can come from
- [x] 1.5 Assert, for every block of every real artifact in this repository's `openspec/` tree, that `markdown.find(block.source)` succeeds. This is the property the whole frontend replacement rests on, so the sample is real documents rather than a hand-written fixture
- [x] 1.6 Add targeted tests for nested lists, tables, fenced code, setext headings, and an HTML block

## 2. Markup in artifacts is text

- [x] 2.1 Disable raw-HTML passthrough: escape `Event::Html` and `Event::InlineHtml` rather than emitting them
- [x] 2.2 Add a test that `<script>alert(1)</script>` in an artifact reaches the caller as escaped text, not as markup

## 3. Anchoring the intended occurrence

- [x] 3.1 Add `search_from: usize` to `anchor::create` and search from that offset, adding it back to the result
- [x] 3.2 Thread the same parameter through `sidecar::add`
- [x] 3.3 Update the CLI and server callers to pass `0`, which is what they mean: a string and no idea where in the file it is
- [x] 3.4 Add a test with two identical list items under different headings, asserting a comment created against the second records the second's offset and the second's heading path
- [x] 3.5 Add a test that `search_from: 0` reproduces today's behaviour exactly, so existing records are provably unaffected

## 4. Verify

- [x] 4.1 `cargo test` across the workspace — ran; three known macOS `notify` watcher tests fail, all non-watcher suites pass
- [x] 4.2 Confirm no existing comment sidecar in this repository resolves differently before and after
