# dashboard-visual-system Specification

## Purpose

Define dashboard-wide observatory identity, typography, semantic state language, responsive composition, motion, accessibility, and runtime-asset discipline.
## Requirements
### Requirement: The dashboard presents one observatory visual identity

The system SHALL present every dashboard route on a near-black layered canvas with bone-toned content, hairline depth, restrained geometry, and cartographic instrument motifs. It SHALL NOT present a generic light dashboard theme or a theme-selection control.

#### Scenario: Dashboard arrives in the observatory theme

- **WHEN** a reviewer opens the dashboard with no stored browser preferences
- **THEN** the system SHALL render the dark observatory interface on first paint without requiring a theme initialization script

#### Scenario: Old theme preference does not alter the interface

- **WHEN** a browser retains a theme preference written by the previous interface
- **THEN** the system SHALL still render the sole observatory theme and SHALL NOT offer a theme toggle

### Requirement: Typography communicates document and instrument roles

The system SHALL use a bundled display serif for product and document gravity, IBM Plex Mono for operational labels and code-shaped values, and a bundled readable text face for sustained prose and comment bodies. It SHALL make no remote font request.

#### Scenario: Document title and metadata have different voices

- **WHEN** a view renders a document or scope title with its path, timestamp, or state metadata
- **THEN** the system SHALL render the title in the display role and the operational metadata in the mono role

#### Scenario: Typography works offline

- **WHEN** the embedded dashboard is used with network access disabled
- **THEN** all display, prose, and mono text SHALL render from assets embedded in the binary

### Requirement: Review meaning is not carried by color alone

The system SHALL render open, addressed, resolved, verdict, delivery, reviewer, and agent meanings through central semantic tokens combined with visible text and distinct glyph, line, or layout treatment. Components SHALL NOT depend on raw local colors to communicate those meanings.

#### Scenario: State remains identifiable without hue

- **WHEN** a reviewer sees open, addressed, and resolved states with color perception unavailable
- **THEN** each state SHALL remain identifiable from its visible label and non-color treatment

#### Scenario: Reviewer and agent remain distinguishable

- **WHEN** a conversation contains both reviewer and agent entries
- **THEN** role labels and structural treatment SHALL distinguish them without relying on color

### Requirement: Celestial artwork frames rather than obscures review content

The system SHALL use celestial, orbital, crosshair, scan-line, and stipple motifs only as atmosphere or wayfinding around content. Artwork SHALL NOT reduce readability, cover controls, intercept interaction, or sit beneath primary document prose.

#### Scenario: Artwork yields to document content

- **WHEN** a view places celestial artwork near a document heading or content region
- **THEN** the artwork SHALL occupy a bounded framing region and the document text SHALL retain an unobstructed high-contrast surface

#### Scenario: Narrow layout removes atmosphere first

- **WHEN** the dashboard is rendered at 390 CSS pixels wide
- **THEN** decorative artwork SHALL shrink, crop, or disappear before any identifier, state, comment, or control is hidden or clipped

### Requirement: Layout remains dense, calm, and responsive

The system SHALL compose information on an 8px rhythm with hairline separation and readable document measure. At narrow widths it SHALL preserve content, identifiers, states, and controls in one logical flow without horizontal page scrolling.

#### Scenario: Desktop preserves hierarchy

- **WHEN** the dashboard is rendered at 1280 CSS pixels or wider
- **THEN** product instrumentation, primary content, metadata, and review state SHALL be visually distinct without reducing the primary content to a narrow card column

#### Scenario: Narrow view preserves information

- **WHEN** the dashboard is rendered at 390 CSS pixels wide
- **THEN** long identifiers SHALL wrap, metadata and states SHALL remain visible, controls SHALL remain reachable, and the page SHALL have no horizontal overflow

### Requirement: Motion explains state without becoming spectacle

The system SHALL limit interface motion to short opacity or slight position transitions that explain navigation or state changes, and SHALL remove nonessential transitions when reduced motion is requested. It SHALL NOT use decorative perpetual motion, parallax, or animated loading ornament.

#### Scenario: Standard motion is restrained

- **WHEN** a panel, route, or review state enters or changes
- **THEN** any transition SHALL complete within 100–200 milliseconds and SHALL not delay access to content or controls

