## MODIFIED Requirements

### Requirement: Default review companion mode
The system SHALL provide review companion mode by default for active OpenSpec changes while supporting an explicit disabled mode for sidecar feedback controls and while preserving local lifecycle action controls.

#### Scenario: Companion mode enabled by default
- **WHEN** a user starts the viewer without disabling review companion mode
- **THEN** the viewer exposes a companion route for active changes
- **AND** the route is labeled as review companion mode

#### Scenario: Companion mode explicitly disabled
- **WHEN** a user starts the viewer with review companion mode disabled
- **THEN** the viewer does not expose browser-side feedback write controls
- **AND** write requests for review feedback are rejected

#### Scenario: Lifecycle actions remain separate from companion feedback mode
- **WHEN** a user opens an active change page while review companion mode is disabled
- **THEN** local lifecycle action controls are governed by lifecycle action availability rather than sidecar feedback mode
- **AND** feedback write requests remain rejected

### Requirement: One-window review companion layout
The review companion SHALL let users review artifacts, read review guidance, inspect feedback, and issue explicitly confirmed local lifecycle actions on a single browser page using a document-first annotation layout.

#### Scenario: Artifacts and guidance shown together
- **WHEN** a user opens the companion route for an active change
- **THEN** the page shows proposal artifacts in a readable main area
- **AND** the same page shows concise review guidance without requiring a separate Pi/chat window

#### Scenario: Existing feedback visible without Pi window
- **WHEN** sidecar feedback records exist for the active change
- **THEN** the companion page shows those feedback records in the browser without requiring a separate Pi/chat window
- **AND** anchored exact or fuzzy feedback is represented near the related rendered artifact content when possible

#### Scenario: Guidance visible during review
- **WHEN** a user scrolls or navigates the review artifacts
- **THEN** review guidance remains available from the companion page so the user does not need another window to read instructions

#### Scenario: Lifecycle actions visible during companion review
- **WHEN** a user opens the companion route for an active change
- **THEN** the page shows appropriate lifecycle action controls near lifecycle guidance
- **AND** sidecar feedback controls remain separate from lifecycle action controls

### Requirement: Review companion preserves intentional mutation boundaries
The review companion SHALL keep sidecar feedback writes separate from explicit local lifecycle action mutations.

#### Scenario: Comment submitted from popover
- **WHEN** a user submits a valid popover comment for selected artifact text
- **THEN** the system stores a sidecar feedback record for the active change
- **AND** the proposal, design, task, and spec delta files under `openspec/` are not modified

#### Scenario: Lifecycle action from companion is explicit
- **WHEN** a user activates a mutating lifecycle action from the companion page
- **THEN** the action requires explicit confirmation before the server runs a command or modifies `tasks.md`
- **AND** the action result is shown separately from sidecar feedback thread results

#### Scenario: Sidecar feedback does not trigger lifecycle transition
- **WHEN** a user creates, edits, resolves, or deletes sidecar review feedback while lifecycle action controls are visible
- **THEN** the system updates sidecar review storage only
- **AND** it does not mark tasks complete, validate, apply, or archive the change automatically
