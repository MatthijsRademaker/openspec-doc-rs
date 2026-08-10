## ADDED Requirements

### Requirement: The document's readable measure is stable across chrome changes

The system SHALL hold the selected document's readable measure constant when surrounding chrome is shown, hidden, collapsed, or expanded, and SHALL absorb the freed or lost width as margin rather than as line length.

A control that collapses a rail exists to get the rail out of the way. If collapsing it also re-wraps every paragraph in the document, the reviewer loses their place in the text as the price of putting the threads away, and the two effects are indistinguishable from each other.

#### Scenario: Collapsing a rail does not re-wrap the document

- **WHEN** a reviewer collapses or expands a rail beside the selected document
- **THEN** the document's line breaks SHALL be unchanged and the freed width SHALL become margin

#### Scenario: The measure still yields at narrow widths

- **WHEN** available width is narrower than the document's readable measure
- **THEN** the document SHALL use the available width, without horizontal page overflow

## MODIFIED Requirements

### Requirement: Celestial artwork frames rather than obscures review content

The system SHALL use celestial, orbital, crosshair, scan-line, and stipple motifs only as atmosphere or wayfinding around content. Artwork SHALL NOT reduce readability, cover controls, intercept interaction, or sit beneath primary document prose or instrumentation text. An artwork's legible extent SHALL be a function of the space its container's content leaves free, and SHALL NOT be a fixed fraction of the viewport that content can grow into. Artwork MAY extend behind the content above it only through a declared fade, and SHALL reach full strength only in free space.

Bounding artwork by the viewport works only while the content beside it stays short. A rail whose list grows with the number of artifacts will eventually reach into a region sized without reference to it, and the resulting overlap appears as a function of fixture size — visible with six artifacts, invisible with three, and therefore invisible to a suite that keeps its fixtures small.

The fade is the distinction that matters. Atmosphere rising behind the foot of a list is what the rails are composed around; stipple at full strength under a mono path is the defect. Making the fade a declared distance rather than a judgement call keeps the second testable without banning the first.

#### Scenario: Artwork yields to document content

- **WHEN** a view places celestial artwork near a document heading or content region
- **THEN** the artwork SHALL occupy a bounded framing region and the document text SHALL retain an unobstructed high-contrast surface

#### Scenario: Artwork yields to instrumentation content

- **WHEN** a rail's instrumentation content grows to occupy the region an artwork would otherwise fill
- **THEN** the artwork's legible region SHALL shrink or disappear, and no identifier, path, state, or label SHALL be rendered over any part of the artwork outside its declared fade

#### Scenario: Narrow layout removes atmosphere first

- **WHEN** the dashboard is rendered at 390 CSS pixels wide
- **THEN** decorative artwork SHALL shrink, crop, or disappear before any identifier, state, comment, or control is hidden or clipped
