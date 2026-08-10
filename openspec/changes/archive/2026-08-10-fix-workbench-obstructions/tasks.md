## 1. Give the e2e lane the ability to see an obstruction

Written first on purpose. Three of the four defects violate a requirement that already existed, and all four shipped, because no assertion in the suite compares one element's rectangle to another's. Fixing the layout before the suite can detect the failure repeats the mistake at a later date.

- [x] 1.1 Add a bounding-box overlap helper to `web/e2e/`: given two locators, fail when their rectangles intersect. Report both rectangles in the failure message — an overlap assertion that says only "expected false" costs the next reader a debugging session.
- [x] 1.2 Confirm the e2e fixture yields a change scope with at least six artifacts and an artifact whose rendered document is taller than the desktop viewport. The artwork defect needs the artifact count and the decision-bar defect needs the document length; a fixture kept small for speed keeps both invisible. If the current fixture is smaller, grow it and say so in the change.
- [x] 1.3 Add the four failing assertions from tasks 2–5 *before* their fixes, and confirm each fails against the current build. An obstruction test that never went red is not evidence of anything.

## 2. The decision instrument stops covering the document

- [x] 2.1 Split the trigger from the drawer in `DecisionInstrument.vue`. The trigger renders in place; the drawer keeps its `Teleport` to `body`, because it is `aria-modal`, traps focus, and dims a backdrop, and a `max-height: 72vh` scrolling panel inside a grid cell will either clip or stretch it.
- [x] 2.2 Place the trigger in the conversation-side instrument region at desktop, which is where *Scope pages instantiate the observatory review workbench* already says it belongs. Keep the narrow dock exactly as it is: at 390px the conversation rail is at the end of a single-column flow and a trigger inside it would need a full-page scroll to reach. The two viewports now use different strategies deliberately.
- [x] 2.3 Preserve, across the move: `aria-controls` and `aria-expanded` on the trigger, the focus trap, Escape-to-close, the focus restore to the trigger on close, and the disabled state during a mutation. This is the only always-present control on the page and relocation is the risk in this change.
- [x] 2.4 Verify the trigger remains visible with the conversation rail collapsed. Collapsing the rail must not hide the scope's decision controls.
- [x] 2.5 Assertions: at 1280×800 scrolled to the document foot, the trigger's rectangle does not intersect any `.review-block`; with an inline composer open near the viewport foot, the trigger does not intersect the composer's textarea or its submit control; at 390×844 the existing narrow clearance still holds.

## 3. The drawer opens at its own beginning

- [x] 3.1 Give the drawer panel `tabindex="-1"` and focus the panel on open instead of `commentInput`. Leave the textarea where it is in the DOM.
- [x] 3.2 Do not "fix" this with `preventScroll: true` on the textarea, or by scrolling the panel back to top after focusing it. Both leave keyboard focus and the visible region disagreeing, so the first Tab reads as a jump backwards. A comment saying so belongs next to the focus call.
- [x] 3.3 Assertions: on open, the panel's `scrollTop` is 0 and `#scope-comment-panel-title`'s rectangle is inside the panel's; `document.activeElement` is the panel; forward Tab from open reaches the close control, then the loose threads, then the textarea.

## 4. The document measure stops moving

- [x] 4.1 Cap `.review-block__content` at a fixed measure so surplus column width becomes margin. Keep the narrow behaviour — below the measure, the document uses the available width.
- [x] 4.2 Record in the change that the collapse control's benefit is now "the threads go away", not "the document gets wider". If that turns out not to be worth a control, deleting the control is the better follow-up than restoring the reflow. Do not leave this as a code comment; it is a product question, not an implementation note.
- [x] 4.3 Assertion: capture the rendered text of the first three blocks with the rail expanded and collapsed at 1280×800; the line breaks are identical.

## 5. The rail artwork stays in its rail

- [x] 5.1 Bound `.scope-utility__art` by the space the rail's content leaves free rather than by `min(46vh, 28rem)` of viewport. The artwork shrinks as the artifact list grows and disappears when the list fills the rail.
- [x] 5.2 Check `.scope-conversation__art` for the same construction. It is clear today only because thread lists are short — the same fixed-viewport-fraction bug is present and merely unexpressed.
- [x] 5.3 Assertion: with the six-artifact fixture at 1280×800, no `.artifact-navigator__path` rectangle intersects the artwork region.

## 6. Close out

- [x] 6.1 Update `openspec/specs/dashboard-html-views/spec.md` and `openspec/specs/dashboard-visual-system/spec.md` with this change's deltas.
- [x] 6.2 Note in `designs/dashboard-review-workbench.md` that the decision instrument is rail-placed at desktop and docked at narrow. The document currently describes one persistent decision surface without distinguishing the two.
- [x] 6.3 Run `bun run check` and `bun run test:e2e`. `test:e2e` builds the frontend and the Rust binary and drives the embedded output; a Vite-only pass is not this change's gate, since the whole point is what the shipped build draws on top of what.
- [x] 6.4 Rebuild `web/dist/` before Cargo, per the standing build order.

## 7. Live verification

The mock lane is where these were found and it is enough to confirm the fixes, but the embedded build is what reviewers run.

- [x] 7.1 Against `openspec-doc serve` on this repository — which has six active changes and artifacts long enough to scroll — open a change scope at 1280×800, scroll to the foot of the longest artifact, and confirm the last sentence is fully legible.
- [x] 7.2 Open the scope-feedback drawer on a scope that has loose comments and confirm its title is the first thing visible.
