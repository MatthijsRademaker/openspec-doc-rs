# On-disk state

Everything the tool writes lives under `.openspec-doc/` at the project root. Nothing is stored in a
database, and every file is readable and hand-editable.

```text
.openspec-doc/
  directives/_session/<session-id>.json    the session's directive slot; also what makes a
                                           session appear in the dashboard at all
  scratch/_session/<session-id>.md         exploration note, before promotion
  scratch/<change-name>.md                 the same note after promotion
  comments/_session/<session-id>.jsonl     append-only comment stream, session-keyed
  comments/<change-name>.jsonl             append-only comment stream, change-keyed
  verdicts/_session/<session-id>.jsonl     append-only phase-verdict stream, session-keyed
  verdicts/<change-name>.jsonl             append-only phase-verdict stream, change-keyed
  verdicts/…<scope>.translated             id of the last verdict translated into a directive
  approval/<change-name>.reported          the unapproved state last reported at a turn boundary,
                                           so the same one is not raised again
```

Session-keyed files sit one directory deeper than change-keyed ones, so a session id can never collide
with a change name. See [Scoping](/concepts/scoping.md).

:::info Not in git
This directory is gitignored. It is the back and forth that produces a change, not the change itself —
the approved outcome lives in `openspec/`. Review state is machine-local by design.
:::

## Append-only streams

Comment and verdict sidecars are JSONL and append-only. Replaying the records in order is what
reconstructs current state, and a later record never rewrites an earlier one. A comment thread is a
`Comment` event plus its `Reply` and `Status` events; a relocation is another event, which is how anchors
survive promotion. Reply events carry `"author": "reviewer"` or `"author": "agent"`. Authorless replies
from older sidecars read as agent-authored because agent-facing CLI was then the only reply writer.

A malformed line is an error, not a skipped record.

## The directive slot

```json
{
  "pending": true,
  "reason": "Review feedback from this project's openspec-doc dashboard: …",
  "createdAt": "2026-08-03T16:12:56.780405+00:00",
  "consumedAt": null
}
```

| State | `pending` | `consumedAt` | Meaning |
| --- | --- | --- | --- |
| Registered, nothing waiting | `false` | `null` | Session is discoverable; nothing to inject |
| Waiting | `true` | `null` | Will be injected at the next turn boundary |
| Consumed | `false` | timestamp | Already injected; kept as an audit trail |

The empty slot is why a session appears in the dashboard before any directive exists. The file is never
deleted — consuming it stamps it rather than removing it.

## Verdict records

```json
{"id":"74eb39be-…","verdict":"move-to-proposal","notes":"","createdAt":"2026-08-03T16:07:28.220528+00:00"}
```

The **standing** verdict for a scope is the last record in the file. The `.translated` sibling holds the
id of the verdict already turned into a directive; a verdict whose id differs from that marker is
untranslated and will be picked up at the next turn boundary.

That marker lives beside the stream rather than in it, because the stream is append-only and a translation
is not a verdict.

## Claim markers in a note

Before promotion, a claimed note carries:

```markdown
<!-- openspec-doc:claims add-widget -->
```

After promotion, the vacated session path carries a redirect instead:

```markdown
<!-- openspec-doc:moved-to .openspec-doc/scratch/add-widget.md -->

This scratch note moved to `.openspec-doc/scratch/add-widget.md`.
```

The two markers are distinct, and promotion refuses a note holding a redirect — otherwise an
already-promoted session stays eligible forever and eventually renames its own redirect onto an unrelated
change.

## Hand-editing

All of it is fair game, and the CLI is usually the better route. Two things to know:

- Writing a directive with `"pending": true` by hand will inject it at the next turn boundary. That is how
  the hook bridge was originally verified.
- Deleting a `.translated` marker makes that scope's standing verdict eligible again, which will produce a
  duplicate directive.
