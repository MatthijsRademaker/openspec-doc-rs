## 1. The shutdown route

- [ ] 1.1 Add the route's path to `crates/core/src/dashboard.rs` beside `IDENTITY_PATH`, for the same reason that one lives there: the server and the CLI's hand-rolled client have to agree about it, and the CLI has no JSON parser or router of its own.
- [ ] 1.2 Add the route to `crates/server/src/routes.rs`, `POST` only, resolving the graceful-shutdown future `crates/server/src/lib.rs` already builds for the idle exit. Do not add a second shutdown path; the idle exit and this one must converge on the same future, and shutting down twice must be a no-op rather than a panic.
- [ ] 1.3 Take the root the caller believes it is stopping, and refuse a shutdown naming a root this server does not serve. This is what closes the fall-forward race, and it is only possible because the mechanism is a conversation — the server is the one participant that knows for certain what it is.
- [ ] 1.4 Require a header a cross-origin HTML form cannot set, so a page in the operator's browser cannot `POST` the dashboard down without clearing a CORS preflight. No token, no nonce — a secret needs somewhere to live, and per-server state is the design this work rejected.
- [ ] 1.5 Answer before exiting, not after. A caller that gets a dropped connection cannot distinguish "shutting down" from "never arrived", and the command's whole contract is reporting what actually happened.
- [ ] 1.6 Tests: a shutdown request stops the server, one naming another root is refused and the server keeps serving, a `GET` to the route is rejected, a request without the required header is rejected, and the idle exit still works with the route present.

## 2. `serve kill`

- [ ] 2.1 Add `ServeCommand::Kill` taking `--project`, `--port`, or `--all`. With no target at all, stop nothing and report that one is required — the destructive reading must never be the default.
- [ ] 2.2 Spell the project target `--project`, a flag taking no value, resolving the project the way every other command does and honouring a global `--root`. **Do not add a subcommand `--root`:** `Cli` already declares `--root` as `global = true`, so that spelling is taken, and the global one already defaults to the resolved project — which is exactly the untargeted case 2.1 forbids. Say in the help text that the flag is required because the behaviour depends on the working directory.
- [ ] 2.3 Resolve the target by enumerating with `add-serve-inventory`'s function. If this change writes its own sweep, the seam is wrong.
- [ ] 2.4 Enumerate immediately before asking rather than trusting a list the operator was shown seconds ago.
- [ ] 2.5 Probe again after asking and report what is actually gone. Never report success from having sent a request.
- [ ] 2.6 Report a target that matched nothing as matching nothing, not as success.
- [ ] 2.7 Report a dashboard that answered the request and is still serving as still running. This is the wedged case the design declines to escalate for, so it must at least be visible when it happens rather than reported as stopped.
- [ ] 2.8 Leave the port assignment alone. `serve forget` is the command for that.
- [ ] 2.9 Tests: a targeted kill stops one and leaves the others, no target stops nothing, `--all` stops everything, a target matching nothing is reported, a dashboard that survives is reported as still running, one that had already exited is reported as not running rather than as killed, a port that changed hands between enumeration and request is not stopped, and a killed project keeps its assignment.

## 3. No signals, and saying so

- [ ] 3.1 Add no signal-sending dependency. If `libc`, `nix`, or `rustix` appears in `Cargo.toml` during this change, the HTTP decision has been quietly reversed and the design needs revisiting rather than the lockfile.
- [ ] 3.2 State the wedged-dashboard limit where an operator hits it: a dashboard that no longer answers HTTP cannot be stopped by this command, and `serve list`'s pid column plus `kill` is the manual escape hatch.
- [ ] 3.3 Do not build `--pid`. It exists in the docs as a manual instruction, not as a flag, until a wedged dashboard is actually observed.

## 4. Live verification

Stopping processes is exactly the kind of thing that passes in a harness against servers the test started itself.

- [ ] 4.1 With dashboards running for two different checkouts, kill one by `--project` and confirm the other keeps serving and its page stays live. Two checkouts, not two sessions in one.
- [ ] 4.2 Kill a dashboard with a review page open in a browser and confirm the page reports the connection ending rather than hanging silently.
- [ ] 4.3 Kill a dashboard that the idle exit has already reaped, and confirm it reports not-running rather than success.
- [ ] 4.4 Kill a project's dashboard, then reach a turn boundary in it, and confirm it comes back on the same port. A kill that quietly drops the assignment passes every unit test and fails this.
- [ ] 4.5 Confirm `--all` from outside any project stops every dashboard on the machine and exits zero with nothing left to find.

## 5. Documentation

- [ ] 5.1 `docs/docs/reference/cli.md`: the command, the required target, the three target forms, and why `--project` has to be typed.
- [ ] 5.2 Replace `add-serve-inventory`'s interim instruction, which tells an operator to find the pid with `serve list` and kill it by hand. That line was written to be replaced by this change and leaving it is a stale answer sitting beside the real one.
- [ ] 5.3 Document the shutdown route as an interface the dashboard exposes, in the same place the identity route is described. A local HTTP route that stops the process is not an implementation detail.
