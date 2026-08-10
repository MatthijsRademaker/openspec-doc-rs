## Why

The scope workbench obstructs its own content at the default desktop size, and it does so in ways the specs already forbid. This was found by driving the mock dashboard at 1280×800 and 390×844, not by reading the code, which is why it survived a green test run.

`dashboard-html-views` already requires, under *Scope pages instantiate the observatory review workbench*: "the persistent decision instrument's trigger and drawer SHALL remain in the conversation-side instrument region and SHALL NOT cover selected-document prose". `DecisionInstrument` teleports to `body` and renders `position: fixed; left: 50%` — dead centre of the viewport, over the document column. Scrolled to the foot of `proposal.md` at 1280×800, the last review block's bottom edge is at y=778 and the decision bar's top edge is at y=747. The proposal's final sentence is printed through. Open an inline composer low in the viewport and the bar lands between the textarea and its `RECORD COMMENT` button.

The narrow case passes: at 390×844 the bar top is 791 and no block overlaps it. The requirement written for the narrow viewport is the one the implementation honours; the desktop scenario next to it is not verified by anything, which is how a stated requirement stayed false.

`dashboard-visual-system` already requires that artwork "SHALL NOT reduce readability". `.scope-utility__art` is a `min(46vh, 28rem)` image behind the instrumentation rail. With six artifacts the navigator's list grows into it and the mono path text for `specs/dashboard-server/spec.md` renders over visible stipple. The conversation rail carries the same artwork and stays clear only because its thread list is short — the defect appears as a function of content, which is why a fixture with fewer artifacts would hide it.

Two further problems have no requirement to violate yet:

Opening the scope-feedback drawer scrolls its own title out of view. `toggleComposer` focuses the textarea, which is the last element in a 920px-tall content box inside a 573px drawer, so the drawer opens at `scrollTop: 347` with `<h2 id="scope-comment-panel-title">` sitting 142px above its top edge. The dialog is `aria-labelledby` that heading: a screen reader announces a title the sighted reviewer has to scroll up to find, along with the paragraph explaining what the sheet is for.

Collapsing the conversation rail rewrites the document. `.review-block__content` is `width: min(100%, 72ch)`, so widening the column from ~600px to ~960px re-wraps every paragraph and two-line paragraphs become one line. A control whose purpose is to give the document more room instead reflows the text the reviewer was reading.

## What Changes

- The decision instrument stops being a page-centred fixed overlay at desktop and moves into the conversation-side instrument region the spec already names. Narrow keeps a dock, because *Narrow decision dock preserves final content* describes one and that behaviour is correct today.
- Opening the drawer focuses the drawer, not the textarea, so the panel opens at its own beginning with the heading the dialog is labelled by on screen. The textarea remains one Tab away.
- The document measure is capped so that collapsing the conversation rail adds margin rather than line length. Chrome changes stop re-wrapping prose.
- The instrumentation rail's artwork is bounded by the rail's content rather than by the viewport, so a long artifact list cannot come to rest on top of it.
- Each of the four gets a browser assertion in the existing e2e lane. The decision-bar overlap is the case that matters most: a requirement stated in `dashboard-html-views` since the workbench redesign has been false in the shipped build, and nothing noticed.
- The e2e fixture's `proposal.md` grows from six blocks to nineteen. It already carried six artifacts, but at six blocks it fit on one desktop screen, so no scroll position existed at which the decision bar could reach the document's foot. A fixture that cannot reach the failing state is not a smaller version of the gate, it is no gate.
- `Desktop decision instrument leaves document unobstructed` required the *drawer*, not only the trigger, to live in the conversation-side rail. That is not what a focus-trapping backdropped modal is, and the shipped drawer never did it. The scenario splits: the trigger is placed and never intersects content, the drawer is allowed to overlay for as long as a reviewer holds it open.

Deliberately **not** in scope:

The session scope stating its title three times — once in the scope header, once as the artifact header's `Exploration scratch`, once as the scratch note's own `#` heading rendered at `clamp(2.5rem, 5vw, 4rem)`. This looked like redundancy until it turned out to be specified: *Route identity does not compete with document gravity* requires that "the selected artifact's source heading SHALL retain primary document gravity and SHALL NOT be hidden or rewritten". Two of the three statements are deliberate. Changing it means arguing against that requirement on its merits, which is a different proposal than this one.

All motion work. This change moves things to where they belong and fixes what focus lands on; **`add-state-change-motion`** animates them once they land in the right place. Sequenced after this one, because animating the drawer into its current position would only make a misplaced panel arrive more gracefully.

## Capabilities

### Modified Capabilities

- `dashboard-html-views`: a modal review surface must open showing its own purpose, rather than scrolled past it by focus placement; and the workbench's decision surfaces are specified as two things — a placed trigger and a modal drawer — rather than one positioned unit.
- `dashboard-visual-system`: the document's readable measure is stable across chrome changes; artwork is bounded by the content it sits behind, not only by the document prose it must not cover.

## Impact

- `web/src/components/review/DecisionInstrument.vue` — the `Teleport`, the fixed positioning, and the open-time focus target.
- `web/src/views/ScopeView.vue` — the decision instrument's position in the layout, and the collapsed-rail grid template.
- `web/src/style.css` — `.decision-instrument*`, `.review-block__content` measure, `.scope-utility__art` bounds.
- `web/e2e/geometry.ts` — new: rectangle-overlap and containment assertions the suite had no way to express.
- `web/e2e/fixture-source.ts` — new: the fixture proposal, shared by the launcher that writes it and the spec that rewrites it.
- `web/e2e/dashboard.spec.ts` — four regression assertions, at both configured viewports.
- `openspec/specs/dashboard-html-views/spec.md`, `openspec/specs/dashboard-visual-system/spec.md` — two new requirements and two modified ones.
- `designs/dashboard-review-workbench.md` — the two decision-instrument strategies, and what the collapse control is now for.

No Rust, API, or router change. `web/dist/` must be rebuilt before Cargo, per the standing build order.

The decision-instrument move is the one change here that can regress something real: it is the only always-present control, and the narrow dock, the focus trap, the Escape handling, and the `aria-controls` relationship all have to survive relocation. The e2e assertions are the gate, not the unit tests.
