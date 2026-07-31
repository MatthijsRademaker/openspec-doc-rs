## Purpose

Provide a default browser review companion for active OpenSpec changes that lets users review artifacts, capture anchored sidecar feedback, discuss comments, track status, and route synthesized feedback through Pi while keeping OpenSpec artifact edits intentional and approval-driven.
## Requirements
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

### Requirement: Default review companion mode
The system SHALL provide review companion mode by default for active OpenSpec changes while supporting an explicit disabled mode.

#### Scenario: Companion mode enabled by default
- **WHEN** a user starts the viewer without disabling review companion mode
- **THEN** the viewer exposes a companion route for active changes
- **AND** the route is labeled as review companion mode

#### Scenario: Companion mode explicitly disabled
- **WHEN** a user starts the viewer with review companion mode disabled
- **THEN** the viewer does not expose browser-side feedback write controls
- **AND** write requests for review feedback are rejected

### Requirement: Document-first companion layout
The review companion SHALL use a document-first layout that provides a wider review canvas while preserving comfortable reading width for rendered artifacts.

#### Scenario: Companion route uses wider review canvas
- **WHEN** a user opens the companion route for an active change
- **THEN** the page uses a wider layout than the default dashboard content width
- **AND** the rendered artifact text remains constrained to a readable line length

#### Scenario: Companion avoids dominant side panel
- **WHEN** a user opens the companion route for an active change
- **THEN** the primary review area is not reduced by a permanent full-height feedback side panel
- **AND** the page emphasizes the rendered OpenSpec artifact content as the main reading surface

### Requirement: Popover-based selected-text comments
The review companion SHALL provide intentional selected-text comment creation through contextual popovers instead of a persistent comment creation form.

#### Scenario: Selection shows comment affordance
- **WHEN** a user selects text in a rendered active-change artifact
- **THEN** the companion page shows an unobtrusive comment affordance near the selected text
- **AND** it does not automatically focus a comment editor
- **AND** it does not scroll the page away from the selected text

#### Scenario: User opens composer from selected text
- **WHEN** a user activates the comment affordance for selected artifact text
- **THEN** the page opens a comment composer near the selected text or active artifact context
- **AND** the composer is populated with the artifact path and selected text context
- **AND** no feedback record is written until the user submits the composer

#### Scenario: Comment submitted from popover
- **WHEN** a user submits a valid popover comment for selected artifact text
- **THEN** the system stores a sidecar feedback record for the active change
- **AND** the proposal, design, task, and spec delta files under `openspec/` are not modified

### Requirement: In-text comment markers and thread popovers
The review companion SHALL represent resolved anchored comments as unobtrusive markers in the rendered artifact content and expose thread details through popovers.

#### Scenario: Exact or fuzzy comment shown as marker
- **WHEN** a stored comment resolves to an exact or fuzzy location in a rendered artifact
- **THEN** the companion page shows a marker near the resolved artifact content
- **AND** the marker communicates that feedback exists without obscuring the document text

#### Scenario: Marker opens thread popover
- **WHEN** a user activates a comment marker
- **THEN** the page opens a popover for that comment thread
- **AND** the popover shows the comment body, selected text context, anchor state, status, and replies

#### Scenario: Thread actions available from marker popover
- **WHEN** a user opens a comment marker popover in review companion mode
- **THEN** the user can reply to the thread, edit the comment body, update comment status, or delete the comment from the popover
- **AND** those actions update sidecar review storage without modifying OpenSpec artifacts

#### Scenario: Comment deleted from review companion
- **WHEN** a user deletes a review comment from the companion page
- **THEN** the comment and its thread replies, edits, and status updates are removed from sidecar feedback storage
- **AND** the deleted comment no longer appears as a document marker or comments overview entry
- **AND** proposal, design, task, and spec delta files under `openspec/` are not modified

### Requirement: Separate comments overview
The review companion SHALL provide a comments overview as a separate page section for review triage and navigation.

#### Scenario: Comments overview lists all feedback
- **WHEN** sidecar feedback records exist for an active change
- **THEN** the companion page shows a comments overview separate from the rendered document content
- **AND** the overview includes exact, fuzzy, orphaned, and missing anchor states

#### Scenario: Overview groups feedback by review state
- **WHEN** the comments overview renders feedback records
- **THEN** it distinguishes open, addressed, and resolved comments
- **AND** each comment remains inspectable regardless of status

#### Scenario: Overview navigates to document marker
- **WHEN** a user activates a jump action for a comment that has a document marker
- **THEN** the page scrolls the primary document surface to the marker
- **AND** the marker is visually emphasized without opening a nested scrolling panel

### Requirement: Calm companion scrolling
The review companion SHALL avoid nested scrolling in the primary review flow and avoid forced scroll jumps during annotation creation.

#### Scenario: Primary page handles review scrolling
- **WHEN** a user reads artifacts and feedback in the companion route
- **THEN** the page uses the browser document as the primary vertical scroll surface
- **AND** feedback controls are not contained in a full-height independently scrolling side panel

