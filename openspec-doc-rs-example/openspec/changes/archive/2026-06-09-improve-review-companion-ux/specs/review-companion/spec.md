## ADDED Requirements

### Requirement: Tabbed review companion stages
The review companion SHALL organize active-change artifacts into tabs for proposal, design, tasks, and spec delta review stages.

#### Scenario: Stage tabs shown for active change
- **WHEN** a user opens the review companion for an active change
- **THEN** the page shows tab controls for Proposal, Design, Tasks, and Spec Deltas
- **AND** each available artifact is shown under its matching stage tab

#### Scenario: Missing stage has clear empty state
- **WHEN** an active change does not have an artifact for a review stage
- **THEN** the matching tab remains visible
- **AND** selecting the tab shows that no artifact is available for that stage

#### Scenario: Switching tabs preserves companion panel
- **WHEN** a user switches between review stage tabs
- **THEN** the companion guidance and feedback panel remains available on the same page
- **AND** the browser does not navigate away from the companion route

### Requirement: Right-click anchored comment creation
The review companion SHALL let users start an anchored comment by right-clicking selected text in a rendered active-change artifact.

#### Scenario: Right-click selected text starts comment
- **WHEN** a user selects text in a rendered active-change artifact
- **AND** the user right-clicks within the selected artifact context
- **THEN** the feedback form is populated with the artifact path and selected text
- **AND** the comment editor is focused for immediate typing

#### Scenario: Right-click without selection falls back safely
- **WHEN** a user right-clicks a rendered active-change artifact without selected text in that artifact
- **THEN** the review companion does not create a feedback record
- **AND** it keeps the existing manual selected-text feedback flow available

### Requirement: Editable review comment bodies
The review companion SHALL let users edit the body of existing sidecar review comments inline without modifying OpenSpec artifacts.

#### Scenario: Inline comment edit saved
- **WHEN** review companion mode is active
- **AND** a user edits an existing review comment body in the browser
- **THEN** the system stores the updated body in sidecar review storage
- **AND** the companion page reflects the edited body and updated timestamp
- **AND** proposal, design, task, and spec delta files under `openspec/` are not modified

#### Scenario: Empty comment edit rejected
- **WHEN** a user submits an empty edited comment body
- **THEN** the system rejects the edit request
- **AND** the original sidecar comment body remains visible

#### Scenario: Comment editing hidden when companion disabled
- **WHEN** a user starts the viewer with review companion mode disabled
- **THEN** the viewer does not expose browser-side comment editing controls
- **AND** comment edit write requests are rejected
