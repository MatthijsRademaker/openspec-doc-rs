## Why

Users need a readable, browser-based way to review OpenSpec documentation without manually navigating Markdown files in an editor. A browser viewer is a core product capability for making specs, changes, requirements, scenarios, and warnings easier to inspect locally.

## What Changes

- Add a browser viewer that renders a target OpenSpec project as navigable HTML.
- Add a `serve` command that starts a local read-only preview server for the viewer.
- Add generated pages for project overview, current specs, active changes, archived changes, and warning details.
- Preserve access to raw Markdown content while adding OpenSpec-aware summaries and navigation.
- Ensure malformed or incomplete OpenSpec content still renders where possible with clear warnings.

## Capabilities

### New Capabilities
- `browser-viewer`: Browser-based rendering and navigation for OpenSpec project documentation.

### Modified Capabilities
- `openspec-summary`: Adds browser viewer usage of the existing scan, parse, and warning data without changing read-only discovery or analysis semantics.

## Impact

- Adds CLI behavior for `openspec-doc serve`.
- Adds rendering and local server layers that consume existing scanner, parser, and analyzer outputs.
- May introduce runtime dependencies for HTTP serving and Markdown-to-HTML rendering.
- Requires tests or fixtures that verify rendered pages and server behavior against sample OpenSpec projects.
