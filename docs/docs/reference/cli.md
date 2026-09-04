# CLI

```
openspec-doc [--root <PATH>] <COMMAND>

  summary                      print the resolved project root, its changes, and its specs
  serve [--host] [--port]      serve the dashboard; --no-open prints the URL instead of opening a browser
  serve url                    print this project's dashboard URL and whether one is serving it
  serve list                   list every dashboard on the port range, and the ports assigned to
                               a project with nothing serving them
  serve kill <--project|--port <PORT>|--all>
                               stop a running dashboard; a target is always required
  serve forget <PATH>          drop a project root's port assignment
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
openspec-doc serve list                     # every dashboard on the machine, whatever it serves
openspec-doc serve kill --project           # stop the dashboard for this project
openspec-doc serve forget ../old-checkout   # give that project's port back
```

`--host` defaults to `127.0.0.1`.

With no `--port`, `serve` binds **the port assigned to this project**: the lowest free port in `4321`–`4352`
the first time the project's canonical root is seen, recorded machine-wide in
`$XDG_STATE_HOME/openspec-doc/ports.json` (defaulting to `~/.local/state`, and the local data directory on
macOS and Windows). The same project therefore binds the same port on every later run, whatever order your
checkouts started in. If something else already holds that port, `serve` falls forward to another port in
the range and records where it landed, so the project is stable there from then on; a range with no free
port at all is an error naming the range. Set `OPENSPEC_DOC_STATE_DIR` to keep the record somewhere else.

`serve forget <PATH>` drops a root's entry from that record, which is how a port is given back — the range
is finite, an entry is spent by every project *ever opened*, and the only entries reclaimed automatically
are those whose directory no longer exists. It is refused while a dashboard for that root is serving,
because dropping it live hands the port to another project with a server still sitting on it.

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

### serve list

Every dashboard answering on the port range, whatever project it serves, beside the ports assigned to a
project with nothing serving them. Needs no project root, so it answers from anywhere — dashboards are
global to the machine and started detached by the turn-end hook, so this is a question about the machine
rather than about the checkout you happen to be standing in.

```
PORT   ROOT                          PID     STATE
4321   /Users/you/repos/acme-api     84213   running
4322   /Users/you/repos/old-spike    -       assigned, not running
4327   /Users/you/checkouts/scratch  88771   running, not its assignment
probed ports 4321-4352 on 127.0.0.1
```

Whether a dashboard is running is decided by probing that port and nothing else. An entry in the port
record says where a project's dashboard *belongs*, never that one is there, so `assigned, not running` is a
project you would find at that port if it were up. `running, not its assignment` is a dashboard the next
turn boundary will record where it actually is. `running, no assignment` is one started by hand with
`--port`. No dashboards at all prints `no dashboards running` and exits zero: that is the ordinary state of
a machine between review sessions, not a failure.

:::warning A dashboard outside `4321`–`4352` cannot be listed
The sweep is bounded, which is why the command names the range it searched. `openspec-doc serve --port 9999`
answers nothing this command asks and will not appear — the alternatives are worse: scanning all 65535
ports, or matching on a process name, which is the `pkill` guesswork this exists to replace. If a server is
missing from the table, that is where to look for it.
:::

### serve kill

Stops a running dashboard, over HTTP on the port it serves rather than by signalling its pid. **A target is
required.** There is no bare `serve kill`.

```bash
openspec-doc serve kill --project      # the dashboard for this project
openspec-doc serve kill --port 4327    # whatever `serve list` shows on that port
openspec-doc serve kill --all          # every dashboard on the range
```

| Target | The situation |
| --- | --- |
| `--project` | "stop the one for this project" — the common case. Resolves the project the way every other command does, honouring `--root`. |
| `--port <n>` | reading a `serve list` table |
| `--all` | "I have lost track, stop everything" |

