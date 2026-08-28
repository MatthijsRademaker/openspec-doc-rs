## Why

A project that uses this tool arrives inert. The three Claude Code hook entries are documented in `docs/docs/quickstart.md` and `docs/docs/reference/hooks.md`, and performed by hand; the pi.dev extension is a TypeScript file someone has to copy out of this repository; and the instructions that tell an agent what the scratch note is *for* exist only as a section of this repository's own `AGENTS.md`, which no other project has. Three artifacts, three manual steps, no command.

`add-setup-diagnostics` opens by naming this: *"Setting this tool up is lossy, and it fails silently."* That is an argument for a command that performs the setup at least as much as for one that inspects it. Once `init` exists and is re-runnable, "is this project wired up?" is largely answered by running it again.

The same proposal also records a judgment that this change reverses. Its design says the wiring *"is deliberately un-committed … That is the right call for a per-machine path, and it is why a clone arrives inert."* The premise is wrong: there is no per-machine path in `openspec-doc hook stop --agent claude`. Every character of the hook configuration is a fact about how the project is reviewed, identical on every machine that checks the project out. It was put in `settings.local.json` because that is where the quickstart said to put it, and the quickstart said so without a reason that survives inspection. Hook configuration is team-owned, so `init` writes `.claude/settings.json`, and this repository's own configuration moves there in this change.

## What Changes

- Adds an `init` subcommand that wires a project up: it detects which agent harnesses are present, prints every write it would make and every problem it found, and performs them only under `--yes`.
- **Dry run is the default.** With no `--yes`, `init` writes nothing and prints its plan. This is the whole of the "conscious interaction" the reviewer asked for, in place of an interactive form: the plan is identical whether or not a human is watching, so the integration tests and CI exercise the path a person sees. An `init` that silently appends to someone's `AGENTS.md` on first invocation would be the wrong default regardless of what else is true.
- **Detection proposes; flags dispose.** With no `--agent`, `init` configures the harnesses it finds evidence of. Any `--agent` replaces detection outright. `openspec init --tools` already asks "which coding agents?" in this same directory; asking it a second time is a worse command than reading the answer off disk.
- Writes three things per configured harness: the three hook entries merged into `.claude/settings.json`, `.pi/extensions/openspec-doc-hook.ts` from a copy embedded in the binary, and a marker-delimited block in `AGENTS.md` carrying the standing instructions about the scratch note and the review loop.
- **Reports a missing `opsx:explore` command rather than installing one.** The `UserPromptExpansion` matcher fires on the bare `command_name`; writing that matcher into a project with no command of that name produces a hook that can never fire, and the symptom is silence — the same silence this tool exists to remove. `init` still writes the matcher, and names the fix (`openspec init --tools claude`). Those four prompt files belong to upstream OpenSpec, and shipping copies of them would mean versioning someone else's prose.
- No `--settings local`. Committed is the default, and the only thing the local variant produces is the state `add-setup-diagnostics` was written to flag as suspicious. A flag whose sole use is to reproduce a known-bad configuration does not earn its place.

Three consequences for this repository, all of which are part of this change rather than a later cleanup:

- `.claude/settings.local.json` becomes `.claude/settings.json`, and the `.gitignore` line hiding it goes. The file holds only the three hook entries — no permissions, no personal state — so the move is clean.
- **This repository takes the managed `AGENTS.md` block**, as its own entry, leaving every hand-written rule untouched. The reviewer's ruling that this file is *this repository's* and not the tool's is satisfied by delimiting the block and tolerating the overlap they explicitly allowed. Not taking it would leave the only code path that appends to an instruction file exercised against fixtures and never against a real project — the same class of gap that produced `add-setup-diagnostics`.
- **The embedded pi extension becomes the single copy**, generated into this repository by `init` like anywhere else. Unlike `AGENTS.md` the extension has no repository-specific content, only about fifty lines of header recording how it was verified during development; that moves to `docs/`, and re-running `init` here produces no diff.

## Capabilities

### New Capabilities
- `project-init`: the `init` subcommand — what it detects, what it plans, what it writes, how it merges into files it does not own, and what it refuses to do.

### Modified Capabilities
- `cli-surface`: the top-level help requirement gains `init`. Restating it also picks up `scratch`, which drifted out of the requirement when `scratch claim` landed. `add-setup-diagnostics` proposes the identical repair; whichever lands second inherits an already-correct list and adds only its own subcommand.

## Impact

One new module in the `cli` crate, one subcommand, a JSON settings merger, a marker-block editor, and two embedded assets. No change to `core`, to any on-disk format under `.openspec-doc/`, or to any existing command's behaviour.

`docs/docs/quickstart.md` and `docs/docs/reference/hooks.md` both instruct the operator to hand-edit `.claude/settings.local.json`. Both are replaced by `init` as the documented path, with the manual JSON kept as the fallback and pointed at the committed file.

`AGENTS.md` says the fix for pi's missing explore hook is "a `pi.on("input")` handler", stated as though it were the only route. Reading pi 0.84.3's extension type declarations shows `before_agent_start` — the event this project's extension already handles, and whose payload it currently discards unread — carries the submitted `prompt`. The prescription is therefore incomplete, and this change corrects that paragraph, because landing `init` is what makes people read it. The gap itself is **not** closed here: `init --agent pi` configures pi and prints a line saying pi explorations are not captured. Closing it is a pi feature, and its design is recorded in this change's exploration note.

`add-setup-diagnostics` is re-scoped as the final task of this change rather than as later cleanup, per the repository's rule that a change carries the edits it falsifies. What survives there is the part `init` cannot do: **executing** the configured commands to prove `openspec-doc` resolves on the agent's `PATH` and is new enough. Writing a command string proves nothing about whether that string runs. Both commands must read one definition of what a wired project contains, or they drift and `doctor` begins passing projects `init` would rewrite.
