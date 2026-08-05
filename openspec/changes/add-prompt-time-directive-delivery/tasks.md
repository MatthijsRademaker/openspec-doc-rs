## 1. The prompt hook command

- [ ] 1.1 Add a `hook prompt --agent <claude|pi>` subcommand, reading only `session_id` from the payload via the existing `parse_session_id`
- [ ] 1.2 Call the existing `translate`, then `load_pending` and `mark_consumed`, and print the reason text on stdout. Reuse them unchanged — consume-once is inherited, not reinvented.
- [ ] 1.3 Print nothing when there is no untranslated verdict and no pending directive, so an ordinary prompt adds no context
- [ ] 1.4 Make every failure path report on stderr and exit zero. This is the deliberate opposite of `hook stop`; note in the code why, so it does not read as a swallowed error.
- [ ] 1.5 Add tests: a standing verdict is emitted, a pending directive is emitted without being re-translated, a session with nothing outstanding emits nothing, and promotion does not run

## 2. Consume-once across both delivery points

- [ ] 2.1 Add a test that prompt-time delivery is followed by an allow decision at the next `hook stop`, not a second block
- [ ] 2.2 Add a test that a directive delivered at a turn boundary leaves the following prompt with nothing to emit
- [ ] 2.3 Add a test that a corrupt verdict or directive sidecar still exits zero with empty stdout, so the reviewer's prompt cannot be refused

## 3. Wiring and live verification

- [ ] 3.1 Add a `UserPromptSubmit` hook entry alongside the existing `Stop` and `UserPromptExpansion` entries
- [ ] 3.2 Establish the exit-code contract by observation: make the hook fail deliberately in a live session and record whether the prompt still goes through. The documentation covers this only for the tool events, and the fail-soft design assumes the worst case without having confirmed it.
- [ ] 3.3 Run the reviewer rhythm end to end: submit a keep-exploring verdict while the agent is idle, then prompt the session, and confirm the feedback is in context for *that* turn rather than arriving when it ends
- [ ] 3.4 Confirm the same cycle ends with an allow at the turn boundary, so nothing is delivered twice
- [ ] 3.5 Update `AGENTS.md`: feedback now arrives either with a prompt or at a turn boundary, so the mechanism section should describe both rather than only the Stop path
