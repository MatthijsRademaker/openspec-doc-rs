## 1. Status lifecycle in core

- [ ] 1.1 Add `Addressed` to `Status` in `crates/core/src/comments/record.rs`
- [ ] 1.2 Extend the sidecar's status-event fold so `addressed` and a reopen back to `open` reconstruct correctly, retaining superseded statuses in the history
- [ ] 1.3 Add a status-count function over a scope key, computed from reconstructed status, returning zeros for a scope with no sidecar
- [ ] 1.4 Make a status operation naming an unknown comment id fail loudly rather than appending an orphan event
- [ ] 1.5 Add tests: open → addressed → resolved, reopen from each later status, history retained across a reopen, counts ignore superseded statuses

## 2. CLI operations

- [ ] 2.1 Add `comment address` and `comment reopen` subcommands alongside the existing `add`/`list`/`reply`/`resolve`
- [ ] 2.2 Include current status in `comment list` output for all three statuses
- [ ] 2.3 Add tests for both new subcommands, including the unknown-comment-id failure

## 3. Close the agent's half of the loop

- [ ] 3.1 Update the verdict-to-directive reason templates from `add-directive-verdict-loop` to instruct reply-and-mark-addressed, never resolve
- [ ] 3.2 Add a test asserting no template instructs the agent to resolve a comment

## 4. Verification

All from the terminal. This change no longer touches the browser: the interface half moved to `replace-dashboard-frontend`, which is deleting the files it would have edited.

- [ ] 4.1 Walk a comment open → addressed → resolved → reopened with `comment list` between each, confirming the reported status and the retained history
- [ ] 4.2 Confirm the status counts are correct for a scope holding comments in all three states, and zero for a scope with no sidecar
- [ ] 4.3 Confirm no directive template instructs the agent to resolve a comment, by reading the text `hook prompt` emits for each verdict kind rather than by reading the source
