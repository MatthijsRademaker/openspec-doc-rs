# Design

## Discovery is stateless, and that is the load-bearing decision

The obvious design is a state file: the server writes `.openspec-doc/serve.json` with its host, port and pid, and a later hook reads it to decide whether to start one. This was the first recommendation during exploration, and it is wrong.

`serve.json` is machine-local runtime state, and `.openspec-doc/` is otherwise durable review history that belongs in the repo — so the file has to be gitignored. **Which means `git clean -xdf` deletes it while its server keeps running.** So does `rm -rf .openspec-doc`, which this project's own verification does routinely.

```
  server running on :51823, serve.json on disk
        │
   git clean -xdf   ──►  file gone, server ALIVE and now unfindable
        │
   next turn boundary ──►  no state ⇒ start ⇒ second server on :51907
        │
   git clean -xdf      ──►  repeat
        ▼
  N orphans, each holding filesystem watchers, none discoverable,
  none killable by anything we could ship — because the only record
  of their port and pid was the file that was deleted
```

A single lingering process is a nuisance. A fleet of undiscoverable ones is a defect, and the state file manufactures it: it exists to answer "is one already running" and answers wrongly in exactly the case where the answer matters.

So discovery asks the network instead:

```
  ensure_dashboard(root)
        │
        ├─ for port in 4321..=4330:
        │     GET /identity  (250ms timeout)
        │       │
        │       ├─ answers with MY canonical root  ──►  reuse it, done
        │       ├─ answers with another root       ──►  another checkout, next port
        │       ├─ answers something else          ──►  not ours, next port
        │       └─ nothing listening               ──►  spawn here
        │
        └─ after spawning: poll /identity until it answers (~2s)
                 │
              still nothing  ──►  report the failure, name serve.log
```

Nothing to lose to `git clean`, `rm -rf`, a reboot, or a `SIGKILL`. `Project::at` already canonicalises the root, so the comparison is exact rather than heuristic.

The cost is that `--port` can no longer default to `0`. An OS-chosen ephemeral port is undiscoverable by a later process by construction, which is precisely the property that broke the state-file design. `4321` is the base because `.pi/prompts/opsx-review.md` already uses it. The range is bounded at ten; exhausting it is a loud, diagnosable error, which is the correct outcome — unlike a silent eleventh duplicate.

### Listing and killing dashboards needs no state either, and that is why it was wrongly excluded

The original scope line excluded "no state file recording the port or pid, no `serve stop`" as one item. That conflated two things. The state file is rejected on the argument above and stays rejected. A list-and-kill command is a different proposition, because everything it needs is already being built here:

- `/identity` reports the canonical root and the pid. The root is the working directory an operator wants to see; the pid is what makes stopping one possible.
- The port range is bounded and **global to the machine** — `4321`–`4330` on localhost is not per-project — so probing all ten enumerates every dashboard running anywhere, which is exactly what "list them all" means.
- The probe is the same fixed localhost `GET` this change has to write regardless.

So `serve list` is ten probes and a table, and `serve kill` is that plus a signal. Nothing on disk, nothing to lose to `git clean`, correct after a reboot or a `SIGKILL`. It is the complement of stateless discovery rather than an exception to it.

Two limits it inherits and must state rather than hide: a dashboard started by hand outside the range — `--port 9999` — is invisible to it; and a pid learned over a socket races a dying process, so the command must re-probe and report what is actually gone rather than inferring success from having sent a signal.

It lands in `add-serve-process-control`, immediately after this change, because this one is already large and carries the riskiest live verification in the project. The dependency runs one way, so nothing is built twice.

### The concurrent-start race resolves itself

Two sessions ending simultaneously both find 4321 free and both spawn. One loses the bind and exits with a bind error into its own log; the other serves. Both hooks then poll `/identity` on that port, and both see a dashboard for their root. The loser of the race is not an error path that needs handling — the post-spawn poll checks *that a dashboard is there*, not *that our child is the one that put it there*.

## Lifetime: the heartbeat is the part that makes an idle exit correct

An idle exit keyed on "no page is subscribed" is wrong, and the review loop's own asynchrony is what breaks it:

```
  t+0    /opsx:explore ──► note location readied, nothing started
  t+2    first turn ends ──► dashboard up, tab opens on the note
  t+3    owner skims the note, closes the tab, walks away
  t+33   idle exit fires. Dashboard gone.
  t+40   agent reaches a turn boundary. No verdict — nobody could submit one.
  t+55   owner returns, clicks the bookmark ──► dead page
```

