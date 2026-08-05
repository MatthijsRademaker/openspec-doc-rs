## 1. The server side

- [ ] 1.1 Change `serve`'s `--port` default from `0` to `4321`, and make an omitted port fall forward through `4330`. An explicit `--port` must fail rather than fall forward; an exhausted range must fail naming the range.
- [ ] 1.2 Add an identity route returning the canonical project root and the process id as JSON. Keep it off the page namespace so it cannot collide with a change or session name.
- [ ] 1.3 Add `--idle-exit` to `serve`, and an idle-exit task that ends the process after 30 minutes with no subscribed page and no registered activity. Read the subscriber count from the hub's existing `broadcast::Sender` rather than adding a second count to keep in step with it.
- [ ] 1.4 Register a request to the identity route as activity against the idle deadline, so a heartbeat and a start probe are the same request.
- [ ] 1.5 Tests: fall-forward picks the next port, an explicit taken port fails, an exhausted range fails, the identity route reports the canonical root, and idleness ends the process only with `--idle-exit` and only when both conditions hold. Drive the idle test on an injected clock or a shortened window — a test that waits 30 minutes will be deleted by whoever runs it next.

## 2. Discovery and detached start

- [ ] 2.1 Add a module owning ensure-a-dashboard: probe `4321`–`4330` for the identity of this canonical root, reuse on a match, skip a port answering with another root or with something that is not a dashboard, and start on the first port answering nothing. Probe with a short connect and read timeout so a hung port cannot stall a hook.
- [ ] 2.2 Hand-roll the probe as a fixed `GET` over `std::net::TcpStream`. The CLI has no HTTP client and this does not justify adding one; say so in a comment so the next reader does not "fix" it.
- [ ] 2.3 Start the server detached via `std::env::current_exe()`, with `CommandExt::process_group(0)` on Unix and `creation_flags` with `DETACHED_PROCESS` on Windows, passing `--idle-exit` and `--no-open`.
- [ ] 2.4 Redirect the started server's stdout and stderr to `.openspec-doc/serve.log`. Confirm `*.log` in `.gitignore` already covers it; add an ignore entry only if it does not.
- [ ] 2.5 After starting, poll the identity route for up to ~2 seconds. Treat *a dashboard for this root answering* as success regardless of which process won the bind, so a concurrent start is not an error path.
- [ ] 2.6 Report a start that never answers, naming `.openspec-doc/serve.log`. Do not fall back to serving nothing quietly.
- [ ] 2.7 Tests: reuse finds an already-serving dashboard, a foreign root is skipped, a non-dashboard listener is skipped, an exhausted range reports, and a probe against a port that accepts but never responds times out instead of hanging. A test that binds real ports must take them from the OS and pass them in rather than assuming `4321` is free on the machine running the suite.

## 3. Wiring the hooks

- [ ] 3.1 `hook explore`: ensure a dashboard, and open a browser only when one was actually started. Keep the note instruction on stdout as the command's primary output; the dashboard URL is a diagnostic and belongs on stderr with the rest.
- [ ] 3.2 `hook stop`: ensure a dashboard only when the session has review material — its scratch note exists, or `scratch::promoted_to` names a change. Never open a browser.
- [ ] 3.3 Report a failure to ensure a dashboard and carry on to the verdict check, the way `report_promotion` already does. A dashboard that would not start must not cost the session its review feedback.
- [ ] 3.4 Tests: a session with a note ensures a dashboard, a session with neither note nor promoted change does not, `hook explore` opens a browser on a start and not on a reuse, and `hook stop` never does. Inject the ensure step rather than starting real servers from the hook tests.

## 4. Live verification

Detachment, the browser moment, and the idle exit all pass in a harness and fail in a real session. None of these can be inferred from a green test run.

- [ ] 4.1 Type `/opsx:explore` in a real Claude Code session with no dashboard running. Confirm one comes up, a browser opens at it, and it is still serving after the hook has exited.
- [ ] 4.2 Type `/opsx:explore` again. Confirm no second dashboard and no second tab.
- [ ] 4.3 Let the session reach several turn boundaries with every tab closed. Confirm the dashboard is still serving — this is the case a subscriber-only idle check gets wrong.
- [ ] 4.4 Kill the dashboard mid-session. Confirm the next turn boundary brings it back.
- [ ] 4.5 Run `git clean -xdf` (or delete `.openspec-doc/`) while a dashboard is running, then reach a turn boundary. Confirm the running dashboard is discovered and no duplicate appears. This is the case the rejected state-file design fails.
- [ ] 4.6 Confirm a session doing unrelated work — no explore command, no note — leaves no dashboard behind.
- [ ] 4.7 Run a pi session through `agent_end` and confirm the dashboard is ensured with no change to `.pi/extensions/openspec-doc-hook.ts`. If this needs extension changes, the claim that pi comes free is wrong and the proposal needs revising rather than patching.
- [ ] 4.8 Determine whether a browser keeps its SSE connection open in a frozen background tab, since that decides whether a minimised window reads as nobody-reviewing. If it does not, record it — with the heartbeat in place it only matters after the session has ended, but it is the one thing that could make 30 minutes the wrong window.

## 5. Documentation and removals

- [ ] 5.1 `AGENTS.md`: starting `serve` by hand stops being a precondition. Describe what starts the dashboard, that it exits when idle, and that `.openspec-doc/serve.log` is where a failed start explains itself.
- [ ] 5.2 `README.md` and `QUICKSTART.md`: the new default port, and the fact that a walkthrough passing `--port` explicitly still gets exactly that port.
- [ ] 5.3 Delete the start-the-viewer step from `.pi/prompts/opsx-review.md`. It instructs the agent to do port-conflict resolution and process supervision that now lives in the binary, and leaving it contradicts the implementation.
- [ ] 5.4 Record the deferred pi note gap where the next reader will find it: pi sessions still get no scratch note, the fix is a `pi.on("input")` handler matching both `/opsx-explore` and `/skill:openspec-explore`, and `pi.registerCommand` must not be used because extension commands suppress the input event and would shadow the existing prompt template.
