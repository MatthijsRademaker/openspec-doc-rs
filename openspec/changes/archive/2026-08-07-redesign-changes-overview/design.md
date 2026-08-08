## Context

The shipped index has the correct data and failure semantics but the wrong visual hierarchy. `IndexView.vue` renders a large product masthead followed by Sessions and Changes through the same `ScopeRegister` treatment. Against this repository's current data, the embedded page renders 22 sessions before 6 changes and reaches 3951px tall at 1280px; Changes begin near the bottom. `add-dashboard-lifecycle` will prevent future sessions without review material from being registered, but explicitly does not migrate existing records, so index composition must remain useful with a large session inventory.

The current raster system has one slot, `index-orbit.webp`, derived from `abstract-star-system.png`. It appears inside a bordered `OrbitalFrame` with a second clean SVG orbit over it. That is technically disciplined but visually timid: artwork is a detachable ornament rather than the load-bearing asymmetrical field shown by `dashboard-mockup.png`.

The reviewer has made image-led art direction the priority. `main-panel-background.png` supplies the main composition: deep black negative space, a halftone face entering from the right, clipped orbital geometry, and sparse instrument marks. `abstract-sun.png`, `abstract-face.png`, and `abstract-star-system.png` supply the lower observation-plate rhythm. These files remain design sources, not runtime assets.

Constraints remain unchanged:

- Vue Router owns navigation and the index API remains `/api/index`.
- Every scope field and identity-keyed link remains available.
- Artwork is atmospheric, non-interactive, and absent from the accessibility tree.
- Runtime remains offline and Rust-embedded.
- Central tokens, bundled fonts, 8px rhythm, visible focus, reduced motion, and 390px behavior remain mandatory.
- Bun 1.3.2 remains the only frontend package-manager path.

## Goals / Non-Goals

**Goals:**

- Make Changes the immediate subject and dominant reading field of `/`.
- Translate the source-board art direction through asymmetry, negative space, halftone imagery, hard crops, and hairline alignment rather than through generic themed cards.
- Use `main-panel-background.png` as source for a purpose-cropped observation field and all three approved abstract sources as a lower cinematic plate strip.
- Keep Sessions separate and fully reachable without letting their count delay access to Changes.
- Preserve exact identifiers, real metadata, status meaning, loading/empty/failure distinctions, keyboard operation, and narrow-width completeness.
- Keep runtime derivatives explicit, optimized, bounded, and replaceable as one asset set.

**Non-Goals:**

- No activity feed, repository summary, lifecycle stage, task progress, sorting control, filtering control, command palette, or new review action.
- No index API or Rust server changes.
- No scope-page redesign; shared scope artwork and document composition stay untouched.
- No source PNG in `web/public/`, no remote asset, and no raster used merely because it exists.
- No pixel-for-pixel copy of the detail-screen mockup. Index interactions remain conventional and index-specific.
- No new image-processing dependency in the frontend build. Derivatives are committed inputs generated deliberately from design sources, as the current runtime image is.

## Decisions

### Changes come first semantically and visually

`IndexView` will place the Changes register before Sessions in DOM order. Desktop CSS will compose a wide primary change field with a narrower session instrument rail; at 390px normal flow will remain Changes then Sessions. This makes keyboard and screen-reader order match task priority instead of using CSS to fake priority.

```text
Desktop, 1280px+

┌ compact identity / index coordinates ───────────────────────────┐
│ CHANGES + real index count       observation artwork from right │
├────────────────────────────────────────────┬────────────────────┤
│ primary change register                    │ session instrument │
│ title / exact key / modified / state       │ bounded rail       │
│                                            │ all links rendered │
├────────────────────────────────────────────┴────────────────────┤
│ sun plate │ eye/face plate │ star-system plate │ instrument copy│
└─────────────────────────────────────────────────────────────────┘

Narrow, 390px

┌ compact heading + bounded hero crop ┐
├ Changes                             ┤
│ complete rows in one flow           │
├ Sessions                            ┤
│ complete rows in one flow           │
└ lower triptych removed first ───────┘
```

Alternative: retain equal stacked registers and only enlarge artwork. Rejected because 22 sessions would still push Changes below several screens. Alternative: tabs with Changes selected. Rejected because they hide one complete register behind interaction and weaken direct overview.