Routing feedback through a turn-boundary hook rather than a chat message exists *because* the reviewer's attention is asynchronous. An idle timer that assumes the reviewer is present whenever the dashboard matters has the premise backwards.

The missing input is session liveness, and it needs no new mechanism: `openspec-doc hook stop` already runs at every turn boundary of every session, on both agents. It has to probe the dashboard anyway to decide whether to start one, so **the probe is the heartbeat**. The idle condition takes both inputs:

```
  exit when   no SSE subscriber for 30 minutes
        AND   no hook heartbeat for 30 minutes
```

The subscriber half needs nothing new either. `watch.rs` already carries the comment `// Fails only when nobody is watching that scope's page right now`, because `broadcast::Sender` tracks its receiver count; the SSE endpoints run with `KeepAlive`, so an open tab holds a live receiver.

What this buys over the alternative:

| | Idle exit + heartbeat | Session-exit hook + refcount |
| --- | --- | --- |
| Cross-process coordination | none | a shared counter or registry |
| Crash safety | inherent — the timer is in the process that must die | a `SIGKILL`ed session leaks a reference forever |
| Agent symmetry | total; lives in the server, knows nothing about agents | a handler in Claude Code **and** in pi |
| Clears pre-existing orphans | yes, they time out | no, they predate the refcount |
| Recovers a crashed dashboard | yes, the next heartbeat restarts it | no |

pi's own extension documentation asks for the outcome this produces:

> Do not start background resources such as processes, sockets, file watchers, or timers from the factory. Defer background resource startup until `session_start` or the command/tool/event that needs the resource. Register an idempotent `session_shutdown` handler to close any session-scoped resources you start.

We satisfy the first half — the dashboard starts from the event that needs it. We decline the second, not because the dashboard is project-scoped rather than session-scoped (that is a lawyerly reading), but because the table above shows `session_shutdown` to be the weaker mechanism on every axis, and the idle exit achieves what pi is actually asking for without the refcount.

### Why a hand-started server does not idle out

`serve` run by hand was explicitly asked for by a human who is standing there. Exiting under it is a surprise. A hook-started one was asked for by nobody, which is what earns it a deadline. Hence `--idle-exit`, passed by the hook and by nothing else. One flag, and it is what makes the process's ownership explicit rather than implicit.

### What lingering actually costs, for the record

Two fears turned out to be unfounded and one turned out to be real, which is why the idle exit is worth building at all:

- **The watcher does not watch the repo.** `scope::target` watches `<root>/.openspec-doc` plus a change's own directory. `target/` is never watched, so a lingering server does not react to `cargo build`.
- **Nothing goes stale.** `Project` is three `PathBuf`s and every route reads the filesystem per request. A server up for a week serves today's changes.
- **But `Hub` never evicts.** `scopes: Mutex<HashMap<Vec<PathBuf>, Watched>>` is inserted into and never removed from, so watchers and their `coalesce` tasks accumulate one per distinct scope ever opened and stay after every tab closes. Worse on the fallback path, where `spawn_polling` runs a 1 Hz recursive `stat` walk forever with no subscriber check. Bounded by scopes-ever-visited, which in a process with no end is unbounded in practice.

Fixing the hub's eviction is the alternative to an idle exit, and it is the wrong fix: it makes an immortal process cheaper instead of making it mortal, and it leaves every other lingering cost — the port, the confusion about which dashboard is which — untouched.

## Which turn boundaries ensure a dashboard

Not all of them. A session where the owner asked one question about one function should not leave a daemon behind. The filter already exists and is already load-bearing: **ensure the dashboard only when this session has something to review** — its scratch note exists, or it was promoted to a change. `hook stop` already calls `scratch::check` on this path, and "a note exists if and only if an exploration actually started" is the invariant `promote::check` depends on.

The effective trigger therefore remains `/opsx:explore`, since that is what causes the note to be written. What changes is that the dashboard now *persists* across every later turn boundary instead of firing once and hoping the process survives — and that the trigger is observed one turn later, at the boundary where the note actually exists, rather than at the command itself.

## Duties split between the two hooks

| | `hook explore` | `hook stop` |
| --- | --- | --- |
| Register the session | no | yes, when it has review material |
| Ensure a dashboard | no | yes, when it has review material |
| Open a browser | never | once, on the turn boundary that first registers the session |
| Heartbeat | n/a | that is the point |

### Why not at the explore command, which is where this started

