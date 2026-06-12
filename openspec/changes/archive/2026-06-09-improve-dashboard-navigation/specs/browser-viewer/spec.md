## MODIFIED Requirements

### Requirement: Browser project overview
The browser viewer SHALL render the root page as a dashboard that summarizes the target OpenSpec project and helps users navigate to the most relevant review and documentation areas.

#### Scenario: Dashboard shows OpenSpec counts
- **WHEN** a user opens the viewer root page
- **THEN** the page shows counts for current specs, active changes, archived changes, requirements, scenarios, and warnings

#### Scenario: Dashboard links to major sections
- **WHEN** a user opens the viewer root page
- **THEN** the page provides navigation links to specs, active changes, archived changes, and warnings

#### Scenario: Dashboard highlights project health
- **WHEN** a user opens the viewer root page
- **THEN** the page shows a project health summary based on available warning and OpenSpec summary data

#### Scenario: Dashboard shows review triage
- **WHEN** active changes exist in the target OpenSpec project
- **THEN** the dashboard lists active changes with direct links to change detail or review pages
- **AND** each listed active change shows available task progress, warning count, and artifact completeness indicators

#### Scenario: Dashboard handles no active changes
- **WHEN** no active changes exist in the target OpenSpec project
- **THEN** the dashboard clearly indicates that there is no active change review queue

#### Scenario: Dashboard shows spec overview
- **WHEN** current specs exist in the target OpenSpec project
- **THEN** the dashboard lists or summarizes current specs with direct links to spec detail pages
- **AND** each listed spec shows available requirement count, scenario count, and warning count

#### Scenario: Dashboard shows warning summary
- **WHEN** analyzer warnings are present
- **THEN** the dashboard shows a warning summary with a direct link to the full warnings page

#### Scenario: Dashboard remains read-only
- **WHEN** a user browses the dashboard
- **THEN** the page does not provide controls that create, modify, or delete files in the target OpenSpec project

## ADDED Requirements

### Requirement: Persistent viewer navigation
The browser viewer SHALL provide persistent navigation across viewer pages so users can reach the dashboard and major sections without returning through intermediate pages.

#### Scenario: Navigation appears on dashboard and section pages
- **WHEN** a user opens the dashboard, specs index, active changes index, archive index, warnings page, or supported detail page
- **THEN** the page shows navigation links for dashboard, specs, active changes, archive, and warnings

#### Scenario: Navigation includes current counts
- **WHEN** the persistent navigation is rendered
- **THEN** it includes available counts for specs, active changes, archived changes, and warnings

#### Scenario: Navigation preserves existing routes
- **WHEN** a user follows a persistent navigation link
- **THEN** the viewer routes to the existing dashboard, specs, active changes, archive, or warnings page for that section

#### Scenario: Navigation remains responsive
- **WHEN** the viewer is displayed on a narrow screen
- **THEN** the persistent navigation remains usable without hiding access to the dashboard, specs, active changes, archive, or warnings sections