### Session rail is bounded on desktop, ordinary flow on narrow screens

All session rows remain rendered. At desktop the session register receives a viewport-bounded internal scroll region with a visible ruled boundary and standard browser scrolling; links, identifiers, metadata, verdicts, and recent marker remain keyboard-reachable. It must not use a custom scrollbar, hover-only reveal, virtual list, or truncated subset. At narrow width the bound and nested scrolling are removed, placing the complete session register after Changes in page flow.

This is a deliberate trade: desktop gains immediate Changes and visible lower plates without deleting old sessions; narrow screens avoid nested scrolling where it is most hostile.

Alternative: collapse old sessions behind `Show all`. Rejected because the current index contract says every scope field remains visible and reachable, and disclosure would make review scope discovery depend on another state.

### Display gravity moves from product branding to page and change content

The product identity becomes compact instrument text. `Changes` leads in Cormorant Garamond; titled changes keep display treatment. Exact keys, timestamps, counts, verdicts, and untitled session identifiers remain IBM Plex Mono.

An untitled session has one identifier serving as both link text and identity. The component will not repeat the same UUID as a serif title and a second mono key. Titled and promoted sessions still show title plus exact identifier because those values carry different information.

Alternative: preserve the giant `OPENSPEC-DOC` masthead for brand consistency. Rejected because it spends the strongest type on shell identity while actual change content becomes subordinate.

### Main artwork is a clipped image field, not wallpaper or a boxed thumbnail

Create `index-observation-field.webp` from `main-panel-background.png`. Crop out the source image's far-left baked-in icon rail and pseudo-control labels: those are visual reference marks that could be mistaken for broken controls in a real interface. Retain enough black negative space for composition, then preserve the right-side face, eye, orbital stack, and globe fragments.

The derivative renders as an `alt=""`, `aria-hidden` image inside an isolated masthead field. CSS clips or masks painted pixels before the text-safe inset; heading and operational text retain an opaque near-black reading plane. Artwork may visually cross panel rules but may not sit under text, focus rings, links, loading messages, or errors.

Do not overlay `OrbitalFrame`'s clean SVG geometry on this derivative. The raster already carries authored orbital structure, and a second geometry language produces the synthetic double-orbit visible today. `OrbitalFrame` remains available to scope pages for its geometry-only use; if removing index image usage leaves its image prop and CSS branch unused, delete that dead branch without refactoring the scope component.

Alternative: use the original PNG as a CSS background. Rejected because Vite would embed a multi-megabyte design source, responsive crop becomes implicit, and source pseudo-controls remain visible. Alternative: place the derivative in another bordered right-hand card. Rejected because it repeats the detached ornament failure.

### Lower plates reproduce rhythm, not activity semantics

Create three hard-cropped, consistent-height WebP plates:

- `index-plate-sun.webp` from `abstract-sun.png`, centered on the dark solar aperture and radiating line field;
- `index-plate-face.webp` from `abstract-face.png`, cropped to eye and halftone texture so it reads differently from the main face field;
- `index-plate-star-system.webp` from `abstract-star-system.png`, cropped to orbital intersections and off-center nodes.

The desktop strip uses square edges, shared hairlines, no shadows, no rounded image-card shells, and no labels implying current state. A fourth text cell may use established product posture (`OBSERVE / PLAN / EXECUTE / VERIFY` and `THE AGENT FOLLOWS THE SPEC. YOU GUIDE THE DIRECTION.`); it must not imply an event or lifecycle stage.

The plate group and images are decorative (`aria-hidden="true"`, empty alt text). At intermediate widths the layout may reduce to two plates. At 390px the plate strip disappears before any content or metadata. There is no carousel, animation, or remote loading.

Alternative: map plates to recent activity types. Rejected because `/api/index` has no activity stream and the interface would fabricate semantics.

### Runtime asset set is replaced atomically

Named runtime files become:

```text
web/public/assets/images/index-observation-field.webp
web/public/assets/images/index-plate-sun.webp
web/public/assets/images/index-plate-face.webp
web/public/assets/images/index-plate-star-system.webp
```

