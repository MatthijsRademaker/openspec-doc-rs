## 1. Remove pointer-responsive frontend integration

- [x] 1.1 Restore `IndexView.vue` to static field-image rendering; remove `ObservationGaze`, tracker imports, field refs, pointer lifecycle setup, teardown, and gaze-specific comments.
- [x] 1.2 Restore `ArtifactDocument.vue` to direct static arrival-image rendering; remove gaze imports, watcher refs, tracker teardown, source-frame wrapper, and gaze-specific comments.
- [x] 1.3 Delete `web/src/components/ObservationGaze.vue`, `web/src/lib/pointer-field.ts`, and `web/src/lib/pointer-field.test.ts`; confirm no remaining frontend import references.

## 2. Restore static artwork CSS

- [x] 2.1 Remove index plate transforms, `--field-gaze-*` consumption, gaze geometry, and index gaze selectors; restore static image sizing and filtering.
- [x] 2.2 Remove artifact source-coordinate plate geometry, pointer drift, gaze selectors, and gaze-era comments; restore direct image `object-fit`/`object-position` rules for normal and wide artifact layouts.
- [x] 2.3 Search frontend sources and styles for `ObservationGaze`, `observation-gaze`, `pointer-field`, `field-gaze`, `eyelid`, and `trackPointerField`; leave no stale implementation identifiers or script-written pointer offsets.

## 3. Restore visual contracts and guidance

- [x] 3.1 Apply the dashboard visual-system delta so pointer movement cannot displace observation artwork and the bounded human-gaze requirement is removed.
- [x] 3.2 Replace gaze exception language in `designs/dashboard-review-workbench.md`, `.pi/skills/openspec-visual-language/SKILL.md`, and `.claude/skills/openspec-visual-language/SKILL.md` with the static-artwork prohibition, keeping both skill copies synchronized.
- [x] 3.3 Run `bun scripts/check-harness-resources.mjs` and fix any guidance drift without changing unrelated visual-language rules.

## 4. Update behavioral coverage

- [x] 4.1 Remove Playwright gaze helpers, pointer-bearing geometry assertions, gaze screenshots, reset/coarse/reduced-motion gaze tests, and source-coordinate registration tests.
- [x] 4.2 Add or retain desktop coverage proving index and wide artifact artwork render as static decorative images with no gaze component, no `--field-gaze-*` properties, no pointer displacement, and no external requests.
- [x] 4.3 Add or retain narrow and intermediate coverage proving artwork yields before content, controls, focus indicators, paths, comments, and semantic state, with no horizontal overflow.
- [x] 4.4 Keep visual-system unit assertions aligned with static artwork, semantic tokens, reduced motion, offline assets, and runtime-image limits.

## 5. Validate embedded dashboard

- [x] 5.1 Run `bun install --frozen-lockfile` and the frontend format, check, unit-test, and production-build commands; build `web/dist/` before Cargo.
- [x] 5.2 Inspect index and wide artifact routes at desktop rest state plus narrow and intermediate widths; confirm static crop matches prior behavior and artwork does not compete with review content.
- [x] 5.3 Run `make build`, `make check`, and `make e2e`; confirm embedded output has no console errors, failed requests, external requests, stale gaze markup, or horizontal overflow.
