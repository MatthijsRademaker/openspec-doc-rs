## Context

Every capability in this project assumes the hooks are wired. `directive-verdict-loop` assumes a verdict reaches the agent; `explore-note-capture` assumes the explore command fires a hook; `prompt-time-directive-delivery` assumes `UserPromptSubmit` is registered. None of them can tell you when that assumption is false, because their failure is the absence of an event, and an absent event looks like an idle project.

The wiring itself is deliberately un-committed: `docs/docs/quickstart.md` sends the operator to `.claude/settings.local.json` so the configuration is not checked in. That is the right call for a per-machine path, and it is why a clone arrives inert.

## Goals / Non-Goals

**Goals:**
- One command answers "is this project wired up", with a specific finding per check.
- The hook checks fail when the install is broken, not merely when the configuration text is absent.
- Non-zero exit on any failed check, so it can gate a setup script.
- Every check that could not be settled says so, in its own line, rather than being omitted or passed.

**Non-Goals:**
- Not a health check for on-disk state. Corrupt sidecars, drifted comment anchors, and unclaimed notes are other capabilities' business.
- Not a check that `serve` is running — `add-dashboard-lifecycle`.
- Not a fixer. It reports; the operator edits settings. A `doctor` that writes to `.claude/settings.local.json` is a much larger and more dangerous change, and nothing yet says the report alone is insufficient.
- Not pi.dev. Its turn boundary is an in-process TypeScript extension, so there is no command string to read and nothing to execute the same way.

## Decisions

- **The hook checks execute the hook; they do not read the settings and stop there.** A settings file is a claim about what will happen. It is silent on whether `openspec-doc` resolves on `PATH`, whether the binary it resolves to is old enough to lack `hook prompt`, and whether the command exits with anything an agent can use. Each of those is a live way to have the text exactly right and the loop entirely dead, and each is the case an operator reaches for `doctor` to settle. Reading is still the first layer — it finds the command string and its origin — but a check that stops there can only ever report that the operator typed something.

- **The probe runs against a throwaway project root, with the configured command verbatim.** `hook stop` is not read-only: it registers the session, runs the promotion check, and *consumes any pending directive*. Probing against the real project with a real session id would silently eat the reviewer's outstanding feedback — `doctor` would cause the class of failure it exists to diagnose. So the probe builds a minimal OpenSpec project in a temporary directory and runs the command with its working directory set there, which is enough for root discovery to land on the throwaway. Alternative considered: a reserved probe session id against the real root, with cleanup afterwards — rejected because it leaves a fake session on the dashboard for the window it runs in, and a cleanup that fails leaves it there permanently.

- **Verbatim means verbatim: `sh -c` on the recorded string, no rewriting.** The agent runs the command through a shell, so splitting on whitespace would probe something the agent never runs. This rules out injecting `--root` into the string, hence the working-directory approach above. If the recorded string contains an explicit `--root`, `doctor` refuses to probe that entry and reports why — running it would point at the real project and consume a directive. One condition, failing loudly, rather than a rewriter that has to be correct about shell quoting.

- **Every hook needs a response that distinguishes healthy from broken, and `hook prompt` does not have one by default.** `hook stop` answers with a decision payload and `hook explore` with a note path, both non-empty. But `hook prompt` was deliberately built to exit zero and print nothing on failure, so that a broken hook can never refuse the reviewer's prompt — which means silence is both "healthy, nothing outstanding" and "completely broken". Probing it with an empty session would therefore pass on any install, including one with no binary at all. So the throwaway root is seeded with a standing verdict for the probe session, and the check requires the probe to emit that directive's text. A fail-soft hook can only be checked by giving it something it must not stay silent about.

- **Report every occurrence of a hook with the file it came from, rather than computing a winner.** Claude Code merges user-level, project, and local settings, and reproducing that precedence exactly is a modelling job whose failure mode is a confidently wrong report. Listing each occurrence with its file needs no model, and it surfaces the two things an operator actually wants: that a hook lives only in a gitignored file, and that the same hook is registered twice.

- **The `UserPromptExpansion` matcher is checked as a string, and the report says it was.** Executing the command bypasses matcher dispatch, so no probe can reach the one failure the docs already flag as costly: a matcher that is valid, present, and never matches. Comparing against `opsx:explore|openspec-explore` is what is available. Labelling it as a static check is not a caveat for its own sake — an operator who reads it as "verified" will conclude the loop is sound while the explore hook never fires, which is the original silent failure wearing a green tick.

## Risks / Trade-offs

- [Risk] **The probe inherits `doctor`'s environment, not the agent's.** If the agent runs hooks under a different `PATH` — a non-login shell, a different launcher, a GUI-launched editor — `doctor` can resolve the binary and pass while the agent cannot and fails. → Mitigation: report the resolved binary path and version rather than a bare pass, so a mismatch is at least visible when the operator compares it against what the agent reports on stderr. This is not closed by anything short of asking the agent to run the check, which is a different change.
- [Risk] A throwaway root proves the binary responds, not that it responds against *this* project. → Mitigation: accepted, and partly covered — root resolution is checked against the real project, separately. State-level problems are out of scope by design; the check answers "is it wired", which is the question that currently has no answer at all.
- [Risk] Seeding a verdict for the `hook prompt` probe couples the check to the directive format. A format change breaks `doctor` rather than the loop. → Mitigation: seed through the same core writers the dashboard uses, so the coupling is to an internal API that a format change already has to update, not to literal JSON.
- [Trade-off] Executing three subprocesses makes `doctor` slower and more side-effecting than a settings reader. That is the whole point — the cheap version is the one that passes on a broken install, and the reviewer rejected it on the exploration for exactly that reason.
