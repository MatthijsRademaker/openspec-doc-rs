## Why

`init` writes the configuration. Nothing proves it runs.

A settings file is a claim about what will happen. It is silent on whether `openspec-doc` resolves on the agent's `PATH`, on whether the binary it resolves to is recent enough to have the subcommand named in it, and on whether that command exits with a response the agent can use. Each of those leaves the configuration text exactly right and the loop entirely dead — and each fails the way everything in this project fails, by the absence of an event, which is indistinguishable from an idle project.

That is the whole of what is left. The manual-setup argument this change opened with — hooks documented in one place and registered in none, a fresh clone arriving inert — belongs to `add-project-init`, which landed it: `openspec-doc init` wires a project and re-running it answers "is this written down correctly?" by printing `unchanged` three times. Asking that question a second way is a second definition of a wired project, and two definitions drift.

## What Changes

- Adds a `doctor` subcommand that **runs** each configured hook and reports what happened, exiting non-zero when any probe fails.
- Reads `.claude/settings.json` for the command strings to execute, identifying this tool's entries with the same predicate `init` writes them by — one module, shared, so a hook that changes shape changes in one place. Two copies drift, and the drift presents as `doctor` passing a project `init` would rewrite.
- Checks that `openspec-doc` on `PATH` is the same binary as the running one, and reports both paths and versions when it is not. The agent runs whichever `PATH` resolves; a check that reports on the running binary's behalf is answering the wrong question.
- Reports everything it does not examine as **not checked**, with the reason. A green line for something never looked at is the failure mode this change exists to remove.

Dropped from this change's earlier scope, because `init` now owns them:

- **Searching every settings file and reporting which one each hook came from.** That finding existed because the wiring lived in a gitignored local file; it is now committed at `.claude/settings.json`, which `init` writes and owns.
- **Comparing the `UserPromptExpansion` matcher against its required value.** `init` writes that matcher from the shared definition, so a mismatch means someone hand-edited the file that `init` will report as modified the next time it runs.
- **Reporting the resolved project root.** Every subcommand resolves it and fails loudly when it cannot, `init` included. A dedicated check restates that.

## Capabilities

### New Capabilities
- `setup-diagnostics`: the `doctor` subcommand — how it runs the hooks, what a probe must see to pass, what it refuses to probe, and what it reports as unchecked.

### Modified Capabilities
- `cli-surface`: the top-level help requirement gains `doctor`. `add-project-init` already corrected the list to include `init` and `scratch`, so this adds one name to an otherwise correct requirement.

## Impact

One new module in the `cli` crate and one subcommand. The definition of a wired project — the three hook entries and the predicate that recognises them — is already in `crates/cli/src/wiring.rs`, written for `init`; `doctor` reads it rather than restating it. No change to `core`, to any on-disk format, or to any existing command's behaviour.

Two things this deliberately does not do. It does not check whether `serve` is running — that is `add-dashboard-lifecycle`'s problem and a different failure. `doctor` answers *does the wiring run*, and the lifecycle change answers *is anyone at the other end*. If `add-dashboard-lifecycle` has landed, `doctor` SHOULD report whether a dashboard serves this root by calling its discovery module — one line, sourced rather than reimplemented; if this change lands first, omit the line rather than growing a second probe. And it does not check pi.dev, whose delivery points are an in-process TypeScript extension rather than a settings entry: there is no command string to read and nothing to invoke the same way. `doctor` reports pi as **not checked** rather than passing it.
