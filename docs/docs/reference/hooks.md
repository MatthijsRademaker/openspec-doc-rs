# Agent hooks

Two hooks per agent. Without them the dashboard still renders whatever is on disk, but nothing reaches the
agent and no exploration note gets started.

| Hook | Event | Purpose |
|---|---|---|
| `hook explore` | command expansion | ready the note location, tell the agent where to write |
| `hook stop` | turn boundary | register the session, run promotion, inject any standing verdict |

## Claude Code

`.claude/settings.json`, or `.claude/settings.local.json` if you would rather not commit it:

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

To verify, add a temporary probe with no matcher, which fires on every command expansion:

```json
{ "type": "command", "command": "cat >> /tmp/openspec-doc-probe.jsonl" }
```

Then type the command once and read the file. The payload carries `command_name`, `command_args`,
`expansion_type`, `command_source`, and the full `prompt`.

After changing settings, open `/hooks` once or restart — the config watcher only watches directories that
had a settings file when the session started.

## pi.dev

pi has no external-process hook; its turn boundary is only reachable from a TypeScript extension. This repo
ships one at `.pi/extensions/openspec-doc-hook.ts`, which handles `agent_end`, shells out to
`openspec-doc hook stop --agent pi`, and re-injects any returned directive with
`pi.sendUserMessage(…, { deliverAs: "followUp" })`.

Set `OPENSPEC_DOC_BIN` to use a binary that is not on `PATH`, such as this repo's own `target/debug/openspec-doc`.

Two quirks that will cost you time if you script against it, both tied to the session flags:

- `--session-id` for a session that does not exist yet, and `--continue`, both perform a session
  *replacement*. Touching the captured `pi`/`ctx` afterwards throws `extension ctx is stale after session
  replacement`. This is why the handler sits on `agent_end` rather than `agent_settled`.
- An `agent_end` handler combined with `--session-id` **hangs pi** before any output, handler never firing.
  Without `--session-id` it works. Cause unknown.

So to exercise it headlessly: launch `pi -p` with **neither** flag and have the agent seed its own session
state via `$PI_SESSION_ID`. Injection happens in-process through `sendUserMessage`, so a single invocation
covers the whole loop — no session resume needed.

## Wire formats

Deliberately different per agent, so pointing one flag at the other's payload fails loudly rather than
half-working.

| | Claude Code | pi.dev |
|---|---|---|
| In | `session_id`, snake_case | `sessionId`, camelCase |
| Out (allow) | `{"continue":true}` | `{"action":"stop"}` |
| Out (block) | `{"decision":"block","reason":…}` | `{"action":"continue","message":…}` |

`hook explore` reads only `session_id`, so it accepts any of an agent's event shapes rather than one
specific event.

## Where output goes

- `hook stop` writes its **decision to stdout** and diagnostics to stderr. Promotion outcomes, including
  `openspec validate` pass or failure, appear on stderr.
- `hook explore` writes its **instruction to stdout**, because for command-expansion and prompt-submission
  events a hook's stdout is added to the model's context.

Never print diagnostics to stdout from either — for `hook stop` it corrupts the decision payload.
