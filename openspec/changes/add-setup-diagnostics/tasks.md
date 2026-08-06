## 1. Read the settings

- [ ] 1.1 Add a settings reader that searches the user-level, project, and local Claude Code settings files and returns every `openspec-doc` hook occurrence with its event, command string, matcher, and the path of the file it came from
- [ ] 1.2 Return occurrences as a list rather than resolving precedence — a hook registered twice is a finding, and modelling Claude Code's merge wrongly produces a confidently wrong report
- [ ] 1.3 Add tests: a hook found only in `settings.local.json` reports that path, the same hook in two files reports twice, a missing settings file is not an error, and a malformed settings file is a reported failure rather than an empty result

## 2. The throwaway probe root

- [ ] 2.1 Build a minimal OpenSpec project in a temporary directory for the probes to run against, so no probe can resolve to the project being checked
- [ ] 2.2 Seed a standing verdict for the probe session in that root, using the same core writers the dashboard uses rather than literal JSON
- [ ] 2.3 Add a test that a pending directive for a live session in the real project is still pending after `doctor` completes. This is the check that stops `doctor` causing the failure it diagnoses.
- [ ] 2.4 Add a test that the real project's `.openspec-doc/` gains no session, note, or directive record from a `doctor` run

## 3. Probe each hook

- [ ] 3.1 Execute each recorded command string verbatim through `sh -c`, with the working directory set to the probe root and a synthetic payload on stdin. No rewriting of the string — the agent runs it through a shell, so anything else probes a command the agent never runs.
- [ ] 3.2 Refuse to probe a command string containing `--root`, reporting why: running it would point at the named project and consume a pending directive
- [ ] 3.3 Require a parseable decision payload from `Stop`
- [ ] 3.4 Require a non-empty note path from `UserPromptExpansion`
- [ ] 3.5 Require the seeded directive's text from `UserPromptSubmit`. Silence cannot be the pass condition here: this hook exits zero and prints nothing on failure by design, so an empty-session probe would pass with no binary installed at all.
- [ ] 3.6 Include the exit status and standard error of any failing probe in the report
- [ ] 3.7 Add tests: an absent binary fails, a binary that does not recognize the subcommand fails, an empty standard output fails per hook, and a working install passes all three

## 4. The remaining checks

- [ ] 4.1 Compare the `UserPromptExpansion` matcher against `opsx:explore|openspec-explore`, printing both values on a mismatch
- [ ] 4.2 Label the matcher finding as a text comparison, stating that a matcher can only be confirmed to fire from a live session. A passing line that reads as "verified" here reintroduces the silent failure with a tick in front of it.
- [ ] 4.3 Report the path and version of the `openspec-doc` on `PATH` and of the running executable, failing when they differ or when nothing resolves on `PATH`
- [ ] 4.4 Report the resolved project root, failing when discovery finds none
- [ ] 4.5 Print a not-checked line for pi.dev and for whether `serve` is running, with the reason for each

## 5. Wire up the subcommand

- [ ] 5.1 Add `doctor` to the `Command` enum and a `doctor` module in the `cli` crate
- [ ] 5.2 Run every check before reporting, so one failure does not truncate the report, and exit non-zero if any failed
- [ ] 5.3 Add a test that the top-level help lists `doctor`, and update the help-listing test for `scratch` while it is being touched
- [ ] 5.4 Add an end-to-end test: `doctor` against a project with all three hooks registered and a working binary exits zero; removing one entry exits non-zero naming it

## 6. Documentation

- [ ] 6.1 Update `docs/docs/reference/hooks.md`: it says "Two hooks per agent" and predates `hook prompt`, so its table and its Claude Code example both need the `UserPromptSubmit` entry
- [ ] 6.2 Replace the hand-wired `/tmp` probe-hook instructions in that page's matcher warning with `doctor`, keeping the warning that the matcher itself is only confirmable from a live session
- [ ] 6.3 Add `doctor` to `docs/docs/reference/cli.md` and point `docs/docs/quickstart.md` at it as the step after wiring the hooks
- [ ] 6.4 Remove the `cli-surface` spec omission from the known-defects table in `docs/docs/roadmap.md`, since this change closes it

## 7. Verify against the failure this exists for

- [ ] 7.1 Run `doctor` on a fresh clone with no local settings and confirm it names all three hooks as unregistered and exits non-zero
- [ ] 7.2 Rename the `openspec-doc` on `PATH` out of the way, leaving the settings untouched, and confirm the hook checks fail. This is the reviewer's case: registration intact, install broken, and the report must not be green.
- [ ] 7.3 Point a hook entry at a stale binary lacking `hook prompt` and confirm that entry fails rather than passing on the strength of its settings text
- [ ] 7.4 Break the `UserPromptExpansion` matcher and confirm the matcher check catches it, then confirm the explore hook's own probe still passes — the two findings are independent and the report must show both
