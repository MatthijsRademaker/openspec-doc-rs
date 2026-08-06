## 1. Point the move-to-proposal directive at the comments

- [x] 1.1 Add the session's comment sidecar path to the move-to-proposal reason text, with an instruction to account for the open comments in the proposal. It names the note and the verdict record today; the keep-exploring template already names the sidecar and is the shape to follow.
- [x] 1.2 Add a test that the move-to-proposal template names the comment sidecar path
- [x] 1.3 Re-check the revised template against the guards `add-directive-verdict-loop` established: still an attributed pointer, still demands no literal output, still says nothing about resolving comments, and still names the claim command with the session id resolved

## 2. The prompt hook command

- [x] 2.1 Add a `hook prompt --agent <claude|pi>` subcommand, reading only `session_id` from the payload via the existing `parse_session_id`
- [x] 2.2 Call the existing `translate`, then `load_pending` and `mark_consumed`, and print the reason text on stdout. Reuse them unchanged — consume-once is inherited, not reinvented.
- [x] 2.3 Print nothing when there is no untranslated verdict and no pending directive, so an ordinary prompt adds no context
- [x] 2.4 Make every failure path report on stderr and exit zero. This is the deliberate opposite of `hook stop`; note in the code why, so it does not read as a swallowed error.
- [x] 2.5 Add tests: a standing verdict is emitted, a pending directive is emitted without being re-translated, a session with nothing outstanding emits nothing, and promotion does not run

## 3. Consume-once across both delivery points

- [x] 3.1 Add a test that prompt-time delivery is followed by an allow decision at the next `hook stop`, not a second block
- [x] 3.2 Add a test that a directive delivered at a turn boundary leaves the following prompt with nothing to emit
- [x] 3.3 Add a test that a corrupt verdict or directive sidecar still exits zero with empty stdout, so the reviewer's prompt cannot be refused

## 4. Wiring and live verification

- [x] 4.1 Add a `UserPromptSubmit` hook entry alongside the existing `Stop` and `UserPromptExpansion` entries
- [x] 4.2 Establish the exit-code contract by observation: make the hook fail deliberately in a live session and record whether the prompt still goes through. The documentation covers this only for the tool events, and the fail-soft design assumes the worst case without having confirmed it. **Answered on Claude Code 2.1.223: exit 1 passes the prompt through with a warning, exit 2 refuses it outright (`UserPromptSubmit operation blocked by hook`). The fail-soft is load-bearing, not defensive.** Recorded in `docs/docs/reference/hooks.md`.
- [x] 4.3 Run the reviewer rhythm end to end: submit a keep-exploring verdict while the agent is idle, then prompt the session, and confirm the feedback is in context for *that* turn rather than arriving when it ends
- [x] 4.4 Confirm the same cycle ends with an allow at the turn boundary, so nothing is delivered twice
- [x] 4.5 Update `AGENTS.md`: feedback now arrives either with a prompt or at a turn boundary, so the mechanism section should describe both rather than only the Stop path

## 5. Prove the MVP criterion

- [x] 5.1 Run the completion criterion on a live session: leave anchored comments on an exploration in the browser, then tell the agent in the terminal to move to proposal, and confirm the proposal it writes **accounts for those comments**. This is the check the whole change exists to enable, and it is the MVP call — nothing below a live run can make it, for the same reason the injection risk needed one. **Passed.** Two anchored comments on an exploration, move-to-proposal verdict submitted through the dashboard, then `lets move to proposal` typed at the session. The agent wrote `openspec/changes/add-setup-diagnostics/` with both comments driving the design rather than mentioned in passing, replied to both, left both open, and ran `scratch claim`.
- [x] 5.2 Confirm the comments arrived *before* the proposal was written rather than at the turn boundary after it. A proposal that addresses the comments only after being revised is a failure of this change, not a pass. **Passed on timestamps: directive consumed 06:50:09, `proposal.md` written 06:52:26.**
- [x] 5.3 Record the outcome, including whether the reviewer had to explain anything the tool should have carried on its own. **Nothing had to be explained — the prompt was four words and carried no context. Three gaps surfaced instead, all in the tooling around the loop rather than the loop itself: the hook config was not installed at all and nothing said so (`add-setup-diagnostics` now exists for this); `docs/docs/reference/hooks.md` documented two hooks per agent and omitted this one; and a session is only discoverable once `hook stop` has written a directive record, so a note written before the first turn boundary 404s on verdict submission.**

