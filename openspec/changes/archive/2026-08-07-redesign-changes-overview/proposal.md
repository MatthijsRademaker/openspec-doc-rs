## Why

The index is functionally complete but compositionally wrong for its primary job: real projects can render dozens of sessions before the reviewer reaches Changes, while the celestial source art is reduced to a small boxed ornament. The result is a competent dark register, not the image-led observatory shown in `designs/visual-language/dashboard-mockup.png`.

This change makes Changes the dominant observation field and uses purpose-cropped derivatives of the supplied visual-language artwork as load-bearing composition, while preserving every honest scope field, route, failure state, and narrow-width guarantee already shipped.

## What Changes

- Recompose `/` around a compact product instrument, a dominant Changes field, and a secondary Sessions rail instead of two equally weighted stacked registers.
- Give the page subject and change titles the display gravity currently spent on the oversized product wordmark; keep identifiers and operational metadata in the mono role.
- Introduce an asymmetric desktop observation field derived from `main-panel-background.png`, with protected negative space for change content and face/orbit artwork entering from the right.
- Introduce a lower observation-plate strip derived from `abstract-sun.png`, `abstract-face.png`, and `abstract-star-system.png`, using aggressive crops and hairline integration rather than generic image cards.
- Replace the existing `index-orbit.webp` slot rather than accumulating a second, conflicting image system. Runtime code continues to reference only optimized WebP derivatives, never source-board PNGs.
- Preserve title, exact identifier, modified time, open-comment count, standing verdict, most-recently-active marker, identifier-keyed navigation, and distinct loading, empty, and failure instruments.
- Keep all sessions reachable without allowing a large session inventory to push Changes below several screens of content. Untitled sessions use their identifier once in the operational type role rather than duplicating it as serif title and mono metadata.
- At 390px, render Changes before Sessions, retain every required field and action, reduce the hero to a bounded crop, and remove lower decorative plates before content is hidden or clipped.
- Keep artwork atmospheric and `aria-hidden`; do not fabricate repository metadata, activity events, lifecycle stages, or review controls absent from the index API.
- Update visual reference, runtime-image allowlist, asset-budget checks, component tests, and embedded desktop/narrow screenshots as one unit.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dashboard-html-views`: Change index hierarchy and responsive composition so Changes are primary, Sessions remain reachable as a secondary instrument, and all existing scope data and state distinctions survive.
- `dashboard-visual-system`: Strengthen index-specific artwork requirements and define explicit optimized runtime slots for the main observation field and lower image plates.

## Impact

- `web/src/views/IndexView.vue` and index-focused components: composition, semantic reading order, and scope presentation.
- `web/src/style.css`: index layout, image framing, crop behavior, and narrow-width rules expressed through existing observatory tokens.
- `web/public/assets/images/`: new purpose-cropped WebP derivatives; removal of obsolete `index-orbit.webp`.
- `designs/dashboard-review-workbench.md`: named runtime slots and index art-direction contract.
- `web/scripts/check-runtime-images.mjs`: allowlist remains exact and continues enforcing the 6 MiB aggregate ceiling.
- Index unit tests and `web/e2e/dashboard.spec.ts`: complete scope data, Changes-first narrow order, focus, overflow, asset health, and deterministic source-board screenshots.
- No index API, Rust route, navigation path, package dependency, remote asset, or theme behavior changes.
- Independent of `add-dashboard-lifecycle`: that change prevents future empty session registration but deliberately does not migrate existing records, so this layout must handle both current large inventories and cleaner future ones.
