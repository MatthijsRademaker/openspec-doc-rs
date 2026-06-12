## ADDED Requirements

### Requirement: Lifecycle-aware companion guidance
The review companion SHALL include lifecycle-aware guidance that connects browser review to propose, apply, and archive decisions.

#### Scenario: Companion shows lifecycle phase
- **WHEN** a user opens the companion route for an active change
- **THEN** the page shows the active change's inferred lifecycle phase
- **AND** it explains how companion review fits into that phase

#### Scenario: Companion review phase guidance
- **WHEN** the companion identifies the change as needing Review-phase attention
- **THEN** it recommends reading proposal, design, tasks, and spec deltas
- **AND** it recommends using sidecar comments or Pi analysis to capture feedback before implementation

#### Scenario: Companion apply phase guidance
- **WHEN** the companion identifies the change as needing Apply-phase attention
- **THEN** it explains that comments and Pi analysis remain guidance
- **AND** it directs the user to explicitly apply artifact or code updates through Pi or CLI workflows

#### Scenario: Companion archive phase guidance
- **WHEN** the companion identifies the change as ready for Archive-phase attention
- **THEN** it recommends validating the completed change
- **AND** it shows archive command guidance without providing a browser-side archive action

### Requirement: Companion lifecycle safety
The review companion SHALL preserve non-mutating review behavior while showing lifecycle guidance.

#### Scenario: Lifecycle guidance does not edit artifacts
- **WHEN** lifecycle guidance is shown in the companion page
- **THEN** no proposal, design, task, or spec delta file is modified by viewing the guidance
- **AND** lifecycle guidance actions are presented as explicit commands or Pi handoffs

#### Scenario: Sidecar feedback remains separate from lifecycle transitions
- **WHEN** a user creates or updates sidecar review feedback while lifecycle guidance is visible
- **THEN** the system stores feedback in sidecar review storage
- **AND** it does not mark the change applied or archived automatically