`--project` is a flag you type rather than what you get for typing nothing, and that is the whole point.
Dashboards are global to the machine while your mental model is per-project, so the natural reading of a bare
`serve kill` is "the one for this project" — which is *not* the destructive reading, and that mismatch is
what would make the destructive behaviour a surprise instead of a choice. Meanwhile a bare `serve kill` that
stopped every dashboard is the command you run by accident with a colleague's review open on another
checkout. Neither is allowed to be the default, so there is no default. `--root` cannot be the project target
either: it is a global flag on every subcommand and already defaults to the resolved project, which makes it
indistinguishable from having named no target at all.

The command **enumerates immediately before it asks** rather than trusting a table you were shown some
seconds ago, **probes again afterwards**, and reports what is actually gone rather than that a request went
out:

```
4327 /Users/you/repos/acme-api: stopped
```

It exits non-zero unless every dashboard it asked to stop is gone — including when the target matched nothing
at all, because a target that matched nothing is not a stop. A dashboard the idle deadline had already reaped
reads as `was not running` rather than as a kill: the state you wanted holds either way, but the report does
not claim credit for it.

A port can change hands between the enumeration and the request: the dashboard that was there exits, and
another project's falls forward onto its port. The request names the root it believes it is stopping, and the
server refuses when that is somebody else — so what you get is a refusal naming who is actually there,
rather than a stopped dashboard belonging to a checkout you never mentioned.

Stopping a dashboard **does not touch its port assignment**, so the project comes back on the same port at
its next turn boundary. `serve forget` is how a port is given back, and it is deliberately a separate act.

There is no `--pid` and no signal of any kind. Rust's std cannot send one, `SIGTERM` does not exist on
Windows, and a pid may have been reused between learning it and acting on it — while a port that stops
answering is the evidence directly, and a server asked to stop itself can check what it is before it does.

:::warning A wedged dashboard cannot be stopped this way
`serve kill` is a conversation, so it needs a dashboard that still answers HTTP. One so wedged it no longer
answers is exactly the one you most want gone, and this command cannot touch it. That is the manual case, and
the pid in the `serve list` table is what it is for:

```bash
openspec-doc serve list
kill 84213
```

Do **not** reach for `pkill -f 'openspec-doc serve'`. It matches on a command string, and during this
project's own development it took out an unrelated shell. The pid from `serve list` is the process that
answered on that port, which is the one you meant.

No signal fallback is built into `serve kill`, because no wedged dashboard has actually been observed. If one
ever is, `--pid` is the follow-up change, justified by that observation rather than by imagining it.
:::

A dashboard the turn-end hook started exits on its own after thirty minutes with nothing subscribed and no
hook asking for it, so most of them need no stopping at all. One you started in a terminal never does.

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

## approval state

```bash
openspec-doc approval state --change add-widget
```

Prints where a change stands with the reviewer — `approved`, `stale`, or `not approved` — and the reason,
then **exits zero only when it is approved**. That exit code is the whole point: an apply workflow runs
this before it starts, and one that exited zero for a stale approval would be read as clearance.

An approval covers `proposal.md`, `design.md`, and every spec delta, and deliberately not `tasks.md`.
Editing a covered artifact after approval makes the state `stale` and names what moved; ticking a checkbox
does not, because a fingerprint over `tasks.md` would invalidate every approval within seconds of work
starting and train everyone to ignore staleness.

A change that does not exist is an error rather than an unapproved change, so a typo is not reported as a
review problem.

:::warning No command records an approval
Approval is the reviewer's act and it is made in the dashboard — `openspec-doc serve url` prints where.
There is no `approve` subcommand and no bulk `--resolve-all`, for the same reason `resolve` is not
something an agent should reach for: a single command that clears an agent's own feedback and approves its
own change is one the agent would have.
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
a directive, reports implementation that has progressed on a change with no current approval, then consumes
and emits that directive.

The approval report is after translation, so the reviewer's own feedback is never queued behind a report
about it, and it is made once per unapproved state rather than at every turn boundary — the state it last
reported is recorded in `.openspec-doc/approval/<change>.reported`.

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
