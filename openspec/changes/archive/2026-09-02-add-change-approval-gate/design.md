## Context

The intended review loop is the pull-request one: the reviewer comments, the comments go to the agent, the agent responds, the reviewer resolves each thread, and the change becomes ready to implement. `add-comment-thread-actions` builds everything up to and including resolution. This change builds the last step, which exists nowhere today.

It is genuinely new. The predecessor implementation in `openspec-doc-rs-example` has no ready-to-apply gate either: its lifecycle phases are inferred from artifact and task presence and marked advisory, the browser executes no transitions, and every use of "approval" there means the agent will not edit files without asking — never that a change is cleared for implementation.

## Goals / Non-Goals

**Goals:**

- Approval is a thing a reviewer says, and can be withdrawn.
- An approval stops counting once what it approved has changed.
- An apply workflow can ask, in one command with a meaningful exit code, whether it may start.
- Satisfying the precondition on a change with a backlog of threads is one act, not one act per thread.
- Implementing without approval is noticed and reported rather than passing silently.

**Non-Goals:**

- Preventing an agent from implementing an unapproved change. Not achievable here; see the decision below.
- Approving individual artifacts or requirements. Approval is per change.
- Multiple reviewers, or any notion of who approved. There is no actor concept anywhere in the sidecars, and inventing one for this is disproportionate.
- Blocking archive on approval. Archive has its own workflow; conflating the two gates helps nobody.

## Decisions

**A Stop hook cannot prevent an action, so the gate is a precheck plus detection.** `openspec-doc hook stop` runs at turn end and its entire output vocabulary is block-and-continue or allow-stop. Blocking makes the agent do *more* work, not less — it is the opposite of a veto. Hard prevention needs a pre-action hook; Claude Code has one, but pi.dev is driven by a project-local extension rather than an external-process hook, so a pre-action gate would not exist on both agents and the loop would be enforceable on one only. What is deliverable on both: a CLI precheck the apply workflow runs before it starts, and after-the-fact detection at stop that raises unapproved implementation as a directive. This is weaker than it sounds when the word "gate" is used, which is why the proposal says so plainly.

**Approval is a verdict kind, and the record carries the fingerprint.** Verdicts are already an append-only per-scope sidecar written by the same page, with an existing kind enum and an existing rendering of history. A second store for approvals would duplicate all of it. The record gains a fingerprint field populated only for approval kinds.

**The fingerprint excludes `tasks.md`.** This is the decision most likely to be got wrong by accident. Ticking checkboxes rewrites `tasks.md` on essentially every turn of implementation, so a fingerprint covering it would invalidate every approval within seconds of work starting and train everyone to ignore staleness. What the reviewer approved is the proposal, the design, and the spec deltas. The cost is real and accepted: adding a *new task* to `tasks.md` will not invalidate an approval, even though arguably it should.

**Staleness is computed, not stored.** The current fingerprint is recomputed on query and compared against the recorded one. Storing a stale flag would need something to set it, which means either a watcher racing the reader or a write on every artifact change.

**Resolve-all-and-approve is one server-side act, and it sweeps `open` comments too.** Resolving in bulk does not weaken the `addressed` rule, because resolution is the reviewer's judgement whether they make it once or twenty times — the rule is about *who* resolves, not how many clicks it takes. It does mean the sweep also resolves `open` comments the agent never replied to, which is the reviewer withdrawing feedback rather than accepting work. That is included deliberately: a control that refused to sweep `open` would be unavailable in exactly the case that motivates it, a change whose threads the reviewer has read in the page and decided are no longer live. It is one route rather than the browser firing N status posts and then an approve, because a partial sequence from the client leaves the reviewer looking at a change that is neither settled nor approved, and emits N SSE events for one intent.

**The act is not atomic, and does not pretend to be.** The sidecars are append-only JSONL. The resolutions are appended, then the approval is evaluated and appended. If the approval fails after the resolutions land — an unreadable artifact while fingerprinting, say — the comments stay resolved and the change stays unapproved, and the response says so. Inventing a rollback over an append log would mean writing compensating reopen records that are indistinguishable from a reviewer reopening a thread, which corrupts the one history this system keeps. Failing loudly with the partial state named is the lesser harm.

**No CLI counterpart.** The bulk act exists in the browser only. Per-comment `openspec-doc comment resolve` already serves a reviewer at a terminal. A `--resolve-all` flag on an approval command would hand the agent a single command that clears its own feedback and approves its own change. The verdict sidecar is a file and nothing stops an agent writing to it — but not shipping the affordance is not the same as it being impossible, and the same reasoning already keeps `addressed` off the dashboard.

**The two approve controls are mutually exclusive, selected by state.** Plain approve renders when nothing is outstanding; resolve-all-and-approve renders when something is. Showing both at once would ask the reviewer to choose between a button that works and a button that is refused, on a page that already knows which is which.

**The precondition rejects `addressed`, not just `open`.** Allowing approval over `addressed` comments would let the agent's own claim of completion satisfy the reviewer's gate, which reintroduces exactly the marking-your-own-homework problem that the three-state lifecycle exists to prevent.

**Detection keys on completed tasks.** `tasks.md` checkbox state is the only implementation signal available without reading the transcript or the git index, and it is the same signal the predecessor's phase inference used.

## Risks / Trade-offs

- [Risk] "Gate" implies prevention, and this does not prevent. → Mitigated only by saying so, in the proposal, in the spec requirement text, and in whatever the CLI prints. A reviewer who believes the tool is stopping the agent will be worse off than one who knows it is reporting on it.
- [Risk] Detection fires after the work is done, so the directive arrives when the damage exists. → Accepted. The directive's value is that the reviewer and the agent both learn the change went in unreviewed; the alternative is that nobody learns it.
- [Risk] Excluding `tasks.md` from the fingerprint leaves a hole: an agent can add tasks to an approved change and implement them. → Accepted for now. Narrowing the fingerprint to only the task *list* while excluding checkbox state would close it, at the cost of parsing `tasks.md` structurally rather than hashing bytes. Worth doing if the hole is ever exercised.
- [Risk] Resolve-all-and-approve makes rubber-stamping a change with twenty open threads a single click, where before it was twenty-one. → Accepted, and the sharpest cost of this control. The counter is that a reviewer determined to rubber-stamp was never going to be slowed by click count, and the honest failure mode is the reviewer who uses it *inattentively* rather than the one who uses it deliberately. Mitigated only by the control naming what it will do — how many comments it will resolve, in which statuses — before it is submitted.
- [Risk] A partial failure leaves comments resolved on a change that did not become approved, and the reviewer has lost the open/addressed distinction on those threads. → Accepted. Recoverable by reopening, which is a control the reviewer already has, and reported rather than swallowed.
- [Risk] Approval with no comments at all is permitted, so a reviewer can rubber-stamp without reading. → Intentional. Requiring a comment before approval would produce ritual comments, and a clean change genuinely needs none. The protection this change offers is that *someone pressed the button*, not that they read anything.
- [Risk] No actor concept means an agent can submit the approval verdict itself via whatever surface exists. → The approve control is browser-side and the verdict sidecar is writable by anything with filesystem access, so this is unenforceable without identity on records. Same limitation, and same accepted trade-off, as the `address`/`resolve` split in `add-comment-thread-actions`.
