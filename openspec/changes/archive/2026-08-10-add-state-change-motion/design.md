## Context

The project's motion rule is restrictive on purpose: 100–200ms, opacity and slight lateral movement, no perpetual motion, no parallax, no animated loading ornament. The rule exists because the visual language is full of orbital diagrams and crosshairs that are begging to be spun, and because a review tool that draws attention to itself is a review tool that is harder to read.

A proposal that adds motion to this codebase therefore has to answer, for every effect, *what state change does this explain?* The three effects below each answer it. The ones that could not are in the proposal's not-in-scope list, and the loading state is the clearest of them: there is no state to explain, only latency to decorate.

## The arrival mark

### Why arrival and not diff

The valuable version of "the agent changed the document" names the paragraphs that changed. That requires knowing that a block in the new artifact array *is* a block from the old one with different text — block identity across a rewrite.

The codebase already has an opinion about this problem, in the anchor resolver: a comment's anchor carries quoted text, a heading path, and offsets, and resolving it after an edit is exactly the question of whether this passage is still that passage. It gets it wrong sometimes, which is why `anchorState` has an `orphaned` value and why the interface has a surface for comments whose anchor was lost.

So per-block diffing is not a small addition to a motion change. It is a second implementation of the hardest existing problem in the product, and if the two disagree the interface will mark a block as unchanged while orphaning a comment anchored to it. That is worse than not marking anything.

Arrival is the part that needs no identity: *this artifact was rewritten, and the rewrite is now on your screen*. It answers the reviewer's actual question — "did something change while I was reading?" — and it composes with the diffing change later rather than being replaced by it.

### Why a decaying mark rather than a state

A mark that persists becomes a state, and a state needs rules: when does it clear, does it survive a route change, does an artifact you have not looked at accumulate marks. The `--delivery` token already exists for events, and an event is a thing that happened, not a condition that holds.

So the mark fades in, holds long enough to be noticed by someone who was looking elsewhere, and fades out. The hold is longer than the 100–200ms transition envelope, and that is not a violation: the envelope governs how long a transition takes to *complete*, and the requirement's own words are that a transition "SHALL not delay access to content or controls". A mark that holds for a second and blocks nothing satisfies that. This distinction should be stated in the spec rather than left for someone to litigate later, because the naive reading — "all motion must finish in 200ms" — would forbid the only useful form of this.

### Where the mark goes

At the document, not in a banner. `.scope-artifact-update` already exists as a banner and only appears when a composer is dirty; promoting it to always-on would put the notice at the top of the page while the reviewer is scrolled to the middle of the document. The mark belongs where the reviewer is looking, which for an arrival with no block identity means the document stage as a whole — its edge, not one of its paragraphs.

## The two-pane lock

### The actual defect

The requirement is met on one side. `.review-block--active` gets a 4px double `--focus-ink` border and two hairline corner ticks, which is unambiguous. `.artifact-conversation__thread--active` gets a 4px double border where the idle thread already has a 2px accent top border — a small difference on a surface the reviewer is being told to look at, at the far side of the screen from the thing they clicked.

This is a conformance failure before it is a motion opportunity, and it could be fixed with no motion at all by making the active thread's treatment differ more from its idle one. That fix should be made regardless; motion is what makes the two panes read as *one* gesture rather than two independent changes that happened to coincide.

### Why the crosshair

`.artifact-conversation__crosshair` already exists — a circle with two hairlines crossing it — and it is currently decoration in the conversation pane's empty state. A crosshair that locks is the product's own idiom for "target acquired", it is in the mockup's vocabulary, and it costs two transitioned lengths.

Pairing it with the document block's corner ticks growing from zero gives the two panes a shared verb. The reviewer clicks a marker; something converges in the document and something converges in the rail, in the same 140ms, with the same easing.

### Why this might be wrong

It is the one effect here whose value cannot be established by measurement. An arrival mark is right because the reviewer currently cannot tell whether the document changed. A drawer transition is right because a 42rem panel appearing in one frame is jarring. A crosshair lock is right only if it reads as an instrument rather than as a flourish, and that is a judgement that survives or does not survive being looked at repeatedly.

The mitigation is not to argue it in advance. It is to build it, look at it for a while, and delete it without ceremony if it does not hold up — falling back to a plain hairline difference, which meets the requirement on its own. Naming the fallback now is what makes deleting it cheap later.

## Transitions on the three surfaces

Nothing subtle. The drawer and its backdrop fade and rise; the inline composer expands so its displacement of the document is legible as the composer opening rather than as the page jolting; the conversation collapse interpolates.

One implementation note that will otherwise cost someone an afternoon: `.scope-layout`'s `grid-template-columns` uses `clamp()` and `minmax()`, and those do not interpolate reliably. The collapse either transitions between fixed lengths at the endpoints or uses a different mechanism. Discovering this halfway through and reaching for a JavaScript height animation is the failure mode to avoid.

These are sequenced last because they are the cheapest and least valuable work in the change, and because `fix-workbench-obstructions` moves two of the three surfaces. Animating a panel into a position that a queued change is about to alter is wasted work twice.

## Reduced motion

The existing block zeroes `--motion-duration` and hard-disables `transform` on a hand-maintained list of selectors. Every effect added here needs an entry, and a hand-maintained list is exactly the thing that goes stale — the next person to add a transition will not know the list exists.

Two options. Keep the list and add to it, accepting the maintenance burden. Or invert it: make the reduced-motion block disable animation and transform globally with a narrow allowlist, so a new effect is disabled by default and has to opt out. The second is more robust and is a bigger change to a file this proposal is otherwise only extending. The recommendation is the second, because "the reduced-motion rule silently stopped covering everything" is a failure nobody notices, and this change is the moment when the list is about to grow enough to matter.

## Deleting `tw-animate-css`

It has been imported since the Tailwind v4 setup and used zero times. A change that adds the project's first real motion is the right moment to remove it: leaving an unused animation library beside hand-written keyframes invites the next contributor to assume it is the sanctioned way to animate, and to add a second, inconsistent motion vocabulary.
