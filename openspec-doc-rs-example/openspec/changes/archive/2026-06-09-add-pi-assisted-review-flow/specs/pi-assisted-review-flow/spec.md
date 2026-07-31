## ADDED Requirements

### Requirement: Pi review workflow entrypoint
The project SHALL provide a Pi-invokable OpenSpec review workflow that helps a user review an active change in the browser viewer.

#### Scenario: Review workflow invoked with active change name
- **WHEN** a user invokes the Pi review workflow with an active OpenSpec change name
- **THEN** the workflow starts or reuses the local `openspec-doc` viewer for the current project
- **AND** the workflow provides a browser URL for reviewing that active change

#### Scenario: Review workflow invoked without active change name
- **WHEN** a user invokes the Pi review workflow without an active change name
- **THEN** the workflow inspects active OpenSpec changes or asks the user to identify which change to review

#### Scenario: Review workflow after proposal creation
- **WHEN** a user has just created a new OpenSpec proposal in Pi
- **THEN** the review workflow can be used without requiring the user to manually construct the `openspec-doc serve` command or active change URL

### Requirement: Pi review workflow remains non-mutating
The Pi review workflow SHALL NOT directly modify target OpenSpec artifacts when starting the browser review session.

#### Scenario: Starting review does not edit artifacts
- **WHEN** the Pi review workflow starts the viewer for an active change
- **THEN** it does not create, modify, or delete files under the target project's `openspec/` directory

#### Scenario: Feedback remains in Pi for first slice
- **WHEN** a user wants to comment on the proposal during this first review workflow slice
- **THEN** the workflow instructs the user to provide comments in Pi rather than through browser-side write controls

### Requirement: Future review companion path
The review workflow SHALL document the intended future path for browser comments to become Pi-routed review feedback without direct browser edits to OpenSpec artifacts.

#### Scenario: Future comment model described
- **WHEN** a contributor inspects the review workflow documentation or design
- **THEN** it describes browser comments as sidecar feedback records that Pi can consume for iteration

#### Scenario: Future implementation remains explicit
- **WHEN** browser-side comment capture is added in a later change
- **THEN** it is introduced through an explicit review companion mode rather than changing the default read-only `serve` behavior silently
