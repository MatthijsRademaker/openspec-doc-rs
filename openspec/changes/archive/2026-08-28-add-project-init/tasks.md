## 1. The definition of a wired project

- [x] 1.1 Add a module in the `cli` crate holding the three Claude Code hook entries — event, command string, matcher, timeout, status message — as data rather than as literals inside the writer
- [x] 1.2 Expose it as the single source both `init` and, later, `doctor` read, so a hook that changes shape changes in one place. Two independent copies drift, and the drift presents as `doctor` passing a project `init` would rewrite
- [x] 1.3 Add a test asserting the entries match the JSON in `docs/docs/reference/hooks.md`, so the documented block and the written block cannot diverge unnoticed

## 2. Harness detection

- [x] 2.1 Detect Claude Code by the presence of `.claude/` and pi.dev by the presence of `.pi/`, both relative to the resolved project root
- [x] 2.2 Do not read `openspec init`'s markers. Its format is upstream's to change, and a detector coupled to it fails by silently configuring nothing
- [x] 2.3 Let repeated `--agent` options replace detection outright rather than adding to it
- [x] 2.4 Exit non-zero when nothing is detected and nothing is named, reporting that `--agent` names a harness explicitly
- [x] 2.5 Add tests: detection with each directory alone, with both, with neither, and `--agent` overriding a detection that disagrees

## 3. The plan

- [x] 3.1 Build a plan value listing the selected harnesses, every intended file write with its kind (create, modify, unchanged), and every warning
- [x] 3.2 Render the plan identically whether or not `--yes` was given, and whether or not a terminal is attached — one code path, so the tests exercise what a person reads
- [x] 3.3 Without `--yes`, print the plan, exit zero, and write nothing
- [x] 3.4 With `--yes`, print the plan and then perform it, reporting each write as performed
- [x] 3.5 Add a test that a dry run against an unconfigured project leaves every file byte-identical
- [x] 3.6 Add a test that a second `--yes` run leaves every managed file byte-identical to its state after the first

## 4. Claude Code settings

- [x] 4.1 Read `.claude/settings.json` when it exists, parse it, and fail loudly naming the file when it is not valid JSON — never overwrite an unparseable settings file
- [x] 4.2 Treat any hook entry whose command *contains* `openspec-doc hook ` as ours and replace it. Substring, not prefix, so `OPENSPEC_DOC_BIN=… openspec-doc hook stop` and an absolute-path invocation are recognised rather than duplicated
- [x] 4.3 Preserve every other key and every other hook entry exactly
- [x] 4.4 Create the file and the `.claude/` directory when absent
- [x] 4.5 Write only `.claude/settings.json`. There is no `--settings local`: the command strings hold nothing machine-specific, so the per-machine argument for the local file does not hold
- [x] 4.6 Add tests: fresh file, file with unrelated hooks preserved, an existing entry replaced rather than duplicated, an env-prefixed entry recognised as ours, malformed JSON failing without a write

## 5. The explore-command check

- [x] 5.1 Search the project for a command or skill named `opsx:explore` or `openspec-explore`, covering both `.claude/commands/` and `.claude/skills/` layouts
- [x] 5.2 Write the `UserPromptExpansion` entry whether or not one is found — a complete configuration with a named hole beats a silently partial one
- [x] 5.3 On a miss, emit a warning stating the hook will never fire and naming `openspec init --tools claude` as the fix
- [x] 5.4 Add tests: a project with the command warns nothing, a project without it warns and still plans the entry

## 6. The pi extension as an embedded asset

- [x] 6.1 Move the development-verification header out of `.pi/extensions/openspec-doc-hook.ts` into `docs/`, beside the existing Claude-side probe recipe, leaving the file as the one that ships
- [x] 6.2 Embed that file in the binary and write it to `.pi/extensions/openspec-doc-hook.ts`, creating the directory when absent
- [x] 6.3 Replace an existing copy that differs, and report it unchanged when it does not
- [x] 6.4 Regenerate this repository's own copy from the embedded asset, so the checked-in file is `init`'s output and a divergence shows up as a diff
- [x] 6.5 Report, whenever pi is configured, that pi sessions produce no exploration note and are therefore not registered for review
- [x] 6.6 Add tests: written into an empty project, a differing copy replaced, an identical copy reported unchanged

