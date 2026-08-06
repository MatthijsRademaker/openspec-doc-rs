## Context

Anchoring is the only thing this tool does that is hard to get right, and the current interface protects it with a blunt instrument: show the file's bytes, so a selection cannot be anything but a substring of them. Every subsequent interface improvement — rendered markdown, hover-to-comment, comments beside their text — is blocked on replacing that protection with something equally trustworthy.

This change exists to settle that question on its own, in Rust, with tests, before anyone commits to a frontend that depends on the answer.

## Goals / Non-Goals

**Goals:**
- An artifact decomposes into blocks whose source text is findable in the artifact, always.
- A comment created from a block's source resolves to the *intended* occurrence, not merely to *an* occurrence.
- Agent-written markdown cannot inject markup into the page that will render it.

**Non-Goals:**
- Not rendering anything. No route serves blocks yet; that is `migrate-dashboard-review-to-vue`.
- Not changing the Exact/Fuzzy/Orphaned resolution ladder, the sidecar format, or the comment record.
- Not a diff or revision model. Blocks are computed from the file as it stands.

## Decisions

### Slice the source, never rebuild it

`into_offset_iter()` gives `(Event, Range<usize>)`. The block's source is `&markdown[range]` — taken from the same string the anchor will later search. Reconstructing the text from the event stream (concatenating `Event::Text` payloads, say) produces something that *looks* like the source and differs from it wherever the parser normalised anything: entity references, hard breaks, indented continuation lines. That difference is invisible until `markdown.find` returns `None` in front of a reviewer.

So the rule is mechanical: the range decides, the events only say where a block starts and ends.

### Blocks, not lines

The reviewer's instinct is GitHub's — hover a line, comment on it. GitHub can do that because a pull request diff is frozen. Nothing here is frozen; the premise of this tool is that an agent rewrites the artifact while the reviewer reads it. Insert a paragraph at the top and every line-anchored comment silently points at different text — silently, which is the failure mode `anchor.rs` was written to avoid, storing before/after context and a heading path precisely so drift is *reported*.

A block is also the only thing with a visual referent once markdown is rendered. A paragraph is one element, not N source lines.

Blocks are: paragraphs, headings, list items, table rows, and individual lines within a code fence. A list item is a block rather than the whole list, because a tasks file is a list and commenting on it as one unit is useless.

### `anchor::create` must be told where to look

This is the finding that turns an open question in the superseded change into a decision here.

`create` does `markdown.find(selected_text)` — the *first* occurrence — and then records `start_offset` and derives `heading_path_before(markdown, start_offset)` from it. `resolve_exact` later checks the slice at that stored offset still matches. So a wrong `create` is not corrected by resolution; it is confirmed by it, forever, under the wrong heading.

Two identical list items in one artifact is not an exotic case:

```
## Tests
- [ ] Add tests

## Docs
- [ ] Add tests
```

Comment on the second and the comment appears against the first, filed under the wrong heading, with no error anywhere. This is a pre-existing property of the anchor model. Select-to-comment made it rare, because a human selection usually drags in surrounding characters. Block-level commenting makes it the common shape, because the block's source is exactly the repeated string.

The block's byte range is known at the moment the comment is created and is thrown away at the API boundary. So:

```rust
pub fn create(artifact_path: &str, markdown: &str, selected_text: &str, search_from: usize)
    -> Result<Anchor, Error>
```

with `sidecar::add` taking the same parameter and passing it through. `find` becomes `markdown[search_from..].find(...)` with the offset added back.

Rejected alternatives:

- **Rely on before/after context to disambiguate.** It cannot: context is consulted by `resolve_fuzzy`, only after `resolve_exact` fails, and `resolve_exact` succeeds — against the wrong text. The record is already wrong before resolution runs.
- **Store the block range in the anchor record.** A schema change to `anchored-comments` for information the existing `start_offset` field already holds correctly, once it is set correctly.
- **Make the parameter `Option<usize>`.** Per rule 7, one path. The CLI passes `0` and means it: it has a string and no idea where in the file it is.

### Raw HTML is text

`pulldown-cmark` passes `Event::Html` through verbatim by default. The markdown reaching this parser is written by an agent, into a file, and rendered into a browser the reviewer is logged into nothing on — but "it is only localhost" is exactly the reasoning that makes a stored-markup bug survive to the day it matters. `Event::Html` and `Event::InlineHtml` are escaped as text, and a test asserts a `<script>` in an artifact does not arrive as markup.

## Risks / Trade-offs

- **The property "every block's source is findable" is asserted by a test over sample documents, not proved.** `pulldown-cmark`'s ranges have edge cases — nested lists, tables, footnotes, lazy continuation lines, setext headings. The test therefore runs over the repository's own real artifacts, not a hand-written fixture, so the sample is the same shape as the input.
- **`search_from` is only as good as the caller.** A server that computes the block's range correctly and then passes `0` reintroduces the bug with the parameter in place. The test that covers this must go through the block path end to end, not call `create` directly with a hand-written offset.
- **Blocks with no unique source still exist** if the same block appears twice and the caller genuinely does not know which. That case is now impossible from the block path and unchanged from the CLI, which is the honest split.

## Migration Plan

Nothing to migrate. Existing comment records are unaffected: `search_from: 0` reproduces today's behaviour exactly, which is what every current caller passes.

## Open Questions

None. The one the superseded change carried — range in the record versus context disambiguation — is answered above: neither, because both address the wrong step.
