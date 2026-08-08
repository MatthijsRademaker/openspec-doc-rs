## ADDED Requirements

### Requirement: Scope routes use an authored image-led observation composition

The system SHALL extend the approved observatory art direction to session and change review routes through an asymmetric face/orbit observation field and a restrained pre-document plate band, while keeping artifact content and anchored conversation on unobstructed high-contrast planes.

#### Scenario: Scope header uses approved observation field

- **WHEN** a scope route renders at desktop width
- **THEN** an optimized shared derivative of `main-panel-background.png` SHALL place halftone face and clipped orbital geometry in a bounded right-side plane distinct from scope title, exact key, verdict, delivery state, and history

#### Scenario: Scope route exposes approved observation plates

- **WHEN** a scope route renders at desktop width
- **THEN** a subordinate pre-document band SHALL use the approved sun, face/eye, and star-system derivatives as hard-edged non-interactive atmosphere without captions implying activity, lifecycle, or review state

#### Scenario: Scope artwork is decorative by construction

- **WHEN** the scope observation field and plate band render
- **THEN** their images SHALL have empty alternative text, remain absent from the accessibility tree, reject pointer interaction, and SHALL NOT sit beneath text, links, focus indicators, comments, composers, status labels, or errors

#### Scenario: Scope artwork yields at narrow width

- **WHEN** a scope route is rendered at 390 CSS pixels wide
- **THEN** the plate band SHALL be absent and the main observation field SHALL reduce to a bounded crop or disappear before any scope identity, document content, comment, composer, state, or control is hidden or clipped

## MODIFIED Requirements

### Requirement: The index runtime artwork is an exact optimized derivative set

The system SHALL ship the index and scope observation artwork only as one shared set of purpose-cropped WebP derivatives in named observatory runtime slots, SHALL keep source PNGs outside the frontend runtime tree, and SHALL remove the superseded index-prefixed filenames rather than duplicating equivalent raster payload for each route.

#### Scenario: Production build contains the shared observatory derivative set

- **WHEN** the frontend production build completes
- **THEN** `dist/` SHALL contain `assets/images/observatory-field.webp`, `assets/images/observatory-plate-sun.webp`, `assets/images/observatory-plate-face.webp`, and `assets/images/observatory-plate-star-system.webp` as allowlisted runtime rasters used by both index and scope compositions

#### Scenario: Index-specific aliases and source artwork are absent from runtime

- **WHEN** the frontend production build completes
- **THEN** `index-observation-field.webp`, every `index-plate-*.webp` alias, `index-orbit.webp`, `main-panel-background.png`, `abstract-sun.png`, `abstract-face.png`, and `abstract-star-system.png` SHALL NOT appear in `dist/`

#### Scenario: Shared route artwork remains within runtime budget

- **WHEN** the production build's runtime raster payload is measured
- **THEN** every raster SHALL be on the explicit allowlist, equivalent index/scope copies SHALL NOT coexist, and the combined payload SHALL remain at or below 6 MiB
