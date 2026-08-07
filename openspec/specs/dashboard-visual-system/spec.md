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
