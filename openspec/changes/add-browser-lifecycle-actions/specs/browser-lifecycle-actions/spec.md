## ADDED Requirements

### Requirement: Local lifecycle action API
The system SHALL provide a local browser action API for allowlisted OpenSpec lifecycle actions.

#### Scenario: Action endpoint accepts allowlisted action
- **WHEN** the local viewer receives a lifecycle action request for an allowlisted action
- **THEN** it validates the action payload against the served OpenSpec project model
- **AND** it returns a structured action result with status and user-facing output

#### Scenario: Action endpoint rejects unknown action
- **WHEN** the local viewer receives a lifecycle action request for an unknown action kind
- **THEN** it rejects the request
- **AND** it does not execute a command or mutate files

#### Scenario: Change action validates active change
- **WHEN** the local viewer receives a change-specific lifecycle action request
- **THEN** it verifies that the requested change is a discovered active change
- **AND** it rejects archived or unknown change names

### Requirement: Local command execution guardrails
The system SHALL execute only predefined lifecycle commands from browser action endpoints.

#### Scenario: Validation command runs locally
- **WHEN** a user confirms a validation lifecycle action for an active change
- **THEN** the server runs `openspec validate <change-name>` in the served project root
- **AND** the browser displays the command exit status and output

#### Scenario: Archive command runs locally
- **WHEN** a user confirms an archive lifecycle action for an active change
- **THEN** the server runs `openspec archive <change-name>` in the served project root
- **AND** the browser displays the command exit status and output
- **AND** a successful archive refreshes the server-side viewer model

#### Scenario: Arbitrary commands are not accepted
- **WHEN** a browser action request includes a command string or command arguments not defined by the action kind
- **THEN** the server ignores those requested command details
- **AND** it executes no arbitrary shell command from the request body

### Requirement: Task state mutation
The system SHALL allow browser lifecycle actions to toggle checklist task state for active change task artifacts.

#### Scenario: Task marked complete
- **WHEN** a user confirms marking an incomplete task complete from a lifecycle action button
- **THEN** the server updates only the matching checklist line in that active change's `tasks.md`
- **AND** subsequent viewer pages show updated task progress

#### Scenario: Task marked incomplete
- **WHEN** a user confirms marking a completed task incomplete from a lifecycle action button
- **THEN** the server updates only the matching checklist line in that active change's `tasks.md`
- **AND** subsequent viewer pages show updated task progress

#### Scenario: Stale task request rejected
- **WHEN** the requested task line no longer matches a checklist item in the active change's current `tasks.md`
- **THEN** the server rejects the task mutation
- **AND** it does not rewrite the task file

### Requirement: Lifecycle action handoffs
The system SHALL provide browser actions for lifecycle steps that require explicit Pi or CLI workflow handoff rather than direct mutation.

#### Scenario: Explore handoff action
- **WHEN** a user activates the Explore lifecycle action from the dashboard
- **THEN** the browser shows a copyable Pi or CLI handoff prompt for exploring an idea
- **AND** no OpenSpec artifacts are modified by the handoff action itself

#### Scenario: Propose handoff action
- **WHEN** a user activates a Propose lifecycle action for an active change
- **THEN** the browser shows a copyable Pi or CLI handoff prompt for proposal refinement
- **AND** no OpenSpec artifacts are modified by the handoff action itself

#### Scenario: Apply handoff action
- **WHEN** a user activates an Apply lifecycle action for an active change
- **THEN** the browser shows a copyable Pi or CLI handoff prompt for applying tasks
- **AND** no code or OpenSpec artifacts are modified by the handoff action itself

### Requirement: Action result visibility
The system SHALL display lifecycle action outcomes in the browser.

#### Scenario: Successful action result displayed
- **WHEN** a lifecycle action completes successfully
- **THEN** the browser shows a success result near the lifecycle controls
- **AND** it includes relevant command output, mutation summary, or handoff text

#### Scenario: Failed action result displayed
- **WHEN** a lifecycle action fails validation, command execution, or file mutation
- **THEN** the browser shows a failure result near the lifecycle controls
- **AND** it includes actionable error details without hiding existing page content