#### Scenario: Reduced motion is immediate

- **WHEN** the reviewer requests reduced motion
- **THEN** transforms and decorative transitions SHALL be removed and state changes SHALL remain understandable

### Requirement: Design references are not runtime payload

The system SHALL keep visual source references outside the frontend runtime asset tree and SHALL embed only explicitly selected, optimized runtime derivatives. The complete runtime raster image payload SHALL not exceed 6 MiB.

#### Scenario: Reference boards are absent from the build

- **WHEN** the frontend production build completes
- **THEN** neither `design-system.png` nor `dashboard-mockup.png` SHALL appear in `dist/`

#### Scenario: Runtime images are bounded

- **WHEN** the production build's runtime raster images are measured
- **THEN** every image SHALL be on the runtime allowlist and their combined size SHALL be at most 6 MiB

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

### Requirement: Scope routes use an authored image-led observation composition

The system SHALL compose session and change review routes as one full-bleed observatory instrument with compact route/state chrome, a primary selected-document stage, exact-path instrumentation, artifact-scoped conversation, and peripheral decision controls. The shared face/orbit observation field SHALL enter asymmetrically through a bounded portion of the selected document's arrival plane, while shared observation plates MAY occupy subordinate peripheral chassis cells; artwork SHALL NOT become a detached pre-document gallery or wallpaper behind review content.

#### Scenario: Scope uses connected full-bleed instrument geometry

- **WHEN** a scope route renders at desktop width
- **THEN** route state, artifact instrumentation, selected document, artifact conversation, and decision controls SHALL read as one hairline-connected workbench that uses available viewport width rather than isolated centered cards

#### Scenario: Selected document appears in initial desktop viewport

- **WHEN** the deterministic change fixture renders at 1536 by 1024 CSS pixels
- **THEN** the selected artifact identity and its first meaningful source content SHALL be visible without scrolling past an oversized route hero or image gallery

#### Scenario: Observation field belongs to document arrival

- **WHEN** a selected artifact renders at desktop width
- **THEN** the optimized shared derivative of `main-panel-background.png` SHALL place halftone face and clipped orbital geometry asymmetrically in a bounded arrival region of the selected document, distinct from protected source text, exact paths, state, comments, and focus indicators

#### Scenario: Route header remains operational rather than cinematic

- **WHEN** a titled scope with verdict and delivery state opens
- **THEN** route title, exact identifier, standing verdict, submission age, delivery state, and history SHALL remain compact and legible without forming a second oversized document hero

#### Scenario: Scope does not render a three-equal-plate preamble

- **WHEN** a scope route renders its selected artifact
- **THEN** the sun, face, and star-system derivatives SHALL NOT appear as an equal-width gallery that delays the selected document

#### Scenario: Peripheral plates remain subordinate

- **WHEN** shared observation plates appear in desktop scope composition
- **THEN** they SHALL occupy hard-edged peripheral rail or footer chassis cells, SHALL carry no interactive behavior or state-implying caption, and SHALL yield before document measure, paths, comments, state, or controls

#### Scenario: Artwork is decorative by construction

- **WHEN** scope observation artwork renders
- **THEN** every image SHALL use empty alternative text, remain absent from the accessibility tree, reject pointer interaction, and SHALL NOT sit beneath prose, links, focus indicators, markers, threads, composers, status labels, loading state, or errors

#### Scenario: Empty conversation uses instrument geometry without fake activity

- **WHEN** the selected artifact contains no threads
- **THEN** its conversation region MAY use hairlines, nodes, crosshairs, or restrained artwork but SHALL NOT invent comments, activity events, agents, validation, or review state

#### Scenario: Scope artwork yields at intermediate width

- **WHEN** available width cannot preserve peripheral artwork alongside readable selected-document and conversation measures
- **THEN** peripheral plate cells SHALL disappear before document, exact-path navigation, state, comments, or controls are reduced or hidden

#### Scenario: Scope artwork yields at narrow width

- **WHEN** a scope route is rendered at 390 CSS pixels wide
- **THEN** peripheral plate cells SHALL be absent and the main observation field SHALL reduce to a bounded shallow crop or disappear before any scope identity, artifact path, document content, comment, composer, state, or control is hidden or clipped

