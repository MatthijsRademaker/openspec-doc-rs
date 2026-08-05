## Context

`add-directive-verdict-loop` proved the loop end to end: a dashboard verdict becomes a directive, the Stop hook blocks, and the agent acts on it. What that proof also exposed is a timing defect. Stop is by construction the end of a turn, so a verdict submitted while the agent is idle is not delivered until the end of the *next* turn — the very turn the reviewer started in order to deliver it.

Everything needed already exists. `translate` finds the standing untranslated verdict and writes the directive; `load_pending` and `mark_consumed` deliver it exactly once. Only the delivery point and the output encoding are new: `UserPromptSubmit` runs before the model sees the prompt, and for that event a hook's plain stdout is added to context — the same mechanism `add-explore-note-capture` already relies on.

## Goals / Non-Goals

**Goals:**
- Feedback submitted before a prompt is in context for the turn that prompt starts.
- A directive is delivered exactly once across both delivery points, with no new consume-once mechanism.
- A failure in this hook can never stop the reviewer from submitting a prompt.

**Non-Goals:**
- No change to the Stop path. It stays the enforcement point and the only one that works when no human prompt is coming.
- No promotion check at prompt time.
- No new verdict kinds, templates, or file formats.

## Decisions

- **The move-to-proposal template fix belongs in this change, not a separate one.** Both defects are the reviewer's comments failing to reach the agent when it needs them: one because the template never names them, one because the naming arrives after the proposal is written. Shipping the timing fix alone would deliver, punctually, a directive that still does not mention the comments — which reads as a fix and is not one. Alternative considered: a separate two-task change ahead of this one — rejected because the live check that closes both (task 3.5) is the same run, and splitting it would mean running it twice or trusting the first half untested.
- **Deliver at prompt time *and* keep the Stop path.** They answer different questions: prompt time answers "the reviewer just said something, act on it now", Stop answers "do not go idle while feedback is outstanding". Dropping Stop would break the autonomous case — an agent working unattended has no prompt to ride in on. Alternative considered: moving delivery to prompt time only — rejected because it makes the loop depend on a human typing, which is the opposite of what the MVP set out to prove.
- **Consume-once is inherited, not reinvented.** Both paths call the same `translate`, `load_pending` and `mark_consumed`. Whichever fires first wins and the other finds nothing pending. No coordination, no new state, and the existing "translated verdict is not retranslated" tests already cover the shared mechanism.
- **No promotion check at prompt time.** Promotion detects change directories that appear as a *result* of agent work, so the end of a turn is when there is something new to find. Running it at prompt time would add a subprocess `openspec validate` to the latency of every prompt the reviewer types, for a check that is almost never satisfiable at that moment.
- **This hook fails soft, and that is a deliberate divergence from the Stop path.** `hook stop` propagates errors and exits non-zero, which is safe because the worst case is a turn that ends normally. The same policy here risks refusing the reviewer's prompt, which is worse than the problem being solved: they would lose the ability to type, not merely the feedback. So a failure is reported on stderr and the command exits zero, delivering nothing. This is the one place in the project where fail-fast loses to not wedging the human, and it is confined to a hook whose entire output is advisory context.

## Risks / Trade-offs

- [Risk] **The exit-code contract for a failing `UserPromptSubmit` hook is unverified.** A non-zero exit may be treated as a blocking error and refuse the prompt. The fail-soft decision above is designed on the assumption that it might, but the actual behaviour is documented only for the tool events. → Mitigation: establish it by observation — deliberately fail the hook in a live session and record whether the prompt still goes through — before this change is considered done. The fail-soft path means the answer does not change the design, only our confidence in it.
- [Risk] Both hooks fire for the same directive in a single cycle: prompt time delivers and consumes, then Stop finds nothing and allows. Correct, but it means a directive delivered at prompt time no longer blocks the turn end, so an agent that ignores the context is not forced to continue. → Mitigation: accepted. Prompt-time delivery trades enforcement for timeliness, and the agent that was observed complying in `add-directive-verdict-loop` did so with context it could corroborate, not because it was blocked. If ignoring proves to be a real pattern, the escalation is for Stop to re-inject a directive whose work is not evidenced — which is a separate change and needs a definition of "evidenced" that does not exist yet.
- [Risk] The reviewer's prompt and the directive arrive together, and the directive may point in a different direction than the prompt. → Mitigation: not this change's problem to resolve. The reviewer is the same person in both channels, and an agent given both a prompt and review feedback has strictly more information than one given the prompt alone.
- [Trade-off] Two delivery points is more surface than one. The alternative is a reviewer-visible rule about when feedback lands, which is worse: the current single point is simpler to describe and actively misleads, since the obvious reviewer action produces a turn that ignores it.
