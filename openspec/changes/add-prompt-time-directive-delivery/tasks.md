## 1. Point the move-to-proposal directive at the comments

- [ ] 1.1 Add the session's comment sidecar path to the move-to-proposal reason text, with an instruction to account for the open comments in the proposal. It names the note and the verdict record today; the keep-exploring template already names the sidecar and is the shape to follow.
- [ ] 1.2 Add a test that the move-to-proposal template names the comment sidecar path
- [ ] 1.3 Re-check the revised template against the guards `add-directive-verdict-loop` established: still an attributed pointer, still demands no literal output, still says nothing about resolving comments, and still names the claim command with the session id resolved

## 2. The prompt hook command

- [ ] 2.1 Add a `hook prompt --agent <claude|pi>` subcommand, reading only `session_id` from the payload via the existing `parse_session_id`
- [ ] 2.2 Call the existing `translate`, then `load_pending` and `mark_consumed`, and print the reason text on stdout. Reuse them unchanged — consume-once is inherited, not reinvented.
- [ ] 2.3 Print nothing when there is no untranslated verdict and no pending directive, so an ordinary prompt adds no context
- [ ] 2.4 Make every failure path report on stderr and exit zero. This is the deliberate opposite of `hook stop`; note in the code why, so it does not read as a swallowed error.
- [ ] 2.5 Add tests: a standing verdict is emitted, a pending directive is emitted without being re-translated, a session with nothing outstanding emits nothing, and promotion does not run

## 3. Consume-once across both delivery points

- [ ] 3.1 Add a test that prompt-time delivery is followed by an allow decision at the next `hook stop`, not a second block
- [ ] 3.2 Add a test that a directive delivered at a turn boundary leaves the following prompt with nothing to emit
- [ ] 3.3 Add a test that a corrupt verdict or directive sidecar still exits zero with empty stdout, so the reviewer's prompt cannot be refused

## 4. Wiring and live verification

- [ ] 4.1 Add a `UserPromptSubmit` hook entry alongside the existing `Stop` and `UserPromptExpansion` entries
- [ ] 4.2 Establish the exit-code contract by observation: make the hook fail deliberately in a live session and record whether the prompt still goes through. The documentation covers this only for the tool events, and the fail-soft design assumes the worst case without having confirmed it.
- [ ] 4.3 Run the reviewer rhythm end to end: submit a keep-exploring verdict while the agent is idle, then prompt the session, and confirm the feedback is in context for *that* turn rather than arriving when it ends
- [ ] 4.4 Confirm the same cycle ends with an allow at the turn boundary, so nothing is delivered twice
- [ ] 4.5 Update `AGENTS.md`: feedback now arrives either with a prompt or at a turn boundary, so the mechanism section should describe both rather than only the Stop path

## 5. Prove the MVP criterion

- [ ] 5.1 Run the completion criterion on a live session: leave anchored comments on an exploration in the browser, then tell the agent in the terminal to move to proposal, and confirm the proposal it writes **accounts for those comments**. This is the check the whole change exists to enable, and it is the MVP call — nothing below a live run can make it, for the same reason the injection risk needed one.
- [ ] 5.2 Confirm the comments arrived *before* the proposal was written rather than at the turn boundary after it. A proposal that addresses the comments only after being revised is a failure of this change, not a pass.
- [ ] 5.3 Record the outcome, including whether the reviewer had to explain anything the tool should have carried on its own
