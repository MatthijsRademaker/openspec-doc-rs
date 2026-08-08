## ADDED Requirements

### Requirement: The index uses an authored image-led observation composition

The system SHALL make the index artwork a load-bearing part of its observatory composition through a dominant asymmetric observation field and a lower strip of hard-cropped visual plates. It SHALL preserve black negative space, halftone texture, clipped celestial geometry, and hairline alignment from the approved visual sources, and SHALL NOT reduce those sources to generic image cards or use them as wallpaper beneath content.

#### Scenario: Main observation field uses the approved face and orbit source

- **WHEN** the index renders at desktop width
- **THEN** it SHALL render an optimized derivative of `designs/visual-language/main-panel-background.png` with the face and orbital field entering asymmetrically from the right and with the source's baked-in far-left pseudo-control rail excluded from the runtime crop

#### Scenario: Lower plates use the approved abstract sources

- **WHEN** the complete desktop index renders
- **THEN** it SHALL include a subordinate hard-edged plate strip using optimized derivatives of `abstract-sun.png`, `abstract-face.png`, and `abstract-star-system.png`, without captions or labels that imply activity or lifecycle state

#### Scenario: Artwork and content occupy distinct planes

- **WHEN** the index observation field or lower plates appear near scope content
- **THEN** artwork SHALL remain non-interactive and absent from the accessibility tree, and SHALL NOT sit beneath text, links, focus indicators, loading messages, failure messages, or state labels

#### Scenario: Index atmosphere yields at narrow width

- **WHEN** the index is rendered at 390 CSS pixels wide
- **THEN** the main artwork SHALL reduce to a bounded crop and the lower plate strip SHALL disappear before any scope identity, metadata, state, or link is hidden or clipped

### Requirement: The index runtime artwork is an exact optimized derivative set

The system SHALL ship the index artwork only as purpose-cropped WebP derivatives in named runtime slots, SHALL keep the source PNGs outside the frontend runtime tree, and SHALL remove the superseded index orbit derivative rather than retaining two competing index art systems.

#### Scenario: Production build contains the named index derivative set

- **WHEN** the frontend production build completes
- **THEN** `dist/` SHALL contain `assets/images/index-observation-field.webp`, `assets/images/index-plate-sun.webp`, `assets/images/index-plate-face.webp`, and `assets/images/index-plate-star-system.webp` as allowlisted runtime rasters

#### Scenario: Superseded and source artwork are absent from runtime

- **WHEN** the frontend production build completes
- **THEN** `index-orbit.webp`, `main-panel-background.png`, `abstract-sun.png`, `abstract-face.png`, and `abstract-star-system.png` SHALL NOT appear in `dist/`

#### Scenario: Expanded index artwork remains within runtime budget

- **WHEN** the production build's runtime raster payload is measured
- **THEN** every raster SHALL be on the explicit allowlist and the combined payload SHALL remain at or below 6 MiB
