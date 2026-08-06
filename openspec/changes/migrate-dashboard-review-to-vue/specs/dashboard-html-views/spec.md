## MODIFIED Requirements

### Requirement: Session page renders scratch note and comments
The system SHALL render, at `/sessions/<session_id>`, the session's scratch note as formatted markdown decomposed into blocks, with its comments shown against the blocks they anchor to.

#### Scenario: Scratch note and comments render
- **WHEN** a browser requests `/sessions/<session_id>` for a session with a scratch note
- **THEN** the system SHALL render the scratch note's rendered content and any existing comments against the blocks their anchors resolve to

### Requirement: Change page renders artifacts and comments
The system SHALL render, at `/changes/<name>`, the change's proposal, design, tasks and spec deltas as formatted markdown decomposed into blocks, with its comments shown against the blocks they anchor to.

#### Scenario: Change artifacts and comments render
- **WHEN** a browser requests `/changes/<name>` for a discovered change
- **THEN** the system SHALL render each artifact's rendered content and any existing comments against the blocks their anchors resolve to

### Requirement: Phase-verdict submission
The system SHALL present a scope's verdict controls persistently rather than at the end of the document, comprising a composer whose submission records an unanchored comment and submits the keep-exploring or comment-resolution verdict, and a control submitting the verdict that advances the work.

#### Scenario: A verdict is submitted with no accompanying text
- **WHEN** a reviewer submits from the composer with it empty
- **THEN** the system SHALL submit the verdict and SHALL NOT record a comment

#### Scenario: Composer text becomes an unanchored comment
- **WHEN** a reviewer submits from the composer with text in it
- **THEN** the system SHALL record that text as an unanchored comment against the scope and SHALL submit the verdict

#### Scenario: The advancing control names the act it performs
- **WHEN** a session page renders its verdict controls
- **THEN** the control submitting `move-to-proposal` SHALL be labelled as moving to proposal, distinctly from the approval that `change-approval` defines

### Requirement: Comment-resolution verdict on change pages
The system SHALL offer, from a change page's persistent controls, submission of the `comment-resolution` verdict with an optional accompanying comment.

The change page has no notes-carrying way to send comments back today: `comment-resolution` carries nothing beyond itself.

#### Scenario: Comment resolution is submitted from a change page
- **WHEN** a reviewer submits the composer on a change page
- **THEN** the system SHALL submit the `comment-resolution` verdict, recording any composer text as an unanchored comment

## ADDED Requirements

### Requirement: Comments are shown against the block they anchor to
The system SHALL render each comment beside the block its anchor resolves to, collapsed behind an expandable marker, and SHALL NOT rely on a pooled list of comments separated from the artifact.

An anchored comment that is only reachable in a list hundreds of lines below the text it annotates is not anchored in any sense the reviewer experiences.

#### Scenario: A comment appears beside its text
- **WHEN** a scope has a comment whose anchor resolves to a block
- **THEN** the system SHALL render a marker against that block, expanding to the comment and its replies

#### Scenario: A drifted comment shows where it landed
- **WHEN** a comment's anchor resolves fuzzily
- **THEN** the system SHALL render it against the block it resolved to and SHALL indicate that it moved

#### Scenario: The reviewer's words and the agent's are distinguishable
- **WHEN** a comment thread contains both the reviewer's comment and an agent's reply
- **THEN** the system SHALL render them visually distinct from one another

### Requirement: The scope's review state is visible without scrolling to it
The system SHALL show a scope's standing verdict, when it was submitted, whether its directive has been delivered to the agent, and its verdict history, without requiring the reviewer to reach the end of the document.

Whether a submitted verdict actually reached the agent is recorded in the directive record and has never been shown anywhere, so submitting one is indistinguishable from submitting into nothing. The verdict history also loses its only home when the pooled list at the bottom of the page is deleted.

#### Scenario: The standing verdict is visible on arrival
- **WHEN** a scope with a recorded verdict is opened
- **THEN** the system SHALL show that verdict and when it was submitted, without the reviewer scrolling to the end of the document

#### Scenario: Delivery state is reported
- **WHEN** a scope has a standing verdict whose directive has not yet been consumed
- **THEN** the system SHALL report that it has not yet been delivered

### Requirement: Commenting is initiated from the block under the pointer
The system SHALL offer a comment action on the block a reviewer is pointing at, SHALL create that comment's anchor from the block's source text and the block's position in the artifact, and SHALL additionally allow a comment to be created from a free-text selection within a block.

Anchoring from the block's position rather than from its text alone is what makes a comment on the second of two identical list items land on the second.

