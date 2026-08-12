## 1. Widen the probe rather than writing a second one

- [ ] 1.1 Change `add-dashboard-lifecycle`'s discovery module to expose enumerating every identity in the range, and have ensure-a-dashboard filter that result for a matching root. If this change ends up writing its own probe, the seam is wrong: widen the shared function instead of copying it.
- [ ] 1.2 **Probe the range concurrently — a thread per port, joined.** Sequential is not an option at thirty-two ports: a 250ms per-port timeout is an eight-second worst case, which is unusable interactively and long enough to be killed as a hook overrun. In practice a closed local port refuses instantly and only a port that accepts without answering costs the full timeout, but the worst case is the one that has to hold. This applies to `ensure`'s pre-bind sweep as much as to `list`, so it belongs in the shared function rather than here.
- [ ] 1.3 Tests: enumeration finds dashboards on several ports, skips ports answering something that is not a dashboard, and returns an empty result rather than an error when nothing is listening.

## 2. `serve list`

- [ ] 2.1 Add `serve list`, printing port, canonical root, and pid for every dashboard found, and naming the range it searched.
- [ ] 2.2 Resolve no project root for this command. It must work from anywhere, including outside any OpenSpec project — the "which of these is which" problem is not scoped to a project.
- [ ] 2.3 Report no dashboards as an ordinary result with exit zero. Exit non-zero only when the probe itself could not run.
- [ ] 2.4 Join the probe result against the port registry `add-dashboard-lifecycle` introduces, and show assigned-but-not-running roots as their own rows. Read the registry for *assignments only*: a row is marked running because a probe answered, never because a file said so. This is the one place the rejected state-file design could quietly re-enter the project.
- [ ] 2.5 Distinguish a dashboard serving on a port that is not its root's assignment. That is the state `ensure` repairs on its next run, and it looking like an ordinary row is how a broken repair path stays invisible.
- [ ] 2.6 Treat a missing or unreadable registry as losing the assigned-not-running rows and nothing else. The running dashboards must still be listed in full.
- [ ] 2.7 Tests: several dashboards are all listed, running outside a project still works, an empty result exits zero, the searched range appears in the output, an assigned-but-idle root appears marked not running, a dashboard away from its assignment is distinguishable, and a missing registry still lists everything running.

## 3. `serve kill`

- [ ] 3.1 Add `serve kill` taking `--root`, `--port`, `--pid`, or `--all`. With no target at all, stop nothing and report that one is required — the destructive reading must never be the default.
- [ ] 3.2 Decide and document whether a bare `--root` with no value defaults to the resolved project root. It is the common case and the convenient spelling, but it makes the command's behaviour depend on the working directory, which is the ambiguity 3.1 exists to remove. Pick one and say why in the help text.
- [ ] 3.3 Probe immediately before signalling rather than trusting a list the operator was shown seconds ago, and prefer stopping by port over pid where both are known: a port going quiet is direct evidence, a vanished pid is consistent with pid reuse.
- [ ] 3.4 Send `SIGTERM`, not `SIGKILL`. Do not build escalation until something is observed to ignore the first signal.
- [ ] 3.5 Re-probe after signalling and report what is actually gone. Never report success from having sent a signal.
- [ ] 3.6 Report a target that matched nothing as matching nothing, not as success.
- [ ] 3.7 Leave the port assignment alone. Stopping a dashboard says nothing about where that project belongs, and a kill that also forgot the assignment would move the project on its next start — which is the instability this whole line of work removed.
- [ ] 3.8 Windows has no `SIGTERM`; decide what stopping means there or state plainly that the command is Unix-only for now. Do not leave it to fail at runtime with a confusing error.
- [ ] 3.9 Tests: a targeted kill stops one and leaves the others, no target stops nothing, `--all` stops everything, a target matching nothing is reported, a dashboard that survives the signal is reported as still running, one that had already exited is reported as not running rather than as killed, and a killed project keeps its assignment.

## 4. `serve forget`

- [ ] 4.1 Add `serve forget <root>`, dropping that root's port assignment. Take a root rather than a port: the operator's question is "I do not use that checkout any more", and making them look the port up first answers a question they did not ask.
- [ ] 4.2 Refuse to forget a root whose dashboard is currently serving, and say to stop it first. Dropping it live hands the port to another project while a server still sits on it, and leaves the forgotten root to rediscover its own dashboard on a port that now belongs to someone else.
- [ ] 4.3 Report a root with no assignment as nothing to forget, not as success.
- [ ] 4.4 Check that the exhausted-range error `add-dashboard-lifecycle` raises names this command with the spelling it actually has. Two changes, one error message: whichever lands second verifies the pair, because an error naming a command that does not exist is worse than the collision it prevented.
- [ ] 4.5 Tests: forgetting an idle assigned root frees its port for a newly seen root, forgetting a served root is refused, forgetting an unassigned root reports nothing to forget.

## 5. The subcommand seam

- [ ] 5.1 Confirm a bare `openspec-doc serve` still starts a server after `serve` gains children, rather than printing help. Every hook, every doc, and `add-dashboard-lifecycle`'s detached spawn invoke it that way, so a clap default-subcommand mistake here breaks the whole lifecycle change silently.
- [ ] 5.2 Test that bare `serve` starts a server, that `serve list`, `serve url`, and `serve forget` do not, and that `serve --port N` is unaffected.

## 6. Live verification

Enumerating processes is exactly the kind of thing that passes in a harness against servers the test started itself.

- [ ] 6.1 With dashboards running for two different checkouts, confirm `serve list` shows both with the right roots. Two checkouts, not two sessions in one — a single root is the case that cannot distinguish a correct implementation from one that reports its own root for everything.
- [ ] 6.2 Kill one by root and confirm the other keeps serving and its page stays live.
- [ ] 6.3 Kill a dashboard that the idle exit has already reaped, and confirm it reports not-running rather than success.
- [ ] 6.4 Start one by hand on a port outside the range and confirm it is absent from `serve list`, and that the reported range makes that explicable rather than looking like a bug.
- [ ] 6.5 Kill a project's dashboard, then reach a turn boundary in it, and confirm it comes back on the same port. A kill that quietly drops the assignment passes every unit test and fails this.

## 7. Documentation

- [ ] 7.1 `docs/docs/reference/cli.md`: all three commands, the required target for `kill`, and `--all`.
- [ ] 7.2 State the out-of-range limit where someone hunting a stray server will actually read it — the hooks reference and the CLI reference, not only this change's design.
- [ ] 7.3 Replace any `pkill -f 'openspec-doc serve'` advice with `serve kill`. That instruction is currently the only answer the project offers and it has already killed the wrong process during development.
- [ ] 7.4 Document where the port registry lives and that `serve forget` is how an entry is removed, in the same place the assignment behaviour is described rather than in a separate corner.
