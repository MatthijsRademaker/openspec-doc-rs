# Anchoring

Comment attaches to markdown span, not line number. File keeps changing underneath it, so anchor must survive edits and report when it cannot.

## Rendered blocks keep exact source

Dashboard renders formatted markdown decomposed into commentable blocks. Every block carries two forms of same parser range:

- sanitized HTML shown in browser
- exact source bytes plus byte range from artifact

Source is sliced, never reconstructed from parser events. Reviewer gets readable document while server still anchors against bytes actually on disk.

## Block comments name occurrence

Block action submits block source and range start. Server re-reads artifact and searches from that byte position. If same text appears twice, comment on second block anchors to second occurrence rather than first.

Free-text selection remains secondary path. Client submits rendered selection plus containing block's start position. If selection crosses inline markup and rendered text does not occur in markdown source, server returns `400` with refusal reason. Client shows that reason; no guessed anchor is written.

Server records:

- selected source text
- enclosing heading path
- 80 bytes context on either side
- byte offsets

File may change between page load and submission, so server always validates current file before writing.

## Drift is reported, not hidden

When comment reads back, anchor resolves against current artifact:

| State | Meaning |
| --- | --- |
| `exact` | recorded offset still holds selected text |
| `fuzzy` | text or context found elsewhere |
| `orphaned` | artifact exists but no landmark matches |
| `missing` | artifact is gone |
| `unanchored` | comment intentionally targets whole scope |

Exact and fuzzy comments stay beside block they resolve to; fuzzy marker says anchor moved. Orphaned and missing comments remain reachable with original quote in comments-without-block panel. Unanchored comments share that panel but are not mislabeled lost.

`fuzzy` is normal after surrounding edits. It reports relocation rather than silently pretending recorded offset remains exact.

## Anchors survive promotion

Promotion relocates comment sidecar by appending relocation event, so anchors recorded against `scratch/_session/<id>.md` resolve against `scratch/<change>.md`. See [Scoping](/concepts/scoping.md).
