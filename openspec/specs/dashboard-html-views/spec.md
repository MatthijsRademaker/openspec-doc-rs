# dashboard-html-views Specification

## Purpose

What the dashboard's pages contain: the session-scoped page rendering an exploration's scratch note, the change-scoped page rendering a change's proposal, design, tasks and spec deltas, the anchored comments shown against each, the select-to-comment interaction that creates them, and the phase-verdict controls a reviewer submits from either page. Artifacts are rendered as their own markdown source, so a browser selection is already a substring of the file the server anchors against and no offset mapping exists to get wrong. Pages are server-rendered HTML with the minimum vanilla JavaScript that selection and live updates require — no framework, no build step. The server, its route table, and the event stream these pages subscribe to belong to `dashboard-server`; the comment records themselves belong to `anchored-comments`.

## Requirements
### Requirement: Session page renders scratch note and comments
The system SHALL render, at `/sessions/<session_id>`, the session's scratch note markdown and its anchored comments.

#### Scenario: Scratch note and comments render
- **WHEN** a browser requests `/sessions/<session_id>` for a session with a scratch note
- **THEN** the system SHALL render the scratch note's content and any existing anchored comments against it

### Requirement: Change page renders artifacts and comments
The system SHALL render, at `/changes/<name>`, the change's `proposal.md`, `design.md`, `tasks.md`, and spec delta files along with their anchored comments.

#### Scenario: Change artifacts and comments render
- **WHEN** a browser requests `/changes/<name>` for a discovered active change
- **THEN** the system SHALL render each present artifact's content and any existing anchored comments against it

### Requirement: Select-to-comment interaction
The system SHALL allow a reviewer to select text within a rendered artifact and submit a comment anchored to that selection, using the anchored-comments capability.

#### Scenario: Comment is created from a text selection
- **WHEN** a reviewer selects text in a rendered artifact and submits a comment
- **THEN** the system SHALL create an anchor context from the raw markdown source underlying that selection and persist the comment via the anchored-comments capability

### Requirement: Phase-verdict submission
The system SHALL allow a reviewer on a session page to submit a phase verdict of either "keep exploring" with free-text notes, or "move to proposal", and SHALL persist it as a session-scoped record distinct from per-comment status.

#### Scenario: Keep-exploring verdict is recorded with notes
- **WHEN** a reviewer submits "keep exploring" with open questions or resolutions entered as free text
- **THEN** the system SHALL persist a phase-verdict record for that session containing the verdict and the free-text content

#### Scenario: Move-to-proposal verdict is recorded
- **WHEN** a reviewer submits "move to proposal"
- **THEN** the system SHALL persist a phase-verdict record for that session containing that verdict

### Requirement: Comment-resolution verdict on change pages
The system SHALL allow a reviewer on a change page to submit a "send to agent: address open comments" verdict.

#### Scenario: Comment-resolution verdict is recorded
- **WHEN** a reviewer on a change page submits "send to agent"
- **THEN** the system SHALL persist a phase-verdict record for that change indicating open comments should be addressed

### Requirement: Live update without reload
The system SHALL update a rendered page's comment list and verdict state when the underlying SSE endpoint emits an event, without requiring a full page reload.

#### Scenario: Comment added in one tab appears in another
- **WHEN** a comment is added against an artifact while a second browser tab has the same page open
- **THEN** the second tab SHALL reflect the new comment without the user reloading the page

