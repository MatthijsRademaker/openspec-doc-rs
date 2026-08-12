## 1. The port registry

- [ ] 1.1 Add a module in `crates/core` owning the root-to-port registry: locate the file, read it, assign, reclaim, write it back. Core rather than the CLI because which port belongs to which root is a domain rule, and because core already has `serde_json` where the CLI does not.
- [ ] 1.2 Locate the file at the platform state directory when there is one — `$XDG_STATE_HOME/openspec-doc/ports.json`, defaulting to `~/.local/state` — and the local data directory otherwise, since macOS and Windows have no state-directory convention. Add `dirs` for this. Do not hand-roll the platform tree: the failure mode is writing to the wrong place on an OS the author does not develop on, which no test on this machine will catch.
- [ ] 1.3 Make the state directory overridable by an environment variable, and use it in every test that touches the registry. **This is not a convenience.** Without it the suite writes into the developer's real assignments and two concurrent `cargo test` runs corrupt each other's.
- [ ] 1.4 Store a mapping from canonical root to port and nothing else. Derive the next free port from the recorded values; do not store a counter, which is a second source of truth that drifts from the table it describes.
- [ ] 1.5 Assign the lowest port in `4321`–`4352` not already assigned. Reclaim only when the range is full, and only entries whose canonical root no longer exists on disk. No LRU, no wraparound.
- [ ] 1.6 Fail loudly when the range is full and nothing is reclaimable, naming the registry file and `serve forget`. Note that `serve forget` itself lands in `add-serve-process-control`; the error text and that command have to agree, so whichever lands second checks the other.
- [ ] 1.7 Write atomically — temp file plus rename in the same directory — so an interrupted write cannot leave a half-written registry.
- [ ] 1.8 Treat a missing registry as normal and silent; treat a corrupt one as loud and non-fatal — report it naming the file, then continue with no assignment. Do not delete or rewrite a file you could not parse.
- [ ] 1.9 Tests: a new root gets the lowest free port; the same root gets the same port again; two roots get different ports; assignments hold while ports remain, including for roots that no longer exist; a full range reclaims a vanished root; a full range of live roots errors; a corrupt registry is reported and does not stop an assignment being skipped; the environment override is honoured.

## 2. The server side

- [ ] 2.1 Change `serve`'s `--port` default from `0` to the port assigned to the resolved project root. An explicit `--port` must neither read nor write the registry, must be honoured exactly, and must fail rather than fall forward.
- [ ] 2.2 On the default path, fall forward within the range when the assigned port is taken by something else, and record where it landed so the project is stable there from then on. An exhausted range must fail naming the range.
- [ ] 2.3 Add an identity route returning the canonical project root and the process id as JSON. Keep it off the page namespace so it cannot collide with a change or session name.
- [ ] 2.4 Add `--idle-exit` to `serve`, and an idle-exit task that ends the process after 30 minutes with no subscribed page and no registered activity. Read the subscriber count from the hub's existing `broadcast::Sender` rather than adding a second count to keep in step with it.
- [ ] 2.5 Register a request to the identity route as activity against the idle deadline, so a heartbeat and a start probe are the same request.
- [ ] 2.6 Tests: the default binds the project's assignment, the same project binds the same port across runs with other projects served in between, two roots bind different ports, a squatted assignment falls forward and rewrites the entry, an explicit taken port fails, an explicit port leaves the assignment untouched, an exhausted range fails, the identity route reports the canonical root, and idleness ends the process only with `--idle-exit` and only when both conditions hold. Drive the idle test on an injected clock or a shortened window — a test that waits 30 minutes will be deleted by whoever runs it next.

## 3. Discovery and detached start

