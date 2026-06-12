## ADDED Requirements

### Requirement: Dashboard lifecycle overview
The browser viewer SHALL show project-level OpenSpec lifecycle guidance on the dashboard.

#### Scenario: Dashboard shows lifecycle overview
- **WHEN** a user opens the viewer root page
- **THEN** the dashboard shows the Explore, Propose, Review, Apply, and Archive lifecycle phases
- **AND** the lifecycle overview is visible near active change triage

#### Scenario: Dashboard highlights active change phases
- **WHEN** active changes exist in the target OpenSpec project
- **THEN** the dashboard shows each active change with its inferred lifecycle phase
- **AND** each listed change includes a concise next-action summary

#### Scenario: Dashboard lifecycle links to active change pages
- **WHEN** the dashboard shows an active change in the lifecycle queue
- **THEN** it links to the active change review page
- **AND** it links to the review companion page when companion mode is enabled

### Requirement: Active changes lifecycle badges
The browser viewer SHALL show lifecycle phase badges on active change lists and detail pages.

#### Scenario: Active changes index shows lifecycle phase
- **WHEN** a user opens the active changes index
- **THEN** each active change item shows its inferred lifecycle phase badge
- **AND** each item includes the primary recommended next action for that phase

#### Scenario: Active change detail shows lifecycle panel
- **WHEN** a user opens an active change detail page
- **THEN** the page shows a lifecycle guidance panel for that change
- **AND** the panel includes the inferred phase, reasons, and next actions

#### Scenario: Archived change detail does not suggest active lifecycle actions
- **WHEN** a user opens an archived change detail page
- **THEN** the page does not suggest active Propose, Review, Apply, or Archive next actions
- **AND** it may indicate that the lifecycle is complete for that archived change

### Requirement: Active change review lifecycle guidance
The browser viewer SHALL include lifecycle guidance on active change review pages.

#### Scenario: Review page shows current lifecycle phase
- **WHEN** a user opens the review page for an active change
- **THEN** the page shows the inferred lifecycle phase near the review header
- **AND** it summarizes why that phase was inferred

#### Scenario: Review page provides phase-specific commands
- **WHEN** a user opens the review page for an active change
- **THEN** the page provides copyable command or Pi guidance for the recommended next phase action
- **AND** the commands are presented as user-run actions rather than browser-executed actions

#### Scenario: Review page explains archive readiness
- **WHEN** a reviewed active change has completed tasks and no change warnings
- **THEN** the review page identifies archive as the recommended next phase
- **AND** it recommends validation before running archive commands
