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

## 3. Thread controls on the page

- [ ] 3.1 Render reply, resolve, and reopen controls per comment in `crates/server/src/page/review.rs`, selected by the comment's current status
- [ ] 3.2 Add the POST routes behind them in `crates/server/src/routes.rs`, for both session and change scopes
- [ ] 3.3 Assert in a test that no rendered control moves a comment to `addressed`
- [ ] 3.4 Render the scope's open/addressed/resolved counts on both page kinds
- [ ] 3.5 Confirm the existing SSE subscription picks up status and reply changes with no new client JS — the sidecar write is already inside the watched tree
- [ ] 3.6 Add tests: a resolved comment renders reopen and not resolve; a status change is pushed to the scope's event stream; the review fragment reflects a status change made after the page was served

## 4. Close the agent's half of the loop

- [ ] 4.1 Update the verdict-to-directive reason templates from `add-directive-verdict-loop` to instruct reply-and-mark-addressed, never resolve
- [ ] 4.2 Add a test asserting no template instructs the agent to resolve a comment
- [ ] 4.3 If `add-directive-verdict-loop` groups 1–2 have not landed, drop 4.1 and 4.2 and confirm the decision is recorded in that change's task 2.1 instead

## 5. Manual verification

- [ ] 5.1 Open a change page in two tabs; resolve a comment in one and confirm the other reflects the new status and updated counts without a reload
- [ ] 5.2 Reply to a comment from the page and confirm the reply renders in the thread
- [ ] 5.3 Reopen a resolved comment and confirm the control set flips back and the counts follow
