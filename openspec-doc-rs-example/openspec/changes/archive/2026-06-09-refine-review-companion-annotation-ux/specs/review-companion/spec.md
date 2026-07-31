## ADDED Requirements

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

## MODIFIED Requirements

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
