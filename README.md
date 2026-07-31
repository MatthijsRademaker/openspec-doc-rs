# openspec-doc

A local review dashboard for [OpenSpec](https://github.com/Fission-AI/OpenSpec) projects, plus an agent
hook bridge. You read a coding agent's proposal or exploration notes in a browser, select text, leave
anchored comments, and submit a verdict; the hook bridge feeds that verdict back into the agent at its
next turn boundary.

Rust rewrite of a TypeScript prototype, which is kept under `openspec-doc-rs-example/` for reference.

> **Status: MVP in progress.** The dashboard renders artifacts, records comments and verdicts, and
> pushes live updates. The hook bridge blocks and reinjects a *hand-written* directive. The wiring
> between the two — turning a dashboard verdict into a directive the agent consumes — is not built
> yet (`openspec/changes/add-directive-verdict-loop`). See [Capability status](#capability-status).

## Quickstart

See **[QUICKSTART.md](QUICKSTART.md)** for a copy-pasteable throwaway project and a walkthrough of
every review interaction, including the two-tab live-update check.

```bash
cargo build
cargo run -- serve              # discovers the project root by walking up from the cwd
```

## Layout

```
crates/core     project discovery, comments, scratch notes, verdicts, hook bridge   (no I/O framework)
crates/server   the axum dashboard: routes, pages, filesystem watcher, SSE
crates/cli      the openspec-doc binary
```

`core` holds every rule and every file format. `server` and `cli` are two front ends over it and
contain no persistence logic of their own.

## Build and test

```bash
cargo test --workspace      # 135 tests, all hermetic (each builds its own temp project)
cargo clippy --workspace --all-targets
cargo fmt --all --check
```

No test needs a network, a browser, or an installed `openspec` binary — except
`scratch::promote`'s validation tests, which shell out to `openspec validate` and will fail if
`openspec` is not on `PATH`.

## CLI

```
openspec-doc [--root <PATH>] <COMMAND>

  summary                    print the resolved project root, its changes, and its specs
  serve [--host] [--port]    serve the dashboard; --no-open prints the URL instead of opening a browser
  comment add|list|reply|resolve
  hook stop --agent <claude|pi>
```

`--root` skips project discovery, which is what the test walkthroughs use to point at a throwaway
project. Every subcommand accepts it.

`comment` operates on the same sidecars the dashboard writes, so `comment list --change <name>` is the
quickest way to confirm what a browser interaction actually recorded:

```bash
openspec-doc comment add --change add-widget \
  --artifact openspec/changes/add-widget/proposal.md \
  --selected-text 'Widgets are slow today.' \
  --body 'Slow by what measure?'
openspec-doc comment list --change add-widget
```

## On-disk state

Everything the tool writes lives under `.openspec-doc/` at the project root. Nothing is stored in a
database, and every file is readable and hand-editable.

```
.openspec-doc/
  directives/_session/<session_id>.json   pending directive for a session; also what makes a
                                          session appear in the dashboard at all
  scratch/_session/<session_id>.md        exploration notes, before a change directory exists
  scratch/<change_name>.md                the same note after promotion
  comments/_session/<session_id>.jsonl    append-only comment stream, session-keyed
  comments/<change_name>.jsonl            append-only comment stream, change-keyed
  verdicts/_session/<session_id>.jsonl    append-only phase-verdict stream, session-keyed
  verdicts/<change_name>.jsonl            append-only phase-verdict stream, change-keyed
  scratch-snapshots/                      per-session active-change snapshots, for promotion detection
```

Session-keyed files sit one directory deeper than change-keyed ones so a session id can never collide
with a change name. Comment and verdict sidecars are append-only: replaying them in order is what
reconstructs current state, and a later record never rewrites an earlier one.

### Scoping

Every route and every sidecar is keyed one of two ways, and this is the single most load-bearing idea
in the codebase:

- **Session-keyed** while an exploration is still pre-proposal. There is no change directory yet, so
  the only artifact is a scratch note.
- **Change-keyed** once the exploration has been formalized into `openspec/changes/<name>/`.

Promotion renames the note and relocates its comment sidecar in lockstep, recording a relocation event
so anchors made against the old path keep resolving.

## Routes

| Route | Method | Purpose |
|---|---|---|
| `/` | GET | every discovered session and active change |
| `/sessions/<id>` `/changes/<name>` | GET | the review page |
| `…/events` | GET | server-sent events; emits `data: changed` when the scope's files change |
| `…/review` | GET | the comment-list + verdict-state fragment an open page refetches |
| `…/comments` | POST | `artifact_path`, `selected_text`, `body` |
| `…/verdict` | POST | `verdict`, `notes` |

An unknown session id or change name is a 404, not an empty page.

### Verdicts

| Verdict | Valid scope | Meaning |
|---|---|---|
| `keep-exploring` | session | stay in the explore phase; `notes` say what is still open (required) |
| `move-to-proposal` | session | the exploration is ready to be formalized |
| `comment-resolution` | change | address the change's open comments |

Submitting a verdict to the wrong kind of scope is a 400, as is `keep-exploring` with empty notes.

## How anchoring works

Artifacts are rendered as their **own markdown source** in a `<pre>`, not converted to HTML. A browser
selection is therefore byte-for-byte a substring of the file, so there is no mapping from rendered DOM
ranges back onto source offsets to get wrong.

The client sends only `selection.toString()`. The server re-finds that text in the file as it stands on
disk and builds the anchor itself — recording the selected text, its enclosing heading path, 80 bytes of
context either side, and byte offsets. A selection that is no longer in the file is refused with a 400
telling the reviewer to reselect, rather than anchored to a guess.

When a comment is read back, its anchor is re-resolved against the current file and reported honestly:

| State | Meaning |
|---|---|
| `exact` | the recorded offset still holds the selected text |
| `fuzzy` | found elsewhere, via heading path or surrounding context |
| `orphaned` | the file is there, but nothing in it matches |
| `missing` | the file is gone |

The page colours a comment's left border by this state. A comment whose text was rewritten out from
under it is shown as such rather than presented as confidently placed.

## Live updates

Each scope gets one filesystem watcher and one broadcast channel. A watcher covers both the scope's
own tree (`openspec/changes/<name>/`) and the whole of `.openspec-doc/`, because comments and verdicts
land in a different tree from the artifacts they discuss. Events are filtered down to the scope's own
paths, and a burst of writes from one edit is coalesced into one update.

When the SSE endpoint pushes, the page refetches `…/review` and swaps that one element — it does not
reload. A reviewer half-way through typing a comment does not lose it because someone else commented.

If the watcher backend fails to initialize, the scope falls back to polling every second and says so on
stderr rather than silently serving no updates.

## Agent hook bridge

`openspec-doc hook stop --agent <claude|pi>` reads the agent's turn-end payload on stdin and prints
that agent's stop decision on stdout. If the session has a pending directive, the turn is blocked and
the directive's reason is reinjected; the directive is marked consumed first, so it cannot fire twice.

```bash
echo '{"session_id":"abc","transcript_path":"/tmp/t.jsonl","cwd":"'"$PWD"'"}' \
  | openspec-doc hook stop --agent claude
```

With no pending directive this prints `{"continue":true}`. With one, `{"decision":"block","reason":…}`.

`--agent pi` speaks a deliberately *different* wire format (camelCase in, `{"action":…}` out), because
pi.dev has no external-process hook — its turn boundary is only reachable from a TypeScript extension.
Pointing one agent's flag at the other's payload fails loudly instead of half-working.

Errors propagate and exit non-zero having emitted no decision. Neither agent treats that as "block",
so a bug here ends the turn noisily rather than wedging the session.

## Capability status

| Capability | State |
|---|---|
| `project-scanner` | done |
| `cli-surface` | done |
| `agent-hook-bridge` | done — consumes hand-written directives |
| `anchored-comments` | done |
| `scratch-note-workflow` | done — promotion is a library call, not yet invoked from the hook |
| `dashboard-server` | done |
| `dashboard-html-views` | done except the two-tab browser check (`tasks.md` 4.2) |
| `directive-verdict-loop` | **not started** — nothing turns a verdict into a directive yet |

Until `directive-verdict-loop` lands, a verdict submitted in the dashboard is recorded and visible but
has no effect on any agent. Write a directive file by hand to exercise the hook end of the loop.

## Conventions

`AGENTS.md` holds the working rules for this repo. The ones that show up most in the code:

- Invalid states fail loudly. No silent defaults, swallowed errors, or fake success values.
- Errors propagate with their source chain intact; the CLI prints `caused by:` lines.
- Prefer small feature-named files over large ones.
- Prefer executable truth in `src/` when docs disagree.
