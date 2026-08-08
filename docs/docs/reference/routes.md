# HTTP routes

Dashboard binds `127.0.0.1` by default. No authentication exists because no remote mode exists — see [what is excluded](/vision.md#what-is-deliberately-excluded).

Vue Router owns all three interface routes. Rust validates scope routes before serving embedded shell, supplies JSON, records mutations through core writers, and pushes filesystem changes over SSE.

| Route | Method | Purpose |
| --- | --- | --- |
| `/` | GET | embedded frontend index |
| `/sessions/<id>` | GET | embedded session review workbench |
| `/changes/<name>` | GET | embedded change review workbench |
| `/api/index` | GET | all discovered sessions and active changes |
| `/api/sessions/<id>` | GET | session artifacts as blocks, comments, counts, verdict history, delivery state |
| `/api/changes/<name>` | GET | change artifacts as blocks, comments, counts, verdict history, delivery state |
| `/api/<scope>/<key>/events` | GET | server-sent JSON events with `artifactsChanged` and `reviewStateChanged` flags |
| `/api/<scope>/<key>/comments` | POST | create anchored or unanchored comment JSON |
| `/api/<scope>/<key>/comments/<comment-id>/replies` | POST | append reply JSON |
| `/api/<scope>/<key>/comments/<comment-id>/status` | POST | reviewer transition to `open` or `resolved` |
| `/api/<scope>/<key>/verdict` | POST | submit scope-appropriate verdict JSON |

Unknown scope data returns `404`, never empty scope. Unknown interface scope also returns `404`; shell is not generic fallback. Embedded assets remain available at their built paths.

Session exists when directive record exists. `hook stop` writes empty slot on first sight, making session discoverable — see [On-disk state](/reference/on-disk-state.md#the-directive-slot).

## Scope detail

Each scope response contains:

- exact scope kind, key, and optional title
- artifacts in reading order, each decomposed into rendered blocks carrying exact source and byte range
- reconstructed comment threads with status, replies, resolved anchor state, and resolved block id
- open, addressed, and resolved counts
- verdict history, standing verdict, and whether standing directive is pending or delivered

Session has one possible artifact: scratch note. Session with no note is valid and returns empty artifact list. Change returns present `proposal.md`, `design.md`, `tasks.md`, spec deltas, and promoted scratch note in reading order.

Artifacts live-update when an SSE event reports `artifactsChanged`. A clean page applies the new rendered blocks and preserves scroll position; a page with unsent composer text keeps its current artifact and reports the pending refresh until that composer is sent or dismissed. Review-state updates never wait.

## Comments

Block comment names exact source position, avoiding first-occurrence ambiguity:

```json
{
  "kind": "anchored",
  "artifactPath": "openspec/changes/add-widget/proposal.md",
  "selectedText": "Repeated requirement.",
  "searchFrom": 412,
  "body": "This second occurrence needs evidence."
}
```

Free-text selection uses same shape. Selection absent from current source returns `400` JSON with refusal reason. Scope-level comment uses:

```json
{ "kind": "unanchored", "body": "Reconcile terminology across artifacts." }
```

Reply body is `{ "body": "…" }`. Server persists and returns dashboard replies with `"author": "reviewer"`; agent-facing CLI replies carry `"author": "agent"`, and older authorless replies read as agent-authored. Reviewer status body is `{ "status": "resolved" }` or `{ "status": "open" }`. Dashboard exposes no `addressed` transition because addressed is agent claim, not reviewer judgement.

## Verdicts

Verdict body has no notes field:

```json
{ "verdict": "keep-exploring" }
```

| Verdict | Valid scope | Meaning |
| --- | --- | --- |
| `keep-exploring` | session | keep exploration open; feedback lives in comments |
| `move-to-proposal` | session | formalize exploration |
| `comment-resolution` | change | address change comments |

Composer text is first recorded as unanchored comment, then verdict submitted. Empty composer submits verdict only. Wrong-scope verdict returns `400`.

Comments accumulate without notifying agent. Verdict triggers directive, and directive points agent at comment sidecar.

## Live updates

Each scope gets filesystem watcher and broadcast channel. Watcher covers scope tree plus `.openspec-doc/`, classifies artifact and review-sidecar paths, and coalesces write bursts into one JSON event. Vue client refetches scope detail after event: replies, statuses, counts, verdict delivery, and clean artifact pages reconcile without reload. Watcher initialization failure falls back to one-second polling and logs fallback.
