## 1. Separate design sources from runtime assets

- [x] 1.1 Move original visual-language PNGs from `web/public/assets/images/` to `designs/visual-language/`, preserving `design-system.png`, `dashboard-mockup.png`, and supporting motif sources for agent review
- [x] 1.2 Update mirrored Pi and Claude visual/frontend skills and `designs/dashboard-review-workbench.md` to use the new source paths, then run the harness-resource drift check
- [x] 1.3 Select only images with a named runtime slot, generate appropriately sized WebP derivatives under `web/public/assets/images/`, and delete every unused runtime copy rather than shipping an asset library
- [x] 1.4 Add an explicit runtime-image allowlist/size check that fails if a source reference reaches `dist/`, an unlisted raster is copied, or combined runtime raster payload exceeds 6 MiB
- [x] 1.5 Record before/after runtime image, `dist/`, and embedded binary sizes so asset reduction is measured rather than inferred

## 2. Establish executable visual foundations

- [x] 2.1 Add pinned bundled Cormorant Garamond through Bun, retaining IBM Plex Sans and IBM Plex Mono only for their defined prose and operational roles
- [x] 2.2 Replace stock light/dark base values in `web/src/style.css` with central observatory canvas, surface, raised, border, bone, muted, accent, focus, role, and review-state tokens
- [x] 2.3 Add explicit display, prose, and mono typography roles plus the 8px spacing rhythm, restrained radius, hairline depth, and visible focus rules
- [x] 2.4 Remove `ThemeToggle`, dark-class initialization, persisted theme handling, obsolete tests, and dead theme dependencies; an old browser preference must not alter first paint
- [x] 2.5 Add 100–200ms instrument transitions and a `prefers-reduced-motion` rule that removes transforms and makes state changes immediate
- [x] 2.6 Add token/typography tests proving semantic states use central classes, display and metadata roles differ, and no remote font request or component-local raw state color remains

## 3. Build bounded observatory primitives

- [x] 3.1 Implement shared instrument-label and status-mark treatments with visible text plus non-color glyph/line distinctions for open, addressed, resolved, verdict, delivery, reviewer, and agent meanings
- [x] 3.2 Implement reusable orbital/crosshair framing with CSS or inline SVG, `aria-hidden` and non-interactive by construction
- [x] 3.3 Implement shared ruled-register and document-heading treatments needed by the index and named scope-migration composition; keep one-use layout local
- [x] 3.4 Adapt existing shadcn button, badge, and structural primitives through observatory variants and central tokens instead of duplicating their behavior
- [x] 3.5 Add component tests for role/state labels, keyboard focus, decorative semantics, and reduced-motion behavior

## 4. Restyle the real index

- [x] 4.1 Replace rounded card/table dashboard composition with the observatory shell and separate ruled session/change registers while preserving current API ownership and Router links
- [x] 4.2 Preserve every scope field and behavior: title fallback, exact identifier, modified time, open-comment count, verdict, most-recently-active marker, and identifier-keyed navigation
- [x] 4.3 Render distinct loading, empty, and failure instruments without fake repository metadata, activity events, scope routes, statuses, or review controls
- [x] 4.4 Add restrained celestial framing that never sits beneath text or controls and yields before content at narrow widths
- [x] 4.5 At 390px, keep all identifiers, metadata, states, and links visible in one flow with no horizontal page overflow
- [x] 4.6 Update unit/component and Router tests for the new semantic structure, removal of theme selection, complete narrow content, and unchanged index behavior

## 5. Replace obsolete direction

- [x] 5.1 Rewrite `designs/dashboard-review-workbench.md` around observatory source, three typography roles, dark-only identity, instrument microcopy, motif boundaries, and conventional accessible interaction
- [x] 5.2 Replace or delete the four generic light/dark SVG frames; any replacement frame must depict realistic long identifiers and review states in the approved visual language
- [x] 5.3 Update mirrored frontend-design and dashboard skills so every future `web/` task reads the source images and rejects generic SaaS/card-grid output
- [x] 5.4 Update frontend contributor documentation wherever it describes theme selection, fonts, design references, or runtime image placement

## 6. Verify embedded result

- [x] 6.1 Run `bun run format`, `bun run check`, and `bun run build` from `web/`
- [x] 6.2 Run runtime-image allowlist/size checks and confirm `design-system.png` and `dashboard-mockup.png` are absent from `dist/`
- [x] 6.3 Run `bun run test:e2e` against the freshly built Rust-embedded app with zero unexpected console errors
- [x] 6.4 Capture and inspect embedded-app index at desktop and 390px against both source images, using realistic long identifiers and non-empty review states
- [x] 6.5 Verify keyboard traversal, visible focus, accessible names/state text, network-offline fonts/assets, and `prefers-reduced-motion`
- [x] 6.6 Confirm `migrate-dashboard-review-to-vue` remains blocked on this change and names the same shell, document spine, anchored conversation, motif, narrow-flow, and verification contracts
