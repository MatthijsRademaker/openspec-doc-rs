# Tasks

## 1. Shared shell width

- [x] 1.1 Add `--shell-width: 100%` and `--shell-padding: clamp(0.25rem, 1.5vw, 1rem)` to `:root` in `web/src/style.css`.
- [x] 1.2 Rewrite `.observatory-shell` to consume both tokens and drop `width: min(100%, 90rem)` and `margin: 0 auto`.
- [x] 1.3 Rewrite `.scope-workbench` to consume the same tokens, leaving its narrow-width `padding-bottom: 7rem` dock allowance intact.
- [x] 1.4 Confirm no other rule restates a shell width or shell padding; delete any that does rather than syncing it.

## 2. Index hero proportions

- [x] 2.1 Delete the `42rem` cap on `.index-observation__content` so the panel is `width: 52%`, matching the divider at `left: 52%` and the artwork crop at `46%`.
- [x] 2.2 Verify at 1280px, 1920px, and 2560px that no bare band opens between the panel edge and the artwork, and that the divider stays on the panel edge at every width.
- [x] 2.3 Confirm the narrow-width hero crop is unchanged.

## 3. Observation field bleed

- [x] 3.1 Promote the observation artwork out of `.index-observation` into a page-level `aria-hidden` field layer in `IndexView.vue`, positioned behind the hero and the workbench.
- [x] 3.2 Give `.observatory-index` the positioning and stacking context the layer needs; put the hero content, the registers, and the plate strip above it.
- [x] 3.3 Remove the hero's `overflow: hidden` clip and bottom border where they would cut the field off at the band edge, keeping the divider as hero chrome.
- [x] 3.4 Declare the fade: the field reaches full strength only in the hero's free space and arrives at the top of the workbench already attenuated, decaying further down it.
- [x] 3.5 End the field above `.index-plates` so it does not compound with the plate artwork.
- [x] 3.6 Confirm `.ruled-register` remains fully opaque so no register text is ever read against the field.
- [x] 3.7 Suppress the bleed at 390px, where the spec requires atmosphere to go before content.
- [x] 3.8 Confirm the field reuses `observatory-field.webp` and that no new raster enters the allowlist or the 6 MiB budget.

## 4. Destination coordinate

- [x] 4.1 Remove `view-transition-name: scope-coordinate` from `.scope-route-acquisition__key`; the loading placeholder stops claiming the coordinate.
- [x] 4.2 Put the coordinate name on the real scope identity rendered by `ScopeHeader.vue`.
- [x] 4.3 Confirm the name is present on exactly one element per page in each direction, so the browser has one origin and one destination and never two.

## 5. Navigation hold

- [x] 5.1 Prefetch the scope detail before committing index-to-scope navigation, and hand the fetched detail to `ScopeView` rather than letting it fetch again.
- [x] 5.2 Measure a local scope fetch against the deterministic fixture and set the hold ceiling from that measurement; record the number and its basis in a comment at the constant.
- [x] 5.3 Commit the route immediately without the gesture when the ceiling elapses.
- [x] 5.4 Commit the route immediately when the prefetch fails, so the scope's own failure state reports the error rather than the click appearing to do nothing.
- [x] 5.5 Keep the index rendered and interactive for the whole hold.
- [x] 5.6 Skip the hold entirely under reduced motion.

## 6. Router-owned route transitions

- [x] 6.1 Move index↔scope transition ownership from the two link sites to the Router, keyed on the route name changing.
- [x] 6.2 Replace the plain `RouterLink to="/"` at `ScopeView.vue:518` with a transitioned route change.
- [x] 6.3 Carry the coordinate back: mark the index entry being returned to so it holds the destination name for the reverse gesture.
- [x] 6.4 Confirm browser Back and Forward get the same treatment in the direction travelled, with history behavior, focus, and rendered route state unchanged.
- [x] 6.5 Leave artifact-query navigation on its existing bespoke path with its paired `artifact-coordinate` / `artifact-lock` names untouched, and confirm the route wrap does not double-wrap it.

## 7. Scoped root suppression

- [x] 7.1 Set a `route` transition marker on the document element for the duration of a route transition and clear it after.
- [x] 7.2 Make `::view-transition-old(root) { opacity: 0 }` conditional on that marker's absence, preserving today's instant switch for artifact replacement.
- [x] 7.3 Give route changes a page-level treatment drawn from the plates' halftone, advancing in discrete passes rather than as a continuous crossfade, within the acquisition bound.
- [x] 7.4 Verify artifact replacement still never ghosts two complete documents over each other.

## 8. Index acquisition becomes the reference gesture

- [x] 8.1 Delete the `.scope-entry::before` lit bar and its `--acquiring` growth.
- [x] 8.2 Resolve the change identity through the plate halftone on acquisition, reusing the existing resolve keyframes, duration tokens, and `steps()` machinery rather than adding a parallel vocabulary.
- [x] 8.3 Confirm the gate outlasts the longest treatment it gates, as the reference-gesture requirement demands.

## 9. Tests

- [x] 9.1 Component tests: index shell consumes the shared tokens and declares no width ceiling of its own.
- [x] 9.2 Component tests: register structure, proportions, separation, and reading order are unchanged by the width change.
- [x] 9.3 Component tests: `ScopeHeader` carries the coordinate name and the loading placeholder does not.
- [x] 9.4 Component tests: the hold commits the route on prefetch failure and on ceiling elapse.
- [x] 9.5 `VisualSystem.test.ts`: the field is `aria-hidden` and non-interactive, registers are opaque, and no text is rendered against the field.
- [x] 9.6 E2E at 2560px: index and scope span the full viewport at the same padding; neither reserves symmetric empty canvas.
- [x] 9.7 E2E at 1280px and 2560px: no bare band in the hero at either width.
- [x] 9.8 E2E: index→scope lands the coordinate on the loaded scope identity, not on the placeholder.
- [x] 9.9 E2E: scope→index by link, and by Back, both animate and both leave correct route state.
- [x] 9.10 E2E at 390px: bleed suppressed, Changes first, every field reachable, no horizontal overflow.
- [x] 9.11 E2E under reduced motion: no hold, no route treatment, correct final state.
- [x] 9.12 Asset check: allowlist and 6 MiB budget unchanged.

## 10. Documentation and spec consistency

- [x] 10.1 Update `designs/dashboard-review-workbench.md` with the index bleed region, its fade, and the field's end above the plate strip.
- [x] 10.2 Confirm nothing else in `openspec/specs/` or `AGENTS.md` still asserts a capped index width, artwork never sitting beneath content, immediate route commitment, or motion never delaying content.

## 11. Gates

- [x] 11.1 `bun install --frozen-lockfile`, lint, typecheck, unit tests, and Playwright pass.
- [x] 11.2 `make build` — `web/dist/` builds before cargo, and the binary compiles with the embedded assets.
- [x] 11.3 `openspec validate widen-index-and-animate-scope-navigation --strict` passes.
