## 1. Approval record

- [ ] 1.1 Add the `approved` and withdrawal verdict kinds to `crates/core/src/verdict.rs`
- [ ] 1.2 Add the artifact fingerprint field to the verdict record, populated only for approval kinds
- [ ] 1.3 Implement fingerprinting over `proposal.md`, `design.md`, and every spec delta, explicitly excluding `tasks.md`
- [ ] 1.4 Add tests: fingerprint changes when a reviewed artifact changes, and does not change when only `tasks.md` changes

## 2. Approval state

- [ ] 2.1 Implement approval-state evaluation returning approved, stale (with the artifacts that changed), or not approved (with the reason)
- [ ] 2.2 Enforce the all-resolved precondition on submission, refusing over `open` or `addressed` and reporting the blocking counts
- [ ] 2.3 Implement withdrawal as an appended record, never a removal
- [ ] 2.4 Add tests for every state transition: approve, edit an artifact, re-approve, withdraw, and approval refused over each blocking status
- [ ] 2.5 Add a test that a change with no comments and no approval record reports not approved

## 3. CLI precheck

- [ ] 3.1 Add the approval-state subcommand printing state and reason
- [ ] 3.2 Exit non-zero for stale and not-approved; exit zero only for approved
- [ ] 3.3 Fail loudly on an unknown change rather than reporting it not approved
- [ ] 3.4 Add tests covering each exit code
- [ ] 3.5 Add the precheck as a first step in `.claude/skills/openspec-apply-change/SKILL.md` and the `opsx:apply` equivalent

## 4. Change page controls

- [ ] 4.1 Render current approval state and reason on the change page
- [ ] 4.2 Add the approve control, and the withdraw control for an approved change
- [ ] 4.3 Render the refusal reason when approve is submitted over outstanding feedback, rather than silently not recording
- [ ] 4.4 Confirm approval state changes reach an open tab over the existing SSE channel
- [ ] 4.5 Add tests: state renders, controls select on state, refusal renders its reason, state change is pushed to the event stream

## 5. Stop-time detection

- [ ] 5.1 Detect at `hook stop` that a change has completed tasks with a stale or absent approval
- [ ] 5.2 Write the pending directive, with reason text that identifies its origin and names the change and the approval-state command, embedding no artifact content
- [ ] 5.3 Suppress repeat directives for the same unapproved state
- [ ] 5.4 Add tests for each: detection fires, stays silent under a current approval, and does not repeat

## 6. Manual verification

- [ ] 6.1 Approve a change from the page with all comments resolved; confirm the state renders approved
- [ ] 6.2 Edit `proposal.md`; confirm the state flips to stale and names the artifact
- [ ] 6.3 Tick a task in `tasks.md`; confirm the state stays approved
- [ ] 6.4 Run the precheck subcommand in each state and confirm the exit codes
- [ ] 6.5 Run a real session implementing an unapproved change and confirm the directive arrives at turn end on both Claude Code and pi.dev
