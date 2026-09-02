## Why

`add-serve-inventory` lets an operator see every dashboard on the machine. Nothing lets them stop one.

That is tolerable while every server is one you started by hand in a terminal you can still see. It stops being tolerable when hooks start them detached, in a fresh process group, across a range of thirty-two ports, for however many checkouts you happen to have open. A dashboard survives the hook that spawned it by design — that is the point of detaching — and the idle exit is a thirty-minute backstop, not a control. Between those two facts sits a half-hour window in which an operator can have several servers running, on ports they did not choose, for roots they may not remember.

The honest version of that complaint, now that the lifecycle change has landed: the window is bounded and the servers are enumerable, so this is no longer the fleet-of-undiscoverable-orphans problem the original proposal described. What is left is smaller and still real. `serve list` prints a pid and the only thing an operator can do with it is `kill` it themselves, which is the manual step the tool exists to remove, and which is one typo away from the `pkill -f 'openspec-doc serve'` that has already killed a shell during this project's development.

## What Changes

- Adds `serve kill`, which **requires a target** — `--project`, `--port`, or `--all`. A bare `serve kill` that stops every dashboard on the machine is the command someone runs by accident while a colleague's review is open on another checkout; a bare `serve kill` that quietly stops *this* project's is a command whose behaviour depends on the working directory. Neither is allowed to be the default.
- Adds a shutdown route to the dashboard, beside the identity route it already answers, and stops a dashboard **over HTTP on its own port rather than by signalling a pid**. This is the change's one real design decision and it deletes most of the difficulty: no pid to be reused, no `SIGTERM`, no new dependency, no Unix-only caveat.
- `serve kill` **re-probes after asking** and reports what is actually gone, rather than inferring success from having sent a request.
- Reports a target that matched nothing as matching nothing, not as success.
- Leaves the port assignment alone. Stopping a dashboard says nothing about where that project belongs.
- Replaces `add-serve-inventory`'s interim documentation, which currently tells an operator to find the pid with `serve list` and kill it by hand.

Deliberately **not** in scope: `--pid` as a target, and signals of any kind. A dashboard so wedged it no longer answers HTTP is the only case a signal reaches and HTTP does not, and no such dashboard has been observed. This is the same rule the predecessor design applied to `SIGKILL` escalation — do not build it until something is seen to need it — applied one level up. `serve list` prints the pid, so the manual escape hatch exists and is honest about being manual.

Also not in scope: `serve restart`, which is `kill` followed by the hooks doing what they already do. No management of dashboards on remote hosts. No record of what is running, for the reasons `add-dashboard-lifecycle`'s design sets out at length.

## Capabilities

### Modified Capabilities

- `dashboard-lifecycle`: gains the last of the operator's half. The capability already covers when a dashboard starts, how a running one is discovered, how it is enumerated, and when it exits on its own; this adds ending one on purpose, and the route it is ended through.

## Impact

- `crates/server/src/routes.rs` — the shutdown route, beside `IDENTITY_PATH`.
- `crates/server/src/lib.rs` — the graceful-shutdown future currently resolves only on the idle condition; it gains a second trigger. The two must not be able to double-shutdown.
- `crates/core/src/dashboard.rs` — the route's path, beside `IDENTITY_PATH`, because the server and the CLI's hand-rolled client have to agree about it.
- `crates/cli/src/discovery.rs` — a hand-rolled `POST` beside the hand-rolled `GET`. Do not add an HTTP client for it; the reasoning that kept one out for the probe is unchanged.
- `crates/cli/src/serve.rs`, `crates/cli/src/cli.rs` — the command. Note that `--root` is **already a global flag** on `Cli`, so the project target cannot be spelled that way.
- `docs/docs/reference/cli.md`, `docs/docs/reference/hooks.md`.

**Depends on `add-serve-inventory`** and must be sequenced after it. Resolving `--project` or `--all` to a set of ports is enumeration, and this change must not build a second one.