## 6. The same delivery point on pi.dev

The requirement this change adds is written against "the agent's prompt-submission contract", not against Claude Code. Wiring only one agent would leave the capability half-delivered against its own spec, and `vision.md` treats both agents as scope rather than a bonus.

- [x] 6.1 Add a `before_agent_start` handler to `.pi/extensions/openspec-doc-hook.ts` calling `openspec-doc hook prompt --agent pi`. That event fires after the prompt is submitted and before the agent loop, which is where `UserPromptSubmit` sits.
- [x] 6.2 Return the directive as a `custom` message from the handler rather than transforming the user's prompt text. The directive is context the reviewer sent, not words the reviewer typed, which is the same distinction Claude Code's contract draws.
- [x] 6.3 Generalize the extension's `runHook` over the subcommand instead of adding a second spawn helper
- [x] 6.4 Verify consume-once across pi's two delivery points, not just Claude's
- [x] 6.5 Verify live: with a verdict standing, the agent's first reply acts on the reviewer's feedback in the same turn as an unrelated prompt. **Passed.** Note the discriminating prompt must not itself demand literal output — a prompt of "Reply with only: X" is obeyed literally even though the directive is in context, which reads as a delivery failure and is not one.
- [x] 6.6 Update `docs/docs/reference/hooks.md`: it documented two hooks per agent and omitted this one entirely. Also correct the `--session-id` quirk, which applies to sessions that do not exist yet — re-entering an existing session does not hang, and that is what makes the prompt-time test possible.

## 7. Close the hole the fail-soft did not cover

Found by running the verification procedure being written for `manual-verification.md`, not by any test. `main.rs` parses arguments and resolves the project root **before** dispatch, so both sit outside `hook::prompt`'s fail-soft and the command cannot close the gap from inside. A clap usage error exits 2, and Claude Code treats a `UserPromptSubmit` hook exiting 2 as a block — so one typo in a hook command string refused every prompt in the session and returned usage text instead of an answer. Confirmed against a real session before and after the fix.

- [x] 7.1 Exit zero from `main` for any error reaching `hook prompt`, root resolution included
- [x] 7.2 Exit zero for an unparseable invocation naming `hook prompt`, reading the intent off the raw arguments because parsing is what failed. A typo in the word `prompt` itself stays unrecoverable; note it rather than guessing.
- [x] 7.3 Preserve every other exit status: usage errors elsewhere stay 2, `--help` and `--version` stay 0, `hook stop` stays loud
- [x] 7.4 Keep stdout empty on these paths — it is injected as context, so clap's usage text must not reach it
- [x] 7.5 Add tests for both classes, and for the sibling commands that must still fail loudly
- [x] 7.6 Amend the `cli-surface` spec: its root-resolution requirement said every subcommand exits non-zero, which this fix now contradicts
- [x] 7.7 Fix the flake these tests exposed in `run_with_stdin`. The helper writes the payload and unwraps the result, but these invocations exit during argument parsing — before stdin is read — so the write races the child's exit and fails with a broken pipe roughly one run in three. First test in the suite whose subject exits without reading stdin, so the sharpness was pre-existing and unhit. `run_with_stdin` stays strict; a tolerant variant covers the early-exit cases.

## 8. Spec drift closed out with this change

`cli-surface` claims to list the whole command surface and did not, a defect the roadmap has carried as low-severity for weeks. It widened when this change added a third `hook` subcommand, so it is fixed here rather than logged again.

- [x] 8.1 Add `scratch` to the documented surface. The CLI test already asserted it — the drift was in the spec alone.
- [x] 8.2 Remove the *Placeholder subcommands fail loudly* requirement. Every subcommand it covers is implemented, so it specifies behaviour the system must no longer have.
