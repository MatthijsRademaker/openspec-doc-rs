## 1. The claim

- [x] 1.1 Add a claim marker to `scratch::note` reusing the `<!-- openspec-doc:… -->` convention the moved-to pointer already uses, with a reader that returns the claimed change or `None`
- [x] 1.2 Add a writer that records a claim while preserving the note's existing content, and replaces rather than appends when a claim is already present
- [x] 1.3 Add `openspec-doc scratch claim --session <id> --change <name>` over that writer
- [x] 1.4 Add tests: a claim round-trips, an exploration survives being claimed, re-claiming replaces, and a note with no marker reads as no claim

## 2. Rewrite promotion around the claim

- [x] 2.1 Refuse a session whose note is a moved-pointer record, checked before anything else. Independent of the claim work — it is a second defect that would survive fixing attribution by any other means.
- [x] 2.2 Resolve the target from the session's claim, and verify the claimed change is an active change before acting
- [x] 2.3 Take no promotion action when there is no claim, when the claim names no active change, or when the session has no note
- [x] 2.4 Leave rename, moved-pointer, comment relocation and validate exactly as they are — only target selection changes
- [x] 2.5 Add tests for each refusal path, and one that a claimed promotion still renames, redirects, relocates comments and validates

## 3. Delete the old mechanism

- [x] 3.1 Delete `crates/core/src/scratch/snapshot.rs` and its module wiring
- [x] 3.2 Remove `Promotion::Ambiguous` and narrow the `hook stop` reporting path that matched on it. While the variant exists so does the concept, and the deletion is not real until it is gone.
- [x] 3.3 Delete the now-dead `.openspec-doc/scratch/_session/*.changes.json` files from this repo rather than leaving a reader to reconstruct a mechanism that no longer exists
- [x] 3.4 Rewrite the purpose statement in `openspec/specs/scratch-note-workflow/spec.md`, which names the snapshot diff and the rejected alternative explicitly. A delta that changes requirements while the purpose still contradicts them is worse than not having written it.
- [x] 3.5 Confirm no non-test caller of the snapshot API remains, the same check that found `scratch::ensure_session` had none

## 4. Wire the claim into the directive

- [x] 4.1 Extend the move-to-proposal reason template with the claim command, session id already resolved, consistent with the existing rule that directives carry resolved paths rather than templates to interpolate
- [x] 4.1a Do the same for the explore instruction, which told every agent entering explore mode that the note is promoted "once a change directory exists". Found by the 5.3 run, not by inspection: task 4 scoped the wiring to the move-to-proposal directive, but this is the other place the tool describes promotion to an agent, and it was describing the mechanism being deleted. `AGENTS.md` said it too. Adds a delta for `explore-note-capture`, whose rationale prose also named the removed mechanism.
- [x] 4.2 Add a test that the template names the claim command and the session id
- [x] 4.3 Re-check the template against the guards `add-directive-verdict-loop` established: still an attributed pointer, still names an in-project path, still demands no literal output, still says nothing about resolving comments

## 5. Verify against the failure that motivated this

- [x] 5.1 Add a regression test for the attribution defect: two sessions, each with a note, one change appearing, and neither promoted without a claim — the scenario that nearly renamed a 29KB exploration onto an unrelated change
- [x] 5.2 Add a regression test for the already-promoted defect: a promoted session, a later change, and its redirect left untouched
- [x] 5.3 Run the live loop once end to end — explore, move-to-proposal verdict, agent claims and creates the change, next Stop promotes — and confirm promotion happens for the claimed change while a second idle session with its own note is untouched. **PASSED against the built binary, with two sessions concurrent throughout.** `hook explore` for both, notes written for both, two turn boundaries with a valid change already sitting in `openspec/changes/` and neither session promoted; move-to-proposal verdict on the working session; its next boundary injected the directive naming `openspec-doc scratch claim --session aaaaaaaa-… --change <the change you created>`; the change was created and the claim command run as the directive named it; the *idle* session's next boundary was a no-op with that change present — the case that used to mis-promote; the working session's next boundary promoted the note to `.openspec-doc/scratch/add-real-thing.md` with its content and its claim marker intact, left the redirect behind, and validated clean. Two further boundaries after that changed nothing: the redirect was not re-promoted and the promoted note was not overwritten. What was **not** live: the dashboard (the verdict record was written directly) and the agent's turn boundary (`hook stop` was invoked directly rather than by a running session). Both halves of that were proven live on real sessions across two agents by `add-directive-verdict-loop` 3.2–3.4; what this change adds on top of them is one command in a template, covered by 4.2.
- [x] 5.4 Record the outcome. The bug this fixes was found by inspection rather than by any test, so a passing suite is not evidence that the replacement behaves under concurrency. **The mechanism is sound; one residual risk is behavioural, not mechanical.** Confidence that no session can now be promoted onto another session's change is **high**, and it rests on the shape of the fix rather than on the tests: promotion reads one thing, a marker inside the session's own note, and there is no path by which a change directory's appearance reaches it. Concurrency is no longer a variable — 5.1 and `only_the_claiming_session_is_promoted` run two sessions at once, but more to the point there is nothing left for a second session to perturb. Checked against this repo's live state as it stands: three session notes, one of them the `a020f7e4` redirect that was the second defect's live instance, five active changes, zero claims — therefore zero promotions, where the old code had each stale note one single-new-change away from firing. Residual risk, **moderate**: an agent that formalizes an exploration and never runs the claim command. That degrades to a visible un-promoted note rather than a destroyed one, which is the trade this change was made for, but it has not been observed either way on a live session yet; the first real `/opsx:explore` → proposal cycle after this lands is the check. Two defects found by this task rather than by the tests, both fixed: the `hook explore` instruction still told the agent promotion happens "once a change directory exists", and `AGENTS.md` said the same — the tool would have been describing a mechanism it no longer has.
