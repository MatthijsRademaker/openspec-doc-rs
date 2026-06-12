## MODIFIED Requirements

### Requirement: Browser project overview
The browser viewer SHALL render the root page as a dashboard that summarizes the target OpenSpec project, prioritizes the most relevant review and documentation areas with a calm readable visual hierarchy, and exposes explicitly confirmed local lifecycle actions when the local action API is enabled.

#### Scenario: Dashboard shows OpenSpec counts
- **WHEN** a user opens the viewer root page
- **THEN** the page shows counts for current specs, active changes, archived changes, requirements, scenarios, and warnings

#### Scenario: Dashboard links to major sections
- **WHEN** a user opens the viewer root page
- **THEN** the page provides navigation links to specs, active changes, archived changes, and warnings

#### Scenario: Dashboard shows secondary project health
- **WHEN** a user opens the viewer root page
- **THEN** the page shows a project health summary based on available warning and OpenSpec summary data
- **AND** the health summary is presented as supporting metadata rather than as the primary hero message

#### Scenario: Dashboard shows secondary project locations
- **WHEN** a user opens the viewer root page
- **THEN** the page makes the target root and OpenSpec config locations available for verification
- **AND** those locations are visually de-emphasized relative to active changes and navigation summaries

#### Scenario: Dashboard highlights active review triage first
- **WHEN** active changes exist in the target OpenSpec project
- **THEN** the dashboard places active changes in a full-width primary section before current specs, archived changes, and secondary health or configuration details
- **AND** each listed active change shows available task progress, warning count, and artifact completeness indicators
- **AND** each listed active change provides direct links to change detail or review pages

#### Scenario: Dashboard handles no active changes
- **WHEN** no active changes exist in the target OpenSpec project
- **THEN** the dashboard clearly indicates that there is no active change review queue in the primary active changes section

#### Scenario: Dashboard shows recent spec preview
- **WHEN** current specs exist in the target OpenSpec project
- **THEN** the dashboard lists only the most recent current specs as a preview with direct links to spec detail pages
- **AND** each listed spec shows available requirement count, scenario count, and warning count
- **AND** the dashboard provides a link to the complete specs index

#### Scenario: Dashboard shows recent archive preview
- **WHEN** archived changes exist in the target OpenSpec project
- **THEN** the dashboard lists only the most recent archived changes as a preview with direct links to archived change detail pages
- **AND** the dashboard provides a link to the complete archive index

#### Scenario: Dashboard shows warning summary
- **WHEN** analyzer warnings are present
- **THEN** the dashboard shows a warning summary with a direct link to the full warnings page

#### Scenario: Dashboard shows local lifecycle actions
- **WHEN** a user opens the dashboard on the local viewer
- **THEN** lifecycle guidance includes explicit local action controls for appropriate project or active-change phases
- **AND** mutating or command-running lifecycle controls require user confirmation before the request is sent

### Requirement: Active change review navigation
The browser viewer SHALL provide a review-oriented route or section for active OpenSpec changes that helps users inspect generated proposal artifacts and issue explicitly confirmed local lifecycle actions.

#### Scenario: Review page opens for active change
- **WHEN** a user opens the review URL for an active change
- **THEN** the viewer shows the change proposal, design, tasks, spec deltas, warnings, and artifact completion details available for that change

#### Scenario: Review page links from active change detail
- **WHEN** a user opens an active change detail page
- **THEN** the viewer provides a navigation link to the review-oriented view for that change

#### Scenario: Review page exposes local lifecycle actions
- **WHEN** a user opens the review-oriented view for an active change
- **THEN** the page exposes local lifecycle action controls for validation, task state changes, handoff actions, and archive when appropriate
- **AND** those controls report action results inline

#### Scenario: Review page explains feedback path
- **WHEN** a user opens the review-oriented view for an active change
- **THEN** the page explains that browser lifecycle actions are local, explicit, and separate from review feedback records

### Requirement: Read-only viewer behavior
The browser viewer SHALL be read-only for target OpenSpec projects except for explicitly confirmed local lifecycle actions and documented sidecar review writes.

#### Scenario: Serve does not mutate target project on page view
- **WHEN** a user runs `openspec-doc serve` against a target project and only browses viewer pages
- **THEN** the command does not create, modify, or delete files in the target project

#### Scenario: Viewer pages expose guarded lifecycle actions
- **WHEN** a user browses lifecycle-enabled viewer pages
- **THEN** pages may provide controls for allowlisted local lifecycle actions
- **AND** those controls require explicit user activation before they can write files or run commands