#### Scenario: Hovering a block offers a comment
- **WHEN** a reviewer points at a rendered block
- **THEN** the system SHALL offer an action creating a comment anchored to that block

#### Scenario: A comment on a block anchors exactly
- **WHEN** a reviewer comments on a rendered block
- **THEN** the system SHALL create the anchor from that block's source text at that block's position, and the anchor SHALL resolve to the exact state

#### Scenario: A repeated block anchors where it was pointed at
- **WHEN** a reviewer comments on a block whose source text also occurs earlier in the same artifact
- **THEN** the comment SHALL anchor to the block that was pointed at

#### Scenario: A selection crossing inline markup is refused with its reason
- **WHEN** a reviewer selects text spanning rendered inline markup, whose text does not occur in the source
- **THEN** the system SHALL refuse the comment and SHALL report why

### Requirement: A comment with no block remains reachable
The system SHALL provide a surface listing every comment that has no block to sit beside — those with no anchor, and those whose anchor no longer resolves — and SHALL show a count of them alongside the scope's controls.

Rendering comments only beside their anchors would remove a comment from the interface at the moment its anchor broke, which is the drift the anchor states exist to report.

#### Scenario: An orphaned comment is still reachable
- **WHEN** a comment's anchor no longer resolves to any text in its artifact
- **THEN** the system SHALL list it with its original quoted text and SHALL report that its anchor was lost

#### Scenario: An unanchored comment is listed with them
- **WHEN** a scope has a comment recorded with no anchor
- **THEN** the system SHALL list it on the same surface

### Requirement: A comment thread can be acted on from the interface
The system SHALL offer, for each comment, controls to reply to it, to resolve it, and to reopen it, selected by the comment's current status. It SHALL NOT offer a control that marks a comment `addressed`.

Creating a comment is the only thread operation the interface has ever supported, so closing one means leaving the browser, running `comment list` to recover a UUID, and pasting it into `comment resolve` — in a tool whose premise is browser-based review. These requirements arrive from `add-comment-thread-actions`, whose interface half was deliberately left for the change that owned the pages.

`addressed` is excluded deliberately and is not an oversight: it asserts work an agent did, not a judgement the reviewer makes. An interface control for it would let the reviewer make the agent's claim on its behalf.

#### Scenario: An open comment offers reply and resolve
- **WHEN** a comment's current status is `open`
- **THEN** the interface SHALL offer a reply control and a resolve control for it

#### Scenario: A resolved comment offers reopen
- **WHEN** a comment's current status is `resolved`
- **THEN** the interface SHALL offer a reopen control for it and SHALL NOT offer a resolve control

#### Scenario: No control marks a comment addressed
- **WHEN** the interface renders any comment in any status
- **THEN** it SHALL NOT offer a control that moves that comment to `addressed`

#### Scenario: A thread action in one tab appears in another
- **WHEN** a reply or status change is submitted while a second browser tab has the same scope open
- **THEN** the second tab SHALL reflect it without the user reloading the page

### Requirement: An addressed comment is shown as a claim, not as a resolution
The system SHALL render a comment marked `addressed` as the agent's report awaiting the reviewer's judgement, offering acceptance and reopening, and SHALL NOT render it as resolved.

Resolving is the reviewer accepting the work. An interface that showed the agent's own claim as a resolution would misreport the single judgement the reviewer is present to make.

#### Scenario: An addressed comment offers acceptance
- **WHEN** a comment has been marked addressed by the agent
- **THEN** the system SHALL show the agent's reply and SHALL offer both accepting it as resolved and reopening it

### Requirement: The scope's comment counts are visible
The system SHALL show, for the scope being reviewed, how many of its comments are `open`, `addressed`, and `resolved`.

This is what tells a reviewer whether there is anything left to act on without reading every thread, and it is the precondition `add-change-approval-gate` consumes.

#### Scenario: Counts are shown for a scope with comments in several states
- **WHEN** a scope holds comments that are open, addressed, and resolved
- **THEN** the interface SHALL show each count

#### Scenario: Counts follow a status change without a reload
- **WHEN** a comment's status changes while the scope is open in a browser
- **THEN** the shown counts SHALL update without the user reloading the page

## REMOVED Requirements

### Requirement: Select-to-comment interaction
**Reason**: Replaced by block-initiated commenting, which retains free-text selection as a secondary path. The original requirement specified selection over rendered *source* as the only gesture, and the source is no longer what is rendered.

**Migration**: The anchoring contract is unchanged — an anchor is still created by finding the submitted text in the artifact's markdown — so existing comment records resolve exactly as before.
