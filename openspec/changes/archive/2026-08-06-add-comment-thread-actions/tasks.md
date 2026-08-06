## 1. Status lifecycle in core

- [x] 1.1 Add `Addressed` to `Status` in `crates/core/src/comments/record.rs`
- [x] 1.2 Extend the sidecar's status-event fold so `addressed` and a reopen back to `open` reconstruct correctly, retaining superseded statuses in the history
- [x] 1.3 Add a status-count function over a scope key, computed from reconstructed status, returning zeros for a scope with no sidecar
- [x] 1.4 Make a status operation naming an unknown comment id fail loudly rather than appending an orphan event
- [x] 1.5 Add tests: open → addressed → resolved, reopen from each later status, history retained across a reopen, counts ignore superseded statuses

## 2. CLI operations

- [x] 2.1 Add `comment address` and `comment reopen` subcommands alongside the existing `add`/`list`/`reply`/`resolve`
- [x] 2.2 Include current status in `comment list` output for all three statuses
- [x] 2.3 Add tests for both new subcommands, including the unknown-comment-id failure

## 3. Close the agent's half of the loop

- [x] 3.1 Update the verdict-to-directive reason templates from `add-directive-verdict-loop` to instruct reply-and-mark-addressed, never resolve
- [x] 3.2 Add a test asserting no template instructs the agent to resolve a comment
- [x] 3.3 Update the comment rule in `AGENTS.md`, which currently reads *"Report back on a comment with `openspec-doc comment reply …`. Do not resolve comments you were asked to address."* Only half of it goes stale: the prohibition on resolving stays exactly as written, because it is the reason `addressed` exists. What changes is that replying is no longer the whole of what an agent can do — it should reply **and** mark the comment addressed, which is the state the rule has been standing in for. A rule telling an agent what not to do, when there is now something it should do instead, reads as an omission rather than a boundary.

## 4. Verification

All from the terminal. This change no longer touches the browser: the interface half moved to `replace-dashboard-frontend`, which is deleting the files it would have edited.

- [x] 4.1 Walk a comment open → addressed → resolved → reopened with `comment list` between each, confirming the reported status and the retained history
- [x] 4.2 Confirm the status counts are correct for a scope holding comments in all three states, and zero for a scope with no sidecar
- [x] 4.3 Confirm no directive template instructs the agent to resolve a comment, by reading the text `hook prompt` emits for each verdict kind rather than by reading the source
