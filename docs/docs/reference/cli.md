# CLI

```
openspec-doc [--root <PATH>] <COMMAND>

  summary                      print the resolved project root, its changes, and its specs
  serve [--host] [--port]      serve the dashboard; --no-open prints the URL instead of opening a browser
  hook stop --agent <claude|pi>
  hook explore --agent <claude|pi>
  comment add|list|reply|resolve
  scratch claim --session <ID> --change <NAME>
```

`--root` skips project discovery and is accepted by every subcommand. Without it, the project root is
found by walking up from the current directory looking for `openspec/config.yaml`.

## summary

Prints the resolved root, active changes, archived changes, and specs. The quickest check that discovery
is working.

## serve

```bash
openspec-doc serve                          # binds 127.0.0.1 on a free port, opens a browser
openspec-doc serve --port 8080 --no-open    # fixed port, prints the URL instead
```

`--host` defaults to `127.0.0.1`. `--port` defaults to `0`, which selects an available port.

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

## Agent wire formats

`--agent pi` speaks a deliberately *different* format from `--agent claude`: camelCase in,
`{"action":…}` out. pi.dev has no external-process hook — its turn boundary is only reachable from a
TypeScript extension, so that format is one this project defines. Pointing one agent's flag at the other's
payload fails loudly instead of half-working.

## Error behaviour

Errors propagate and exit non-zero having emitted no decision, printing a `caused by:` chain. Neither
agent treats a non-zero hook exit as "block", so a bug ends the turn noisily rather than wedging the
session.
