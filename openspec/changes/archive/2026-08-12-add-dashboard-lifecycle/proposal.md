## Why

The review loop has a manual precondition nobody remembers: the owner must have `openspec-doc serve` running in another terminal before any of it works. `AGENTS.md` documents this, and forgetting it is invisible — the explore hook still readies the note, the agent still writes its exploration, the turn boundary still checks for a verdict, and there is simply no reviewer at the other end. The loop does not fail; it silently never starts.

The predecessor tool already tried to fix this the wrong way, and `.pi/prompts/opsx-review.md` still carries the attempt: it asks the *agent* to run `openspec-doc serve`, to determine whether port 4321 "is already serving this project", to pick another port on conflict, and to "keep the process running while the user reviews". Every one of those is a deterministic decision with an exact right answer, routed through a language model, executed through a bash tool that either blocks until timeout or orphans its child. Port conflict resolution and liveness detection belong in the binary.

## What Changes

- The dashboard is started by the hooks, not by hand — and specifically by `hook stop`, at the first turn boundary of a session that has something to review, then kept alive at every subsequent one. `hook explore` does not touch it.
- The browser opens **once per session, at that session's own page**, on the turn boundary where the session is first registered. Not when the exploration starts: at that moment the note does not exist — `hook explore` readies its location and nothing more — so the page renders no artifact, and the artifact is not refetched on live-update, so the tab would sit empty until reloaded by hand. One turn later the note is there.
- Registration stops being unconditional. Today every session gets a directive record whether it explored or not, so the index lists sessions with nothing on them; the same review-material predicate now gates registering, ensuring a dashboard, and therefore listing.
- **Records already written for never-explored sessions are not migrated.** Nothing goes back over `.openspec-doc/directives/_session/` to delete the ones the old unconditional rule created, so an existing checkout keeps listing sessions that this change would not have registered — this repo has four records and three notes, and `3631c68a` explored nothing. `.openspec-doc/` is local and disposable, so deleting the stale records by hand is the answer rather than a migration nobody would run twice. Stated here because the alternative is a silent inconsistency between old and new records, where the index disagrees with the rule that now governs it.
- A running dashboard is found by asking the network, not by reading a file. `serve` gains an identity route reporting the canonical project root it serves and its pid; discovery probes a port range and reuses the server whose root matches.
- **Each project gets its own port, and keeps it.** A machine-global registry maps canonical project root to a port in `4321`–`4352`, assigned in ascending order the first time a root is seen, so a project's URL is the same today as it was last week regardless of which checkouts started first. This replaces the current ephemeral `--port 0` default: an ephemeral port cannot be discovered by a later process, and a port that depends on start order cannot be bookmarked, printed in advance, or stated by an agent.
- **The registry is a hint, never an authority.** Liveness is still asked of the network, always. The assigned port is probed first because it is nearly always right; before binding anything, the whole range is swept for this root and the entry repaired if the dashboard turns out to be elsewhere. Deleting the registry therefore costs stability and never correctness — which is precisely the test the rejected `.openspec-doc/serve.json` design fails, and why the two are not the same proposal.
- **The URL becomes something that can be stated before the server exists.** `serve url` prints this project's URL and whether a dashboard is live on it, answering correctly with nothing running. `hook explore` prints that URL beside the note path — possible only because the port is now predictable a full turn before the dashboard is started — and `AGENTS.md` tells the agent to answer with it when asked.
- A hook-started dashboard exits once nothing needs it: no page subscribed *and* no hook heartbeat for 30 minutes. A dashboard started by hand does not, because it was asked for.
- A dashboard that fails to start says so: the spawned server's output goes to `.openspec-doc/serve.log`, and a start that does not come up is reported through the hook rather than swallowed.

Deliberately **not** in scope: **no file recording which dashboard is running, on what port, under what pid** — the port registry records an assignment, not a process, and is verified against the network before it is acted on. Also out: no session-exit hook, no reference counting, and no new pi extension code. The reasoning for each is in `design.md`.

Listing and stopping running dashboards is **`add-serve-process-control`**, sequenced immediately after this change. It was originally excluded here alongside the state file, which conflated two things: the state file is rejected on its merits, but a list-and-kill command built on the identity probe needs no state at all. It is separated rather than folded in because this change is already large and carries the riskiest live verification in the project, and because the dependency runs one way — that change consumes the discovery module this one introduces.

## Capabilities

### New Capabilities

- `dashboard-lifecycle`: which port a project's dashboard belongs on, when the dashboard is started, how a running one is discovered and reused, how it survives the process that spawned it, how long it lives, and how a failed start is surfaced.

### Modified Capabilities

- `dashboard-server`: the default port stops being an OS-chosen ephemeral one and becomes the port assigned to this project, stable across restarts; the server gains an identity route and an opt-in idle exit.

## Impact

- `crates/cli/src/hook.rs` — both hook commands gain the ensure-the-dashboard step; `hook explore` also prints the project's URL.
- `crates/cli/src/cli.rs` — `serve` gains `--idle-exit` and a `url` subcommand; the `--port` default changes.
- `crates/cli/src/serve.rs` — `serve url`.
- `crates/server/src/lib.rs`, `routes.rs`, `watch.rs` — the identity route, the bind, and the idle-exit task reading subscriber counts the hub already tracks.
- A new module in `crates/core` owning the port registry: read, assign, reclaim, and the state-directory location. Core because which port belongs to which root is a domain rule, and because core already has `serde_json` where the CLI does not.
- A new module in the CLI owning discovery and detached spawning. It hand-rolls a fixed localhost `GET` over `std::net::TcpStream` rather than adding an HTTP client, which the CLI has no dependency for today.
- **One new dependency, `dirs`**, for the platform state directory. Weighed against the hand-rolled probe in `design.md`: the rule is that a dependency must be smaller than the correctness risk of doing it by hand, and three platforms' directory conventions fail that test where a fixed `GET` passes it.
- `openspec/specs/dashboard-server/spec.md` — the port requirement.
- `AGENTS.md`, `README.md` — "start `serve` first" stops being a precondition the owner has to remember, and the agent is told to answer with this project's URL when asked. `docs/docs/reference/cli.md` currently documents `--port` as defaulting to `0`; that line becomes false the moment this lands.
- `.pi/prompts/opsx-review.md` — its start-the-viewer step becomes obsolete and should be deleted rather than left contradicting the binary.

Two corrections to this proposal's earlier text, found while revising it: there is **no `QUICKSTART.md`** in this repository, so the previous impact list sent the implementer to a file that does not exist; and the claim that `.gitignore`'s `*.log` line already covers `.openspec-doc/serve.log` is stated as fact here but only as a thing to check in `tasks.md` — checking it is correct, asserting it was not. Checked during implementation: `.gitignore` carries both `*.log` and `.openspec-doc/`, so it is covered twice and needed no new entry.

**Three decisions were taken without an answer from the reviewer**, since the verdict arrived with empty notes. Each is stated as an assumption here and argued in `design.md`: autostart is built rather than the do-nothing alternative of printing the command for the owner to run; a browser is opened once per session, from `hook stop`, at the turn boundary that first registers it — not from `hook explore`, and not gated on whether a dashboard was actually started; and the idle window is fixed at 30 minutes with no flag to tune it.

This change is not complete on passing tests alone. Detachment, the browser moment, and the idle exit are all things that pass in a harness and fail in a real session, so it carries live-verification tasks that cannot be inferred from a green test run.