#### Scenario: Selection does not force scroll
- **WHEN** a user selects text or right-clicks selected text in a rendered artifact
- **THEN** the page does not automatically scroll to a separate form or panel
- **AND** the selected document context remains visible

### Requirement: One-window review companion layout
The review companion SHALL let users review artifacts, read review guidance, and inspect feedback on a single browser page using a document-first annotation layout.

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
The review companion SHALL resolve stored comment anchors against rendered artifact content, show resolved comments as document markers, and make unresolved anchors visible in the comments overview.

#### Scenario: Exact anchor found
- **WHEN** a stored comment's selected text and context match current artifact content
- **THEN** the companion page shows the comment as attached to the matching rendered text with a document marker
- **AND** the comment anchor state is treated as exact

#### Scenario: Fuzzy anchor found
- **WHEN** a stored comment's exact selected text location no longer matches
- **AND** the system can locate a likely matching section or surrounding context in the artifact
- **THEN** the companion page shows a comment marker near the likely matching content
- **AND** the comment anchor state is marked as fuzzy

#### Scenario: Anchor cannot be found
- **WHEN** a stored comment references an existing artifact but the system cannot locate the selected text or likely context
- **THEN** the companion page still lists the comment in the comments overview
- **AND** the comment anchor state is marked as orphaned
- **AND** the original selected text remains visible in the comment details

#### Scenario: Referenced artifact missing
- **WHEN** a stored comment references an artifact that is no longer available for the active change
- **THEN** the companion page still lists the comment in the comments overview
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
The review companion SHALL let users intentionally start an anchored comment from selected text in a rendered active-change artifact without forcing focus or scrolling away from the document context.

#### Scenario: Right-click selected text shows comment action
- **WHEN** a user selects text in a rendered active-change artifact
- **AND** the user right-clicks within the selected artifact context
- **THEN** the companion page shows or opens a contextual comment action for the selected text
- **AND** it does not automatically scroll to a separate feedback form

#### Scenario: User intentionally opens comment editor
- **WHEN** a contextual comment action is available for selected text
- **AND** the user activates that action
- **THEN** the comment editor opens near the selected text or active artifact context
- **AND** the editor has the artifact path and selected text available for submission

#### Scenario: Right-click without selection falls back safely
- **WHEN** a user right-clicks a rendered active-change artifact without selected text in that artifact
- **THEN** the review companion does not create a feedback record
- **AND** it does not interrupt normal review navigation

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

### Requirement: Browser Pi review entrypoint
The review companion SHALL provide browser-native entrypoints for live Pi analysis by default unless review agent mode is explicitly disabled or unavailable.

#### Scenario: Ask Pi action appears by default
- **WHEN** a user opens an active change companion page during a default companion serve session
- **THEN** the page MUST show an Ask Pi action near the review guidance or comments overview

#### Scenario: Ask Pi action explains included context
- **WHEN** the user prepares to start Pi analysis from the companion page
- **THEN** the page MUST summarize whether open feedback, selected feedback, artifact snapshots, and warnings will be sent to Pi

#### Scenario: Disabled state appears when opted out
- **WHEN** a user opens an active change companion page while review agent analysis mode is explicitly disabled
- **THEN** the page MUST show a concise disabled state or omit the active Ask Pi control without disrupting existing comment creation and thread review behavior

#### Scenario: Unavailable state appears when Pi cannot start
- **WHEN** a user opens an active change companion page and Pi review analysis is unavailable because setup, credentials, or model availability are missing
- **THEN** the page MUST show an actionable unavailable state without disrupting existing comment creation and thread review behavior

### Requirement: In-browser Pi review transcript
The review companion SHALL render live Pi analysis as an in-browser review transcript.

#### Scenario: Transcript displays streaming response
- **WHEN** the browser receives Pi review text deltas
- **THEN** the companion page MUST append them to the active review transcript in reading order

#### Scenario: Transcript displays completion state
- **WHEN** a Pi review session completes successfully
- **THEN** the companion page MUST show that the analysis is complete and keep the response visible

#### Scenario: Transcript displays failure state
- **WHEN** a Pi review session cannot start or fails while running
- **THEN** the companion page MUST show an actionable error message without losing existing review comments

#### Scenario: Transcript offers retry after safe failure
- **WHEN** a Pi review session fails and retry is safe
- **THEN** the companion page MUST provide a retry action that starts a new session from the current review context and preserves the failed transcript for reference

### Requirement: Browser follow-up messages
The review companion SHALL allow the user to continue an active browser Pi review session with follow-up instructions when analysis mode is enabled.

#### Scenario: Follow-up sent after initial response
- **WHEN** a completed or idle Pi review session exists for the active change and the user submits a follow-up instruction
- **THEN** the companion page MUST send the message to the same review session and render the resulting streamed response

#### Scenario: Follow-up during running response
- **WHEN** the user submits a follow-up while the Pi review session is still streaming
- **THEN** the system MUST either queue the follow-up using Pi's supported streaming behavior or show a clear message that the user must wait

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

