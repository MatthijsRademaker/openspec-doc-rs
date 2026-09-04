# Design

## Stop it over HTTP, not with a signal

The original version of this design spent its longest section mitigating pid reuse, in three layers, and its third layer read: *prefer stopping by port over pid, because a port that stops answering is direct evidence while a vanished pid is consistent with the pid having been reused*. It then built the command on signals anyway, and mitigated the hazard it had just described as avoidable.

The dashboard already answers HTTP on the port. Asking it to stop through that port is the same preference taken to its conclusion:

```
   signal path                        HTTP path
   ───────────                        ─────────
   probe → learn pid                  probe → learn port
   kill(pid, SIGTERM)                 POST <port>/api/shutdown
   ↑ pid may already be reused        ↑ the port is the identity
   ↑ needs libc / nix / rustix        ↑ hand-rolled POST, no dependency
   ↑ no SIGTERM on Windows            ↑ identical on every platform
   ↑ tells the process to stop        ↑ the process stops itself
   re-probe → confirm                 re-probe → confirm
```

What it buys, in order:

- **The pid-reuse hazard does not exist.** It is not narrowed by probing late, it is absent, because no pid is ever used. Three paragraphs of mitigation become none.
- **No new dependency.** Rust's std cannot send a signal; `SIGTERM` costs `libc`, `nix`, or `rustix`, and this workspace has an explicit written test for adding a crate — it must be smaller than the correctness risk of doing the thing by hand. `discovery.rs` already notes it reached for `process_group(0)` specifically because it is in std. A hand-rolled `POST` beside the hand-rolled `GET` costs nothing.
- **Windows stops being a question.** The original task list carried an open item about what stopping means on a platform with no `SIGTERM`. There is no such item here.
- **The server chooses when to stop.** `axum`'s graceful shutdown already exists for the idle exit; the route resolves the same future. That is a more direct route to a clean shutdown than a signal handler would be, and the clean shutdown path is already written.

That last point came with a bill, and it is worth recording rather than smoothing over. The graceful shutdown was *written*, but it was only ever **exercised against a dashboard with nothing subscribed to it** — because the idle condition requires zero subscribers, so the drain had never had a connection to drain. A shutdown route removes that precondition. `axum`'s graceful shutdown waits for in-flight responses, and a server-sent-events stream is a response that never finishes on its own, so the first version of this stopped a dashboard nobody was looking at and hung indefinitely on one with a review page open — which is precisely the dashboard an operator is most likely to be stopping, and precisely the silent hang `serve kill` exists to replace.

The fix is small and belongs to the same mechanism rather than beside it: the event streams watch the shutdown too, and end when it begins. The drain then completes on its own with no deadline anywhere. It also turns the reviewer's side of the stop into a report — the page's stream closes and it says the live channel disconnected — where a hard-cut connection would have been indistinguishable from a network blip. The general lesson is narrower than "graceful shutdown is hard": **a path that exists is not a path that has been exercised, and the condition guarding the only caller is where to look for what it never had to handle.**

The cost, stated rather than discovered: **a dashboard so wedged it no longer answers HTTP cannot be stopped this way**, and that is exactly the dashboard an operator most wants to kill. This design does not build a signal fallback for it, on the same rule the predecessor applied to `SIGKILL` escalation — speculative escalation for a failure mode nothing has been observed to hit. `serve list` prints the pid, so the manual escape hatch is one `kill` away and the docs say so. If a wedged dashboard is ever actually seen, `--pid` and a signal is the follow-up change, and it will be justified by an observation instead of an imagination.

Confidence: high that this is the right primary mechanism, moderate that no fallback is needed. The second one is the one to revisit.

## A local page must not be able to shut your dashboard down

A route that stops a process, reachable from a browser, on a fixed and predictable port, deserves one paragraph rather than none.

The bind is `127.0.0.1`, so nothing off the machine reaches it. What remains is a page in the operator's own browser: a plain HTML form can `POST` cross-origin without a preflight, and the port is guessable within thirty-two tries. The blast radius is a stopped dev dashboard — annoying, not dangerous — but it is free to close. Require a header on the shutdown request that a simple form cannot set, which forces the browser into a CORS preflight it will not get past. The CLI writes its own request bytes, so setting a header costs one line there.

Do not reach for a token or a nonce. That needs somewhere to keep the secret, and "somewhere to keep per-server state" is the design this whole line of work rejected.

## Why a target is required, and what the targets are called

`serve kill` with no argument is one keystroke from stopping a dashboard on a checkout you are not looking at. The dashboards are global to the machine while the operator's mental model is per-project, so the natural reading of a bare `serve kill` — "stop the one for *this* project" — is not the destructive one, which means the destructive behaviour would be a surprise rather than a choice.

The original design listed `--root <path>` as the project target and left an open task asking whether a bare `--root` should default to the resolved project root. Both halves of that were wrong:

**`--root` is already taken.** `Cli` declares it `global = true`, so `openspec-doc serve kill --root /repos/x` parses today, as the project-root override that every other subcommand honours. A per-subcommand `--root` cannot be added beside it.

**And the global one already defaults to the resolved project.** That is what makes the open question unanswerable as posed: the convenient spelling the task was weighing is the one spelling indistinguishable from having given no target at all, which is the exact case the required-target rule exists to reject.

So the project target is its own explicit switch, taking no value:

| Target | The situation |
| --- | --- |
| `--project` | "stop the one for this project" — the common case. Resolves the project the way every other command does, honouring a global `--root` if one is given. |
| `--port <n>` | reading a `serve list` table |
| `--all` | "I have lost track, stop everything" |

`--project` is a flag rather than a default because typing it is the act of choosing. The behaviour still depends on the working directory, which is fine — so does every other command in this CLI — but it depends on it only once the operator has said "the project" out loud.

`--all` exists because the honest answer to losing track is a command that admits it, rather than an operator running `pkill -f openspec-doc` and taking out whatever else matches. That has already happened in this project's own development.

## A stop reports what is gone, not what was sent

Even without pids, a target is a snapshot. Between the enumeration that resolved `--project` to a port and the request, the dashboard may have exited on its own idle deadline, or been stopped by someone else, or — the interesting one — that port may now hold a *different* project's dashboard, because the first one exited and an `ensure` elsewhere fell forward onto it.

Three rules, in order of how much they buy:

1. **Enumerate immediately before asking**, not from a list the operator was shown some seconds ago. Narrows the window to milliseconds; does not close it.
2. **Send the root the request expects to be stopping**, and have the server refuse a shutdown naming a root it does not serve. This closes the fall-forward case outright, which the signal design could not do at any price: a pid is just a number, but a running server can check its own identity.
3. **Probe again afterwards** and report what is actually gone. This is what the command reports, rather than "sent a shutdown request to 4321".

Rule 2 is worth noticing as the second thing the HTTP path gets for free. The thing being stopped is the only participant that knows for certain what it is, and asking it is only possible because the mechanism is a conversation rather than a signal.

## What a stop does not touch

The port assignment. Stopping a dashboard says nothing about where that project belongs, and a kill that also forgot the assignment would move the project on its next start — which is the instability this whole line of work removed. `serve forget` is the command for that, and it is deliberately a separate act.
