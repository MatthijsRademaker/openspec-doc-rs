## ADDED Requirements

### Requirement: Companion browser launch
The review companion SHALL open in a browser tab by default using the server's resolved local URL.

#### Scenario: Browser opens with resolved port
- **WHEN** a user starts the browser viewer without disabling browser auto-open
- **AND** the server binds to a requested or automatically selected port
- **THEN** the CLI opens the default browser to the resolved viewer URL
- **AND** the opened URL uses the actual host and port reported by the running server

#### Scenario: Browser auto-open disabled
- **WHEN** a user starts the browser viewer with browser auto-open disabled
- **THEN** the CLI prints the resolved viewer URL
- **AND** it does not attempt to open a browser tab

### Requirement: Anchored selected-text comments
The review companion SHALL let users create comments attached to selected text in rendered active-change artifacts.

#### Scenario: Comment created from selected text by default
- **WHEN** a user starts the browser viewer without disabling review companion mode
- **AND** the user selects text in a rendered active-change artifact
- **AND** the user submits a comment for that selection
- **THEN** the system stores a sidecar feedback record for the active change
- **AND** the record includes the comment body, artifact path or identifier, selected text, timestamp, and available anchor context
- **AND** the proposal, design, task, and spec delta files under `openspec/` are not modified by the submission

#### Scenario: Comment creation requires selection context
- **WHEN** a user submits an anchored comment without selected text or artifact context
- **THEN** the system rejects the anchored comment request
- **AND** it does not write an anchored comment record

#### Scenario: Anchored comments hidden when explicitly disabled
- **WHEN** a user starts the browser viewer with review companion mode disabled
- **THEN** the viewer does not expose selected-text comment controls
- **AND** anchored comment write requests are rejected

### Requirement: Comment anchor resolution
The review companion SHALL resolve stored comment anchors against rendered artifact content and make unresolved anchors visible.

#### Scenario: Exact anchor found
- **WHEN** a stored comment's selected text and context match current artifact content
- **THEN** the companion page shows the comment as attached to the matching rendered text
- **AND** the comment anchor state is treated as exact

#### Scenario: Fuzzy anchor found
- **WHEN** a stored comment's exact selected text location no longer matches
- **AND** the system can locate a likely matching section or surrounding context in the artifact
- **THEN** the companion page shows the comment near the likely matching content
- **AND** the comment anchor state is marked as fuzzy

#### Scenario: Anchor cannot be found
- **WHEN** a stored comment references an existing artifact but the system cannot locate the selected text or likely context
- **THEN** the companion page still lists the comment in the companion panel
- **AND** the comment anchor state is marked as orphaned
- **AND** the original selected text remains visible in the comment details

#### Scenario: Referenced artifact missing
- **WHEN** a stored comment references an artifact that is no longer available for the active change
- **THEN** the companion page still lists the comment in the companion panel
- **AND** the comment anchor state is marked as missing

### Requirement: Review comment threads and status
The review companion SHALL support back-and-forth discussion and status tracking for sidecar review comments.

#### Scenario: User replies to comment
- **WHEN** review companion mode is active and a user replies to an existing review comment
- **THEN** the system appends the reply to the sidecar review thread for that comment
- **AND** it does not modify files under `openspec/`

#### Scenario: Comment status updated
- **WHEN** review companion mode is active and a user changes a review comment status
- **THEN** the system records the new status in sidecar review storage
- **AND** the companion page reflects the updated status for that comment

#### Scenario: Resolved comments remain inspectable
- **WHEN** a review comment is marked resolved or addressed
- **THEN** the companion page still allows the user to inspect the original comment, replies, anchor context, and status history

### Requirement: Review feedback synthesis
The Pi-routed feedback iteration workflow SHALL synthesize review companion comments into a structured review digest and proposed OpenSpec artifact changes.

#### Scenario: Open comments compiled into digest
- **WHEN** the user invokes the feedback iteration workflow for an active change with open review companion comments
- **THEN** Pi reads the sidecar review comments for that change
- **AND** it groups related feedback by theme, artifact, or affected requirement
- **AND** it presents a review digest that references the source comment ids

#### Scenario: Digest proposes artifact updates
- **WHEN** Pi synthesizes open review feedback
- **THEN** the digest identifies proposed updates to relevant proposal, design, task, or spec delta artifacts when such updates are inferable
- **AND** the digest distinguishes proposed updates from unresolved questions

#### Scenario: Ambiguous feedback prompts clarification
- **WHEN** Pi cannot infer the intended artifact change from one or more review comments
- **THEN** it asks the user for clarification before applying changes to OpenSpec artifacts

#### Scenario: Artifact edits require approval
- **WHEN** Pi presents synthesized review feedback and proposed artifact updates
- **THEN** OpenSpec artifacts are not modified unless the user explicitly asks Pi to apply or capture the proposed changes

#### Scenario: No open comments available
- **WHEN** the user invokes the feedback iteration workflow for an active change with no open review companion comments
- **THEN** Pi reports that no open review companion feedback is available for that change
