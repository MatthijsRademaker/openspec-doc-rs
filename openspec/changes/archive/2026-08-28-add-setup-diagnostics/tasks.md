## 1. Read what to probe

- [x] 1.1 Enumerate the hooks from `crates/cli/src/wiring.rs` — the module `init` writes them from — rather than restating them. Two definitions drift, and the drift presents as `doctor` passing a project `init` would rewrite
- [x] 1.2 Read `.claude/settings.json` and pick out this tool's entries with `wiring::is_ours`, keeping the command string exactly as recorded. A developer pointing an entry at a local build is running that string; the canonical one checks a binary the agent never invokes
- [x] 1.3 Report a hook the shared definition names and the file does not as failing, naming `openspec-doc init` as what registers it
- [x] 1.4 Add tests: an env-prefixed and an absolute-path command are both found and returned verbatim, a missing settings file reports all three unregistered, and a malformed settings file is a reported failure rather than an empty result

## 2. The throwaway probe root

- [x] 2.1 Build a minimal OpenSpec project in a temporary directory for the probes to run against, so no probe can resolve to the project being checked
- [x] 2.2 Seed a standing verdict for the probe session in that root, using the same core writers the dashboard uses rather than literal JSON
- [x] 2.3 Add a test that a pending directive for a live session in the real project is still pending after `doctor` completes. This is the check that stops `doctor` causing the failure it diagnoses.
- [x] 2.4 Add a test that the real project's `.openspec-doc/` gains no session, note, or directive record from a `doctor` run

## 3. Probe each hook

- [x] 3.1 Execute each recorded command string verbatim through `sh -c`, with the working directory set to the probe root and a synthetic payload on stdin. No rewriting of the string — the agent runs it through a shell, so anything else probes a command the agent never runs.
- [x] 3.2 Refuse to probe a command string containing `--root`, reporting why: running it would point at the named project and consume a pending directive
- [x] 3.3 Require a parseable decision payload from `Stop`
- [x] 3.4 Require a non-empty note path from `UserPromptExpansion`
- [x] 3.5 Require the seeded directive's text from `UserPromptSubmit`. Silence cannot be the pass condition here: this hook exits zero and prints nothing on failure by design, so an empty-session probe would pass with no binary installed at all.
- [x] 3.6 Include the exit status and standard error of any failing probe in the report
- [x] 3.7 Add tests: an absent binary fails, a binary that does not recognize the subcommand fails, an empty standard output fails per hook, and a working install passes all three

## 4. The remaining checks

- [x] 4.1 Report the path and version of the `openspec-doc` on `PATH` and of the running executable, failing when they differ or when nothing resolves on `PATH`
- [x] 4.2 Print a not-checked line, with its reason, for pi.dev, for whether `serve` is running, and for the `UserPromptExpansion` matcher — executing a command bypasses matcher dispatch, and `init` is what keeps that value correct

## 5. Wire up the subcommand

- [x] 5.1 Add `doctor` to the `Command` enum and a `doctor` module in the `cli` crate
- [x] 5.2 Run every check before reporting, so one failure does not truncate the report, and exit non-zero if any failed
- [x] 5.3 Add `doctor` to the top-level help test's subcommand list, which `add-project-init` already corrected for `init` and `scratch`
- [x] 5.4 Add an end-to-end test: `doctor` against a project `init --yes` has wired, with a working binary, exits zero; removing one entry exits non-zero naming it

## 6. Documentation

- [x] 6.1 Replace the hand-wired `/tmp` probe-hook recipe in `docs/docs/reference/hooks.md` with `doctor`, keeping the warning that the matcher itself is only confirmable from a live session
- [x] 6.2 Add `doctor` to `docs/docs/reference/cli.md`, and point `docs/docs/quickstart.md` at it as the step after `init` — `init` says the wiring is written, `doctor` says it runs

## 7. Verify against the failure this exists for

- [x] 7.1 Rename the `openspec-doc` on `PATH` out of the way, leaving the settings untouched, and confirm the hook checks fail. This is the reviewer's case: registration intact, install broken, and the report must not be green.
- [x] 7.2 Point a hook entry at a stale binary lacking `hook prompt` and confirm that entry fails rather than passing on the strength of its settings text
- [x] 7.3 Run `doctor` immediately after `openspec-doc init --yes` on a fresh clone whose binary is not installed, and confirm `init` reports everything written while `doctor` reports it dead. That divergence is the entire reason both commands exist.
