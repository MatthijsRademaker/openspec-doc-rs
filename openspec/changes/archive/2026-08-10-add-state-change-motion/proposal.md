## Why

The dashboard is a live document read by two parties at once, and every change the reviewer did not personally cause arrives silently.

The agent rewrites `proposal.md` on disk. The watcher fires, an SSE frame lands, `ScopeView.replaceLiveState` swaps the artifact array and restores the scroll offset to the pixel. This is correct and required — *Reading position survives an update* asks for exactly that — but its side effect is that the prose the reviewer was mid-sentence on is now different prose, and nothing says so. A good change and no change look identical. The interface has no channel for "that paragraph is new".

The second silence is the link between a passage and its thread. `dashboard-html-views` requires that "the corresponding source block and thread SHALL receive linked visible treatment that does not rely on color alone" when a marker or a thread is activated. The document side honours it well: a double `--focus-ink` border and hairline corner ticks. The conversation side does not. An idle thread already carries a left accent bar, and an active one carries a slightly different one, so clicking a gutter marker changes essentially nothing in the pane the reviewer is being told to look at. Driven side by side, the before and after screenshots of a marker click differ only in the document column. On a 1280px screen those two panes are a thousand pixels apart and the relationship between them is the single claim this product exists to make.

The third silence is smaller and purely mechanical: there is no `<Transition>` anywhere in `web/src` and no `@keyframes`. Every surface in the app appears instantly — the decision drawer and its backdrop, the inline composer that displaces the document by its own height, the conversation rail collapsing, a posted comment materialising in the rail on the next frame. `tw-animate-css` is imported in `style.css` and used zero times.

What exists today is hover polish, and it is finished: `--motion-duration: 160ms`, `--motion-easing: cubic-bezier(0.2, 0, 0.2, 1)`, a global `button, a` transition, the scope-entry lift, the review-block cross-fade, the comment-action reveal, and a reduced-motion block that genuinely zeroes the token. More of that layer would add nothing. Motion's remaining job here is not polish, it is notification.

## What Changes

- An artifact change that arrives from outside the reviewer's own actions is marked where it lands. Scope of this change is arrival, not diffing: the reviewer is told the document was rewritten and where the rewrite arrived, using the `--delivery` token that already exists for this class of event. Per-block change detection is explicitly out of scope and argued below.
- Thread activation becomes legible in both panes. The crosshair already in `.artifact-conversation__crosshair` locks on activation — its two hairlines drawing to their intersection — and the active block's existing corner ticks grow from zero rather than appearing. One gesture, read across the whole width. This is a conformance fix to *Active conversation locks its block visibly* as much as it is new motion.
- A comment the reviewer posts arrives with the same mark as an agent edit, closing the gap between pressing send and seeing the thread exist. Today that gap is filled by disabled buttons.
- The drawer, the inline composer, and the conversation collapse get entry and exit transitions inside the 100–200ms envelope the spec already sets. This is deliberately last on the list: it is the cheapest work here and the least valuable, and it is worth doing only after **`fix-workbench-obstructions`** has moved those surfaces to where they belong.
- The mock lane gains the ability to simulate an agent rewriting an artifact. It cannot today, and this is the reason the arrival mark has no way to be seen or demonstrated.

Deliberately **not** in scope:

**Per-block change detection.** Marking *which* paragraphs differ is the version of this everyone actually wants, and the animation is the easy tenth of it. The hard part is block identity across a rewrite, which is the same problem the anchoring code already wrestles with when a comment's anchor orphans. Building it inside a motion change would produce a change whose real content is a diff algorithm. It is its own proposal, and it should reuse whatever the anchor resolver knows about identity rather than inventing a second answer.

**A loading animation.** The index and scope loading states stay static text. This is where the pressure to build a beautiful orbital loader is strongest and where *Motion explains state without becoming spectacle* is most explicit. A `127.0.0.1` server reading files off disk resolves in milliseconds; a loader would be an animation of nothing.

**Animating the celestial artwork.** Same requirement, same reason. The orbital SVGs and stippled plates stay still.

## Capabilities

### Modified Capabilities

- `dashboard-visual-system`: motion's remit extends from explaining the reviewer's own navigation to marking arrivals the reviewer did not cause, with a bounded mark that decays rather than becoming a persistent state.
- `dashboard-html-views`: an artifact update that arrives without the reviewer acting is reported at the point it lands, not only preserved in place.
- `dashboard-development-harness`: the mock lane can originate an artifact change, so the live-update path is exercisable without a running agent.

## Impact

- `web/src/views/ScopeView.vue` — `replaceLiveState` and `mutate` gain an arrival signal; nothing about scroll preservation or generation guards changes.
- `web/src/components/review/ArtifactDocument.vue`, `ArtifactConversation.vue` — the arrival mark and the two-pane lock.
- `web/src/components/review/DecisionInstrument.vue` — drawer entry and exit.
- `web/src/style.css` — arrival keyframes, the crosshair lock, transitions on the three surfaces, and matching entries in the reduced-motion block. `tw-animate-css` is deleted from the imports and from `package.json`: it has never been used, and a change that adds the project's first real motion should not leave an unused animation library sitting next to it.
- `web/src/mocks/handlers.ts`, `web/src/mocks/data.ts` — an artifact-rewrite trigger and a broadcast that is not a side effect of a reviewer mutation.
- `web/e2e/dashboard.spec.ts` — reduced-motion coverage for every new effect.
- `openspec/specs/dashboard-visual-system/spec.md`, `openspec/specs/dashboard-html-views/spec.md`.

## Sequencing

**After `fix-workbench-obstructions`.** That change moves the decision instrument out of the document's way, fixes where the drawer's focus lands, and stops the collapse re-wrapping the document. Two of the three surfaces this change animates are surfaces that change moves; animating them first means animating them into the wrong position and then moving them.

**`dashboard-development-harness` is complete but not archived**, so the mock-lane capability this change modifies is not yet in `openspec/specs/`. This change should not land before that one is archived, or its delta will target a capability the baseline does not have.

## Risk

The honest one: the crosshair lock may read as a gimmick rather than as an instrument. It is the most expressive thing in this change and the only one whose value is a judgement call rather than an observation. It is also cheap enough to build and delete. If it does not survive being looked at for ten minutes, the linked-treatment requirement can be met with a plain hairline change instead, and that fallback should be taken without argument rather than defended.
