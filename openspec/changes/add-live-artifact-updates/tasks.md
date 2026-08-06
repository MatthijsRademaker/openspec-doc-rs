## 0. Before starting

- [ ] 0.1 Confirm `migrate-dashboard-review-to-vue` has landed. There is no composer to be dirty and no rendered artifact to refresh until it has

## 1. The event says what changed

- [ ] 1.1 Extend the SSE event to indicate whether the scope's artifacts changed, its review state changed, or both. `scope::target` already records the artifact paths and the sidecar paths separately, so the distinction exists at the point of emission and is simply not carried
- [ ] 1.2 Debounce, so one logical rewrite produces one event rather than however many the watcher backend chooses to fire
- [ ] 1.3 Tests: an artifact write is reported as an artifact change; a comment is reported as a review-state change; a write touching both is reported as both

## 2. Artifacts refetch

- [ ] 2.1 Refetch and re-render the scope's artifacts when the event says artifacts changed. This is the defect: the watcher already fires and the client throws the information away
- [ ] 2.2 Keep review-state refetching as it is, and never defer it — a comment arriving does not disturb what the reviewer is typing, and holding it back would make the counts lie
- [ ] 2.3 Preserve scroll position across an artifact refresh. An update that returns the reviewer to the top of a 400-line proposal is one they will learn to suppress, and suppressing it defeats the mechanism

## 3. Deferral while a composer is dirty

- [ ] 3.1 Track whether a composer is open with unsent text. An empty textarea is not dirty; one character is
- [ ] 3.2 While it is dirty, indicate that the artifact has changed and do not replace the content
- [ ] 3.3 Apply the pending update once the composer is sent or dismissed
- [ ] 3.4 Hold one pending flag rather than a queue — two artifact changes while dirty means only the latest matters
- [ ] 3.5 Tests: an event with a clean composer applies; an event with a dirty composer defers and then applies on send; a second event while already deferred does not stack

## 4. Specs and docs

- [ ] 4.1 Update `dashboard-server` for the event that distinguishes what changed
- [ ] 4.2 Update `dashboard-html-views` for live artifact updates and the deferral rule
- [ ] 4.3 Update `openspec/changes/add-dashboard-lifecycle/design.md`, which names the change that fixes this defect
- [ ] 4.4 Update `docs/docs/` wherever it describes what live update covers

## 5. Verify by hand

This cannot be settled by a test. It is the claim the product is sold on and it has never been observed working.

- [ ] 5.1 Open a session page, have an agent rewrite the note, and confirm the note updates without a reload
- [ ] 5.2 Open a session page at explore time, before the note exists, and confirm it fills in when the agent writes it rather than staying on "No artifact on disk to review yet"
- [ ] 5.3 Rewrite an artifact under an open page with a composer holding text, and confirm the refresh is offered rather than applied
- [ ] 5.4 Rewrite an artifact under an open page with nothing unsent, and confirm it updates silently and the scroll position holds
- [ ] 5.5 Confirm an agent rewriting a note produces one refresh, not a burst
