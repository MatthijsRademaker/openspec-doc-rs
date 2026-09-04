# HTTP routes

Dashboard binds `127.0.0.1` by default. No authentication exists because no remote mode exists — see [what is excluded](/vision.md#what-is-deliberately-excluded).

Vue Router owns all three interface routes. Rust validates scope routes before serving embedded shell, supplies JSON, records mutations through core writers, and pushes filesystem changes over SSE.

| Route | Method | Purpose |
| --- | --- | --- |
| `/` | GET | embedded frontend index |
| `/sessions/<id>` | GET | embedded session review workbench |
| `/changes/<name>` | GET | embedded change review workbench |
| `/api/identity` | GET | the canonical project root this dashboard serves, and the pid serving it |
| `/api/shutdown` | POST | begin this dashboard's graceful shutdown |
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

## Identity and shutdown

Two routes are about the process rather than about the review. Both are the interface `serve url`,
`serve list`, and `serve kill` are built on, which is why they are documented rather than treated as
internal — a local HTTP route that stops the process is not an implementation detail.

`GET /api/identity` returns `{"root": "<canonical project root>", "pid": <n>}`. This is how a dashboard is
discovered: whether one is serving a project is decided by probing the port range and comparing canonical
roots, never by reading a file. The turn-end hook has to ask this anyway to decide whether to start a
dashboard, so the probe doubles as the heartbeat the idle deadline watches — see
[`serve list`](/reference/cli.md#serve-list).

`POST /api/shutdown` begins the graceful shutdown, resolving the same future the idle exit resolves. It has
three conditions, and each one is load-bearing:

- **`POST` only.** A `GET` returns `405`, so no link, prefetch, or address bar reaches it.
- **`X-OpenSpec-Doc-Root: <canonical root>` is required**, and must name the root this dashboard actually
  serves. A missing header returns `400`; a root belonging to somebody else returns `409` and the dashboard
  keeps serving. Naming the root is what closes the case where the port changed hands between a caller's
  enumeration and its request: the dashboard is the only participant that knows for certain what it is, and
  asking it is only possible because the mechanism is a conversation rather than a signal. Requiring a header
  no cross-origin HTML form can set is also what stops a page in your own browser from posting a dashboard
  down on a guessable local port — it forces a CORS preflight that will not pass. There is no token and no
  nonce: a secret needs somewhere to live, and per-server state is what this design rejects.
- **The answer comes before the exit**, as `202 Accepted`. A caller handed a dropped connection cannot tell
  a shutdown that started from a request that never arrived.

Open event streams end when a shutdown begins, rather than holding the drain open forever. A page watching a
dashboard that is stopped sees its stream close, which is the honest report; without it, graceful shutdown
would wait on a response that never finishes.

A dashboard bound outside `4321`–`4352` is unreachable by any of this, because the sweep these commands run
is bounded. See [the warning under `serve list`](/reference/cli.md#serve-list).

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
| `approval-withdrawn` | change | a recorded approval no longer stands |

`approved` is not accepted here and returns `400`: an approval carries the fingerprint of the artifacts it approves, so it is recorded through the approval route below. Composer text is first recorded as unanchored comment, then verdict submitted. Empty composer submits verdict only. Wrong-scope verdict returns `400`.

Comments accumulate without notifying agent. Verdict triggers directive, and directive points agent at comment sidecar.

## Approval

`POST /api/changes/<name>/approval` — the reviewer's three acts on a change's approval, one route, tagged by `act`:

```json
{ "act": "approve" }
{ "act": "resolve-all-and-approve" }
{ "act": "withdraw" }
```

Response is `{ "resolved": <count>, "approval": { "state": …, "reason": …, "changedArtifacts": [] } }`. `state` is `approved`, `stale`, or `not-approved`, and the same object appears on `GET /api/changes/<name>` as `approval` (null for a session, which is an exploration and has nothing to approve).

`approve` is refused with `400` and its reason while any comment is `open` or `addressed`. `resolve-all-and-approve` resolves every comment not already `resolved` and then approves, reporting how many it moved; the two halves are not atomic, and a failure after the resolutions land returns `400` naming both — the comments are resolved and the change is not approved, and nothing compensating is written. `withdraw` is refused when there is no approval standing.

Sweeping `addressed` comments here does not contradict the per-comment route exposing no `addressed` transition. That rule is about *who* may record acceptance, not about how many clicks it takes: `addressed` is the agent's claim, and both routes leave the move to `resolved` with the reviewer. What this route adds is doing it once instead of once per thread, which is also why it is one request — a client firing N status posts and then an approve leaves a change neither settled nor approved when it fails halfway, and emits one live update per comment for a single intent.

There is deliberately **no command-line counterpart** to the bulk act. Per-comment `openspec-doc comment resolve` already serves a reviewer at a terminal; a `--resolve-all` flag would hand the agent one command that clears its own feedback and approves its own change.

## Live updates

Each scope gets filesystem watcher and broadcast channel. Watcher covers scope tree plus `.openspec-doc/`, classifies artifact and review-sidecar paths, and coalesces write bursts into one JSON event. Vue client refetches scope detail after event: replies, statuses, counts, verdict delivery, and clean artifact pages reconcile without reload. Watcher initialization failure falls back to one-second polling and logs fallback.
