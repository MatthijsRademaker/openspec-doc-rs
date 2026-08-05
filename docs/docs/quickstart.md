# Quickstart

Getting from nothing to one complete review loop. Budget ten minutes.

Looking to exercise every interaction against a throwaway project instead? That is
[Manual verification](/development/manual-verification.md).

## 1. Install

```bash
git clone https://github.com/matthijsrademaker/openspec-doc-rs
cd openspec-doc-rs
cargo install --path crates/cli
openspec-doc --version
```

`~/.cargo/bin` needs to be on your `PATH`. Re-run with `--force` after pulling changes.

You also need the `openspec` CLI on `PATH` — promotion runs `openspec validate` against the change it
promotes onto.

## 2. Wire the hooks

In the OpenSpec project you want to review, add to `.claude/settings.local.json`:

```json
{
  "hooks": {
    "Stop": [
      { "hooks": [{ "type": "command", "command": "openspec-doc hook stop --agent claude", "timeout": 30 }] }
    ],
    "UserPromptExpansion": [
      {
        "matcher": "opsx:explore|openspec-explore",
        "hooks": [{ "type": "command", "command": "openspec-doc hook explore --agent claude", "timeout": 30 }]
      }
    ]
  }
}
```

Then **open `/hooks` once, or restart**. The config watcher only watches directories that already had a
settings file when the session started, so a newly created one is not picked up mid-session.

For pi.dev, and for verifying the matcher actually fires, see [Agent hooks](/reference/hooks.md).

## 3. Start the dashboard

In a second terminal, at the project root:

```bash
openspec-doc serve
```

Leave it running. Nothing in the loop works without it, and nothing tells you so — the hooks still fire,
the agent still writes its note, and there is simply no reviewer at the other end.

## 4. Explore

In the agent session, type the explore command:

```
/openspec-explore I want to work out whether the widget cache is worth keeping
```

The hook fires and prints the note's resolved path into the agent's context. The agent writes its
exploration to `.openspec-doc/scratch/_session/<session-id>.md` as it thinks.

Reload the dashboard. The session appears on `/`, and its page renders the note.

:::tip Nothing on the page?
Two likely causes. Either the agent has not written the note yet — the page renders exactly one artifact
and skips it if absent — or the matcher did not fire. Check
`ls .openspec-doc/scratch/_session/` for the `.md` file.
:::

## 5. Review

On the session page:

1. **Select text in the note.** The comment composer appears only once you have a selection *inside* the
   artifact — it is a plain form, so fill it in and submit.
2. **Submit a phase verdict** under *Phase verdict*. Pick `keep exploring` and write what is still open in
   *Open questions or resolutions*. That field is required for this verdict; its entire content is what
   remains unsettled, so a blank one says nothing.

Comments alone reach nobody. **The verdict is the trigger.**

## 6. Send it back

Send the agent session any message. At the end of that turn the Stop hook translates your verdict into a
directive, blocks the turn, and feeds it back. The agent reads the files the directive names — your notes,
the note, any comments — and carries on.

:::info That turn runs blind
The feedback lands when the turn *ends*, so the message you use to poke the agent is processed before it
sees your verdict. Submitting the verdict while the agent is still working avoids this.
:::

## 7. Formalize

When the exploration is ready, submit a `move to proposal` verdict. The next directive tells the agent to
create the change and then run:

```bash
openspec-doc scratch claim --session <id> --change <name>
```

At the following turn boundary the note is promoted to `.openspec-doc/scratch/<name>.md`, your comments
follow it, a redirect is left behind, and `openspec validate` runs. The change page now renders the
proposal, design, tasks, spec deltas, and the exploration that produced them.

If the agent forgets to claim, nothing breaks — the note stays where it is. Run the command yourself.

## Checking what was recorded

Everything is a plain file, and the CLI reads the same sidecars the dashboard writes:

```bash
openspec-doc comment list --session <id>
openspec-doc summary
cat .openspec-doc/verdicts/_session/<id>.jsonl
```

See [On-disk state](/reference/on-disk-state.md) for the full layout.
