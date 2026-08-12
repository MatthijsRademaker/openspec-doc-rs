# Design

## The whole change is one probe, run wide instead of narrow

`add-dashboard-lifecycle` probes `4321`–`4352` looking for *one* answer: a dashboard whose canonical root matches mine. It stops at the first match and reuses it.

This change runs the same loop and keeps every answer instead of the first matching one.

```
  ensure_dashboard(root)          serve list
        │                              │
   probe 4321..4352              probe 4321..4352
        │                              │
   first match wins  ──►          collect all  ──►  port | root | pid
   nothing found ⇒ spawn          nothing found ⇒ "no dashboards running"
```

That is the entire mechanism. If implementing this requires a second probe, the shared function was scoped too narrowly and should be widened to return all identities, with `ensure` filtering — not copied.

## Assignments are shown beside the running dashboards, never instead of them

`add-dashboard-lifecycle` gives each project root a port it keeps, recorded in a machine-global registry. That changes what a useful `serve list` looks like, and it is worth being exact about how, because the obvious reading of "read the registry" is the mistake that change spent a section rejecting.

The registry answers *which port does this project own*. The probe answers *what is running*. `serve list` prints the join, and the second column is never inferred from the first:

```
  PORT   ROOT                          PID     STATE
  4321   ~/repos/openspec-doc-rs       84213   running
  4322   ~/repos/acme-api              91004   running
  4323   ~/repos/old-spike             —       assigned, not running
  4327   ~/checkouts/scratch           88771   running, not its assignment
  ↑ probed 4321–4352
```

Three things fall out of that table that a probe-only list could not show. A project that owns a port but has nothing running is visible, which is the "where would this checkout appear" question. A dashboard sitting somewhere other than its assignment is visible as an anomaly rather than looking normal — that is the state `ensure` repairs on its next run, and seeing it is how an operator notices the repair path is not firing. And a row with no assignment at all is a dashboard started by hand with `--port`, which is worth distinguishing from one the hooks placed.

A row is only ever marked running because a probe of that port answered. Losing the registry costs this table its assigned-not-running rows and nothing else.

## `serve forget` exists because assignment can fail

The range is finite, and `add-dashboard-lifecycle` fails loudly rather than colliding when every port is assigned to a root that still exists on disk. That error names `serve forget` as the way out. An error that names a command which does not exist is worse than the collision it prevented, so the two have to land together in spirit even though they land in different changes — whichever is implemented second checks that the error text and the command agree.

It takes a root, not a port: the operator's question is "I do not use that checkout any more", and answering it by port requires them to look the port up first. Forgetting a root whose dashboard is currently running is refused rather than allowed — the assignment would be handed to another project while a server still sits on it, and the next `ensure` for the forgotten root would find its own dashboard on a port belonging to someone else. Stop it first.

## Why a target is required

`serve kill` with no argument is one keystroke from stopping a dashboard on a checkout you are not looking at. The dashboards are global to the machine and the operator's mental model is per-project, so the natural reading of a bare `serve kill` — "stop the one for *this* project" — is not the destructive one, which means the destructive behaviour would be a surprise rather than a choice.

Three target forms, because the three ways an operator arrives at this command are different:

| Target | The situation |
| --- | --- |
| `--root <path>` (or defaulting to the resolved project root) | "stop the one for this project" — the common case |
| `--port <n>` | reading a `serve list` table |
| `--pid <n>` | reading `ps`, or a stale pid from somewhere else |
| `--all` | "I have lost track, stop everything" |

`--all` exists because the honest answer to losing track is a command that admits it, rather than an operator running `pkill -f openspec-doc` and taking out whatever else matches. That has already happened in this project's own development.

## Killing races the thing being killed

The pid comes from a socket read, which is a snapshot. Between the probe and the signal the process may have exited on its own idle deadline, been killed by someone else, or — worst — exited and had its pid reused by an unrelated process.

Pid reuse is the one that turns a convenience command into a hazard. Mitigation, in order of how much they buy:

1. **Probe immediately before signalling**, not from a list the operator was shown some seconds ago. Narrows the window to milliseconds; does not close it.
2. **Probe again after signalling** and report what is actually gone. This is what the command reports, rather than "sent SIGTERM to 12345".
3. **Prefer stopping by port over pid where both are known.** A port that stops answering `/identity` is direct evidence the dashboard is gone; a pid that no longer exists is weaker evidence, because it is consistent with the pid having been reused.

`SIGTERM`, not `SIGKILL`. The server should get its shutdown path. A follow-up `SIGKILL` after a timeout is deliberately not built until something is observed to ignore the first signal — speculative escalation for a process that has never been seen to hang.

## What `list` prints when there is nothing

"No dashboards running" and exit zero. Not an error: no dashboards is the ordinary state of a machine between review sessions, and an operator running `serve list` to check is getting the answer they asked for.

Exit non-zero only when the probe itself could not run.

## The out-of-range blind spot is stated, not solved

A dashboard on `--port 9999` does not answer within `4321`–`4352` and will not appear. Every alternative that closes this gap reintroduces something worse: scanning all 65535 ports is slow and hostile; reading `/proc` or `ps` is platform-specific and matches on a command string, which is the `pkill` approach this command exists to replace; a state file is rejected at length in `add-dashboard-lifecycle`'s design.

So it is a documented limit. `serve list` names the range it searched in its output, so an operator who does not find what they expect can see immediately why rather than concluding the tool is broken.
