## MODIFIED Requirements

### Requirement: Browser project overview
The browser viewer SHALL render the root page as a dashboard that summarizes the target OpenSpec project and prioritizes the most relevant review and documentation areas with a calm, readable visual hierarchy.

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

#### Scenario: Dashboard remains read-only
- **WHEN** a user browses the dashboard
- **THEN** the page does not provide controls that create, modify, or delete files in the target OpenSpec project
