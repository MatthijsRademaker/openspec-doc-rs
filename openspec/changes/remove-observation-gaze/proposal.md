## Why

The new pointer-responsive observation gaze is visually distracting and reads as a gimmick rather than useful review instrumentation. Remove it before it becomes part of the dashboard contract; static observatory artwork is clearer and keeps review content primary.

## What Changes

- **BREAKING** Remove the `ObservationGaze` eyelid/iris overlay from the index observation field and artifact arrival plate.
- **BREAKING** Delete pointer-bearing state, quantization, event listeners, teardown code, and tests that exist only to drive the gaze.
- Remove gaze-specific SVG geometry, CSS selectors, custom properties, lifecycle comments, and visual-regression assertions.
- Keep shared static observation-field artwork, responsive cropping, opaque content planes, and decorative accessibility behavior unchanged.
- Restore visual-language guidance so observation artwork remains static and pointer-following artwork is prohibited.
- Update dashboard visual-system specs and validation coverage to describe absence of the gaze rather than sanctioning it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dashboard-visual-system`: remove the bounded observation-gaze exception; observation-field artwork remains static and non-pointer-responsive.

## Impact

Affected frontend code includes `web/src/components/ObservationGaze.vue`, `web/src/lib/pointer-field.ts` and its tests, `IndexView.vue`, `ArtifactDocument.vue`, `web/src/style.css`, visual-system tests, and Playwright coverage. Update the dashboard design reference and synchronized Pi/Claude visual-language guidance. No Rust API, routes, persisted data, runtime raster assets, dependency, or accessibility contract changes are intended. Existing static field and plate assets remain in use.
