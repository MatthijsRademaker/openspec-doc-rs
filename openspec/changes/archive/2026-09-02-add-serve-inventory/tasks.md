## 1. Widen the probe rather than writing a second one

- [x] 1.1 Change `crates/cli/src/discovery.rs` to expose enumerating every identity in the range, and rewrite `find_in` as a filter over that enumeration rather than its own loop. If this change ends up with two loops over `probe`, the seam is wrong.
- [x] 1.2 Probe the range concurrently — a thread per port, joined with `std::thread::scope`. The worst case is a port that accepts and never answers, which costs `CONNECT_TIMEOUT + READ_TIMEOUT`; thirty-two of those sequentially is around twelve seconds and unusable for an interactive command. Put it in the shared function so `ensure`'s miss path benefits too, but do not repeat the predecessor's claim that `ensure` needs it equally: `find` short-circuits on the assignment and usually costs one probe.
- [x] 1.3 Keep the existing probe untouched. The hand-rolled `GET`, the timeouts, and the "anything that is not an identity is a no" rule are already correct and already tested; this change is about how many times it runs, not what it does.
- [x] 1.4 Tests: enumeration finds dashboards on several ports, skips ports answering something that is not a dashboard, returns an empty result rather than an error when nothing is listening, and the existing `find`/`ensure` tests still pass unchanged.

## 2. `serve list`

- [x] 2.1 Add `ServeCommand::List`, printing port, canonical root, and pid for every dashboard found, and naming the range it searched.
- [x] 2.2 Resolve no project root for this command. It must work from anywhere, including outside any OpenSpec project — the "which of these is which" problem is not scoped to a project.
- [x] 2.3 Report no dashboards as an ordinary result with exit zero. Exit non-zero only when the probe itself could not run.
- [x] 2.4 Join the probe result against the port registry and show assigned-but-not-running roots as their own rows. Read the registry for *assignments only*: a row is marked running because a probe answered, never because a file said so. This is the one place the rejected state-file design could quietly re-enter the project.
- [x] 2.5 Distinguish a dashboard serving on a port that is not its root's assignment. That is the state `ensure` repairs on its next run, and it looking like an ordinary row is how a broken repair path stays invisible.
- [x] 2.6 Treat a missing or unreadable registry as losing the assigned-not-running rows and nothing else. The running dashboards must still be listed in full.
- [x] 2.7 Tests: several dashboards are all listed, running outside a project still works, an empty result exits zero, the searched range appears in the output, an assigned-but-idle root appears marked not running, a dashboard away from its assignment is distinguishable, and a missing registry still lists everything running.

## 3. `serve forget`

- [x] 3.1 Add a way to read the whole registry and to drop one entry in `crates/core/src/dashboard.rs`, beside `assign`, `assigned`, and `record`. Same atomic-rename write path; do not open a second one.
- [x] 3.2 Add `ServeCommand::Forget` taking a root, dropping that root's assignment. Take a root rather than a port: the operator's question is "I do not use that checkout any more", and making them look the port up first answers a question they did not ask.
- [x] 3.3 Canonicalize the given root the same way `Project` does before keying on it, or a relative path and an absolute one forget different things.
- [x] 3.4 Refuse to forget a root whose dashboard is currently serving, and say to stop it first. Dropping it live hands the port to another project while a server still sits on it, and leaves the forgotten root to rediscover its own dashboard on a port that now belongs to someone else.
- [x] 3.5 Report a root with no assignment as nothing to forget, not as success.
- [x] 3.6 Tests: forgetting an idle assigned root frees its port for a newly seen root, forgetting a served root is refused, forgetting an unassigned root reports nothing to forget, and a non-canonical path forgets the same entry a canonical one does.

## 4. The error message that has been naming a dead end

- [x] 4.1 Check that `PortRangeFull` in `crates/core/src/error.rs` names `serve forget` with the spelling the command actually has, now that the command exists.
- [x] 4.2 Correct the source comment above it. It says `serve forget` lands in `add-serve-process-control` and that whichever of the two is touched second checks the other; the command has moved here, and this is the touch it was waiting for. Delete the comment rather than repointing it — once both exist there is nothing left to coordinate.
- [x] 4.3 Test that exhausting the range produces an error naming a subcommand the binary accepts. A string assertion is not enough; the point of the pairing is that the named command runs.

## 5. The subcommand seam

`add-dashboard-lifecycle` already made `serve` a parent command with `Url` under an `Option<ServeCommand>`, so the risk this section originally carried is spent. What is left is confirming two more variants do not spend it back.

- [x] 5.1 Confirm a bare `openspec-doc serve` still starts a server with three subcommands present rather than printing help.
- [x] 5.2 Extend the existing `serve url` test so `serve list` and `serve forget` are covered by the same "does not start a server" assertion, and `serve --port N` is unaffected.

## 6. Live verification

Enumerating processes is exactly the kind of thing that passes in a harness against servers the test started itself.

- [x] 6.1 With dashboards running for two different checkouts, confirm `serve list` shows both with the right roots. Two checkouts, not two sessions in one — a single root is the case that cannot distinguish a correct implementation from one that reports its own root for everything.
- [x] 6.2 Run `serve list` from a directory that is not inside any OpenSpec project and confirm it still lists both.
- [x] 6.3 Start one by hand on a port outside the range and confirm it is absent from `serve list`, and that the reported range makes that explicable rather than looking like a bug.
- [x] 6.4 Forget an idle project's assignment, then reach a turn boundary in a checkout the registry has never seen, and confirm it takes the freed port.

## 7. Documentation

- [x] 7.1 `docs/docs/reference/cli.md`: both commands, beside the existing `serve url` entry.
- [x] 7.2 State the out-of-range limit where someone hunting a stray server will actually read it — the hooks reference and the CLI reference, not only this change's design.
- [x] 7.3 Say how to stop a dashboard, which the docs currently do not say anywhere. Until `add-serve-process-control` lands that answer is "find it with `serve list` and kill the pid it prints", and it should say so plainly rather than leaving a gap an operator fills with `pkill -f 'openspec-doc serve'` — which has already killed the wrong process during this project's development. Revisit this line when that change lands.
- [x] 7.4 Document where the port registry lives and that `serve forget` is how an entry is removed, in the same place the assignment behaviour is described rather than in a separate corner.
