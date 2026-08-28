## Context

Uncommitted dashboard work adds `ObservationGaze.vue` to the index field and wide artifact header. It also adds `trackPointerField`, `--field-gaze-*` custom properties, pointer-driven plate transforms, source-coordinate wrappers, and extensive browser coverage. The feature has no product state behind it and the moving eyelid/iris competes with the static halftone artwork.

The repository already had a static image composition before this work. Removing the feature should restore that simpler DOM and crop model, not introduce a replacement effect. Static artwork remains decorative, non-interactive, responsive, and subordinate to review content.

## Goals / Non-Goals

**Goals:**

- Remove the overlay and every pointer-response path that exists only for it.
- Restore direct static image rendering and existing object-fit/crop behavior in both placements.
- Remove dead gaze geometry, CSS, custom properties, lifecycle code, tests, and documentation.
- Restore one consistent visual contract across the base spec, design reference, and Pi/Claude guidance.
- Prove artwork stays still, offline, accessible, and clear of content at desktop and narrow widths.

**Non-Goals:**

- Do not edit or replace source raster assets.
- Do not remove the observation field, artifact arrival artwork, lower plates, route behavior, or review interactions.
- Do not add a new pointer effect, hover substitute, feature flag, compatibility alias, or fallback component.
- Do not change Rust APIs, persisted review data, routes, dependencies, or runtime asset budgets.

## Decisions

### 1. Delete feature rather than hide it

Remove `ObservationGaze.vue` and `pointer-field.ts` with its focused tests. Remove imports, refs, watchers, teardown hooks, and template instances from `IndexView.vue` and `ArtifactDocument.vue`. A CSS-only hide or dormant tracker would preserve dead complexity and contradict the static-artwork contract.

### 2. Restore static placement DOM and CSS

Return the index field to one static image and the artifact arrival artwork to its direct image child. Remove the gaze wrapper, source-coordinate plate geometry, pointer custom properties, inverse transforms, transition rules, and gaze selectors. Restore the pre-gaze `object-fit` and `object-position` crop rules, including the wide artifact branch. Preserve surrounding masks, opaque panels, responsive hiding, and content layout unless verification identifies a regression caused by this removal.

Keeping the old wrapper was rejected: it existed to give the gaze a source-coordinate frame, so retaining it would leave unexplained geometry and make future pointer behavior look sanctioned.

### 3. Make the contract prohibit pointer artwork

Update `dashboard-visual-system` so motion and celestial-artwork requirements contain no observation-gaze exception. Remove the bounded human-gaze requirement introduced by the current WIP and require the observation field to remain static and pointer-independent. Apply the same wording to `designs/dashboard-review-workbench.md`, `.pi/skills/openspec-visual-language/SKILL.md`, and `.claude/skills/openspec-visual-language/SKILL.md`; run the repository drift checker rather than maintaining divergent copies.

### 4. Test absence and preserved behavior

Delete gaze-specific Playwright helpers, geometry assertions, pointer screenshots, and reset tests. Keep or add narrow coverage that both placements contain only static decorative images, expose no gaze selectors or `--field-gaze-*` styles, make no external requests, preserve no horizontal overflow, and retain existing responsive content priority. Unit coverage for bearing and tracker lifecycle disappears with the deleted module.

### 5. Verify embedded output after frontend build

Run Bun formatting, checks, tests, build, and harness-resource validation. Build `web/dist/` before any Cargo build so the embedded binary cannot accidentally retain stale dashboard assets. Inspect desktop index and wide artifact artwork at rest, plus narrow layouts, for unchanged static composition and unobstructed review content.

## Risks / Trade-offs

- **Restoring direct image cropping changes wide artifact framing** → Compare desktop and wide artifact geometry against the pre-gaze behavior and update only stale gaze-era assertions.
- **A stale import or selector survives deletion** → Search for `ObservationGaze`, `pointer-field`, `field-gaze`, `eyelid`, and `observation-gaze` before validation; let type checking and the build fail loudly.
- **Documentation copies drift** → Run `bun scripts/check-harness-resources.mjs` after synchronizing both agent skill files.
- **A pointer listener remains through another path** → Confirm no dashboard source imports `trackPointerField` and Playwright observes unchanged plate transforms after pointer movement.

## Migration Plan

Delete the feature code and revert its consumers, CSS, tests, visual guidance, and delta spec in one change. No persisted data or server migration exists. Build frontend assets, then run the normal Rust embedding/build gates. Rollback is a source-level revert; there is no runtime state to migrate or restore.

## Open Questions

None. The desired end state is the existing static observation artwork with no pointer-responsive overlay or plate drift.
