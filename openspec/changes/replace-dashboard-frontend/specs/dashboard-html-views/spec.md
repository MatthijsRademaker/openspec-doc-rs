## MODIFIED Requirements

### Requirement: Artifacts are rendered as markdown, decomposed into anchorable blocks
The system SHALL render an artifact as formatted markdown rather than as its source, decomposing it into blocks that each carry their rendered form and the exact source text they were sliced from, and SHALL create anchors from that source text.

Showing the source guaranteed that a browser selection was a substring of the file. The block model keeps that guarantee by a different route: the client echoes back a string the server sliced out of the file, so the anchor's substring search cannot fail for a block that still exists. What is given up is that the guarantee is now correct by construction rather than visible to the reviewer.

#### Scenario: Every block's source is findable in its artifact
- **WHEN** an artifact is decomposed into blocks
- **THEN** each block's source text SHALL occur in that artifact's markdown

#### Scenario: A comment on a block anchors exactly
- **WHEN** a reviewer comments on a rendered block
- **THEN** the system SHALL create the anchor from that block's source text and SHALL resolve it to the exact state

#### Scenario: Markup in an artifact does not reach the page as markup
- **WHEN** an artifact contains raw HTML
- **THEN** the system SHALL render it as text rather than as markup

### Requirement: Comments are shown against the block they anchor to
The system SHALL render each comment beside the block its anchor resolves to, collapsed behind an expandable marker, and SHALL NOT rely on a pooled list of comments separated from the artifact.

An anchored comment that is only reachable in a list hundreds of lines below the text it annotates is not anchored in any sense the reviewer experiences.

#### Scenario: A comment appears beside its text
- **WHEN** a scope has a comment whose anchor resolves to a block
- **THEN** the system SHALL render a marker against that block, expanding to the comment and its replies

#### Scenario: A drifted comment shows where it landed
- **WHEN** a comment's anchor resolves fuzzily
- **THEN** the system SHALL render it against the block it resolved to and SHALL indicate that it moved

### Requirement: A comment with no block remains reachable
The system SHALL provide a surface listing every comment that has no block to sit beside — those with no anchor, and those whose anchor no longer resolves — and SHALL show a count of them alongside the scope's controls.

Rendering comments only beside their anchors would remove a comment from the interface at the moment its anchor broke, which is the drift the anchor states exist to report.

#### Scenario: An orphaned comment is still reachable
- **WHEN** a comment's anchor no longer resolves to any text in its artifact
- **THEN** the system SHALL list it with its original quoted text and SHALL report that its anchor was lost

#### Scenario: An unanchored comment is listed with them
- **WHEN** a scope has a comment recorded with no anchor
- **THEN** the system SHALL list it on the same surface

### Requirement: Commenting is initiated from the block under the pointer
The system SHALL offer a comment action on the block a reviewer is pointing at, and SHALL additionally allow a comment to be created from a free-text selection within a block.

#### Scenario: Hovering a block offers a comment
- **WHEN** a reviewer points at a rendered block
- **THEN** the system SHALL offer an action creating a comment anchored to that block

#### Scenario: A selection crossing inline markup is refused with its reason
- **WHEN** a reviewer selects text spanning rendered inline markup, whose text does not occur in the source
- **THEN** the system SHALL refuse the comment and SHALL report why

### Requirement: Phase verdicts are submitted from a persistent control
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

### Requirement: The rendered artifact reflects the artifact on disk
The system SHALL update a rendered artifact when its file changes, and SHALL defer that update while the reviewer has unsent text in a composer, applying it once there is none.

The filesystem watcher already reports artifact changes and the event already reaches the browser; only the review state is refreshed in response. An exploration that is readable "while it is happening" cannot leave the exploration itself stale.

#### Scenario: An artifact rewritten under an idle page updates
- **WHEN** an artifact changes while its page is open with no unsent composer text
- **THEN** the system SHALL render the new content without the reviewer acting

#### Scenario: An artifact rewritten under a page with unsent text offers a refresh
- **WHEN** an artifact changes while a composer holds unsent text
- **THEN** the system SHALL indicate that the artifact has changed and SHALL NOT replace the content until the composer is sent or dismissed

### Requirement: The scope's review state is visible without scrolling to it
The system SHALL show a scope's standing verdict, when it was submitted, and whether its directive has been delivered to the agent, without requiring the reviewer to reach the end of the document.

Whether a submitted verdict actually reached the agent is recorded in the directive record and has never been shown anywhere, so submitting one is indistinguishable from submitting into nothing.

#### Scenario: Delivery state is reported
- **WHEN** a scope has a standing verdict whose directive has not yet been consumed
- **THEN** the system SHALL report that it has not yet been delivered

### Requirement: An addressed comment is shown as a claim, not as a resolution
The system SHALL render a comment marked `addressed` as the agent's report awaiting the reviewer's judgement, offering acceptance and reopening, and SHALL NOT render it as resolved.

Resolving is the reviewer accepting the work. An interface that showed the agent's own claim as a resolution would misreport the single judgement the reviewer is present to make.

#### Scenario: An addressed comment offers acceptance
- **WHEN** a comment has been marked addressed by the agent
- **THEN** the system SHALL show the agent's reply and SHALL offer both accepting it as resolved and reopening it

### Requirement: A comment thread can be acted on from the interface
The system SHALL offer, for each comment, controls to reply to it, to resolve it, and to reopen it, selected by the comment's current status. It SHALL NOT offer a control that marks a comment `addressed`.

Creating a comment is the only thread operation the interface has ever supported, so closing one means leaving the browser, running `comment list` to recover a UUID, and pasting it into `comment resolve` — in a tool whose premise is browser-based review. These requirements arrive from `add-comment-thread-actions`, whose interface half targeted files this change deletes.

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

### Requirement: The scope's comment counts are visible
The system SHALL show, for the scope being reviewed, how many of its comments are `open`, `addressed`, and `resolved`.

This is what tells a reviewer whether there is anything left to act on without reading every thread, and it is the precondition `add-change-approval-gate` consumes.

#### Scenario: Counts are shown for a scope with comments in several states
- **WHEN** a scope holds comments that are open, addressed, and resolved
- **THEN** the interface SHALL show each count

#### Scenario: Counts follow a status change without a reload
- **WHEN** a comment's status changes while the scope is open in a browser
- **THEN** the shown counts SHALL update without the user reloading the page

### Requirement: The index survives the replacement
The system SHALL continue to present an index of sessions and changes carrying, per row, the scope's title where one exists, its identifier, when its artifacts were last modified, how many of its comments are open, and its standing verdict.

The index is the only navigation into every other page, and its content is a requirement this capability already carries — shipped, not proposed. Replacing the server-rendered pages without re-providing it would delete working behaviour and the sole route to the pages this change is rewriting. It is called out because the rest of this change is scoped in terms of session and change pages, and an index that is merely not mentioned is an index that does not get built.

#### Scenario: Every discovered scope is reachable from the index
- **WHEN** the interface is opened at its root
- **THEN** it SHALL list every discovered session and change, each linking to its own page

#### Scenario: Index rows keep the fields the previous interface showed
- **WHEN** the index renders a scope
- **THEN** it SHALL show that scope's title where one exists, its identifier, its last-modified time, its open-comment count, and its standing verdict

## REMOVED Requirements

### Requirement: Select-to-comment interaction
**Reason**: Replaced by block-initiated commenting, which retains free-text selection as a secondary path. The original requirement specified selection over rendered *source* as the only gesture.

**Migration**: The anchoring contract is unchanged — an anchor is still created by finding the submitted text in the artifact's markdown — so existing comment records resolve exactly as before.
