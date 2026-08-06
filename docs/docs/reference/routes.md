# HTTP routes

The dashboard binds `127.0.0.1` by default. There is no authentication, because there is no remote mode —
see [what is excluded](/vision.md#what-is-deliberately-excluded).

| Route | Method | Purpose |
|---|---|---|
| `/` | GET | every discovered session and active change |
| `/sessions/<id>` | GET | the session review page |
| `/changes/<name>` | GET | the change review page |
| `…/events` | GET | server-sent events; emits `data: changed` when the scope's files change |
| `…/review` | GET | the comment-list + verdict-state fragment an open page refetches |
| `…/comments` | POST | `artifact_path`, `selected_text`, `body` |
| `…/verdict` | POST | `verdict`, `notes` |

An unknown session id or change name is a `404`, not an empty page.

A session appears here only if it has a directive record. `hook stop` writes an empty slot on first sight,
which is what makes a session discoverable — see [On-disk state](/reference/on-disk-state.md#the-directive-slot).

## What a page renders

The index lists each session and change with its **title** — the first heading of its scratch note, which is
what the exploring agent wrote there — followed by its identifier, when its artifacts were last modified, how
many of its comments are open, and its standing verdict. A session whose note has been promoted away is listed
as *Promoted to `<change>`* — the redirect left at its note path is where that comes from. A scope with neither
is listed by its identifier alone; nothing invents a name for it. The session with the most recent directive or verdict
activity is marked *most recently active*, which is a statement about timestamps on disk and not a heartbeat:
nothing here knows whether that session is still running.

Titles are display text. Every link, sidecar filename and hook payload is keyed on the identifier, so
rewriting a note's heading changes what the index says and nothing else.

A session page renders exactly one artifact: its scratch note. **If no exploration has been written, the
page has nothing to select** — the composer stays hidden and no comment can be made. That is not a bug;
it means nothing has been explored yet.

A change page renders `proposal.md`, `design.md`, `tasks.md`, each spec delta under `specs/<capability>/`,
and the promoted scratch note if there is one.

:::warning Artifacts render on full page load only
Live updates refetch the `…/review` fragment — comments and verdicts — not the artifacts. A note written
while the page is open will not appear until you reload.
:::

## Comments

```
POST /sessions/<id>/comments
  artifact_path=.openspec-doc/scratch/_session/<id>.md
  selected_text=The foo must be reconciled with the bar.
  body=Is the bar load-bearing?
```

Responds `303` back to the page on success. If `selected_text` is not in the artifact as it stands on
disk, `400` — the reviewer is told to reselect rather than having the comment anchored to a guess.

## Verdicts

```
POST /sessions/<id>/verdict
  verdict=keep-exploring
  notes=Work out whether the bar is load-bearing
```

| Verdict | Valid scope | Meaning |
|---|---|---|
| `keep-exploring` | session | stay in the explore phase; `notes` say what is still open (**required**) |
| `move-to-proposal` | session | the exploration is ready to be formalized |
| `comment-resolution` | change | address the change's open comments |

Submitting a verdict to the wrong kind of scope is a `400`, as is `keep-exploring` with empty notes — its
entire content is what remains open, so a blank one says nothing.

:::tip A verdict is the trigger; a comment is not
Comments accumulate while you are still forming a view. Only a verdict produces a directive. If you leave
comments and no verdict, the agent hears nothing.
:::

## Live updates

Each scope gets one filesystem watcher and one broadcast channel. A watcher covers the scope's own tree
(`openspec/changes/<name>/`) and the whole of `.openspec-doc/`, because comments and verdicts land in a
different tree from the artifacts they discuss. Events are filtered down to the scope's own paths, and a
burst of writes from one edit is coalesced into a single update.

When the SSE endpoint pushes, the page refetches `…/review` and swaps that one element rather than
reloading — a reviewer half-way through typing a comment does not lose it because someone else commented.

If the watcher backend fails to initialize, the scope falls back to polling every second and says so on
stderr rather than silently serving no updates.
