## Why

The browser viewer currently exposes useful OpenSpec pages, but the root experience is a simple count overview rather than an easily navigable dashboard. Users need a faster way to understand project health, identify active changes that need review, and jump to relevant specs, changes, archive entries, and warnings.

## What Changes

- Replace the root overview with a dashboard-oriented landing page that summarizes OpenSpec health, active work, spec coverage, warnings, and archived work.
- Add a persistent viewer navigation shell so major sections remain easy to reach from dashboard, index, detail, review, companion, archive, and warning pages.
- Add dashboard sections for review triage, including active changes with warnings, missing artifacts, and task progress.
- Add concise spec and archive overview sections with direct links into detail pages.
- Preserve existing routes and read-only behavior; this change does not add browser editing or target project mutation.

## Capabilities

### New Capabilities

### Modified Capabilities
- `browser-viewer`: Changes the browser viewer requirements for the root page and navigation experience from a simple overview into a dashboard-oriented, easily navigable interface.

## Impact

- Affected code: `src/viewer/render.ts`, possibly `src/viewer/model.ts`, and viewer-related tests.
- Affected behavior: `openspec-doc serve` root page and shared browser viewer layout/navigation.
- APIs/dependencies: no new runtime dependency is expected; existing server-rendered HTML can support the first dashboard slice.
- Compatibility: existing viewer routes should continue to work.
