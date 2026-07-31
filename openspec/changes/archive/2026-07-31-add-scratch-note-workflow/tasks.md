## 1. Scratch note file lifecycle

- [x] 1.1 Define the session-scoped scratch note path (`.openspec-doc/scratch/_session/<session_id>.md`) and a helper to read/create it
- [x] 1.2 Add tests: note persists untouched while no change directory exists for the session

## 2. Promotion detection

- [x] 2.1 Implement an active-changes snapshot capture (list of change names at a point in time)
- [x] 2.2 Implement diff-based detection of a newly appeared active change since the last snapshot, scoped to sessions with a pending scratch note
- [x] 2.3 Implement the ambiguity guard: skip auto-promotion and record a notice when more than one new active change appears between snapshots
- [x] 2.4 Add tests for: single new change promotes cleanly; zero new changes is a no-op; multiple new changes triggers the ambiguity guard

## 3. Rename and validate

- [x] 3.1 Implement the rename from `.openspec-doc/scratch/_session/<session_id>.md` to `.openspec-doc/scratch/<name>.md`
- [x] 3.2 Leave a moved-pointer record at the old path referencing the new path
- [x] 3.3 Run `openspec validate <name>` after rename and record pass/fail plus output as part of the promotion result
- [x] 3.4 Add tests: rename plus moved-pointer plus validate outcome recorded, for both a passing and a failing validation

## 4. Manual verification

- [x] 4.1 Simulate a session scratch note, create a matching `openspec/changes/<name>/proposal.md` by hand, trigger promotion, and confirm the rename, pointer, and validate result all land correctly
