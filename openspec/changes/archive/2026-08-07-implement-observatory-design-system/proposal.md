## Why

The Vue dashboard foundation shipped before the product's visual language existed, so its generic shadcn cards, neutral themes, sans headings, and table-first index contradict the observatory interface now defined by `design-system.png` and `dashboard-mockup.png`. Building scope review pages on that foundation would multiply the wrong decisions across the SPA and turn the design source into decoration rather than direction.

## What Changes

- Introduce an observatory visual system for every dashboard route: near-black layered surfaces, bone ink, serif document gravity, mono operational language, hairline depth, an 8px grid, cartographic motifs, and instrument-grade motion.
- **BREAKING:** remove the selectable light theme and theme toggle. The dashboard becomes one deliberate dark observatory interface rather than a faithful dark screen plus an invented stock light variant.
- Bundle a display serif matching the source direction. Keep IBM Plex Mono for operational metadata and restrict IBM Plex Sans to long prose and comments where it improves sustained reading.
- Centralize visual, role, review-state, focus, spacing, and motion tokens in `web/src/style.css`; semantic states retain visible text and glyph/shape differences rather than relying on color.
- Add the smallest shared visual primitives needed by both index and scope views, then restyle the existing index against real session/change data so the system is proven before scope migration consumes it.
- Replace the obsolete generic review-workbench reference and frames with guidance faithful to the source images, while preserving document-first hierarchy, conventional interaction, responsive behavior, and accessibility.
- Move source/reference images out of Vite's runtime `public/` tree. Ship only deliberately selected, optimized derivatives; do not embed the 24 MB reference set into every binary.
- Make `migrate-dashboard-review-to-vue` consume this change and update that change's proposal, design, specs, and tasks together.

## Capabilities

### New Capabilities

- `dashboard-visual-system`: Dashboard-wide visual identity, typography roles, semantic state language, responsive composition, motion, accessibility, and runtime-asset discipline.

### Modified Capabilities

- `dashboard-html-views`: Replace the foundation's selectable light/dark theme requirement with the sole dark observatory interface and require the existing index to instantiate the shared visual system without losing its data and failure-state behavior.

## Impact

- `web/src/style.css`, `web/src/views/IndexView.vue`, `web/src/components/`, existing index tests, and Playwright assertions.
- `web/package.json` and `bun.lock` for one bundled open-source display serif.
- `web/public/assets/images/` and a stable design-reference location under `designs/`; `web/dist/` output and embedded binary size.
- `designs/dashboard-review-workbench.md`, `designs/frames/`, mirrored Pi/Claude frontend skills, and harness-resource drift checks.
- `openspec/changes/migrate-dashboard-review-to-vue/` as one re-scoped change unit.
- No HTTP API, sidecar format, Rust runtime behavior, or review-state transition changes.

## Dependencies and ordering

- `add-vue-dashboard-foundation` and `add-dashboard-development-harness` provide the executable frontend and verification harness this change replaces visually.
- The separate frontend-distribution policy mismatch must be reconciled before those completed changes are archived; this change does not hide that unrelated contract change inside visual work.
- `migrate-dashboard-review-to-vue` starts only after this change lands.
