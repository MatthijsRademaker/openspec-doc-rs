## 1. See what Windows actually does, before changing anything

The suite has ~153 test functions and only three are `#[cfg(unix)]`-gated, so about 150 already compile on Windows and none of them has ever been run. Predicting their failures from a Mac is guessing.

- [ ] 1.1 Add a `windows-latest` leg to the gate run from `add-continuous-verification`, with the three tests still gated, and read the result. This is an instrument, not a deliverable — it may be red, and that is the point.
- [ ] 1.2 Record what fails that this change did not anticipate. If it is nothing, say so; if it is substantial, the scope question comes back to the reviewer before the work continues rather than after.

## 2. Exec form in the wiring

- [ ] 2.1 Change the three entries in `crates/cli/src/wiring.rs` to carry an executable and an argument vector instead of one string.
- [ ] 2.2 Replace `COMMAND_MARKER` and the entry-recognition predicate. **This is the risky edit in the change.** The predicate decides whether an entry is replaced or a second one is added beside it, and two `Stop` hooks both starting a dashboard is a bad, confusing failure. Match structurally: the executable's file stem is ours and the leading argument is the hook subcommand.
- [ ] 2.3 Keep matching an entry whose executable is an absolute path to a local build. That case survives exec form and the existing tests cover it.
- [ ] 2.4 Keep matching **old single-string entries**, so that `init` over an already-wired project replaces them. This is the only place the old format is understood, and it is understood in order to delete it — not to keep working alongside the new one.
- [ ] 2.5 Test from the direction of the failure: `init` run twice, and `init` run over a settings file containing shell-form entries, must each leave exactly three entries for this tool. Not "at least three".
- [ ] 2.6 Delete the environment-assignment case from the predicate and from its tests. `OPENSPEC_DOC_BIN=… openspec-doc hook stop` is shell syntax with no shell left to interpret it, and a predicate that still recognises it is recognising something that can no longer be executed.

## 3. The probe stops using a shell

- [ ] 3.1 Replace `Command::new("sh").arg("-c").arg(command)` in `crates/cli/src/doctor/probe.rs` with a direct spawn of the recorded executable and its recorded arguments. The stdin payload, the working directory, the state-directory environment, and the captured output all stay as they are.
- [ ] 3.2 Report an entry still carrying a single argument string as failing, naming `init` as the fix. Do not split the string. A checker that tokenises a command line is a shell, badly, and the split it chooses is not the one any real shell would have chosen.
- [ ] 3.3 Confirm the probe's failure-path comment still holds: a command that does not exist exits before reading stdin, so a failed write is the child's exit rather than a fault of ours. Direct spawn changes when that happens.

## 4. Find the binary on Windows

- [ ] 4.1 Use `std::env::consts::EXE_SUFFIX` when joining the binary name onto each `PATH` entry in `crates/cli/src/doctor/binary.rs`. Today the join is the bare name, so on Windows the lookup misses a correctly installed binary and reports it absent — a failing check, on a working install, in the command written to be trusted about that.
- [ ] 4.2 Look again at `is_executable`'s `#[cfg(not(unix))]` arm, which returns `path.is_file()`. With the suffix applied it is closer to right than it was, but "a file with this name exists" is a weaker claim than the Unix arm's permission-bit check and the difference should be a decision rather than a leftover.
- [ ] 4.3 Test the resolution on Windows in CI, not by reasoning. This is the exact class of bug that was found by reading and would have been found years earlier by running.

## 5. The agent version floor

- [ ] 5.1 Establish the minimum Claude Code version that executes hook argument vectors, from the agent's own changelog, and record the number and where it came from in the design. Preliminary finding: `2.1.139`. Confirm it rather than inheriting it.
- [ ] 5.2 Add a `doctor` check that resolves the agent's executable, reports its version, and fails when it is below the floor, naming the installed version, the required one, and how to upgrade.
- [ ] 5.3 Report the check as **not checked** when no agent executable resolves, rather than passing or failing it. The capability requires that anything not examined says so.
- [ ] 5.4 State the minimum in `docs/docs/quickstart.md` and `README.md`. A requirement that lives only in a spec is a requirement a user meets by accident or not at all.
- [ ] 5.5 Do not make bare `openspec-doc` exit zero, and do not write the old form for old agents. Both were considered and rejected in the design; either one appearing means the decision was reversed without revisiting it.

## 6. Un-gate the three tests

- [ ] 6.1 Replace `std::os::unix::fs::symlink` in `path_with_binary` with a file copy. The helper's purpose is a binary reachable by bare name on a temporary `PATH`, and the comment explaining that a symlink canonicalises to the same file needs to be replaced by whatever the copy makes true instead — `doctor` compares canonical paths, so a copy is a different file and this may require the assertion to change, not just the helper.
- [ ] 6.2 Replace the hardcoded `format!("{}:/usr/bin:/bin", …)` in the `doctor` helper with `std::env::join_paths`, and drop the Unix system directories or supply the platform's equivalents.
- [ ] 6.3 Remove `#[cfg(unix)]` from the three tests and their three helpers.
- [ ] 6.4 Confirm the three run and pass on all three platforms. If 6.1 cannot be made to work by copying — because identity, not reachability, is what the test asserts — say so and solve it deliberately rather than weakening the assertion.

## 7. Windows joins the covered platforms

- [ ] 7.1 Remove the Windows omission `add-continuous-verification` recorded, in this change. Leaving it would state that Windows is uncovered while the lane covers it, which is the drift the omission record exists to prevent.
- [ ] 7.2 Confirm the Windows leg is green with all three previously gated tests running. A green leg that still skips them is the failure this change was written to remove.

## 8. The text this falsifies

- [ ] 8.1 Rewrite the hook entries quoted in `docs/docs/reference/hooks.md`. Every one of them is a shell command line.
- [ ] 8.2 Rewrite the same quotations in `README.md`, `AGENTS.md`, and `docs/docs/quickstart.md`. `AGENTS.md` describes the hooks as `openspec-doc hook prompt --agent <claude|pi>` in its standing instructions, which after this change is a description of a command a human types, not of what is registered.
- [ ] 8.3 Re-run `openspec-doc init` against this repository and commit the rewritten `.claude/settings.json`. The project's own wiring is the first consumer of this change and the first place a mistake in section 2 will show.
- [ ] 8.4 Leave the pi extension alone. It invokes the CLI directly and never went through a shell; a change to it here would be scope this change did not earn.

## 9. Live verification

- [ ] 9.1 Run a real Claude Code session in this repository after 8.3, submit a prompt, and reach a turn boundary. The unit tests cannot tell you the agent accepted the entries — only the agent can.
- [ ] 9.2 Run `openspec-doc doctor` in this repository and confirm every hook probes green through the shell-free path.
- [ ] 9.3 If a Windows machine is available, run `init` and `doctor` there. If one is not, say so in the change rather than implying the platform was exercised by hand; CI is then the only evidence and it is worth being explicit that it is.