## 7. The managed instruction block

- [x] 7.1 Write the generic instruction text as an embedded asset: what the scratch note is for, that it is the only artifact the reviewer can read, the claim command, the comment reply and address commands, and how directives arrive
- [x] 7.2 Keep it generic. Everything specific to this repository — its port, its own known gaps, its settings path — stays in this repository's hand-written section and out of the shipped block
- [x] 7.3 Append the block between `<!-- openspec-doc:begin -->` and `<!-- openspec-doc:end -->` when neither marker is present, preserving prior content byte-for-byte
- [x] 7.4 Replace only the span between the markers when both are present
- [x] 7.5 Fail loudly when exactly one marker is present, rather than appending a second block to the file that tells the agent how to behave
- [x] 7.6 Create `AGENTS.md` when it does not exist
- [x] 7.7 Add `--skip-instructions` to plan and perform no write to it
- [x] 7.8 Add tests: append, replace in place, half-marked file rejected, file created when absent, `--skip-instructions` writing nothing

## 8. Wire it into the CLI

- [x] 8.1 Add the `Init` variant with `--agent`, `--yes`, and `--skip-instructions`, and a one-line description consistent with the existing subcommands
- [x] 8.2 Add an integration test in `crates/cli/tests/cli.rs` driving `init` against a temporary project through the binary
- [x] 8.3 Update the top-level help assertion to list `init`, and `scratch` with it

## 9. This repository adopts what it ships

- [x] 9.1 Move `.claude/settings.local.json` to `.claude/settings.json` and commit it — the file holds only the three hook entries, so the move carries no personal state
- [x] 9.2 Remove the `.claude/settings.local.json` line from `.gitignore`
- [x] 9.3 Install the managed `AGENTS.md` block here by running `init`, as its own entry, leaving every hand-written rule untouched. This is what keeps the marker-editing path from being one only fixtures ever traverse
- [x] 9.4 Verify `openspec-doc init` against this repository reports every managed file unchanged afterwards

## 10. Documentation this change falsifies

- [x] 10.1 Rewrite `docs/docs/quickstart.md` around `init`, keeping the manual JSON as the fallback and pointing it at `.claude/settings.json`
- [x] 10.2 Update `docs/docs/reference/hooks.md` the same way, and drop the "if you would rather not commit it" framing — the reason it gave does not survive inspection
- [x] 10.3 Correct the pi paragraph in `AGENTS.md`. It presents `pi.on("input")` as the only route to the missing explore hook; pi 0.84.3's type declarations show `agent_start` and `agent_settled` carry no payload at all, `agent_end` carries `messages` but fires too late, and `before_agent_start` carries the submitted `prompt` — on the event this project's extension already handles and whose payload it discards unread. Record both routes and why the command-name match is still preferred: `before_agent_start` sees the prompt *after* expansion, so matching there means matching prose that upstream owns
- [x] 10.4 Do not close the pi gap here. Its design is in this change's exploration note; the fix is a pi feature

## 11. Re-scope `add-setup-diagnostics`

Last, after everything above is implemented, so the re-scope is written against what landed rather than what was planned.

- [x] 11.1 Rewrite its "Why": the manual-setup argument now belongs to `init`, and what remains is that a written configuration proves nothing about whether it runs
- [x] 11.2 Delete the claim in its design that the wiring is deliberately un-committed because it is a per-machine path. It is not a per-machine path, and it is now committed
- [x] 11.3 Point its settings reader at the shared definition from task 1 instead of its own copy
- [x] 11.4 Drop its checks that `init` now makes unreachable, and keep the execution probes — `PATH` resolution, binary version, hook response — which are the part `init` cannot do
- [x] 11.5 Reduce its `cli-surface` delta to adding `doctor` to the list this change already corrected
- [x] 11.6 Prune its tasks to match, and confirm proposal, design, tasks, and specs all describe the same reduced change