- [ ] 3.1 Add a module owning ensure-a-dashboard. Probe the assigned port first, because it is nearly always right. **Then sweep the whole range for this root before binding anything**, reuse whatever is found, and repair the registry entry when the dashboard turns out to be elsewhere. Skip ports answering with another root or with something that is not a dashboard. Probe with a short connect and read timeout so a hung port cannot stall a hook.
- [ ] 3.2 Never bind on the strength of the registry alone. An implementation that trusts the entry and skips the sweep has rebuilt the rejected `serve.json` design with extra steps, and fails the same way: one deleted file becomes one duplicate server. Say this in a comment at the seam.
- [ ] 3.3 Hand-roll the probe as a fixed `GET` over `std::net::TcpStream`. The CLI has no HTTP client and this does not justify adding one; say so in a comment so the next reader does not "fix" it — and does not read the new `dirs` dependency as licence to add more.
- [ ] 3.4 Start the server detached via `std::env::current_exe()`, with `CommandExt::process_group(0)` on Unix and `creation_flags` with `DETACHED_PROCESS` on Windows, passing `--idle-exit` and `--no-open`.
- [ ] 3.5 Redirect the started server's stdout and stderr to `.openspec-doc/serve.log`. **Verify** that `*.log` in `.gitignore` covers it rather than assuming it; add an entry if it does not.
- [ ] 3.6 After starting, poll the identity route for up to ~2 seconds. Treat *a dashboard for this root answering* as success regardless of which process won the bind, so a concurrent start is not an error path. The loser of a race falls forward and records where it landed.
- [ ] 3.7 Report a start that never answers, naming `.openspec-doc/serve.log`. Do not fall back to serving nothing quietly.
- [ ] 3.8 Tests: reuse finds an already-serving dashboard on its assignment; reuse finds one on a *different* port and repairs the entry; a deleted registry still finds a running dashboard and starts nothing; a foreign root is skipped; a non-dashboard listener is skipped; an exhausted range reports; and a probe against a port that accepts but never responds times out instead of hanging. A test that binds real ports must take them from the OS and pass them in rather than assuming the real range is free on the machine running the suite — the developer's own dashboards are sitting in it.

## 4. Stating the URL

- [ ] 4.1 Add `serve url`: print the current project's dashboard URL and whether a dashboard is serving it. It must print the right URL with nothing running — that is the capability the registry exists to provide — and must report the actual port when a dashboard is found somewhere other than its assignment.
- [ ] 4.2 Have `hook explore` print that URL alongside the note location. This is the moment the reviewer asks to be shown the exploration and, before the registry, the moment at which no URL could be produced.
- [ ] 4.3 Keep the URL string built in one place, used by `serve`'s startup print, `serve url`, `hook explore`, and the browser-opening step. Four spellings of `http://{host}:{port}` is four chances to disagree about the trailing slash.
- [ ] 4.4 Tests: `serve url` with nothing running prints the assigned URL and reports it not running; with a dashboard running reports it running; with a dashboard on a non-assigned port reports the real one; and `hook explore`'s output contains the URL.

## 5. Wiring the hooks

- [ ] 5.1 `hook explore` does **not** touch the dashboard. It readies the note location, prints the instruction and the URL, and that is all. Nothing needs a dashboard yet: at that moment the note does not exist, so the session page renders no artifact — verified, it prints "No artifact on disk to review yet." and nothing selectable.
- [ ] 5.2 `hook stop`: ensure a dashboard only when the session has review material — its scratch note exists, or `scratch::promoted_to` names a change. This is where the dashboard starts.
- [ ] 5.3 Make registration at `hook stop` conditional on that same predicate, so a directive record comes to mean *this session has something to review* rather than *a turn ended in this project*. Today every session is registered: this repo has four records and three notes, and `3631c68a` explored nothing. One predicate should gate registering, ensuring, and listing.
- [ ] 5.4 Note in the change that existing records for never-explored sessions are not migrated. `.openspec-doc/` is local and disposable so deleting them is the answer, but it must be stated rather than left as a silent inconsistency between old and new records.
- [ ] 5.5 Open a browser at `/sessions/<id>` **the first time this session is registered** — the turn boundary at which its record is created — and never again for that session. This is the first moment the page has anything on it. Derive "first" from the record not having existed before this invocation; do not add a flag for it.
- [ ] 5.6 Do not gate the browser on whether a dashboard was *started* versus *reused*. A dashboard already up for another session on this root is still a dashboard whose page for *this* session is new. Reuse suppression is no longer what prevents tab pile-up; registering once per session is.
- [ ] 5.7 Report a failure to ensure a dashboard and carry on to the verdict check, the way `report_promotion` already does. A dashboard that would not start must not cost the session its review feedback.
- [ ] 5.8 Tests: a session with a note registers and ensures a dashboard; a session with neither note nor promoted change does neither; the browser opens once at that session's page on the registering turn boundary and not at later ones; a second exploration in the same session opens no second tab; `hook explore` starts nothing. Inject the ensure step rather than starting real servers from the hook tests.

