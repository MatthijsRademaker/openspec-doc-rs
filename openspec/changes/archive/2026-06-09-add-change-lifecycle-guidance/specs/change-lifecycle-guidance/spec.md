## ADDED Requirements

### Requirement: Lifecycle phases
The system SHALL present OpenSpec change work as a lifecycle with Explore, Propose, Review, Apply, and Archive phases.

#### Scenario: Lifecycle phases shown in order
- **WHEN** a user views lifecycle guidance
- **THEN** the system shows the phases in the order Explore, Propose, Review, Apply, Archive
- **AND** each phase includes a concise description of its purpose

#### Scenario: Lifecycle guidance preserves OpenSpec terminology
- **WHEN** lifecycle guidance refers to OpenSpec work
- **THEN** it uses OpenSpec terms for changes, proposals, design, tasks, specs, validation, and archive actions

### Requirement: Project-level explore guidance
The system SHALL provide explore-phase guidance when a project has no active OpenSpec changes.

#### Scenario: No active changes prompts exploration
- **WHEN** the target project has no active changes
- **THEN** the dashboard lifecycle guidance presents Explore as the current recommended phase
- **AND** it explains that the user can clarify an idea before creating a proposal
- **AND** it provides a non-mutating next action for starting exploration in Pi or the agent workflow

#### Scenario: Active changes replace explore prompt with queue
- **WHEN** the target project has one or more active changes
- **THEN** the dashboard lifecycle guidance presents the active changes as a lifecycle queue
- **AND** it does not imply that a new change is required before continuing existing work

### Requirement: Active change phase inference
The system SHALL infer an advisory lifecycle phase for each active change from scanned OpenSpec state.

#### Scenario: Incomplete artifacts indicate propose phase
- **WHEN** an active change is missing proposal, design, tasks, or spec delta artifacts expected for review
- **THEN** the system identifies the change as needing Propose-phase attention
- **AND** it lists the missing artifact signals as reasons

#### Scenario: Complete artifacts before implementation indicate review phase
- **WHEN** an active change has proposal, design, tasks, and spec delta artifacts
- **AND** task progress does not show implementation work in progress
- **THEN** the system identifies the change as ready for Review-phase attention
- **AND** it recommends reviewing artifacts and resolving feedback before implementation

#### Scenario: Incomplete tasks indicate apply phase
- **WHEN** an active change has tasks
- **AND** at least one task remains incomplete
- **AND** the change is not missing proposal artifacts that require proposal attention
- **THEN** the system identifies the change as needing Apply-phase attention
- **AND** it recommends implementing tasks and running validation

#### Scenario: Completed tasks and clean active change indicate archive phase
- **WHEN** an active change has all tasks complete
- **AND** required proposal artifacts are present
- **AND** no analyzer warnings apply to that change
- **THEN** the system identifies the change as ready for Archive-phase attention
- **AND** it recommends validating and archiving the change explicitly

### Requirement: Lifecycle next actions
The system SHALL show phase-specific next actions without executing lifecycle transitions from the browser.

#### Scenario: Propose next actions
- **WHEN** lifecycle guidance identifies Propose-phase attention
- **THEN** it recommends creating or refining proposal artifacts
- **AND** it shows copyable command or Pi guidance for proposal work

#### Scenario: Review next actions
- **WHEN** lifecycle guidance identifies Review-phase attention
- **THEN** it recommends opening the active change review or companion page
- **AND** it explains that review feedback is advisory until the user explicitly applies artifact updates

#### Scenario: Apply next actions
- **WHEN** lifecycle guidance identifies Apply-phase attention
- **THEN** it recommends implementing tasks and running validation
- **AND** it shows copyable command or Pi guidance for applying the change

#### Scenario: Archive next actions
- **WHEN** lifecycle guidance identifies Archive-phase attention
- **THEN** it recommends validating the change before archive
- **AND** it shows copyable archive command guidance
- **AND** it does not archive the change from the browser

### Requirement: Lifecycle safety messaging
The system SHALL distinguish lifecycle guidance from artifact mutation.

#### Scenario: Browser guidance is non-mutating
- **WHEN** lifecycle guidance presents next actions
- **THEN** it states that the browser page does not directly edit proposal, design, task, or spec delta artifacts
- **AND** it directs the user to explicit CLI or Pi workflows for mutations

#### Scenario: Approval-gated apply guidance
- **WHEN** lifecycle guidance recommends applying feedback or implementation work
- **THEN** it explains that proposed artifact or code changes require explicit user approval before they are applied
