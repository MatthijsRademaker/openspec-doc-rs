## Context

This is the last of the four changes that replace the dashboard's interface, and the only one that delivers a claim the product already makes. The other three fix things that are ugly, awkward or risky. This one fixes something that is stated and false.

It is separate from the migration because it is the only part with a design decision that is not obvious, and because putting it inside a thirty-five-task change means it gets built at the end, tired, by someone who wants to be finished.

## Goals / Non-Goals

**Goals:**
- The artifact on screen matches the artifact on disk.
- Achieving that never costs the reviewer a half-written sentence.
- The client refetches what changed and not more.

**Non-Goals:**
- Not a diff or a change highlight. The new content replaces the old; showing what moved is a different feature.
- Not conflict handling. There is one reviewer and the agent never edits comments.
- Not making the watcher more granular than per-scope. The event says *which kind* of thing changed, not which file.

## Decisions

### The event says what changed

Today the event is a bare notification. A client receiving it knows only that something under the scope moved, so it has three options: refetch everything, refetch the cheap thing, or guess. The current client refetches the cheap thing, which is why the defect exists.

`scope::target` already records the artifact paths and the sidecar paths separately, so the distinction is available at the point the event is emitted and is simply not carried. The event gains it: artifacts, review state, or both.

Rejected: having the client refetch everything and diff. It moves the decision to the wrong side of the wire and refetches a 32KB artifact every time a comment is resolved.

### Offer while dirty, apply when clean

The client tracks exactly one thing: whether a composer is open with unsent text.

```
SSE event says artifacts changed
      │
      ├── composer has unsent text ──► show "this has changed — refresh"
      │                                  │
      │                                  └── on send or cancel ──► apply silently
      │
      └── nothing unsent ─────────────► apply silently
```

Nothing is ever yanked out from under someone mid-sentence, and nobody is made to click a banner they do not care about.

Rejected alternatives:

- **Always swap.** Loses a half-written comment. This is exactly why the current client refetches only the fragment, and reproducing that mistake with better rendering is not progress.
- **Always offer.** A banner on every update, on a page whose entire point is that it updates. The reviewer clicks it reflexively within a day and it has become a reload button.
- **Preserve the composer and swap around it.** Sounds better and is not: the composer is anchored to a block, and the block may not exist after the rewrite. Then the reviewer is typing into a comment that will anchor to nothing, which is worse than being told the page is stale.

Review-state events are never deferred. A comment arriving does not disturb what the reviewer is typing, and holding it back would make the counts lie.

### Scroll position is part of the feature, not polish

An artifact refresh that returns the reviewer to the top of a 400-line proposal is a refresh they will learn to dread, and dreading it means keeping a composer open to suppress it. The mechanism then defeats itself. Preserving scroll is therefore a requirement of the update, not a nicety attached to it.

### This has to be verified by hand

There is no test that settles it. The claim is "open a page, have an agent rewrite the note, and watch it move", and every part of that — the watcher firing on the real filesystem, the debounce, the event reaching a real EventSource, the refetch, the render — is integration a unit test stubs out. It has never been observed working. So the change is not done until it has been watched happening, and that is a task rather than a hope.

## Risks / Trade-offs

- **Filesystem watchers fire more than once per logical write.** An agent rewriting a note may produce several events; without debouncing the page refetches repeatedly. `dashboard-server` already requires a polling fallback when the watcher fails to initialize, so the surrounding machinery exists — the debounce is the part this change must not assume.
- **"Unsent text" is a judgement.** An open composer with an empty textarea is not dirty; one with a single character is. Getting this wrong in the strict direction means the page stops updating whenever a composer is open, which is a quiet regression to today's behaviour with extra machinery.
- **A deferred update can be superseded.** Two artifact changes arrive while the composer is dirty; only the latest matters. The pending state is a flag, not a queue.

## Migration Plan

Nothing to migrate. The event payload changes and the only client is shipped in the same binary.

## Open Questions

None.
