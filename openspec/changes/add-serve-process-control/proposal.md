## Why

Once `add-dashboard-lifecycle` lands, dashboards start themselves. Nothing lets the operator see them or stop them.

That is tolerable while every server is one you started by hand in a terminal you can still see. It stops being tolerable when hooks start them detached, in a fresh process group, across a range of thirty-two ports, for however many checkouts you happen to have open. A dashboard survives the hook that spawned it by design — that is the point of detaching — and the idle exit is a thirty-minute backstop, not a control. Between those two facts sits a half-hour window in which an operator can have several servers running, on ports they did not choose, for roots they may not remember, with no way to enumerate them and nothing to do about it but `pkill` and hope.

This is not hypothetical. Developing this project, the author has repeatedly lost track of hand-started servers, resorted to `pkill -f 'openspec-doc serve'`, and killed the wrong thing — including, on one occasion, a shell.

The gap is narrow and the fix is small, because `add-dashboard-lifecycle` already builds every part of it. `/identity` reports the canonical root and the pid. The port range is bounded and global to the machine rather than per-project, so probing it enumerates every dashboard running anywhere. The probe itself is already written. What is missing is a command that runs the probe across the range and prints the answer.

## What Changes

- Adds `serve list`: probes the whole port range and prints every dashboard found, with its port, the canonical project root it serves, and its pid.
- Adds `serve kill`, which **requires a target** — a project root, a port, or a pid — with `--all` for the deliberate case of stopping everything. A bare `serve kill` that stops every dashboard on the machine is the command someone runs by accident while a colleague's review is open on another checkout; the convenience is not worth the failure.
- `serve kill` **re-probes after signalling** and reports what is actually gone, rather than inferring success from having sent a signal. The pid was learned over a socket, so it races a dying process and may already be stale.
- Both commands report the limit rather than hiding it: a dashboard started by hand on a port outside the range is invisible to them. This is inherited from stateless discovery and is the honest trade for a design that survives `git clean`, a reboot, and a `SIGKILL`.
- Neither command needs a project root. `serve list` run anywhere lists every dashboard on the machine, which is what makes it useful for the "which of these is which" problem.
- `serve list` also shows **ports assigned to a project but not currently serving**, distinguished from the running ones. `add-dashboard-lifecycle` gives every project a port it keeps, so the question "which port is this checkout on" now has an answer whether or not anything is running, and an operator looking at a table of four running dashboards should be able to see that a fifth project owns a port too.
- Adds `serve forget <root>`, which drops a project's port assignment. This exists because assignment can fail: the range is finite, and when every port is assigned to a root that still exists, `add-dashboard-lifecycle` fails loudly and names this command. Without it that error names no way out, which makes it a dead end rather than a diagnosis.

Deliberately **not** in scope: no record of *what is running* — no port-and-pid file — for the reasons `add-dashboard-lifecycle`'s design sets out at length: a gitignored record of port and pid is deleted by `git clean -xdf` while its server keeps running, manufacturing the exact fleet of undiscoverable orphans this command exists to prevent. The port assignments these commands read are a different thing entirely, and neither `list` nor `kill` treats them as evidence that anything is running: both learn that from the network, every time. No `serve restart`, which is `kill` followed by the hooks doing what they already do. No management of dashboards on remote hosts.

## Capabilities

### Modified Capabilities

- `dashboard-lifecycle`: gains the operator's half. The capability already covers when a dashboard starts, how a running one is discovered, how long it lives and when it exits on its own; this adds enumerating them and ending one on purpose. Same capability, because both halves are answers to "which dashboards exist and why" and both are built on the same identity probe — splitting them into two capabilities would put the probe's two consumers in different specs.

## Impact

- `crates/cli/src/cli.rs` — `serve` gains two subcommands, which makes `serve` itself a command with children rather than a leaf. Check that a bare `openspec-doc serve` still starts a server rather than printing help, since every hook and every doc invokes it that way.
- `crates/cli/src/serve.rs` — the two new commands.
- The discovery module `add-dashboard-lifecycle` introduces — reused, not reimplemented. If this change finds itself writing a second probe, the seam is in the wrong place and that module should be widened instead.
- `docs/docs/reference/cli.md`, `docs/docs/reference/hooks.md` — the new commands, and the out-of-range limit stated where someone hunting a stray server will read it.

**Depends on `add-dashboard-lifecycle`** and must be sequenced after it. Without the identity route there is nothing to probe, and without the fixed port range there is nothing to probe *across*: today's ephemeral `--port 0` default makes a running server undiscoverable by construction, which is the same property that rules out the state file.
