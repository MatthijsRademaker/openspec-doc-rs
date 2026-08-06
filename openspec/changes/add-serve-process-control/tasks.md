## 1. Widen the probe rather than writing a second one

- [ ] 1.1 Change `add-dashboard-lifecycle`'s discovery module to expose enumerating every identity in the range, and have ensure-a-dashboard filter that result for a matching root. If this change ends up writing its own probe, the seam is wrong: widen the shared function instead of copying it.
- [ ] 1.2 Probe ports concurrently, or with a short enough per-port timeout that ten sequential probes stay interactive. `ensure` stops at the first match; enumeration always pays for the whole range, so the worst case is ten timeouts rather than one.
- [ ] 1.3 Tests: enumeration finds dashboards on several ports, skips ports answering something that is not a dashboard, and returns an empty result rather than an error when nothing is listening.

## 2. `serve list`

- [ ] 2.1 Add `serve list`, printing port, canonical root, and pid for every dashboard found, and naming the range it searched.
- [ ] 2.2 Resolve no project root for this command. It must work from anywhere, including outside any OpenSpec project — the "which of these is which" problem is not scoped to a project.
- [ ] 2.3 Report no dashboards as an ordinary result with exit zero. Exit non-zero only when the probe itself could not run.
- [ ] 2.4 Tests: several dashboards are all listed, running outside a project still works, an empty result exits zero, and the searched range appears in the output.

## 3. `serve kill`

- [ ] 3.1 Add `serve kill` taking `--root`, `--port`, `--pid`, or `--all`. With no target at all, stop nothing and report that one is required — the destructive reading must never be the default.
- [ ] 3.2 Decide and document whether a bare `--root` with no value defaults to the resolved project root. It is the common case and the convenient spelling, but it makes the command's behaviour depend on the working directory, which is the ambiguity 3.1 exists to remove. Pick one and say why in the help text.
- [ ] 3.3 Probe immediately before signalling rather than trusting a list the operator was shown seconds ago, and prefer stopping by port over pid where both are known: a port going quiet is direct evidence, a vanished pid is consistent with pid reuse.
- [ ] 3.4 Send `SIGTERM`, not `SIGKILL`. Do not build escalation until something is observed to ignore the first signal.
- [ ] 3.5 Re-probe after signalling and report what is actually gone. Never report success from having sent a signal.
- [ ] 3.6 Report a target that matched nothing as matching nothing, not as success.
- [ ] 3.7 Windows has no `SIGTERM`; decide what stopping means there or state plainly that the command is Unix-only for now. Do not leave it to fail at runtime with a confusing error.
- [ ] 3.8 Tests: a targeted kill stops one and leaves the others, no target stops nothing, `--all` stops everything, a target matching nothing is reported, a dashboard that survives the signal is reported as still running, and one that had already exited is reported as not running rather than as killed.

## 4. The subcommand seam

- [ ] 4.1 Confirm a bare `openspec-doc serve` still starts a server after `serve` gains children, rather than printing help. Every hook, every doc, and `add-dashboard-lifecycle`'s detached spawn invoke it that way, so a clap default-subcommand mistake here breaks the whole lifecycle change silently.
- [ ] 4.2 Test that bare `serve` starts a server, that `serve list` does not, and that `serve --port N` is unaffected.

## 5. Live verification

Enumerating processes is exactly the kind of thing that passes in a harness against servers the test started itself.

- [ ] 5.1 With dashboards running for two different checkouts, confirm `serve list` shows both with the right roots. Two checkouts, not two sessions in one — a single root is the case that cannot distinguish a correct implementation from one that reports its own root for everything.
- [ ] 5.2 Kill one by root and confirm the other keeps serving and its page stays live.
- [ ] 5.3 Kill a dashboard that the idle exit has already reaped, and confirm it reports not-running rather than success.
- [ ] 5.4 Start one by hand on a port outside the range and confirm it is absent from `serve list`, and that the reported range makes that explicable rather than looking like a bug.

## 6. Documentation

- [ ] 6.1 `docs/docs/reference/cli.md`: both commands, the required target, and `--all`.
- [ ] 6.2 State the out-of-range limit where someone hunting a stray server will actually read it — the hooks reference and the CLI reference, not only this change's design.
- [ ] 6.3 Replace any `pkill -f 'openspec-doc serve'` advice with `serve kill`. That instruction is currently the only answer the project offers and it has already killed the wrong process during development.
