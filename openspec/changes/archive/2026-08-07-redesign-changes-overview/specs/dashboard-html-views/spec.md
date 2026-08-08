## MODIFIED Requirements

### Requirement: The index instantiates the observatory visual system

The system SHALL render the existing session and change index through the shared observatory visual system as a dominant Changes observation field and a separate secondary Sessions instrument, while preserving every scope field, identifier-keyed link, and loading, empty, and failure distinction already required of the index. A large session inventory SHALL NOT delay visual or keyboard access to Changes.

#### Scenario: Changes are the primary index subject

- **WHEN** the index returns discovered sessions and changes
- **THEN** the system SHALL present Changes before Sessions in semantic reading and keyboard order, and SHALL give Changes the dominant desktop content region

#### Scenario: Real scopes retain complete index data

- **WHEN** the index renders its change and session instruments
- **THEN** it SHALL show each supplied title, exact identifier, modified time, open-comment count, verdict, and most-recently-active marker, and each primary link SHALL remain keyed by exact identifier

#### Scenario: A large session inventory does not bury changes

- **WHEN** the index contains more sessions than changes at desktop width
- **THEN** the Changes heading and change links SHALL remain available in the primary initial content region while every session and its metadata remain reachable through the secondary session instrument

#### Scenario: An untitled session does not duplicate its identifier as display text

- **WHEN** a session has neither a title nor a promotion name
- **THEN** the system SHALL use its exact identifier once as its linked identity in the operational typography role and SHALL NOT repeat the same value as a separate display-serif title

#### Scenario: Index atmosphere does not invent product state

- **WHEN** the index renders celestial framing, observation plates, or instrument labels
- **THEN** it SHALL NOT show a fake repository, activity event, lifecycle stage, review control, scope route, or status unsupported by current data

#### Scenario: Index failure remains distinct from emptiness

- **WHEN** loading the index endpoint fails
- **THEN** the observatory interface SHALL render a failure state distinct from both loading and a successfully loaded empty project

#### Scenario: Index loading remains explicit

- **WHEN** the index endpoint has not completed
- **THEN** the observatory interface SHALL expose an explicit busy loading instrument rather than an empty register or decorative-only state

#### Scenario: Empty registers remain successful results

- **WHEN** the index endpoint returns no changes, no sessions, or neither
- **THEN** each empty register SHALL be identified as successfully empty and SHALL NOT be presented as a failure

#### Scenario: Index remains complete and Changes-first at narrow width

- **WHEN** the index is rendered at 390 CSS pixels wide
- **THEN** Changes SHALL precede Sessions in one logical flow, every scope identifier, modified value, open-comment count, verdict, primary link, and most-recently-active marker SHALL remain visible and reachable, and the page SHALL have no horizontal scrolling