## 6. Live verification

Detachment, the browser moment, the idle exit, and port stability across restarts all pass in a harness and fail in a real session. None of these can be inferred from a green test run.

- [ ] 6.1 Type `/opsx:explore` in a real Claude Code session with no dashboard running. Confirm the printed URL names a port, that nothing has started yet, and then that at the end of that turn a dashboard comes up **on the port that was printed**, a browser opens at that session's page with the note already rendered, and it is still serving after the hook has exited.
- [ ] 6.2 Type `/opsx:explore` again in the same session. Confirm no second dashboard and no second tab.
- [ ] 6.2a Explore in a *second* session against the same root while the first dashboard is up. Confirm no second dashboard and one new tab, at the second session's own page. This is the case the old reuse-suppression rule got wrong.
- [ ] 6.3 Let the session reach several turn boundaries with every tab closed. Confirm the dashboard is still serving — this is the case a subscriber-only idle check gets wrong.
- [ ] 6.4 Kill the dashboard mid-session. Confirm the next turn boundary brings it back **on the same port**.
- [ ] 6.5 Run `git clean -xdf` (or delete `.openspec-doc/`) while a dashboard is running, then reach a turn boundary. Confirm the running dashboard is discovered and no duplicate appears. This is the case the rejected state-file design fails.
- [ ] 6.5a Delete the port registry from the state directory while a dashboard is running, then reach a turn boundary. Confirm the running dashboard is discovered, no duplicate appears, and the entry is rewritten to the port it is actually on. This is the case that distinguishes a hint from an authority, and the one an implementation that skips the sweep will fail.
- [ ] 6.6 Confirm a session doing unrelated work — no explore command, no note — leaves no dashboard behind.
- [ ] 6.7 Run a pi session through `agent_end` and confirm the dashboard is ensured with no change to `.pi/extensions/openspec-doc-hook.ts`. If this needs extension changes, the claim that pi comes free is wrong and the proposal needs revising rather than patching.
- [ ] 6.8 Determine whether a browser keeps its SSE connection open in a frozen background tab, since that decides whether a minimised window reads as nobody-reviewing. If it does not, record it — with the heartbeat in place it only matters after the session has ended, but it is the one thing that could make 30 minutes the wrong window.
- [ ] 6.9 With dashboards up for two different checkouts, stop both, start them in the opposite order, and confirm each returns to the port it had. This is the whole point of the change and the one thing fall-forward gets wrong; a test that starts them in the same order every time cannot tell the two designs apart.
- [ ] 6.10 Bookmark a project's URL, restart the machine, reach a turn boundary in that project, and open the bookmark. It must be that project.

## 7. Documentation and removals

- [ ] 7.1 `AGENTS.md`: starting `serve` by hand stops being a precondition. Describe what starts the dashboard, that each project has its own stable URL, that the agent should answer with it when asked, that the dashboard exits when idle, and that `.openspec-doc/serve.log` is where a failed start explains itself.
- [ ] 7.2 `README.md`: per-project ports, where the registry lives, and the fact that a walkthrough passing `--port` explicitly still gets exactly that port and changes nothing about the project's assignment.
- [ ] 7.3 `docs/docs/reference/cli.md`: it currently says `--port` defaults to `0` and selects an available port, which this change makes false. Replace it, and document `serve url`. There is **no `QUICKSTART.md`** in this repository — an earlier draft of this change's impact list named one.
- [ ] 7.4 Delete the start-the-viewer step from `.pi/prompts/opsx-review.md`. It instructs the agent to do port-conflict resolution and process supervision that now lives in the binary, and leaving it contradicts the implementation.
- [ ] 7.5 Record the deferred pi note gap where the next reader will find it: pi sessions still get no scratch note, the fix is a `pi.on("input")` handler matching both `/opsx-explore` and `/skill:openspec-explore`, and `pi.registerCommand` must not be used because extension commands suppress the input event and would shadow the existing prompt template.
