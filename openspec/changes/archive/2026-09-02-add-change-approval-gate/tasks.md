## 1. Approval record

- [x] 1.1 Add the `approved` and withdrawal verdict kinds to `crates/core/src/verdict.rs`
- [x] 1.2 Add the artifact fingerprint field to the verdict record, populated only for approval kinds
- [x] 1.3 Implement fingerprinting over `proposal.md`, `design.md`, and every spec delta, explicitly excluding `tasks.md`
- [x] 1.4 Add tests: fingerprint changes when a reviewed artifact changes, and does not change when only `tasks.md` changes

## 2. Approval state

- [x] 2.1 Implement approval-state evaluation returning approved, stale (with the artifacts that changed), or not approved (with the reason)
- [x] 2.2 Enforce the all-resolved precondition on submission, refusing over `open` or `addressed` and reporting the blocking counts
- [x] 2.3 Implement withdrawal as an appended record, never a removal
- [x] 2.4 Add tests for every state transition: approve, edit an artifact, re-approve, withdraw, and approval refused over each blocking status
- [x] 2.5 Add a test that a change with no comments and no approval record reports not approved

## 3. Resolve-all-and-approve

- [x] 3.1 Implement the single act: resolve every comment not already `resolved`, then evaluate and append the approval, returning the count resolved
- [x] 3.2 Skip comments already `resolved` rather than appending a redundant status record
- [x] 3.3 On approval failure after the resolutions land, return the partial outcome as an error naming both halves; do not append compensating reopen records
- [x] 3.4 Expose it in the server surface only — no CLI subcommand or flag performs bulk resolution
- [x] 3.5 Add tests: mixed `open`/`addressed` all resolved and approved with the count returned, already-resolved comments untouched, nothing outstanding still approves, partial failure reports both halves

## 4. CLI precheck

- [x] 4.1 Add the approval-state subcommand printing state and reason
- [x] 4.2 Exit non-zero for stale and not-approved; exit zero only for approved
- [x] 4.3 Fail loudly on an unknown change rather than reporting it not approved
- [x] 4.4 Add tests covering each exit code
- [x] 4.5 Add the precheck as a first step in `.claude/skills/openspec-apply-change/SKILL.md` and the `opsx:apply` equivalent

## 5. Change page controls

- [x] 5.1 Render current approval state and reason on the change page
- [x] 5.2 Add the approve control, and the withdraw control for an approved change
- [x] 5.3 Render the refusal reason when approve is submitted over outstanding feedback, rather than silently not recording
- [x] 5.4 Add the resolve-all-and-approve control and its route, rendered in place of the plain approve control while any comment is `open` or `addressed`
- [x] 5.5 Label it with the count it will resolve, broken down by `open` and `addressed`
- [x] 5.6 Render the partial outcome when the approval fails after the resolutions landed
- [x] 5.7 Confirm approval state changes reach an open tab over the existing SSE channel, and that the bulk act emits one event rather than one per comment
- [x] 5.8 Add tests: state renders, controls select on state, the two approve controls are never rendered together, the count label is correct, refusal renders its reason, partial failure renders both halves, state change is pushed to the event stream

## 6. Stop-time detection

- [x] 6.1 Detect at `hook stop` that a change has completed tasks with a stale or absent approval
- [x] 6.2 Write the pending directive, with reason text that identifies its origin and names the change and the approval-state command, embedding no artifact content
- [x] 6.3 Suppress repeat directives for the same unapproved state
- [x] 6.4 Add tests for each: detection fires, stays silent under a current approval, and does not repeat

## 7. Manual verification

- [x] 7.1 Approve a change from the page with all comments resolved; confirm the state renders approved
- [x] 7.2 Edit `proposal.md`; confirm the state flips to stale and names the artifact
- [x] 7.3 Tick a task in `tasks.md`; confirm the state stays approved
- [x] 7.4 Run the precheck subcommand in each state and confirm the exit codes
- [x] 7.5 On a change with both `open` and `addressed` comments, confirm the plain approve control is absent, submit resolve-all-and-approve, and confirm every thread reads resolved and the change reads approved in a second open tab without reloading
- [ ] 7.6 Run a real session implementing an unapproved change and confirm the directive arrives at turn end on both Claude Code and pi.dev

## 8. Text this change falsifies

- [x] 8.1 Update `AGENTS.md`, which tells agents the reviewer submits "a phase verdict: keep exploring, move to proposal, or send the open comments back for work". This change makes that list of three wrong by adding a fourth, and it is the list an agent reads to know what feedback can arrive. Add `approved`, and say what an agent should do on receiving it — which for this change is the one verdict that gates work rather than requesting it.
- [x] 8.2 Five docs enumerate the verdict kinds independently of the spec and each needs the fourth: `docs/docs/concepts/review-loop.md`, `docs/docs/concepts/pointer-not-embed.md`, `docs/docs/reference/routes.md`, `docs/docs/quickstart.md`, and `docs/docs/development/manual-verification.md`. `routes.md` and `manual-verification.md` carry the accepted form-values and worked `curl` examples, so they go wrong in a way a reader can execute rather than merely read.
- [x] 8.3 `docs/docs/reference/routes.md` states that the dashboard exposes no `addressed` transition because addressed is the agent's claim. The bulk act resolves `addressed` comments without the reviewer opening each thread, which is consistent with that sentence but not obviously so; document the bulk route beside the per-comment status route and say why sweeping `addressed` is still the reviewer's judgement.
