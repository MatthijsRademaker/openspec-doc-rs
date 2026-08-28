# CLI

```
openspec-doc [--root <PATH>] <COMMAND>

  summary                      print the resolved project root, its changes, and its specs
  serve [--host] [--port]      serve the dashboard; --no-open prints the URL instead of opening a browser
  serve url                    print this project's dashboard URL and whether one is serving it
  hook stop --agent <claude|pi>
  hook explore --agent <claude|pi>
  comment add|list|reply|resolve
  scratch claim --session <ID> --change <NAME>
  init [--agent <claude|pi>] [--yes] [--skip-instructions]
  doctor                       run the configured hooks and report whether they work
```

`--root` skips project discovery and is accepted by every subcommand. Without it, the project root is
found by walking up from the current directory looking for `openspec/config.yaml`.

## summary

Prints the resolved root, active changes, archived changes, and specs. The quickest check that discovery
is working.

## serve

```bash
openspec-doc serve                          # binds this project's assigned port, opens a browser
openspec-doc serve --port 8080 --no-open    # exactly that port, prints the URL instead
openspec-doc serve url                      # the URL, and whether a dashboard is serving it
```

`--host` defaults to `127.0.0.1`.

With no `--port`, `serve` binds **the port assigned to this project**: the lowest free port in `4321`–`4352`
the first time the project's canonical root is seen, recorded machine-wide in
`$XDG_STATE_HOME/openspec-doc/ports.json` (defaulting to `~/.local/state`, and the local data directory on
macOS and Windows). The same project therefore binds the same port on every later run, whatever order your
checkouts started in. If something else already holds that port, `serve` falls forward to another port in
the range and records where it landed, so the project is stable there from then on; a range with no free
port at all is an error naming the range. Set `OPENSPEC_DOC_STATE_DIR` to keep the record somewhere else.

An explicit `--port` neither reads nor writes that record: it binds exactly the port you named and fails
rather than binding a different one. Naming a port is a statement about this invocation, not about where
the project lives.

`--idle-exit` makes the server exit after thirty minutes with no page subscribed to it *and* no external
activity registered against it. The turn-end hook passes it; nothing else does, because a server a person
started in a terminal must not vanish under them.

### serve url

Prints the project's dashboard URL and whether a dashboard is currently serving it — correctly when
nothing is running, which is what the port assignment exists to make possible. A dashboard found on some
other port in the range is reported where it actually is.

```
http://127.0.0.1:4323 (not running)
http://127.0.0.1:4323 (serving)
```

## comment

Operates on the same sidecars the dashboard writes, so `comment list` is the quickest way to confirm what
a browser interaction actually recorded.

```bash
openspec-doc comment add --change add-widget \
  --artifact openspec/changes/add-widget/proposal.md \
  --selected-text 'Widgets are slow today.' \
  --body 'Slow by what measure?'

openspec-doc comment list --change add-widget
openspec-doc comment reply --change add-widget --comment <id> --body 'Measured: 400ms p99.'
openspec-doc comment resolve --change add-widget --comment <id>
```

Exactly one of `--change` or `--session` is required; supplying both or neither is an error.

`--selected-text` must appear in the artifact as it stands on disk. If it does not, the command fails and
writes nothing — see [Anchoring](/concepts/anchoring.md).

:::warning `resolve` is the reviewer's act
An agent addressing feedback should `reply`, never `resolve`. Resolving records the reviewer accepting the
work; an agent that closes its own threads has ended the review loop rather than completed a pass of it.
:::

## scratch claim

```bash
openspec-doc scratch claim --session <id> --change <name>
```

Records which change a session's exploration became, so the next turn boundary promotes its scratch note
onto that change. Writes a claim marker into the session's note, preserving whatever the note already
contains; re-claiming replaces the previous claim.

Fails if the session has no note — there is nothing to claim with. Promotion acts on this and nothing
else, so a session that never claims is never promoted. See [Scoping](/concepts/scoping.md).

## hook stop

Reads the agent's turn-end payload on stdin, prints that agent's stop decision on stdout.

```bash
echo '{"session_id":"abc","transcript_path":"/tmp/t.jsonl","cwd":"'"$PWD"'"}' \
  | openspec-doc hook stop --agent claude
```

With nothing outstanding: `{"continue":true}`. With a directive: `{"decision":"block","reason":…}`.

In order, it: registers the session if new, runs the promotion check, translates any standing verdict into
a directive, then consumes and emits that directive.

## hook explore

Reads a payload on stdin, readies the session's note directory, and prints the instruction that tells the
agent where to write. Only `session_id` is read, so it accepts any of the agent's event shapes.

Its **stdout is the payload** — for the events it is wired to, a hook's stdout is added to the model's
context. See [Agent hooks](/reference/hooks.md).

## doctor

Runs each hook registered in `.claude/settings.json` and reports what came back, one line per check,
exiting non-zero if any failed. `init` answers *is this written down correctly*; `doctor` answers *does it
run* — a perfect settings file for a binary that is not on the agent's `PATH` is exactly as silent as a
healthy one.

```bash
openspec-doc doctor
```

What it does, and why each part is the way it is:

- **It executes the command string recorded in the settings file**, verbatim, through `sh -c`. Not the
  canonical one: an entry pointing at a local build is what the agent runs, so that is what gets probed.
- **Every probe runs against a throwaway project** in a temporary directory, never this one. `hook stop`
  consumes a pending directive, so a probe against the real project would eat the reviewer's outstanding
  feedback — `doctor` would cause the failure it exists to diagnose. An entry whose command names its own
  `--root` is therefore reported as unchecked rather than run.
- **Each hook must answer with the thing that hook exists to say**: a decision payload from `Stop`, a note
  location from `UserPromptExpansion`, and — because `hook prompt` exits zero and prints nothing on failure
  by design — the text of a directive seeded in the throwaway project for `UserPromptSubmit`. Silence
  cannot be the pass condition for a hook built to fail silently.
- **It reports the binary on `PATH` against the running one** and fails when they differ. The agent runs
  whichever `PATH` resolves; a report about the other one is answering the wrong question.
- **Anything it does not examine gets a line saying so**: pi.dev's extension, whether a dashboard is
  serving this project, and the `UserPromptExpansion` matcher, which executing the command bypasses. A
  green line for something never looked at is what this command exists to remove.

`openspec-doc init` is the fixer. `doctor` names what is broken and never writes to the project.

## Agent wire formats

`--agent pi` speaks a deliberately *different* format from `--agent claude`: camelCase in,
`{"action":…}` out. pi.dev has no external-process hook — its turn boundary is only reachable from a
TypeScript extension, so that format is one this project defines. Pointing one agent's flag at the other's
payload fails loudly instead of half-working.

## Error behaviour

Errors propagate and exit non-zero having emitted no decision, printing a `caused by:` chain. Neither
agent treats a non-zero hook exit as "block", so a bug ends the turn noisily rather than wedging the
session.
