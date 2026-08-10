## Context

Four defects, found by driving the mock dashboard rather than by reading it. Two violate requirements that already exist; two need a requirement written. The interesting design question is not how to fix any one of them — each fix is small — but why a requirement stated in the spec was false in the shipped build, and what to do so the answer is not "write more requirements".

## Why the decision instrument is fixed-positioned today

Not by accident. A `Teleport` to `body` plus `position: fixed` is the standard way to get a modal drawer out of a transformed or overflow-clipped ancestor, and `.scope-layout` is a grid with `overflow: hidden` regions around it. The trigger and the drawer are one component, so the drawer's need for an escape hatch dragged the trigger out of the layout with it. That is the actual cause: the trigger was never separately placed, it inherited the drawer's positioning strategy.

So the fix is not "unset `position: fixed`". It is to stop treating the trigger and the drawer as one positioned unit.

### Decision: the trigger goes into the conversation rail; the drawer stays overlaid

The trigger is a persistent control and belongs in the layout, in the conversation-side instrument region the spec names. The drawer is a modal dialog and should stay an overlay — it is `aria-modal="true"`, it traps focus, it dims a backdrop, and putting a `max-height: 72vh` scrolling panel inside a grid cell would either clip it or stretch the cell.

Considered and rejected: putting both in flow at the foot of the document column. It reads well in the mockup, where `SEND 4 COMMENTS TO AGENT` sits under the document — but the mockup's bar is the *submit* control for a fixed-height document, and ours is a persistent trigger for a scrolling one. In flow at the document foot it would be below the fold on every artifact longer than a viewport, which is worse than overlapping: an obstruction is at least visible.

Considered and rejected: keeping it fixed and adding bottom padding to the document column to clear it. This is the cheap fix and it is what the narrow layout effectively does. At desktop it means reserving a permanent empty strip across the full width of the page for a 209px-wide control that the spec says should not be there at all. It also does not fix the inline-composer collision, because a composer opened near the viewport foot is inside the reserved strip.

### Consequence: the narrow dock stays as it is

At 390px the layout is one column, the conversation rail is at the end of the flow, and a trigger placed in it would be unreachable without scrolling to the bottom of the page. The narrow dock exists for that reason and it currently satisfies its scenario — bar top 791, zero overlapping blocks. It stays. This change makes the two viewports behave differently on purpose, where today they share one strategy that only one of them is suited to.

## Why the drawer opens scrolled past its own heading

`toggleComposer` sets `open`, awaits a tick, and calls `commentInput.focus()`. Focusing an element inside a scroll container scrolls it into view. The textarea is last in the drawer, so the drawer scrolls to the bottom.

Focusing the textarea was a reasonable instinct — the reviewer opened a feedback sheet, presumably to type. It is wrong here because the drawer is not only a composer: it also lists every loose and orphaned comment on the scope, which is content the reviewer needs to read before writing, and it carries a paragraph explaining what the sheet is for. Focus should land where the dialog begins.

### Decision: focus the panel element, not a control inside it

Give the drawer `tabindex="-1"` and focus it. This is the conventional pattern for a dialog with content above its first input, it puts the accessible name that `aria-labelledby` points at into view, and Tab from there reaches the close button and then the threads in reading order, with the textarea at the end where it already is.

Rejected: focusing the textarea with `preventScroll: true`. It fixes the symptom while leaving keyboard focus at the bottom of a panel scrolled to the top, so the first Tab appears to jump backwards, and Shift+Tab from the textarea walks up through content the reviewer has not seen. Rejected: scrolling the drawer to top after focusing. Same disagreement between focus and viewport, arrived at by a longer route.

## Why capping the measure is the right fix for the reflow

`.review-block__content` is `width: min(100%, 72ch)`. `72ch` is a good measure and is not the problem; `min(100%, …)` is, because at ~600px the column is the binding constraint and at ~960px the measure is, so the collapse crosses the boundary and re-wraps everything.

Capping the block content at a fixed measure and letting the surplus become margin means the collapse changes the document's position on screen but not its line breaks. The reviewer keeps their place in the text.

This does raise a question this change does not answer: if collapsing the rail cannot widen the text, what is the collapse *for*? It is for getting the threads out of the way, and for the artwork and hairline chassis to breathe. That is a smaller benefit than "more room for the document", and it is worth saying out loud rather than discovering later that the control now looks pointless. If the answer turns out to be that it is pointless, removing it is a better change than restoring the reflow.

**Recorded outcome.** As implemented, `.review-block__content` is capped at `48ch` — deliberately below the ~487px the document column narrows to at the 58rem breakpoint with the rail expanded, since anything above that reintroduces the boundary crossing this fix exists to remove. So the collapse control's benefit is now exactly "the threads go away", and the decision trigger stays in the rail either way. Whether that is worth a control is a product question for whoever next looks at the rail; the answer "no" means deleting the control, not widening the measure. This is written here rather than as a code comment for that reason.

## Why the rail artwork escapes its rail

`.scope-utility__art` is positioned `absolute; right: 0; bottom: 0; left: 0` with `height: min(46vh, 28rem)`, anchored to the bottom of a `position: sticky` rail whose `min-height` is `calc(100vh - var(--space-3))`. The artwork's height is therefore a fraction of the *viewport*, and the content that might occupy that region is a function of the *artifact count*. Nothing relates the two. Six artifacts is enough for the navigator to reach into it; the fixtures used during the workbench redesign evidently had fewer.

The fix is to make the artwork's region a function of the rail's remaining space rather than of the viewport — the artwork occupies what the content leaves, and disappears when the content leaves nothing. This is what "artwork yields before review content" already means; it was implemented as a fixed fraction because at the time the fraction was always free.

## The real finding

Three of these four are in a spec, or follow directly from one, and all four shipped. The e2e suite asserts reachability and absence of horizontal clipping; none of the assertions ask whether one element is drawn on top of another. A layout requirement expressed as prose — "SHALL NOT cover selected-document prose" — has no failing test until someone writes a bounding-box comparison.

So each fix in this change carries a geometric assertion, not a "renders without error" assertion: compare rectangles, at both configured viewports, with a fixture large enough to trigger the overflow. The fixture size matters as much as the assertion — the artwork defect needs six artifacts to appear, and a suite that keeps a small fixture for speed will keep passing.

That generalizes beyond this change and is the part worth carrying forward: for this workbench, obstruction is a class of bug the current gate structurally cannot catch.
