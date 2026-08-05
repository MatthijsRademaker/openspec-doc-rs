# Anchoring

A comment is attached to a span of markdown, not to a line number. The problem is that the file keeps
changing underneath it — the agent is still writing — so an anchor has to survive edits and, when it
cannot, say so instead of pretending.

## Artifacts render as their own source

Markdown is shown verbatim inside a `<pre>`, not converted to HTML.

That one decision removes an entire class of bug. A browser selection is byte-for-byte a substring of the
file, so there is no mapping from rendered DOM ranges back onto source offsets to get wrong. The cost is
that the page shows raw markdown; the benefit is that a selection cannot be subtly misplaced.

## The client sends only the selected text

The page posts `selection.toString()` and nothing else. The **server** re-finds that text in the file as
it stands on disk and builds the anchor itself, recording:

- the selected text
- its enclosing heading path
- 80 bytes of context either side
- byte offsets

A selection no longer present in the file is refused with a `400` telling the reviewer to reselect. It is
never anchored to a guess.

This ordering matters: the client cannot be trusted about offsets, because the file may have changed
between page load and submission. The server reads the file at the moment of the write.

## Drift is reported, not hidden

When a comment is read back its anchor is re-resolved against the current file:

| State | Meaning |
|---|---|
| `exact` | the recorded offset still holds the selected text |
| `fuzzy` | found elsewhere, via heading path or surrounding context |
| `orphaned` | the file is there, but nothing in it matches |
| `missing` | the file is gone |

The page colours a comment's left border by state — amber for `fuzzy`, red for `orphaned` and `missing`.
A comment whose text was rewritten out from under it is shown as such rather than presented as
confidently placed.

`fuzzy` is the normal case in practice, not a failure. A comment anchored to an exploration note that
then doubles in length will report `fuzzy at offset <n>` — the anchor found its text, just not where it
was left. That is the honest answer, and it is more useful than either silently re-pointing or dropping
the comment.

## Anchors survive promotion

Promotion relocates a comment sidecar by appending a relocation event, so anchors recorded against
`scratch/_session/<id>.md` continue to resolve against `scratch/<change>.md`. See
[Scoping](/concepts/scoping.md).