The first version of this table put both the start and the browser on `hook explore`, reasoning that the explore command is the moment the reviewer asked, and that a tab opening at a turn boundary is a jump scare — focus stolen while they read the agent's output, for a page they did not ask for.

That reasoning was about *focus*, and it missed a plainer problem about *content*. `hook explore` readies the note's location and deliberately does not create the file, because an empty placeholder makes the agent's first write fail. So at the instant the explore command returns, the session has no artifact. Checked against a running dashboard: the page renders zero artifacts and says "No artifact on disk to review yet."

It does not fix itself either. The live-update path refetches only the comments-and-verdicts fragment; the artifact is never refetched, which `add-live-artifact-updates` exists to fix. So a tab opened at explore time shows an empty page, stays empty while the exploration is written, and gives no sign that reloading would help. That is worse than no tab: it is a dashboard that appears to have nothing to say about the exploration you just started, which is one step from the silent no-op this whole change exists to remove.

One turn later, the note is on disk. So the browser goes to the first turn boundary at which the session has review material — which is also, not coincidentally, the first boundary at which it gets registered and becomes discoverable at all.

The focus objection survives in weakened form and is answered by *bounding*, not by timing. Opening is tied to **first registration**, so it happens exactly once per session:

- a second `/opsx:explore` in the same session finds the record already there and opens nothing;
- a second *session* against the same root gets its own tab for its own page, which is what you want and what the old reuse-suppression rule got wrong — it would have suppressed the tab merely because someone else's dashboard was already up;
- a dashboard that dies and is restarted at a later turn boundary opens nothing, because the session is long since registered.

That last point is why "first registration" beats the more obvious "when a dashboard was actually started": the two agree on the common case and disagree on both of the interesting ones.

No new state is needed for this. "Was this session already registered" is answered by whether its directive record existed before this invocation, which the hook is reading anyway.

## pi needs no new code, and that is a consequence, not a goal

`.pi/extensions/openspec-doc-hook.ts` already calls `hook stop --agent pi` at `agent_end`. Putting ensure-and-heartbeat there means pi gets the whole of this change for free.

During exploration this was bundled with a pi `input` handler, on the grounds that `hook explore` has no pi caller at all. That bundling was a mistake. The missing pi caller is a real defect — pi sessions get no scratch note, so pi explorations are invisible to the reviewer exactly as `add-explore-note-capture` describes — but it is a *pre-existing* defect about notes, not about the dashboard, and it belongs in its own change. Recorded there rather than lost: pi's `pi.on("input")` fires before skill and template expansion and sees raw input text, so the matcher must cover both `/opsx-explore` (the prompt template) and `/skill:openspec-explore` (the skill). `pi.registerCommand` must **not** be used for it: extension commands are checked before the input event and suppress it, so registering `opsx-explore` would shadow `.pi/prompts/opsx-explore.md` and silently break the existing prompt.

## Detaching without new dependencies

The spawned server must outlive the hook. Claude Code kills a hook that overruns its timeout, and a child in the hook's process group dies with it — the failure mode here works perfectly by hand and never survives a real hook invocation.

`std::os::unix::process::CommandExt::process_group(0)` puts the child in a fresh process group and is in std, so no `libc` dependency for `setsid`. Windows needs `creation_flags` with `DETACHED_PROCESS`.

The child's stdout and stderr go to `.openspec-doc/serve.log`, already covered by the `*.log` line in `.gitignore`. Not `/dev/null`: a server that cannot bind would then be a start that appears to succeed, a poll that times out, and no way to learn why — and the next turn boundary would repeat it identically, forever. That is the silent-failure shape this project's rules single out, and the log plus a reported timeout is what makes it loud.

The hook re-spawns itself via `std::env::current_exe()`, which also keeps the debug binary consistent with the one the hook configuration points at.

## Assumptions taken without an answer

The `move-to-proposal` verdict arrived with empty notes, leaving three questions from the exploration unanswered. Each is decided here and each is cheap to reverse:

1. **Autostart is built**, rather than the Rule 2 minimum of probing and printing the command for the owner to run. The manual step is the step that gets forgotten, which is what motivated the request; the do-nothing option was offered explicitly during exploration and not taken.
2. **A browser opens only from `hook explore`, only on an actual start.** Argued above.
3. **The idle window is 30 minutes, with no flag.** Long enough to cover a reviewer reading a page whose tab a browser may have frozen; short enough that an abandoned dashboard does not outlive the working day. A flag to tune it is speculative until something wants a different value.
