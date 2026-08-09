## MODIFIED Requirements

### Requirement: Scope pages instantiate the observatory review workbench

The system SHALL compose each session and change route from the shared observatory visual system as a compact route/state instrument, an explicit document-coordinate rail, a primary artifact spine, spatially anchored conversation, and persistent decision controls outside the document's reading plane. Authored celestial artwork SHALL establish the route's atmosphere without obscuring document text, state, focus, comments, composers, or controls.

#### Scenario: Session and change routes share one workbench

- **WHEN** a reviewer opens either a discovered session route or a discovered change route
- **THEN** both SHALL use the same scope-workbench landmarks, image-led composition, artifact rendering, anchored-conversation behavior, and responsive rules, with only real route-kind behavior differing

#### Scenario: Route identity does not compete with document gravity

- **WHEN** a titled scope opens and its first artifact repeats that title as source content
- **THEN** the compact route header SHALL show title and exact identifier while the artifact's rendered heading SHALL retain primary document gravity and SHALL NOT be hidden or rewritten

#### Scenario: Desktop keeps document primary

- **WHEN** a scope page is rendered at 1280 CSS pixels or wider
- **THEN** the artifact SHALL retain a readable primary document spine while exact route metadata and artifact anchors occupy a utility rail and expanded anchored conversation occupies a distinct conversation rail aligned with its block

#### Scenario: Scope route links back to implemented index

- **WHEN** a reviewer opens a session or change scope
- **THEN** the route instrumentation SHALL provide a real navigation link to `/` without inventing any unimplemented destination

#### Scenario: Anchored conversation remains spatial on desktop

- **WHEN** a reviewer expands a comment marker against a rendered block at desktop width
- **THEN** the thread and its actions SHALL render in the conversation region beside that same block and SHALL NOT be duplicated into a detached pooled feed

#### Scenario: Anchored conversation enters narrow document flow

- **WHEN** a scope page is rendered at 390 CSS pixels wide
- **THEN** route metadata and document coordinates SHALL precede the artifact, each expanded anchored thread SHALL follow its block in document flow, persistent controls SHALL remain reachable, and the page SHALL have no horizontal overflow

#### Scenario: Desktop decision instrument leaves document unobstructed

- **WHEN** the persistent decision instrument is closed or open at desktop width
- **THEN** its trigger and drawer SHALL remain in the conversation-side instrument region and SHALL NOT cover the document spine, artifact links, block comment actions, or focus indicators

#### Scenario: Narrow decision dock preserves final content

- **WHEN** the decision instrument is rendered at 390 CSS pixels wide
- **THEN** the final artifact block, expanded thread, active composer, and decision controls SHALL all be scrollable fully clear of the dock and safe-area inset

#### Scenario: Artwork yields before review content

- **WHEN** available width cannot preserve the observation field or plate band alongside review content
- **THEN** the plates SHALL disappear and the main artwork SHALL shrink, crop, or disappear before any artifact text, identifier, state, thread, composer, or control is hidden or clipped

#### Scenario: Scope chrome reports only real state

- **WHEN** a scope page renders image-led instrument chrome inspired by the visual source
- **THEN** it SHALL NOT present repository metadata, validation results, activity events, lifecycle phases, navigation destinations, or review state absent from the scope or index APIs
