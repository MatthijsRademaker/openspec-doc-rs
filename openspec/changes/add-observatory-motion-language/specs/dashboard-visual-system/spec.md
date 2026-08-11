## MODIFIED Requirements

### Requirement: Motion explains state without becoming spectacle

The system SHALL express dashboard motion as a short event-driven instrument language for acquiring coordinates, triangulating linked content, transmitting reviewer intent, receiving content, resolving semantic state, and reconfiguring interface chrome. Motion SHALL be tied to real navigation, mutation, state, or remote-update events; SHALL complete standard transitions within 100–200 milliseconds; and SHALL NOT delay access to content or controls. The system SHALL NOT use decorative perpetual motion, parallax, pointer-following fields, cursor trails, sound, whole-page glitch effects, spring spectacle, or animated loading ornament.

Motion MAY use opacity, slight position, registration-line growth, hard masks, clipping, or glyph and line-geometry changes when those effects explain the event and leave no decorative movement after settling. A mark reporting an arrival MAY remain visible longer than a transition takes to complete, and SHALL decay on its own rather than persisting as a state the interface must later clear. The duration bound governs how long a transition takes to finish and how long content is withheld; it does not require a nonblocking notice to vanish before it has been seen.

#### Scenario: Navigation acquires a real coordinate

- **WHEN** a reviewer navigates to a scope, artifact, or linked thread destination
- **THEN** any acquisition motion SHALL originate from a real selected coordinate or control, SHALL terminate at the real destination, and SHALL NOT delay route, history, focus, or content updates until decoration finishes

#### Scenario: Transmission follows real request state

- **WHEN** a reviewer submits a comment, reply, status change, or verdict
- **THEN** transmission treatment SHALL remain in flight only while the real mutation is unresolved, SHALL terminate at the actual successful destination, and SHALL NOT present successful receipt when the mutation fails

#### Scenario: Semantic state remains immediate

- **WHEN** an open, addressed, resolved, verdict, or delivery state changes
- **THEN** visible text and final non-color geometry SHALL update from real state without waiting for its decorative transition to complete

#### Scenario: Remote content receives a bounded arrival mark

- **WHEN** content arrives through a live update the reviewer did not initiate
- **THEN** receive treatment MAY mark the surface where replacement lands but SHALL NOT obscure content, intercept interaction, move reading position, or claim which document blocks changed

#### Scenario: Standard motion is bounded

- **WHEN** a panel, route, review state, or instrument event enters or changes
- **THEN** each standard transition SHALL complete within 100–200 milliseconds and SHALL leave the interface still after the event settles

#### Scenario: An arrival mark decays without blocking

- **WHEN** the interface marks content that arrived without the reviewer acting
- **THEN** the mark SHALL fade out on its own, SHALL leave no residual state once it has, and SHALL NOT intercept interaction or delay access to the content it marks

#### Scenario: Loading is not decorated

- **WHEN** the interface is waiting for a scope, an index, or a mutation to resolve
- **THEN** it SHALL report the wait in text and SHALL NOT present an animated loading ornament

#### Scenario: Narrow motion preserves content order

- **WHEN** an event occurs at 390 CSS pixels wide and its desktop source and destination no longer occupy adjacent planes
- **THEN** the system SHALL use local or reading-direction treatment and SHALL NOT draw a connector across document content, reorder content, create horizontal overflow, or hide controls

#### Scenario: Reduced motion is immediate

- **WHEN** the reviewer requests reduced motion
- **THEN** native view transitions, smooth scrolling, transforms, masks, clipping transitions, and decorative displacement SHALL be removed or made immediate, while text, focus, semantic state, and final geometry remain understandable

#### Scenario: Reduced motion covers effects added later

- **WHEN** a new transition or animation is added to the interface
- **THEN** it SHALL be suppressed under reduced motion without requiring that it be listed individually, and any effect that must survive reduced motion SHALL opt out explicitly

#### Scenario: Celestial artwork remains still

- **WHEN** orbital, face, stipple, scan-line, or star-system artwork frames an interaction
- **THEN** the artwork SHALL remain static unless a bounded non-interactive overlay reports a real event, and it SHALL NOT track the pointer or animate perpetually
