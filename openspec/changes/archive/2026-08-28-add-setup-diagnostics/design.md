## Context

Every capability in this project assumes the hooks are wired. `directive-verdict-loop` assumes a verdict reaches the agent; `explore-note-capture` assumes the explore command fires a hook; `prompt-time-directive-delivery` assumes `UserPromptSubmit` is registered. None of them can tell you when that assumption is false, because their failure is the absence of an event, and an absent event looks like an idle project.

`add-project-init` closed half of that. `openspec-doc init` writes the wiring, and re-running it reports every managed file as unchanged, so "is this project configured?" is a question the tool now answers. What it cannot answer is whether the configuration *runs*: `init` will write a perfect settings file for a binary that is not on the agent's `PATH`, and it has no way to find out.

That boundary is the whole of this change. `init` writes the configuration; `doctor` runs it. Both read one definition of what a wired project contains — `crates/cli/src/wiring.rs`, which already exists — or they drift, and the drift presents as `doctor` passing a project `init` would rewrite.

## Goals / Non-Goals

**Goals:**
- One command answers "does the wiring actually run", with a specific finding per hook.
- The hook checks fail when the install is broken, not merely when the configuration text is absent.
- Non-zero exit on any failed check, so it can gate a setup script.
- Every check that could not be settled says so, in its own line, rather than being omitted or passed.

**Non-Goals:**
- Not a second definition of a wired project. The entries and the predicate that recognises them belong to `wiring`, shared with `init`.
- Not a check that the configuration is *present and correct* — `init` reports that, and reports it as a diff.
- Not a health check for on-disk state. Corrupt sidecars, drifted comment anchors, and unclaimed notes are other capabilities' business.
- Not a check that `serve` is running — `add-dashboard-lifecycle`.
- Not a fixer. `init` is the fixer.
- Not pi.dev. Its delivery points are an in-process TypeScript extension, so there is no command string to read and nothing to execute the same way.

## Decisions

- **The hook checks execute the hook; reading the settings only finds the string to run.** A settings entry proves the operator, or `init`, typed something. It is silent on whether `openspec-doc` resolves on `PATH`, whether the binary it resolves to is old enough to lack `hook prompt`, and whether the command exits with anything an agent can use. Each of those is a live way to have the text exactly right and the loop entirely dead, and each is the case an operator reaches for `doctor` to settle.

- **The command strings come from `.claude/settings.json`, and ours are identified by `wiring::COMMAND_MARKER`.** Not from the shared definition's own `command` field: a developer pointing an entry at a local build — `OPENSPEC_DOC_BIN=… openspec-doc hook stop --agent claude` — is running that string, and probing the canonical one would check a binary the agent never invokes. The shared definition supplies the events to enumerate and the predicate that recognises ours; the file supplies what to run.

- **The probe runs against a throwaway project root, with the configured command verbatim.** `hook stop` is not read-only: it registers the session, runs the promotion check, and *consumes any pending directive*. Probing against the real project with a real session id would silently eat the reviewer's outstanding feedback — `doctor` would cause the class of failure it exists to diagnose. So the probe builds a minimal OpenSpec project in a temporary directory and runs the command with its working directory set there, which is enough for root discovery to land on the throwaway. Alternative considered: a reserved probe session id against the real root, with cleanup afterwards — rejected because it leaves a fake session on the dashboard for the window it runs in, and a cleanup that fails leaves it there permanently.

- **Verbatim means verbatim: `sh -c` on the recorded string, no rewriting.** The agent runs the command through a shell, so splitting on whitespace would probe something the agent never runs. This rules out injecting `--root` into the string, hence the working-directory approach above. If the recorded string contains an explicit `--root`, `doctor` refuses to probe that entry and reports why — running it would point at the real project and consume a directive. One condition, failing loudly, rather than a rewriter that has to be correct about shell quoting.

- **Every hook needs a response that distinguishes healthy from broken, and `hook prompt` does not have one by default.** `hook stop` answers with a decision payload and `hook explore` with a note path, both non-empty. But `hook prompt` was deliberately built to exit zero and print nothing on failure, so that a broken hook can never refuse the reviewer's prompt — which means silence is both "healthy, nothing outstanding" and "completely broken". Probing it with an empty session would therefore pass on any install, including one with no binary at all. So the throwaway root is seeded with a standing verdict for the probe session, and the check requires the probe to emit that directive's text. A fail-soft hook can only be checked by giving it something it must not stay silent about.

## Risks / Trade-offs

- [Risk] **The probe inherits `doctor`'s environment, not the agent's.** If the agent runs hooks under a different `PATH` — a non-login shell, a different launcher, a GUI-launched editor — `doctor` can resolve the binary and pass while the agent cannot and fails. → Mitigation: report the resolved binary path and version rather than a bare pass, so a mismatch is at least visible when the operator compares it against what the agent reports on stderr. This is not closed by anything short of asking the agent to run the check, which is a different change.
- [Risk] A throwaway root proves the binary responds, not that it responds against *this* project. → Mitigation: accepted. State-level problems are out of scope by design; the check answers "does it run", which is the question `init` cannot answer at all.
- [Risk] Seeding a verdict for the `hook prompt` probe couples the check to the directive format. A format change breaks `doctor` rather than the loop. → Mitigation: seed through the same core writers the dashboard uses, so the coupling is to an internal API that a format change already has to update, not to literal JSON.
- [Risk] **A hook registered somewhere other than `.claude/settings.json` is not probed.** Claude Code merges user-level and local settings too, and this change reads one file. → Accepted. `init` owns that file and writes nothing else; an entry elsewhere is one nothing in this project put there. The earlier scope reported every occurrence with its origin, which existed because the wiring used to live in a gitignored file — a state `init` removed.
- [Trade-off] Executing three subprocesses makes `doctor` slower and more side-effecting than a settings reader. That is the whole point — the cheap version is the one that passes on a broken install, and it is also the version `init --yes` already performs for free.
