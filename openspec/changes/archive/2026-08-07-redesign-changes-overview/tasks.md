## 1. Lock runtime artwork and reference contract

- [x] 1.1 Add four named index slots to `designs/dashboard-review-workbench.md`, recording each source image, rendered role, focal crop, decorative semantics, responsive disappearance, and replacement of `index-orbit.webp`
- [x] 1.2 Generate `index-observation-field.webp` from `main-panel-background.png` at slot-appropriate dimensions, cropping out the far-left baked-in pseudo-control rail while preserving black safe space and the right-side face/orbit field
- [x] 1.3 Generate consistent hard-cropped `index-plate-sun.webp`, `index-plate-face.webp`, and `index-plate-star-system.webp` derivatives, with the face plate focused on eye/halftone texture rather than repeating the hero composition
- [x] 1.4 Record each derivative's source, crop rectangle, output dimensions, WebP quality, and byte size in the review-workbench reference so the committed outputs are intentional rather than unexplained binaries
- [x] 1.5 Update `web/scripts/check-runtime-images.mjs` to allow exactly the four new derivatives, keep source-board and 6 MiB checks, and reject `index-orbit.webp`
- [x] 1.6 Delete `web/public/assets/images/index-orbit.webp` only after index references move to the new slots; confirm no design-source PNG is copied under `web/public/`

## 2. Specify index structure through tests

- [x] 2.1 Update `IndexView` tests to assert Changes precede Sessions in DOM order for loaded data and remains the primary heading when sessions outnumber changes
- [x] 2.2 Extend index test data with multiple sessions and changes, including titled, promoted, untitled, modified/unmodified, open-comment, verdict, and most-recently-active states
- [x] 2.3 Update `ScopeRegister` tests so an untitled session exposes one mono identifier identity without a duplicate serif copy, while titled and promoted sessions retain title plus exact key
- [x] 2.4 Add tests proving observation-field and plate images have empty alt text, remain outside the accessibility tree, and do not replace loading, empty, or failure text
- [x] 2.5 Preserve tests for identifier-keyed links, every returned scope field, stale-theme immunity, and explicit zero/dash values

## 3. Recompose index markup

- [x] 3.1 Replace the oversized product masthead with a compact product instrument and a Changes-led observation heading without adding repository, activity, lifecycle, or review claims
- [x] 3.2 Render the Changes register before Sessions in `IndexView.vue`; keep loading and failure branches semantically explicit and keep successful empty registers independent
- [x] 3.3 Add the main observation-field image as non-interactive decorative markup with a dedicated text-safe content plane rather than as a page background or boxed image card
- [x] 3.4 Add a lower decorative plate strip for sun, cropped face/eye, and star-system derivatives plus only approved product-posture copy
- [x] 3.5 Adapt `ScopeRegister` for primary Changes and secondary Sessions presentation while retaining one shared source of link, age, count, verdict, and recent-marker logic
- [x] 3.6 Make untitled scope identity conditional: render a single operational identifier link when title equals no separate information; retain secondary exact key when a real title exists
- [x] 3.7 Reuse existing shadcn-vue structural and badge behavior; do not introduce a duplicate card, badge, scroll-area, or image primitive for one-use composition

## 4. Build image-led desktop composition

- [x] 4.1 Add index-specific central styles for asymmetric observation field, protected black text inset, clipped right-side artwork, and aligned hairline geometry without component-local raw colors
- [x] 4.2 Create desktop workbench grid with wide primary Changes field and narrower Sessions instrument; keep primary content useful at 1280px and above
- [x] 4.3 Bound desktop Sessions with native viewport-relative scrolling while rendering every row and preserving visible focus, exact identifiers, metadata, verdicts, and recent marker
- [x] 4.4 Restyle primary change rows for dense title/key/metadata hierarchy without hiding zero, null, or long values and without rounded card-grid treatment
- [x] 4.5 Integrate lower plates as one hard-edged ruled strip with aggressive crops, shared panel lines, no shadows, no state-like captions, and no perpetual or parallax motion
- [x] 4.6 Remove the clean SVG overlay from index artwork; if `OrbitalFrame`'s optional raster prop and image styles become unused, delete only that dead branch while preserving scope-page geometry behavior

## 5. Preserve intermediate and narrow behavior

- [x] 5.1 Define intermediate-width behavior that reduces observation-field width and plate count before reducing Changes measure or hiding metadata
- [x] 5.2 At 390px, remove desktop grid placement and session height bounds so Changes then Sessions render as one ordinary document flow with no nested scrolling
- [x] 5.3 Keep one bounded hero crop at 390px, hide the lower plate strip, and verify artwork never intersects headings, links, status labels, loading state, or failure alert
- [x] 5.4 Verify long change names, UUID session identifiers, multiword verdicts, and most-recently-active labels wrap without page-level horizontal overflow
- [x] 5.5 Preserve visible keyboard focus and immediate reduced-motion behavior across primary change links and the desktop session rail

## 6. Verify realistic embedded behavior

- [x] 6.1 Expand deterministic embedded fixture data so desktop verification includes enough sessions to pressure hierarchy plus several changes and non-empty review states
- [x] 6.2 Update Playwright assertions to prove Changes precede Sessions semantically, remains in the initial primary content region, and all session links remain keyboard-reachable through the bounded rail
- [x] 6.3 Update 390px assertions to prove complete Changes-first flow, all required metadata/state visibility, removed session scroll bound, hidden lower plates, and document width no greater than viewport
- [x] 6.4 Preserve browser-health assertions for zero console errors, page errors, failed asset/API requests, external requests, and offline font/image loading
- [x] 6.5 Capture deterministic desktop and narrow embedded screenshots; compare desktop composition against `dashboard-mockup.png` and all four selected source assets for asymmetry, negative space, crop quality, and lower-strip rhythm
- [x] 6.6 Inspect focus traversal and native session scrolling manually at desktop, then verify narrow flow with keyboard and reduced-motion media emulation

## 7. Run completion gates and measure payload

- [x] 7.1 Run `bun run format`, `bun run check`, and `bun run build` from `web/`
- [x] 7.2 Confirm runtime-image check reports exactly four allowlisted rasters, no source PNG or `index-orbit.webp` in `dist/`, and aggregate runtime raster bytes at or below 6 MiB
- [x] 7.3 Run `bun run test:e2e` against the freshly built Rust-embedded dashboard for desktop and narrow projects
- [x] 7.4 Record final runtime raster total and inspect generated `dist/` asset requests to confirm every image is local, used, and purpose-named
- [x] 7.5 Review final git diff for obsolete image references, dead index CSS, duplicate status logic, unintended scope-view edits, generated npm metadata, or changes outside this proposal's boundary
