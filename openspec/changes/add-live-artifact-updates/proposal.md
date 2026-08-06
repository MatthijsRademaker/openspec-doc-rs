## Why

The vision's headline is that an exploration is readable in a browser *while it is happening*. It is not. The artifact never live-updates.

Everything needed already exists and one line throws it away. `scope::target` deliberately watches the artifact tree, so the watcher fires when the agent rewrites its note, and an SSE event reaches the browser. The client then refetches only the comments-and-verdicts fragment. The artifact itself is never refetched.

On a session page the scratch note *is* the only artifact, so the reviewer opens the page the product is sold on and watches an exploration that does not move. `add-dashboard-lifecycle` found the sharper version of this: a browser opened at explore time shows "No artifact on disk to review yet", the agent writes the note one turn later, and the page stays empty forever with no sign that reloading would help.

Fixing it is not simply "refetch everything on every event". Two things stand in the way, and both are why this is its own change rather than a line in the migration:

- **The event does not say what changed.** A client that cannot tell an artifact change from a comment arriving must refetch everything every time, including the artifact under a reviewer who is mid-sentence.
- **Replacing the document under an open composer destroys unsent text.** That is the reason the current client refetches only the fragment, and it is a real reason. Swapping the artifact out from under a half-written comment is worse than not updating at all.

## What Changes

- **The SSE event says what changed** — artifacts, review state, or both. The watcher already distinguishes them: `scope::target` records the artifact paths and the sidecar paths separately.
- **The client refetches artifacts when the event says artifacts changed**, which is the defect.
- **The client tracks whether a composer holds unsent text.** While it does, an artifact change is *offered* rather than applied; once it does not, pending updates apply silently.
- **Scroll position survives an artifact refresh.** An update that returns the reviewer to the top of a 400-line proposal is an update they will learn to dread.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dashboard-server`: the update event gains what changed.
- `dashboard-html-views`: live update extends from the comment list and verdict state to the rendered artifact, with the deferral rule.

## Impact

- `crates/server/src/watch/` and the SSE emission — the event payload gains what changed.
- `crates/server/src/routes.rs` — the SSE handler.
- `web/` — the event handling, the dirty-composer tracking, the refresh affordance, scroll preservation.
- `openspec/specs/dashboard-server/spec.md` and `openspec/specs/dashboard-html-views/spec.md`.
- `openspec/changes/add-dashboard-lifecycle/design.md` — it names the change that fixes this defect.

## Dependencies and ordering

- **`migrate-dashboard-review-to-vue` must land first.** There is no composer to be dirty and no rendered artifact to refresh until the scope pages are Vue.
- **`add-dashboard-lifecycle` is adjacent, not blocking.** It opens a browser at the right moment; this change makes the page it opens worth having open. Its design cites the live-update defect and the change that fixes it, so that reference moves here.
- **Nothing depends on this**, which is why it is separable at all. Artifacts do not live-update today, so `migrate-dashboard-review-to-vue` shipping without it is not a regression — it is the same behaviour in a better-looking page.

## What is given up, deliberately

**A reviewer with an open composer can be looking at a stale artifact.** They are told so, and the alternative is losing their sentence. The choice is deliberate and the affordance is what makes it honest.
