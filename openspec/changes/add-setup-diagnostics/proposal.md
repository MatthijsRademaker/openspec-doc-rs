## Why

Setting this tool up is lossy, and it fails silently. The hook configuration lives in `.claude/settings.local.json`, which is gitignored, so a fresh clone has hooks documented in `docs/docs/reference/hooks.md` and none registered. Nothing says so. `serve` starts, the dashboard renders, the session and change pages are empty — which is exactly what a correctly wired project looks like before an agent has done anything. The operator has no way to tell "not set up" from "nothing has happened yet".

The same shape recurs one level down. `UserPromptExpansion` matches on the bare `command_name`, and a matcher that does not match never runs its hook and reports nothing. `agent-hook-bridge` already carries a `:::danger` block about it, and that block's advice is "verify it fires rather than assuming" — followed by instructions to hand-wire a probe hook that appends to `/tmp` and read the file yourself. That is a diagnostic the tool should own.

## What Changes

- Adds a `doctor` subcommand that reports whether this project is wired up, and exits non-zero when it is not.
- **The hook checks run the hooks.** For each configured entry, `doctor` executes the command string recorded in settings, against a throwaway project root, with a synthetic payload on stdin, and checks that it responds. Reading the settings file proves the text is there and nothing else: it cannot tell you that `openspec-doc` resolves on the agent's `PATH`, that the binary it resolves to is new enough to have the subcommand named, or that the command exits with a usable response. A check that passes on a broken install is worse than no check, because it converts "I don't know" into a wrong answer.
- Reports, for each of the three Claude Code entries (`Stop`, `UserPromptSubmit`, `UserPromptExpansion`), whether it is registered, **which settings file it came from**, and what happened when it ran. The origin is part of the finding, not decoration — a hook present only in gitignored local settings is precisely the state that produced this change.
- Checks that `openspec-doc` on `PATH` is the same binary as the running one, and reports both paths and versions when it is not.
- Checks that the project root resolves, and reports the resolved path.
- Reports the `UserPromptExpansion` matcher as a **string comparison, labelled as one**. Running the command directly bypasses matcher dispatch entirely, so this is the one check that cannot be settled by execution, and the report says so rather than implying the matcher was exercised.

## Capabilities

### New Capabilities
- `setup-diagnostics`: the `doctor` subcommand — the checks it runs, how it runs the hook checks, what it prints, and what it exits with.

### Modified Capabilities
- `cli-surface`: the top-level help requirement gains `doctor`. Restating that requirement also picks up `scratch`, which the roadmap already records as drifted out of the spec when `scratch claim` landed. Both are the same sentence; fixing one and knowingly leaving the other wrong is not the surgical option, it is the sloppy one.

## Impact

One new module in the `cli` crate, one subcommand, and a settings reader. No change to `core`, to any on-disk format, or to any existing command's behaviour. `docs/docs/reference/hooks.md` still says "Two hooks per agent" and predates `hook prompt`; the check enumerates three, so that page is updated with it.

Two things this deliberately does not do. It does not check whether `serve` is running — that is `add-dashboard-lifecycle`'s problem and a different failure. And it does not check pi.dev, whose turn-boundary hook is an in-process TypeScript extension rather than a settings entry: there is no command string to read and nothing to invoke the same way. `doctor` reports pi as **not checked** rather than passing it, because a green line for something never examined is the failure mode this whole change exists to remove.
