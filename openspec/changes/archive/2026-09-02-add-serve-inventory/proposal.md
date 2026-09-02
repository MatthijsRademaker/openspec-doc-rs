## Why

`add-dashboard-lifecycle` shipped. Dashboards now start themselves, detached, on a port each project keeps, and nothing lets the operator see them.

That gap has a caller waiting on it *today*, not hypothetically. `crates/core/src/error.rs` tells a user who has exhausted the port range to "forget one with `openspec-doc serve forget`". That command does not exist. The error the lifecycle change added to avoid a silent collision currently names a dead end, and it has named one since the day that change landed.

The rest is the ordinary version of the same gap. Every project owns a port whether or not anything is serving it, so "which port is this checkout on, and is anything on it" is now a question with an answer — and `serve url` answers it for one project, from inside that project. Nothing answers it for the machine. An operator with four checkouts open has four dashboards they cannot enumerate.

This is small because `add-dashboard-lifecycle` already built all of it. `crates/cli/src/discovery.rs` has the probe, the bounded range, and the registry. What is missing is running the probe wide instead of narrow and printing the result.

## What Changes

- Widens the discovery module to enumerate every dashboard in the range, with `ensure` filtering that result for a matching root. One probe, run wide instead of narrow. If this change writes a second probe, the seam is in the wrong place.
- Probes the range concurrently, a thread per port. This matters more for enumeration than the narrow case it is shared with: `find` short-circuits on the assignment and usually costs one probe, while `list` always costs thirty-two.
- Adds `serve list`: every dashboard found, with its port, canonical root, and pid, plus the ports assigned to a project with nothing serving them, plus the range that was searched. Requires no project root, so it works from anywhere.
- Adds `serve forget <root>`, dropping a project's port assignment, refused while a dashboard for that root is serving.
- Corrects `crates/core/src/error.rs` so the comment above `PortRangeFull` names the change that actually delivers `serve forget`.
- Documents how to stop a dashboard. The project currently offers no answer at all, which is how an operator arrives at `pkill -f 'openspec-doc serve'` and takes out a shell — as happened during this project's development.

Deliberately **not** in scope: stopping a dashboard. That is `add-serve-process-control`, which depends on this change for the enumeration it needs to resolve a target. It is separated because every hard question in that area — how a stop is delivered, what a stop confirms, what a target may be named — is a question `list` and `forget` do not have to answer, and holding two unambiguous commands behind them is what kept both unbuilt for a month.

Also not in scope: any record of *what is running*. No port-and-pid file, for the reasons `add-dashboard-lifecycle`'s design sets out at length — a gitignored record is deleted by `git clean -xdf` while its server keeps running. The registry this reads holds *assignments*, and `list` never treats an assignment as evidence that anything is running. That distinction is this change's single most important invariant, because `list` is exactly where the rejected design would re-enter.

## Capabilities

### Modified Capabilities

- `dashboard-lifecycle`: gains enumeration. The capability already covers how one dashboard is discovered, how a port is assigned, and when a dashboard exits on its own; this adds seeing all of them at once and dropping an assignment. Same capability, because it is the same probe and the same registry.

## Impact

- `crates/cli/src/discovery.rs` — the probe widens to enumerate, and gains concurrency. `find_in` becomes a filter over the enumeration rather than its own loop.
- `crates/cli/src/serve.rs` — the two new commands.
- `crates/cli/src/cli.rs` — `ServeCommand` gains `List` and `Forget` beside `Url`. The seam that made `serve` a parent command is already built and already tested, so this is two variants and no risk to bare `serve`.
- `crates/core/src/dashboard.rs` — reading the whole registry, and dropping one entry.
- `crates/core/src/error.rs` — the comment above `PortRangeFull`, which currently names the wrong change.
- `docs/docs/reference/cli.md`, `docs/docs/reference/hooks.md` — the new commands, and the out-of-range limit stated where someone hunting a stray server will read it.

**Depends on `add-dashboard-lifecycle`**, which is archived. There is no remaining blocker.
