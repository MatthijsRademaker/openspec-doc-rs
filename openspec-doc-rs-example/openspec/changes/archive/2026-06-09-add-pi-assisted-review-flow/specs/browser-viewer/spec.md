## ADDED Requirements

### Requirement: Active change review navigation
The browser viewer SHALL provide a review-oriented route or section for active OpenSpec changes that helps users inspect generated proposal artifacts.

#### Scenario: Review page opens for active change
- **WHEN** a user opens the review URL for an active change
- **THEN** the viewer shows the change proposal, design, tasks, spec deltas, warnings, and artifact completion details available for that change

#### Scenario: Review page links from active change detail
- **WHEN** a user opens an active change detail page
- **THEN** the viewer provides a navigation link to the review-oriented view for that change

#### Scenario: Review page remains read-only
- **WHEN** a user opens the review-oriented view for an active change
- **THEN** the page does not expose controls that write comments or edits to the target project in this first implementation slice

#### Scenario: Review page explains feedback path
- **WHEN** a user opens the review-oriented view for an active change
- **THEN** the page explains that proposal feedback should be provided in Pi for now and that browser-native comments are reserved for a later review companion mode
