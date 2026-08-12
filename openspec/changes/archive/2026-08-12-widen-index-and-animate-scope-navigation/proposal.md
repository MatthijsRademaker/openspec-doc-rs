## Why

The index and the scope workbench look like two products. The scope route runs edge to edge; the index stops growing at 1440px and centres the remainder, so on a 2560px display it renders as a column with roughly 560px of dead canvas on each side while the page beside it fills the screen. The two widths are written as independent copies of the same decision in `.observatory-shell` and `.scope-workbench`, which is how they came to disagree.

Navigating between those two pages is worse than under-animated. `view-transition-name: scope-coordinate` is declared in exactly two places, and the destination one is `.scope-route-acquisition__key` — the `<code>` element inside the *loading placeholder*, not `ScopeHeader`. Clicking a change title morphs it into the loading state's key text; the real header then appears with no transition at all, because the view transition has already finished and `::view-transition-old(root) { opacity: 0 }` discards the root snapshot instantly. The return leg has nothing: `ScopeView.vue:518` is a plain `RouterLink` with no transition, and browser Back is unhooked in both directions.

The index also still reports acquisition with the treatment `add-observatory-motion-language` removed from the scope route — `.scope-entry::before` is a lit bar growing across the entry, which is the shape the spec's own reference-gesture requirement names and rejects.

## What Changes

- State the shell's width and padding once as `--shell-width` and `--shell-padding` and have both `.observatory-shell` and `.scope-workbench` consume them, removing the index's 1440px ceiling and centred margin so the index grows with the window at the scope route's tighter padding.
- Keep the index's layout structure exactly as it is. The registers, their proportions, their separation, and the plate strip do not change; only the width the page is allowed to occupy does.
- Make the index hero proportional at every width by deleting the `42rem` cap on `.index-observation__content`, so the text panel, the divider pinned at 52%, and the artwork clipped at 46% stay in agreement instead of opening a bare band that grows with the display.
- Let the observation field bleed out of the hero and continue as the background behind the change and session panelling, through a declared fade, with the opaque registers reading as panels over it. Artwork remains still, non-interactive, and absent from the accessibility tree.
- Move `view-transition-name: scope-coordinate` off the loading placeholder and onto the real scope identity in `ScopeHeader`, and hold the navigation until the scope has resolved so the gesture terminates at that real header rather than at scaffolding. The hold has a declared ceiling past which the route commits without the gesture.
- Give the return leg and browser Back the same treatment, by owning index↔scope route transitions at the Router rather than at the two link sites, while leaving artifact-query navigation on its existing paired-coordinate path.
- Scope the root-snapshot suppression to artifact replacement, which is what it was written for, so a route change between two different pages can carry a page-level treatment instead of inheriting a hard cut.
- Replace the index's lit-bar acquisition with the reference gesture the spec already defines, so the index and the scope route report the same event the same way.
- Permit motion to hold a navigation before it commits, replacing the blanket prohibition on delaying content. The permission is bounded: the origin stays rendered and interactive throughout, the hold has a declared ceiling, and a slow or failed load never leaves an action unanswered.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dashboard-visual-system`: State that both routes occupy the full viewport width from one shared shell declaration; permit the index observation field to continue behind opaque panelling through a declared fade; replace the absolute no-delay motion rule with a bounded permission to hold a navigation so a gesture can reach a real destination.
- `dashboard-html-views`: Require index↔scope acquisition to terminate at the real scope identity in both directions including Back, and require the index to use the full viewport width with the bleeding observation field, without changing its content, reading order, routes, or narrow-width guarantees.

## Impact

- `web/src/style.css`: shell width and padding tokens, index hero proportions, the bleed layer and its fade, the replacement acquisition treatment, and the scoping of the root-snapshot suppression.
- `web/src/views/IndexView.vue`: the bleed layer's element and the returning-coordinate name on the entry being navigated back to.
- `web/src/components/ScopeRegister.vue`: acquisition treatment and the move of navigation ownership to the Router.
- `web/src/components/review/ScopeHeader.vue`: carries the destination coordinate name.
- `web/src/views/ScopeView.vue`: the plain index `RouterLink` becomes a transitioned route change; the loading placeholder stops claiming the coordinate name.
- `web/src/lib/view-transition.ts` and `web/src/router.ts`: route-level transition ownership, the navigation hold and its ceiling, and the `route` transition marker the scoped root rule reads.
- `openspec/specs/dashboard-visual-system/spec.md` and `openspec/specs/dashboard-html-views/spec.md`: two requirements each currently forbid what this change does — artwork "SHALL NOT sit beneath text" on the index, and the Router "SHALL navigate to its identifier-keyed route immediately". Both move here rather than in a later cleanup.
- `designs/dashboard-review-workbench.md`: the index art-direction contract gains the bleed region and its fade.
- Index and register component tests, `web/src/components/VisualSystem.test.ts`, and `web/e2e/dashboard.spec.ts`: width, bleed legibility, both transition directions, Back, and reduced motion.
- No Rust route, index or scope API payload, package dependency, motion dependency, or embedded-asset boundary change. The bleed reuses `observatory-field.webp`, so the allowlist and the 6 MiB runtime raster budget are untouched.
