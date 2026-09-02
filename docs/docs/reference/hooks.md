# Agent hooks

Three hooks for Claude Code, two for pi.dev. Without them the dashboard still renders whatever is on disk,
but nothing reaches the agent and no exploration note gets started.

| Hook | Event | Purpose |
|---|---|---|
| `hook explore` | command expansion | ready the note location, tell the agent where to write |
| `hook prompt` | prompt submission | deliver any standing verdict as context for the turn the prompt starts |
| `hook stop` | turn boundary | register the session, run promotion, inject any standing verdict |

`hook prompt` and `hook stop` are two delivery points for the same directive, and it is consumed exactly
once across them. Prompt-time is the usual path — feedback arrives at the *start* of the turn it belongs to,
so a proposal accounts for the reviewer's comments instead of being revised after the fact. The Stop path is
what stops the agent going idle while feedback is outstanding, and the only path that works when no prompt
is coming.

pi.dev has no command-expansion event, so it gets `prompt` and `stop` only; start an exploration there by
writing the note yourself at `.openspec-doc/scratch/_session/$PI_SESSION_ID.md`.

Wire all of this with `openspec-doc init`, which detects the harnesses a project uses and writes the
entries below, pi's extension, and a block of standing instructions into `AGENTS.md`. It prints its plan
and writes nothing without `--yes`. Then run `openspec-doc doctor`, which executes what `init` wrote:
a settings file is a claim about what will happen, and `doctor` is what turns it into an observation. See
[Quickstart](/quickstart.md).

## The dashboard the hooks start

