## ADDED Requirements

### Requirement: The index instantiates the observatory visual system

The system SHALL render the existing session and change index through the shared observatory visual system while preserving every scope field, link, and loading, empty, and failure distinction already required of the index.

#### Scenario: Real scopes render as instrument registers

- **WHEN** the index returns discovered sessions and changes
- **THEN** the system SHALL present separate session and change registers with each title, exact identifier, modified time, open-comment count, verdict, and most-recently-active marker supplied by the endpoint

#### Scenario: Index atmosphere does not invent product state

- **WHEN** the index renders celestial framing or instrument labels
- **THEN** it SHALL NOT show a fake repository, activity event, review control, scope route, or status unsupported by current data

#### Scenario: Index failure remains distinct from emptiness

- **WHEN** loading the index endpoint fails
- **THEN** the observatory interface SHALL render a failure state distinct from both loading and a successfully loaded empty project

#### Scenario: Index remains complete at narrow width

- **WHEN** the index is rendered at 390 CSS pixels wide
- **THEN** every scope identifier, modified value, open-comment count, verdict, primary link, and most-recently-active marker SHALL remain visible and reachable without horizontal page scrolling

## REMOVED Requirements

### Requirement: The interface offers a light and a dark theme

**Reason**: The selectable neutral themes were a toolchain proof created before art direction existed. The approved source defines one dark observatory identity; retaining light mode would preserve an invented generic variant and double every visual-state contract.

**Migration**: Remove theme selection and persisted theme handling. Render the observatory theme from first paint regardless of any obsolete browser preference.
