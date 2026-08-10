## MODIFIED Requirements

### Requirement: Motion explains state without becoming spectacle

The system SHALL limit interface motion to short opacity or slight position transitions that explain navigation, state changes, or the arrival of content the reviewer did not cause, and SHALL remove nonessential transitions when reduced motion is requested. It SHALL NOT use decorative perpetual motion, parallax, or animated loading ornament.

A mark reporting an arrival MAY remain visible longer than a transition takes to complete, and SHALL decay on its own rather than persisting as a state the interface must later clear. The duration bound governs how long a transition takes to finish and how long content is withheld; it does not require a notice to vanish before it has been seen.

#### Scenario: Standard motion is restrained

- **WHEN** a panel, route, or review state enters or changes
- **THEN** any transition SHALL complete within 100–200 milliseconds and SHALL not delay access to content or controls

#### Scenario: An arrival mark decays without blocking

- **WHEN** the interface marks content that arrived without the reviewer acting
- **THEN** the mark SHALL fade out on its own, SHALL leave no residual state once it has, and SHALL NOT intercept interaction or delay access to the content it marks

#### Scenario: Loading is not decorated

- **WHEN** the interface is waiting for a scope, an index, or a mutation to resolve
- **THEN** it SHALL report the wait in text and SHALL NOT present an animated loading ornament

#### Scenario: Reduced motion is immediate

- **WHEN** the reviewer requests reduced motion
- **THEN** transforms and decorative transitions SHALL be removed and state changes SHALL remain understandable

#### Scenario: Reduced motion covers effects added later

- **WHEN** a new transition or animation is added to the interface
- **THEN** it SHALL be suppressed under reduced motion without requiring that it be listed individually, and any effect that must survive reduced motion SHALL opt out explicitly