`hook stop` brings a dashboard up at the first turn boundary of a session that has something to review, on
[the port assigned to this project](/reference/cli.md#serve), and keeps it up at every later one. Nobody has
to remember to start one, which also means a machine accumulates dashboards nobody chose to start.

`openspec-doc serve list` is what enumerates them, from anywhere, with the pid of each. It works by probing
`4321`–`4352` and **nothing outside that range answers it** — a server started by hand as
`openspec-doc serve --port 9999` will not appear, however much it is running. That is the first thing to
check when a dashboard you know is up is missing from the table.

To stop one, take the pid from `serve list` and `kill` it; a hook-started dashboard also exits by itself
after thirty minutes with no page subscribed and no turn boundary asking for it. Do not `pkill -f` on the
command string — it has already killed the wrong process during this project's development.

If a start fails, that server's output is in `.openspec-doc/serve.log` and the hook says so on stderr.

## Claude Code

`.claude/settings.json` at the project root — the file `init` writes, and the file to edit if you are
wiring it by hand. Committed: every character of it is a fact about how the project is reviewed,
identical on every machine that checks the project out, so a per-machine file is the wrong place for it.

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "openspec-doc hook stop --agent claude",
            "timeout": 30,
            "statusMessage": "Checking openspec-doc review feedback"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "openspec-doc hook prompt --agent claude",
            "timeout": 30,
            "statusMessage": "Checking openspec-doc review feedback"
          }
        ]
      }
    ],
    "UserPromptExpansion": [
      {
        "matcher": "opsx:explore|openspec-explore",
        "hooks": [
          {
            "type": "command",
            "command": "openspec-doc hook explore --agent claude",
            "timeout": 30,
            "statusMessage": "Opening the openspec-doc exploration note"
          }
        ]
      }
    ]
  }
}
```

:::danger The matcher is the bare command name
`UserPromptExpansion` matches against `command_name`, which has **no leading slash and no namespace** —
typing `/openspec-explore` produces `command_name: "openspec-explore"`.

A matcher that does not match **fails silently**: the hook never runs, no note is readied, and the session
page stays empty. That is indistinguishable from the bug this hook exists to fix, so verify it fires rather
than assuming.
:::

`openspec-doc doctor` runs each registered hook and reports what came back, which settles everything about
the wiring except this: it executes the command directly, and executing a command bypasses matcher
dispatch. A matcher that matches nothing still probes clean.

The matcher is only confirmable from a live session. Type the command once and check that the note was
readied:

```bash
ls .openspec-doc/scratch/_session/
```

Nothing there means the hook never fired. `openspec-doc init` writes the matcher from the same definition
the binary carries, so the fix is to re-run it rather than to edit the value by hand.

After changing settings, open `/hooks` once or restart — the config watcher only watches directories that
had a settings file when the session started.

:::danger `hook prompt` must never exit non-zero
Verified by observation on Claude Code 2.1.223: a `UserPromptSubmit` hook exiting **1** lets the prompt
through with a warning, but exiting **2** refuses it outright — the model never sees what you typed, only
`UserPromptSubmit operation blocked by hook`.

This hook sits in front of your own input, so that failure is strictly worse than the delayed feedback it
exists to fix. `hook prompt` therefore reports every failure on stderr and exits zero, deliberately unlike
`hook stop`. It is the one place in the project that does not propagate an error, and the fail-soft is
load-bearing rather than defensive.

**Known hole:** argument parsing and project-root resolution happen before dispatch, so they bypass the
fail-soft. A clap usage error — a typo'd flag, a missing `--agent` — exits **2** and therefore blocks every
prompt in the session. Paste the command string rather than typing it until this is fixed.
:::

## pi.dev

pi has no external-process hook; neither of its delivery points is reachable except from a TypeScript
extension. One copy of it is embedded in the binary; `openspec-doc init --agent pi` writes it to
`.pi/extensions/openspec-doc-hook.ts`, where pi auto-discovers it once the project is trusted. This
repository's own copy is that same output. It handles both delivery points:

- `before_agent_start` — fires after the prompt is submitted and before the agent loop, pi's equivalent of
  `UserPromptSubmit`. Shells out to `openspec-doc hook prompt --agent pi` and returns any directive as a
  `custom` message, which lands in the conversation before the model runs.
- `agent_end` — shells out to `openspec-doc hook stop --agent pi` and re-injects any returned directive with
  `pi.sendUserMessage(…, { deliverAs: "followUp" })`.

`init --agent pi` also reports what pi does **not** get: there is no command-expansion event, so nothing
calls `hook explore`, so a pi session writes no note and is never registered for review.

Set `OPENSPEC_DOC_BIN` to use a binary that is not on `PATH`, such as this repo's own `target/debug/openspec-doc`.

### Verification status

Proven end to end in a real interactive pi session: the directive was found at the turn boundary, injected
as a user message, and the agent started a fresh turn acting on it. A real captured payload from the
extension is the `PI_STOP` fixture in `crates/core/src/hook/adapter.rs`.

Also proven from a single scripted `pi -p` run, against a verdict rather than a hand-written directive: the
agent seeds a scratch note and a keep-exploring verdict for its own session via `$PI_SESSION_ID`, and at
`agent_end` the hook translates that verdict, the directive comes back, and the agent answers the reviewer's
question instead of the prompt it was given. No session resume is involved — injection happens in-process
through `sendUserMessage`, so one invocation covers the whole loop.

Two quirks that will cost you time if you script against it, both tied to the session flags:

- `--session-id` for a session that does not exist yet, and `--continue`, both perform a session
  *replacement*. Touching the captured `pi`/`ctx` afterwards throws `extension ctx is stale after session
  replacement`. This is why the handler sits on `agent_end` rather than `agent_settled`.
- An `agent_end` handler combined with `--session-id` for a session that does **not exist yet** hangs pi
  before any output, handler never firing. Cause unknown. Re-entering a session that already exists with
  `--session-id` does not hang (checked on pi 0.83.0), which is what makes the prompt-time recipe below
  possible.

To exercise the **stop** path headlessly: launch `pi -p` with **neither** flag and have the agent seed its
own session state via `$PI_SESSION_ID`. Injection happens in-process through `sendUserMessage`, so a single
invocation covers the whole loop — no session resume needed.

To exercise the **prompt** path you need the verdict standing before the prompt, so the session must already
exist. Run `pi -p` once to create one, take its id from the directive record `hook stop` just wrote under
`.openspec-doc/directives/_session/`, write the verdict for that id, then re-enter with
`pi -p --session-id <id>`. Prompt-time delivery is proven when the agent's *first* reply answers the
reviewer rather than the prompt you gave it.

## Wire formats

Deliberately different per agent, so pointing one flag at the other's payload fails loudly rather than
half-working.

| | Claude Code | pi.dev |
|---|---|---|
| In | `session_id`, snake_case | `sessionId`, camelCase |
| `stop` out (allow) | `{"continue":true}` | `{"action":"stop"}` |
| `stop` out (block) | `{"decision":"block","reason":…}` | `{"action":"continue","message":…}` |
| `prompt` out | the directive text, plain on stdout | the directive text, plain on stdout |
| `prompt` out (nothing standing) | empty | empty |

`hook prompt` is the exception to the per-agent split: its **output** is plain text either way, because it
carries context rather than a decision. Only the stdin shape differs, and it reads nothing but the session
id. Claude Code adds that stdout to the model's context itself; for pi the extension reads it and returns it
as a `custom` message, so the wrapping happens on pi's side of the bridge rather than in the binary.

`hook explore` likewise reads only `session_id`, so it accepts any of an agent's event shapes rather than
one specific event.

## Where output goes

- `hook stop` writes its **decision to stdout** and diagnostics to stderr. Promotion outcomes, including
  `openspec validate` pass or failure, appear on stderr.
- `hook explore` writes its **instruction to stdout**, because for command-expansion and prompt-submission
  events a hook's stdout is added to the model's context.

Never print diagnostics to stdout from either — for `hook stop` it corrupts the decision payload.
