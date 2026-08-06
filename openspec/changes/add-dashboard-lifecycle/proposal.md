## Why

The review loop has a manual precondition nobody remembers: the owner must have `openspec-doc serve` running in another terminal before any of it works. `AGENTS.md` documents this, and forgetting it is invisible — the explore hook still readies the note, the agent still writes its exploration, the turn boundary still checks for a verdict, and there is simply no reviewer at the other end. The loop does not fail; it silently never starts.

The predecessor tool already tried to fix this the wrong way, and `.pi/prompts/opsx-review.md` still carries the attempt: it asks the *agent* to run `openspec-doc serve`, to determine whether port 4321 "is already serving this project", to pick another port on conflict, and to "keep the process running while the user reviews". Every one of those is a deterministic decision with an exact right answer, routed through a language model, executed through a bash tool that either blocks until timeout or orphans its child. Port conflict resolution and liveness detection belong in the binary.

## What Changes

- The dashboard is started by the hooks, not by hand: `hook explore` starts it when an exploration begins, and `hook stop` keeps it alive at every subsequent turn boundary of a session that has something to review.
- `hook explore` opens the browser at **that session's own page**, and registers the session so the page exists. A session is discoverable only if it has a directive record, and only `hook stop` writes one today — so without this the tab opens on a 404, and the index does not list the session either. Registration moves to where an exploration actually starts, and stops being unconditional at the turn boundary, so a directive record comes to mean *this session explored* rather than *a turn ended in this project*.
- A running dashboard is found by asking the network, not by reading a file. `serve` gains an identity route reporting the canonical project root it serves and its pid; discovery probes a small port range and reuses the server whose root matches.
- `serve` binds `4321` by default, falling forward through `4321`–`4330`, replacing the current ephemeral `--port 0` default. An ephemeral port cannot be discovered by a later process, which is the whole problem.
- A hook-started dashboard exits once nothing needs it: no page subscribed *and* no hook heartbeat for 30 minutes. A dashboard started by hand does not, because it was asked for.
- A dashboard that fails to start says so: the spawned server's output goes to `.openspec-doc/serve.log`, and a start that does not come up is reported through the hook rather than swallowed.

Deliberately **not** in scope: no state file recording the port or pid, no session-exit hook, no reference counting, and no new pi extension code. The reasoning for each is in `design.md`.

Listing and stopping running dashboards is **`add-serve-process-control`**, sequenced immediately after this change. It was originally excluded here alongside the state file, which conflated two things: the state file is rejected on its merits, but a list-and-kill command built on the identity probe needs no state at all. It is separated rather than folded in because this change is already large and carries the riskiest live verification in the project, and because the dependency runs one way — that change consumes the discovery module this one introduces.

## Capabilities

### New Capabilities

- `dashboard-lifecycle`: when the dashboard is started, how a running one is discovered and reused, how it survives the process that spawned it, how long it lives, and how a failed start is surfaced.

### Modified Capabilities

- `dashboard-server`: the default port stops being an OS-chosen ephemeral one and becomes a discoverable base port with a fall-forward range; the server gains an identity route and an opt-in idle exit.

## Impact

- `crates/cli/src/hook.rs` — both hook commands gain the ensure-the-dashboard step.
- `crates/cli/src/cli.rs` — `serve` gains `--idle-exit`; the `--port` default changes.
- `crates/server/src/lib.rs`, `routes.rs`, `watch.rs` — the identity route, the fall-forward bind, and the idle-exit task reading subscriber counts the hub already tracks.
- A new module owning discovery and detached spawning. It hand-rolls a fixed localhost `GET` over `std::net::TcpStream` rather than adding an HTTP client to the CLI, which has no such dependency today.
- `openspec/specs/dashboard-server/spec.md` — the port requirement.
- `AGENTS.md`, `README.md`, `QUICKSTART.md` — "start `serve` first" stops being a precondition the owner has to remember.
- `.pi/prompts/opsx-review.md` — its start-the-viewer step becomes obsolete and should be deleted rather than left contradicting the binary.

**Three decisions were taken without an answer from the reviewer**, since the verdict arrived with empty notes. Each is stated as an assumption here and argued in `design.md`: autostart is built rather than the do-nothing alternative of printing the command for the owner to run; a browser is opened only by `hook explore`, and only when a dashboard was actually started; and the idle window is fixed at 30 minutes with no flag to tune it.

This change is not complete on passing tests alone. Detachment, the browser moment, and the idle exit are all things that pass in a harness and fail in a real session, so it carries live-verification tasks that cannot be inferred from a green test run.