`index-orbit.webp` is deleted once no source reference remains. `designs/dashboard-review-workbench.md` names all four slots and their source/crop intent. `check-runtime-images.mjs` allowlists exactly those four files and still rejects every unlisted raster, either source board in `dist/`, and an aggregate runtime raster payload above 6 MiB.

Target derivatives are sized to rendered slots rather than source dimensions: roughly 1200×720 for the observation field and a common roughly 2:1 plate ratio around 640×320. Exact crop rectangles and WebP quality are selected during implementation against desktop and narrow screenshots, then recorded with the generated files; source identity, slot ratio, focal subject, and payload ceiling are fixed by this design.

Alternative: keep `index-orbit.webp` as fallback. Rejected. It preserves two competing index art systems and leaves dead compatibility behavior.

### Existing data contract remains source of truth

No visual element may imply facts unavailable from `Scope`: title, key, modified time, open comments, verdict, and most-recently-active. Counts already rendered from array lengths are permitted. Change ordering remains server-owned; this change does not silently reinterpret alphabetical discovery as recent activity.

Loading, successful emptiness, and failure continue to be separate branches. Main artwork can frame each state but cannot replace status text, `aria-live`, `aria-busy`, or alert semantics.

### Verification uses embedded output and realistic inventory pressure

Unit tests continue to assert every field and identifier-keyed link. Add structure tests for Changes-first DOM order, non-duplicated untitled session identity, and decorative image semantics. Extend fixture data or a focused component fixture so more sessions than changes prove that Changes remain first and session rows remain reachable.

Embedded Playwright remains completion evidence at 1280×800 and 390×844. Desktop capture must include the main art field, primary Changes area, session rail, and lower plates in a useful first-screen composition. Narrow capture must prove Changes precede Sessions, required values remain visible, decorative plates yield, and document width never exceeds viewport. Existing browser-health assertions continue rejecting console, page, failed-request, and external-request errors.

## Risks / Trade-offs

- **[Risk] Artwork becomes wallpaper and weakens legibility.** → Keep a measurable text-safe inset, stop painted pixels before content, and inspect actual screenshots rather than accepting CSS intent.
- **[Risk] Baked-in marks from `main-panel-background.png` resemble controls.** → Crop out the far-left source rail and keep remaining image regions non-interactive and absent from accessibility tree.
- **[Risk] Four rasters bloat the embedded binary.** → Use slot-sized WebP crops, exact allowlist, existing 6 MiB hard ceiling, and record aggregate bytes after build.
- **[Risk] Internal session scrolling harms navigation.** → Render every row, use native scrolling, preserve focus visibility, test keyboard reachability, and remove the bound entirely at narrow width.
- **[Risk] Lower plates make the index feel like a gallery rather than an instrument.** → Keep them subordinate to registers, hard-edged, uncaptained, and aligned to the same hairline grid; remove them before content under width pressure.
- **[Risk] Screenshot tests become crop-sensitive.** → Assert semantic structure and asset health in tests; use screenshots for reviewed evidence, not pixel-perfect comparison.
- **[Risk] Existing uncommitted scope-view work is disturbed.** → Restrict code changes to index components/styles, shared branches made dead by index removal, asset policy, tests, and design reference.

## Migration Plan

1. Generate and inspect four WebP derivatives from the design sources; add named slots to the review-workbench reference and exact runtime allowlist.
2. Recompose index markup in Changes-first semantic order and adapt existing register primitives without changing API ownership.
3. Add desktop observation field, bounded session rail, and lower plate strip through central index styles.
4. Add narrow rules that remove nested session scrolling and decorative plates while retaining one bounded hero crop and complete data.
5. Remove `index-orbit.webp` and any image-only `OrbitalFrame` branch made dead by the new composition.
6. Update unit, component, Router, asset, and embedded Playwright evidence.
7. Run Bun format/check/build/e2e gates, inspect desktop and narrow captures, and verify runtime raster payload.

No persisted state or server migration exists. Rollback restores index markup, old named slot, allowlist, reference, and tests together; partial rollback is invalid because runtime filenames are embedded at build time.

## Open Questions

None. Image-led direction, all four source derivatives, Changes-first hierarchy, and no new activity/API scope are established.
