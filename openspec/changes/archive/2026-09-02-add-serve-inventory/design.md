# Design

## The whole change is one probe, run wide instead of narrow

`add-dashboard-lifecycle` shipped `find_in` in `crates/cli/src/discovery.rs`: chain the assignment onto the range, and `.find()` the first identity whose root matches. It stops at the first match.

This change runs the same loop and keeps every answer.

```
  ensure(project)                 serve list
        │                              │
   probe 4321..4352              probe 4321..4352
        │                              │
   first match wins  ──►          collect all  ──►  port | root | pid
   nothing found ⇒ spawn          nothing found ⇒ "no dashboards running"
```

`find_in` becomes a filter over the enumeration rather than a second loop. That is the entire mechanism, and it is the reason this change is small: the hard parts — the hand-rolled `GET`, the timeouts, the identity parse, the "anything that is not an identity is a no" rule — are already written and already tested.

## Concurrency, and where the argument actually applies

The predecessor change probes sequentially and that has been fine. It is fine because `find` almost always answers on its first probe: the assignment is tried first and is nearly always right. The full sweep only happens on the path that is about to spawn a process anyway.

`serve list` has no such short-circuit. It probes all thirty-two ports every time, and it is interactive.

The worst case is bounded by `CONNECT_TIMEOUT + READ_TIMEOUT` — 400ms — and only a port that *accepts a connection and never answers* costs it. Thirty-two of those is roughly twelve seconds, which is unusable. Closed ports refuse instantly, so the common case is milliseconds either way.

A thread per port, joined with `std::thread::scope`, is about five lines and makes the worst case one timeout instead of thirty-two. It goes in the shared function rather than in `list`, so `ensure`'s miss path gets it too — but the honest statement of the benefit is that it is for `list`, and `ensure` is along for the ride.

## Assignments are shown beside the running dashboards, never instead of them

The registry answers *which port does this project own*. The probe answers *what is running*. `serve list` prints the join, and the second column is never inferred from the first:

```
  PORT   ROOT                          PID     STATE
  4321   ~/repos/openspec-doc-rs       84213   running
  4322   ~/repos/acme-api              91004   running
  4323   ~/repos/old-spike             —       assigned, not running
  4327   ~/checkouts/scratch           88771   running, not its assignment
  ↑ probed 4321–4352
```

Three things fall out of that table that a probe-only list could not show. A project that owns a port but has nothing running is visible, which answers "where would this checkout appear". A dashboard sitting somewhere other than its assignment is visible as an anomaly rather than looking normal — that is the state `ensure` repairs on its next run, and seeing it is how an operator notices the repair path is not firing. And a row with no assignment at all is a dashboard started by hand with `--port`, worth distinguishing from one the hooks placed.

A row is only ever marked running because a probe of that port answered. Losing the registry costs this table its assigned-not-running rows and nothing else.

## `serve forget` has a caller waiting

`crates/core/src/error.rs` already reads:

> every port in 4321-4352 is assigned to a project that still exists: forget one with `openspec-doc serve forget`, or remove its entry from …

with a source comment saying the command lands in `add-serve-process-control` and that whichever of the two is touched second checks the other. That comment is now wrong twice over: the command has moved to this change, and nothing has been touched second, so the error has been naming a nonexistent command since the day it shipped. Correcting the comment is a task here, not a later cleanup — the text this change falsifies moves with it.

`forget` takes a root, not a port: the operator's question is "I do not use that checkout any more", and answering it by port makes them look something up they did not ask about. Forgetting a root whose dashboard is currently serving is refused — the assignment would be handed to another project while a server still sits on that port, and the next `ensure` for the forgotten root would find its own dashboard on a port belonging to someone else. Stop it first.

## What `list` prints when there is nothing

"No dashboards running" and exit zero. Not an error: no dashboards is the ordinary state of a machine between review sessions, and an operator running `serve list` to check is getting the answer they asked for. Exit non-zero only when the probe itself could not run.

## The out-of-range blind spot is stated, not solved

A dashboard on `--port 9999` does not answer within `4321`–`4352` and will not appear. Every alternative reintroduces something worse: scanning all 65535 ports is slow and hostile; reading `/proc` or `ps` is platform-specific and matches on a command string, which is the `pkill` approach this work exists to replace; a state file is rejected at length in `add-dashboard-lifecycle`'s design.

So it is a documented limit. `serve list` names the range it searched in its own output, so an operator who does not find what they expect sees immediately why, rather than concluding the tool is broken.

## Why this is not one change with `serve kill`

It was, for a month, and nothing was built.

`list` and `forget` have no open questions. `kill` has, or had, five: what a target may be called given `--root` is already a global flag, whether an untargeted invocation may default to the current project, how a stop is delivered, what platform that delivery works on, and what a stop is allowed to claim afterwards. None of those questions touch enumeration, and enumeration is the part with a broken error message pointing at it.

The dependency runs one way — `kill` resolves its target by enumerating, so it consumes this change and nothing is built twice. That is the same shape as this change's own dependency on `add-dashboard-lifecycle`, and it worked there.
