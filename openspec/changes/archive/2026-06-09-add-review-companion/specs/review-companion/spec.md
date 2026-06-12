## ADDED Requirements

### Requirement: Explicit review companion mode
The system SHALL provide an explicit opt-in review companion mode for active OpenSpec changes.

#### Scenario: Companion mode enabled
- **WHEN** a user starts the viewer with review companion mode enabled
- **THEN** the viewer exposes a companion route for active changes
- **AND** the route is labeled as review companion mode

#### Scenario: Companion mode disabled by default
- **WHEN** a user starts the viewer without review companion mode enabled
- **THEN** the viewer does not expose browser-side feedback write controls
- **AND** write requests for review feedback are rejected

### Requirement: One-window review companion layout
The review companion SHALL let users review artifacts, read review guidance, and inspect feedback on a single browser page.

#### Scenario: Artifacts and companion panel shown together
- **WHEN** a user opens the companion route for an active change
- **THEN** the page shows proposal artifacts in a readable main area
- **AND** the same page shows a companion panel with review guidance and feedback controls

#### Scenario: Existing feedback visible without Pi window
- **WHEN** sidecar feedback records exist for the active change
- **THEN** the companion page shows those feedback records in the browser without requiring a separate Pi/chat window

#### Scenario: Guidance visible during review
- **WHEN** a user scrolls or navigates the review artifacts
- **THEN** review guidance remains available from the companion page so the user does not need another window to read instructions

### Requirement: Sidecar feedback capture
The review companion SHALL capture browser comments as sidecar feedback records outside OpenSpec artifacts.

#### Scenario: Comment saved as sidecar record
- **WHEN** a user submits a companion review comment for an active change
- **THEN** the system appends a feedback record outside the target project's `openspec/` directory
- **AND** the proposal, design, task, and spec delta files are not modified by that submission

#### Scenario: Feedback record has review context
- **WHEN** the system stores a feedback record
- **THEN** the record includes the change name, timestamp, comment body, and any available artifact or selected-text context

#### Scenario: Invalid change rejected
- **WHEN** a feedback write request references a missing, archived, or path-traversal change name
- **THEN** the system rejects the request and does not write a feedback record

### Requirement: Pi-routed feedback iteration
The project SHALL provide a Pi-invokable workflow for consuming review companion feedback.

#### Scenario: Pi reads open feedback
- **WHEN** a user invokes the feedback iteration workflow for an active change
- **THEN** the workflow reads sidecar feedback records for that change
- **AND** summarizes open feedback for review in Pi

#### Scenario: Pi applies changes intentionally
- **WHEN** the user asks Pi to address review feedback
- **THEN** Pi proposes or applies updates to OpenSpec artifacts through the normal agent workflow rather than through automatic browser edits

#### Scenario: No feedback found
- **WHEN** the feedback iteration workflow runs for a change with no sidecar feedback records
- **THEN** it reports that no review companion feedback is available for that change
