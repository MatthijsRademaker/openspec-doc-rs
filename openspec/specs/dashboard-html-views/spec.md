# dashboard-html-views Specification

## Purpose

What the embedded Vue dashboard renders: an observatory index plus session and change review workbenches composed around formatted artifact blocks, spatially anchored conversation, thread actions, visible verdict delivery state, and persistent decision controls. The Rust server supplies JSON and event streams; Vue Router owns browser navigation and the built assets remain embedded in the binary. Comment records belong to `anchored-comments`; HTTP routing and push transport belong to `dashboard-server`.
## Requirements
### Requirement: Session page renders scratch note and comments

The system SHALL render, at `/sessions/<session_id>`, the session's scratch note as formatted markdown decomposed into blocks, with its comments shown against the blocks they anchor to.

#### Scenario: Scratch note and comments render

- **WHEN** a browser requests `/sessions/<session_id>` for a session with a scratch note
- **THEN** the system SHALL render the scratch note's rendered content and any existing comments against the blocks their anchors resolve to

### Requirement: Change page renders artifacts and comments

The system SHALL render, at `/changes/<name>`, an exact-path artifact navigator and one complete selected proposal, design, tasks, or spec-delta artifact as formatted markdown decomposed into blocks, with every comment belonging to that artifact reachable through its resolved block marker and artifact-scoped conversation instrument.

#### Scenario: Change opens with one selected artifact

- **WHEN** a browser requests `/changes/<name>` for a discovered change containing artifacts
- **THEN** the system SHALL select one supplied artifact, render its complete formatted content, and expose every other supplied artifact through exact-path navigation rather than concatenating all artifact contents

#### Scenario: Comments are scoped to visible artifact

- **WHEN** the selected artifact has anchored comments
- **THEN** the system SHALL show markers against their resolved blocks and SHALL show those threads in the selected artifact's conversation instrument

#### Scenario: Hidden artifact comments remain represented by scope state

- **WHEN** comments exist on artifacts other than the selected artifact
- **THEN** the system SHALL keep them represented in scope comment counts and artifact navigation state without presenting their bodies as though they annotate the visible document

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

#### Scenario: Comment resolution is submitted from a change page

- **WHEN** a reviewer submits the composer on a change page
- **THEN** the system SHALL submit the `comment-resolution` verdict, recording any composer text as an unanchored comment

### Requirement: Scope pages instantiate the observatory review workbench

The system SHALL compose each session and change route from one shared observatory workbench as compact route/state instrumentation, exact artifact navigation, one primary selected-document stage, artifact-scoped conversation, and persistent decision controls outside the document's reading plane. Authored celestial artwork SHALL form part of the selected document's bounded arrival composition without obscuring document text, state, focus, comments, composers, or controls.

A persistent trigger and a modal drawer are different surfaces with different obligations. The trigger is always present and therefore SHALL be placed rather than overlaid; the drawer is dismissible, backdropped, and focus-trapping, and is allowed to overlay the page for as long as a reviewer has chosen to have it open. Treating them as one positioned unit is what put a page-centred fixed bar across the foot of the document column.

#### Scenario: Session and change routes share one workbench

- **WHEN** a reviewer opens either a discovered session route or a discovered change route
- **THEN** both SHALL use the same scope-workbench landmarks, selected-document renderer, comment behavior, image-led composition, responsive rules, and mutation ownership, with only real route-kind and artifact-cardinality behavior differing

#### Scenario: Session does not imitate a multi-artifact change

- **WHEN** a session route contains its sole scratch artifact
- **THEN** the workbench SHALL identify the scratch document without rendering tabs, empty folders, or inactive artifact destinations

#### Scenario: Route identity does not compete with document gravity

- **WHEN** a titled scope opens and its selected artifact repeats that title as source content
- **THEN** compact route instrumentation SHALL show title and exact identifier while the selected artifact's source heading SHALL retain primary document gravity and SHALL NOT be hidden or rewritten

#### Scenario: Desktop keeps one document primary

- **WHEN** a scope page is rendered at 1280 CSS pixels or wider
- **THEN** the selected artifact SHALL retain a readable primary document measure while exact paths and scope state occupy an instrumentation rail and selected-artifact conversation occupies a distinct conversation rail

#### Scenario: Scope route links back to implemented index

- **WHEN** a reviewer opens a session or change scope
- **THEN** route instrumentation SHALL provide a real navigation link to `/` without inventing any unimplemented destination

#### Scenario: Desktop conversation remains tied to visible source

- **WHEN** the selected artifact has comments whose anchors resolve to blocks
- **THEN** the conversation rail SHALL contain only real threads associated with that artifact and SHALL provide bidirectional navigation between each thread and its resolved source block

#### Scenario: Active conversation locks its block visibly

- **WHEN** a reviewer activates a block marker or a thread in the conversation rail
- **THEN** the corresponding source block and thread SHALL receive linked visible treatment that does not rely on color alone

#### Scenario: Narrow workbench preserves one logical flow

- **WHEN** a scope page is rendered at 390 CSS pixels wide
- **THEN** scope state and artifact selection SHALL precede the complete selected document, selected-artifact conversation SHALL remain reachable after it, persistent decisions SHALL remain reachable, and the page SHALL have no horizontal overflow

#### Scenario: Desktop decision trigger occupies the conversation-side rail

- **WHEN** a scope page is rendered at desktop width, at any scroll position, with the conversation rail expanded or collapsed
- **THEN** the persistent decision trigger SHALL be laid out inside the conversation-side instrument region, SHALL remain visible without page scrolling, and its rectangle SHALL NOT intersect selected-document prose, artifact navigation, block comment actions, inline composers, thread actions, or focus indicators

#### Scenario: Decision drawer overlays only while a reviewer holds it open

- **WHEN** a reviewer opens the decision drawer
- **THEN** it MAY overlay the page as a backdropped modal surface, and on dismissal it SHALL restore focus to its trigger and leave no obstruction behind

#### Scenario: Narrow decision dock preserves final content

- **WHEN** the decision instrument is rendered at 390 CSS pixels wide
- **THEN** the final selected-document block, artifact-scoped thread action, active composer, and decision controls SHALL all be scrollable fully clear of the dock and safe-area inset

#### Scenario: Artwork yields before review content

- **WHEN** available width cannot preserve document-stage artwork or peripheral observation cells alongside review content
- **THEN** peripheral cells SHALL disappear and main artwork SHALL shrink, crop, or disappear before any artifact path, document text, state, thread, composer, or control is hidden or clipped

#### Scenario: Scope chrome reports only real state

- **WHEN** a scope page renders image-led instrument chrome inspired by the visual source
- **THEN** it SHALL NOT present repository metadata, validation results, activity events, lifecycle phases, navigation destinations, or review state absent from the scope or index APIs

### Requirement: A modal review surface opens at its own beginning

The system SHALL open a modal review surface scrolled to its own start, with the heading that names it visible, and SHALL place initial keyboard focus on the surface itself rather than on a control positioned below content the reviewer has not yet seen.

A surface that lists a scope's loose and orphaned comments is something to read before it is something to type into. Focusing its composer scrolls the panel to the composer, which puts the panel's own title and purpose off-screen at the moment it opens — and when the dialog's accessible name comes from that title, a screen reader announces a heading the sighted reviewer cannot see.

#### Scenario: The surface's own title is visible when it opens

- **WHEN** a reviewer opens a modal review surface whose content is taller than the surface
- **THEN** the surface SHALL be scrolled to its start and the heading providing its accessible name SHALL be within its visible bounds

#### Scenario: Initial focus is the surface, not its composer

- **WHEN** a modal review surface opens
- **THEN** keyboard focus SHALL be on the surface, and the composer SHALL remain reachable by forward keyboard traversal in reading order

#### Scenario: Reading order and focus order agree

- **WHEN** a reviewer traverses forward by keyboard from the moment a modal review surface opens
- **THEN** focus SHALL move through the surface's content in the order it is rendered, without moving backwards through content the reviewer has not passed

### Requirement: Comments are shown against the block they anchor to

The system SHALL render a marker against every block holding a resolved anchored comment and SHALL expose the comment in conversation scoped to the currently selected artifact. The interface SHALL maintain an explicit, navigable relationship between thread and resolved source block and SHALL NOT mix comments from hidden artifacts into the visible document's conversation.

A thread rail is contextual conversation rather than detached chat only when selecting either side reveals and visibly links the other side.

#### Scenario: A comment marks its source block

- **WHEN** the selected artifact has a comment whose anchor resolves to a block
- **THEN** the system SHALL render a status-distinct marker against that block and a corresponding thread entry in selected-artifact conversation

#### Scenario: Block marker activates its thread

- **WHEN** a reviewer activates a block's comment marker
- **THEN** the system SHALL activate the corresponding thread, make its source block visibly current, and move focus or reading position to keep the conversation reachable

#### Scenario: Thread entry navigates to source

- **WHEN** a reviewer activates a thread whose anchor resolves to a block in the selected artifact
- **THEN** the system SHALL bring that exact block into view and visibly link it to the active thread

#### Scenario: Repeated source navigates to exact occurrence

- **WHEN** an active thread anchors to the second of two identical source blocks
- **THEN** thread navigation SHALL bring the second block into view rather than the first matching text

#### Scenario: A drifted comment shows where it landed

- **WHEN** a comment's anchor resolves fuzzily
- **THEN** the system SHALL link it to the block where it resolved and SHALL indicate that it moved

#### Scenario: Hidden artifact thread does not annotate visible document

- **WHEN** a scope comment belongs to an artifact other than the selected artifact
- **THEN** the system SHALL omit that thread from visible artifact conversation while keeping it reachable by selecting its exact artifact path

#### Scenario: Reviewer and agent remain distinguishable

- **WHEN** a comment thread contains both reviewer and agent entries
- **THEN** visible role labels and structural treatment SHALL distinguish them without relying on color

#### Scenario: Narrow conversation can return to anchor

- **WHEN** a selected-artifact thread follows the document in narrow flow and its anchor resolves to a block
- **THEN** the thread SHALL provide a reachable action returning the reviewer to that exact block

### Requirement: The scope's review state is visible without scrolling to it

The system SHALL show a scope's standing verdict, when it was submitted, whether its directive has been delivered to the agent, and its verdict history, without requiring the reviewer to reach the end of the document.

Delivery state comes from the directive record, while verdict history comes from the append-only verdict sidecar.

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

### Requirement: Live update without reload
The system SHALL update a rendered page's comment list, verdict state and rendered artifacts when the underlying SSE endpoint emits an event, without requiring a full page reload. It SHALL defer an artifact update while the reviewer has unsent text in a composer, indicating that the artifact has changed and applying the update once the composer is sent or dismissed. It SHALL NOT defer a review-state update.

The filesystem watcher already reports artifact changes and the event already reaches the browser; only the review state is refreshed in response. An exploration that is readable "while it is happening" cannot leave the exploration itself stale. Replacing the document under an open composer is why that refresh was limited in the first place, so the limit is replaced with a rule rather than removed.

#### Scenario: Comment added in one tab appears in another
- **WHEN** a comment is added against an artifact while a second browser tab has the same page open
- **THEN** the second tab SHALL reflect the new comment without the user reloading the page

#### Scenario: An artifact rewritten under an idle page updates
- **WHEN** an artifact changes while its page is open with no unsent composer text
- **THEN** the system SHALL render the new content without the reviewer acting

#### Scenario: An artifact rewritten under a page with unsent text offers a refresh
- **WHEN** an artifact changes while a composer holds unsent text
- **THEN** the system SHALL indicate that the artifact has changed and SHALL NOT replace the content until the composer is sent or dismissed

#### Scenario: A deferred update applies once the composer closes
- **WHEN** a composer holding unsent text is sent or dismissed after an artifact change was deferred
- **THEN** the system SHALL apply the deferred update

#### Scenario: Reading position survives an update
- **WHEN** an artifact update is applied to a page the reviewer has scrolled
- **THEN** the system SHALL preserve the reviewer's position in the document

#### Scenario: An artifact appearing for the first time is rendered
- **WHEN** a scope's page is open with no artifact on disk and an artifact is then written
- **THEN** the system SHALL render it without the reviewer reloading

### Requirement: The index identifies each scope by more than its key

The system SHALL render, for each session and change on the index, its title where one exists, its identifier, when its artifacts were last modified, how many of its comments are open, and its standing verdict.

An index of bare session ids gives a reviewer nothing to choose between. Every field named here is already on disk in the scratch, comment and verdict sidecars; the index simply does not read it.

#### Scenario: A titled session shows its title and its id

- **WHEN** the index renders a session whose scratch note has a level-one heading
- **THEN** the system SHALL show that heading as the row's primary text and the session id as secondary text on the same row

#### Scenario: A promoted session is named by the change it became

- **WHEN** the index renders a session whose note has been promoted away, leaving a redirect with no heading to title it from
- **THEN** the system SHALL show the change that exploration became as the row's primary text, and SHALL still link to the session's own page

#### Scenario: An untitled session falls back to its id

- **WHEN** the index renders a session with neither a title nor a promotion
- **THEN** the system SHALL show the session id as the row's primary text and SHALL still link to that session's page

#### Scenario: Review state is visible without opening the page

- **WHEN** the index renders a scope with open comments or a recorded verdict
- **THEN** the system SHALL show the open-comment count and the standing verdict on that scope's row

#### Scenario: The session that spoke to the reviewer last is marked

- **WHEN** the index renders sessions and one of them has the most recent directive or verdict sidecar activity
- **THEN** the system SHALL mark that session as the most recently active one, and SHALL NOT describe it as live

#### Scenario: Links are keyed on the identifier

- **WHEN** the index renders any scope
- **THEN** the link SHALL address that scope by its identifier and SHALL NOT incorporate its title

### Requirement: A scoped page is headed by its title

The system SHALL use a scope's title as the heading and document title of its page where one exists, and SHALL show the scope's identifier on the page regardless.

The identifier is what an operator pastes into `openspec-doc comment list`, so it stays visible even once a title is available to lead with.

#### Scenario: A titled session page leads with its title

- **WHEN** a browser requests the page of a session that has a title
- **THEN** the system SHALL render that title as the page's heading and SHALL also render the session id

### Requirement: The index instantiates the observatory visual system

The system SHALL render the existing session and change index through the shared observatory visual system as a dominant Changes observation field and a separate secondary Sessions instrument, while preserving every scope field, identifier-keyed link, and loading, empty, and failure distinction already required of the index. A large session inventory SHALL NOT delay visual or keyboard access to Changes. The index SHALL occupy the full viewport width through the shared shell declaration, and its observation field SHALL continue behind the registers as attenuated ground. Widening the index SHALL NOT change its register composition, proportions, separation, plate strip, semantic reading order, or narrow-width flow.

#### Scenario: Changes are the primary index subject

- **WHEN** the index returns discovered sessions and changes
- **THEN** the system SHALL present Changes before Sessions in semantic reading and keyboard order, and SHALL give Changes the dominant desktop content region

#### Scenario: The index fills the display the scope route fills

- **WHEN** the index and a scope route are rendered in turn at 2560 CSS pixels wide
- **THEN** both SHALL span the full viewport width at the same shell padding, and the index SHALL NOT centre its content inside reserved empty canvas

#### Scenario: Widening the index preserves its composition

- **WHEN** the index is rendered at 1280 and at 2560 CSS pixels wide
- **THEN** the register structure, their relative proportions, their separation, the plate strip, and the reading order SHALL be the same at both widths, and only the space the page occupies SHALL differ

#### Scenario: Real scopes retain complete index data

- **WHEN** the index renders its change and session instruments
- **THEN** it SHALL show each supplied title, exact identifier, modified time, open-comment count, verdict, and most-recently-active marker, and each primary link SHALL remain keyed by exact identifier

#### Scenario: A large session inventory does not bury changes

- **WHEN** the index contains more sessions than changes at desktop width
- **THEN** the Changes heading and change links SHALL remain available in the primary initial content region while every session and its metadata remain reachable through the secondary session instrument

#### Scenario: An untitled session does not duplicate its identifier as display text

- **WHEN** a session has neither a title nor a promotion name
- **THEN** the system SHALL use its exact identifier once as its linked identity in the operational typography role and SHALL NOT repeat the same value as a separate display-serif title

#### Scenario: Index atmosphere does not invent product state

- **WHEN** the index renders celestial framing, observation plates, or instrument labels
- **THEN** it SHALL NOT show a fake repository, activity event, lifecycle stage, review control, scope route, or status unsupported by current data

#### Scenario: Index failure remains distinct from emptiness

- **WHEN** loading the index endpoint fails
- **THEN** the observatory interface SHALL render a failure state distinct from both loading and a successfully loaded empty project

#### Scenario: Index loading remains explicit

- **WHEN** the index endpoint has not completed
- **THEN** the observatory interface SHALL expose an explicit busy loading instrument rather than an empty register or decorative-only state

#### Scenario: Empty registers remain successful results

- **WHEN** the index endpoint returns no changes, no sessions, or neither
- **THEN** each empty register SHALL be identified as successfully empty and SHALL NOT be presented as a failure

#### Scenario: Index remains complete and Changes-first at narrow width

- **WHEN** the index is rendered at 390 CSS pixels wide
- **THEN** Changes SHALL precede Sessions in one logical flow, every scope identifier, modified value, open-comment count, verdict, primary link, and most-recently-active marker SHALL remain visible and reachable, and the page SHALL have no horizontal scrolling

### Requirement: Scope review presents one complete selected artifact

The system SHALL present exactly one complete artifact as the primary document at a time, SHALL preserve that artifact's rendered block order and source-accurate commenting behavior, and SHALL make every other real artifact in the scope reachable without concatenating their contents into the same document stream.

#### Scenario: Session selects its sole scratch artifact

- **WHEN** a session scope contains one scratch artifact
- **THEN** the system SHALL select and render that complete artifact without presenting a fake multi-artifact chooser

#### Scenario: Change exposes every exact artifact path

- **WHEN** a change scope contains proposal, design, tasks, and nested spec artifacts
- **THEN** the system SHALL expose every supplied exact artifact path in its artifact navigation and SHALL render only the selected artifact in the primary document stage

#### Scenario: Selected artifact renders completely

- **WHEN** a reviewer selects an artifact containing multiple headings and blocks
- **THEN** the system SHALL render every block in that artifact in source order and SHALL NOT reduce the document to one focused requirement, block, or excerpt

#### Scenario: Selecting another artifact replaces document context

- **WHEN** a reviewer selects another exact artifact path
- **THEN** the primary document and artifact-scoped conversation SHALL change to that artifact while scope identity, counts, verdict state, and decision controls remain in place

#### Scenario: Scope with no artifact remains explicitly empty

- **WHEN** a discovered scope has no artifact on disk
- **THEN** the system SHALL render a successful empty document instrument and SHALL NOT invent an artifact selection or treat the result as a loading failure

### Requirement: Artifact selection is route-addressable and exact

The system SHALL identify a selected change artifact by its exact path in the existing scope route's `artifact` query parameter, SHALL make selection compatible with browser history and copied URLs, and SHALL fail visibly when an explicitly requested artifact does not exist.

#### Scenario: Unqualified change route selects first supplied artifact

- **WHEN** a reviewer opens a change route with no `artifact` query and the scope contains artifacts
- **THEN** the system SHALL select the first server-supplied artifact and SHALL canonically represent that selection in the route without adding a meaningless browser-history step

#### Scenario: Encoded nested artifact opens directly

- **WHEN** a reviewer opens a change route whose `artifact` query decodes to an exact nested path such as `specs/dashboard-html-views/spec.md`
- **THEN** the system SHALL select that artifact and render its complete document

#### Scenario: Browser history restores artifact selection

- **WHEN** a reviewer selects multiple artifacts and then uses browser Back or Forward
- **THEN** the system SHALL restore the artifact identified by the resulting route

#### Scenario: Copied artifact URL restores review context

- **WHEN** a reviewer opens a copied change URL containing a valid exact `artifact` query
- **THEN** the system SHALL restore that selected artifact without requiring prior client state

#### Scenario: Invalid explicit selection fails visibly

- **WHEN** a change route contains an `artifact` query that does not exactly match any supplied artifact
- **THEN** the system SHALL retain scope identity and available artifact navigation, SHALL report that the requested artifact is unavailable, and SHALL NOT silently select another artifact

#### Scenario: Live update removes selected artifact

- **WHEN** a live artifact refresh no longer contains the exact path selected by the route
- **THEN** the system SHALL preserve the requested route, report that the selected artifact is unavailable, and keep every remaining artifact reachable


### Requirement: An update the reviewer did not cause is reported where it lands

The system SHALL report, at the point on screen where it arrives, that a scope's artifact content or comment content has been replaced by an update the reviewer did not initiate, and SHALL do so without moving the reviewer's reading position and without requiring them to act on the report.

The interface already preserves reading position across a live update, which is correct and must not change. Its consequence is that a rewritten document and an unchanged one are indistinguishable: the agent edits `proposal.md`, the frame lands, the prose is different, and nothing says so. Reporting the arrival is the only channel the interface has for the difference.

The report concerns arrival, not authorship or extent. Identifying *which* passages changed requires deciding that a block in the new content is the same block as one in the old, which is the problem the anchor resolver already owns and already gets wrong in known ways. A second, disagreeing answer to it would let the interface call a block unchanged while orphaning a comment anchored to that block.

#### Scenario: An artifact rewritten under an idle page is reported

- **WHEN** an artifact changes while its page is open with no unsent composer text
- **THEN** the system SHALL render the new content, SHALL preserve the reviewer's position, and SHALL report at the document that the content was replaced

#### Scenario: A deferred update is reported when it applies

- **WHEN** an artifact update deferred behind an unsent composer is applied after that composer is sent or dismissed
- **THEN** the system SHALL report the replacement at the moment the content changes, not at the moment the change was detected

#### Scenario: The reviewer's own submission is confirmed where it appears

- **WHEN** a comment or reply the reviewer submitted is rendered after the mutation resolves
- **THEN** the system SHALL report its arrival at the thread it created or extended

#### Scenario: The report requires no dismissal

- **WHEN** the system has reported an arrival
- **THEN** the report SHALL clear on its own, SHALL NOT require the reviewer to dismiss it, and SHALL NOT obscure or intercept interaction with the content it reports on

#### Scenario: An arrival report is not a change summary

- **WHEN** an artifact is replaced
- **THEN** the system SHALL NOT assert which blocks differ, and SHALL NOT present unchanged content as changed or changed content as unchanged

### Requirement: Route and artifact navigation acquire exact coordinates

The system SHALL present index-to-scope and artifact-selection navigation as acquisition of the exact identifier or artifact path being selected, while Vue Router remains owner of navigation and final behavior remains independent of motion support.

Index-to-scope acquisition SHALL terminate at the scope's real rendered identity and SHALL NOT terminate at a loading placeholder, a transient key, or any element that the destination discards once it has loaded. A gesture that lands on scaffolding reports the arrival of something the reviewer did not select. Acquisition SHALL be symmetric: returning to the index, whether by its link or by browser history, SHALL be the same gesture in the other direction rather than an unanimated cut, and route transitions SHALL therefore be owned once at the Router rather than at individual links, which is what leaves history navigation uncovered.

#### Scenario: Index selection acquires a scope coordinate

- **WHEN** a reviewer follows a scope link from the index
- **THEN** the acquisition response SHALL use that scope's exact identifier, SHALL terminate at the scope's real rendered identity, the Router SHALL own the navigation and reach its identifier-keyed route, and the destination SHALL remain correct when motion is unavailable

#### Scenario: A held navigation still reaches its route

- **WHEN** index-to-scope navigation is held so the gesture can terminate at the loaded scope identity, and the scope is slow to load, fails to load, or motion is unavailable
- **THEN** the Router SHALL still reach the identifier-keyed route, the index SHALL remain interactive until it does, a failure SHALL be reported by the scope's own failure state, and no reviewer action SHALL go unanswered

#### Scenario: Returning to the index acquires the coordinate in reverse

- **WHEN** a reviewer returns to the index from a scope by its index link
- **THEN** the scope's identity SHALL transfer back to that scope's entry in the index register, the Router SHALL own the navigation, and the index SHALL render its complete register state when the gesture ends

#### Scenario: History navigation is the same gesture

- **WHEN** a reviewer moves between the index and a scope with browser Back or Forward
- **THEN** the same acquisition treatment SHALL apply as for the corresponding link, in the direction travelled, and history behavior, focus, and rendered route state SHALL be unchanged by it

#### Scenario: Artifact selection acquires an exact path

- **WHEN** a reviewer selects another artifact coordinate
- **THEN** the acquisition response SHALL connect that exact path to the selected-document stage while preserving the route's `artifact` query, complete document replacement, focus movement, and browser history behavior

#### Scenario: Artifact replacement is unaffected by route treatment

- **WHEN** a reviewer selects an artifact within a scope
- **THEN** the paired artifact coordinate transfer and the immediate root-snapshot switch that keeps two complete documents from ghosting SHALL behave exactly as before, and any page-level treatment introduced for route changes SHALL NOT apply

#### Scenario: Direct navigation needs no acquisition origin

- **WHEN** a reviewer opens a copied scope or artifact URL, reloads, or uses Back or Forward without an initiating dashboard element
- **THEN** the system SHALL render the exact route state without requiring a source animation node or transient navigation state

#### Scenario: Invalid artifact coordinate remains visible

- **WHEN** an explicitly requested artifact path does not exist
- **THEN** acquisition treatment SHALL NOT hide or replace the required unavailable state and remaining exact-path navigation

### Requirement: Source and conversation navigation triangulates linked review content

The system SHALL mark the origin and exact destination of bidirectional navigation between a resolved source block and its artifact-scoped thread, using bounded direction treatment that does not cross or obscure primary content.

#### Scenario: Marker activation triangulates its thread

- **WHEN** a reviewer activates a source-block marker
- **THEN** the source origin SHALL receive an acquisition indication before navigation and the exact thread SHALL receive the corresponding destination lock after focus or reading position moves

#### Scenario: Thread activation triangulates its source

- **WHEN** a reviewer activates a thread's source action
- **THEN** the thread origin SHALL receive an acquisition indication and the exact resolved block SHALL receive the corresponding destination lock after focus or reading position moves

#### Scenario: Narrow triangulation remains local

- **WHEN** source and conversation are separated in the 390 CSS pixel reading flow
- **THEN** triangulation SHALL use local origin, destination, or reading-direction marks rather than drawing a persistent connector through intervening content

#### Scenario: Triangulation remains understandable without motion

- **WHEN** reduced motion is requested or transitions are unsupported
- **THEN** focus movement, active block treatment, active thread treatment, accessible names, and control relationships SHALL still identify both ends of the navigation

### Requirement: Reviewer mutations expose truthful transmission and receipt

The system SHALL report comment, reply, status, and verdict mutations as transmissions tied to their actual request lifetime and SHALL mark the real destination after success. It SHALL preserve explicit failure output and SHALL NOT animate success optimistically.

#### Scenario: Comment submission terminates at created thread

- **WHEN** a reviewer submits a new anchored comment and the mutation succeeds
- **THEN** transmitting state SHALL end and receipt SHALL be reported at the thread created from the server response

#### Scenario: Reply submission terminates at extended thread

- **WHEN** a reviewer submits a reply and the mutation succeeds
- **THEN** transmitting state SHALL end and receipt SHALL be reported at the thread that contains the new reply

#### Scenario: Status submission terminates at changed state

- **WHEN** a reviewer resolves or reopens a comment and the mutation succeeds
- **THEN** transmitting state SHALL end and the affected thread's real semantic state SHALL receive the resolve response

#### Scenario: Verdict submission terminates at standing verdict

- **WHEN** a reviewer submits a scope verdict and the mutation succeeds
- **THEN** transmitting state SHALL end and receipt SHALL be reported at the refreshed standing-verdict instrument

#### Scenario: Failed transmission reports failure instead of receipt

- **WHEN** any reviewer mutation fails
- **THEN** transmitting state SHALL end, no successful receipt treatment SHALL run, and the existing explicit action failure SHALL remain visible

#### Scenario: Busy state is named accessibly

- **WHEN** a reviewer mutation remains unresolved
- **THEN** the submitting surface SHALL expose text or status semantics naming that transmission rather than relying only on disabled controls or decorative motion

### Requirement: Live updates receive content where it lands

The system SHALL give remote artifact and review-state updates bounded receive treatment at the surface whose content changed, while preserving existing dirty-composer deferral, reading position, and arrival-report behavior.

#### Scenario: Remote artifact replacement marks document stage

- **WHEN** an artifact update is applied to an idle page
- **THEN** the complete new content SHALL be available immediately and the selected-document stage SHALL receive a non-interactive replacement mark without identifying individual blocks as changed

#### Scenario: Deferred artifact replacement marks application time

- **WHEN** an artifact update held behind unsent composer text is later applied
- **THEN** receive treatment SHALL begin when the document is replaced rather than when the deferred event was first detected

#### Scenario: Remote review state marks known destination only

- **WHEN** a live review-state refresh adds or changes a known thread, status, verdict, or delivery state
- **THEN** receive treatment MAY mark that known destination and SHALL NOT claim unrelated document content changed

### Requirement: Instrument chrome reconfigures without changing interaction contracts

The system SHALL use hard-edged chassis transitions for conversation-rail and decision-surface reconfiguration while preserving document measure, modal semantics, focus behavior, safe-area clearance, and final content reachability.

#### Scenario: Conversation rail retracts without document reflow

- **WHEN** a reviewer collapses or expands the conversation rail
- **THEN** chassis treatment SHALL settle within the standard motion envelope, the selected document's line breaks SHALL remain unchanged, and the final toggle state SHALL remain keyboard operable

#### Scenario: Decision surface deploys from its trigger

- **WHEN** a reviewer opens the decision drawer
- **THEN** hard-edged deployment treatment MAY connect trigger and drawer, while the drawer still opens at its own beginning, traps focus, exposes its heading, and leaves content unobscured after dismissal

#### Scenario: Decision surface retracts to its trigger

- **WHEN** a reviewer dismisses the decision drawer by its close control, backdrop, or Escape
- **THEN** dismissal treatment SHALL not delay removal of the modal obstruction beyond the standard motion envelope and focus SHALL return to the persistent trigger

### Requirement: Approval control on change pages
The system SHALL render, on a change page, a control submitting the `approved` verdict, and a control withdrawing approval when the change is currently approved.

#### Scenario: Approve control renders when feedback is settled
- **WHEN** a browser requests a change page whose every comment is `resolved`, or which has no comments, and which is not currently approved
- **THEN** the system SHALL render an approve control

#### Scenario: Approve control is refused while feedback is outstanding
- **WHEN** a reviewer submits the approve control for a change with `open` or `addressed` comments
- **THEN** the system SHALL refuse the verdict and SHALL render why, rather than recording an approval

#### Scenario: Withdraw control renders for an approved change
- **WHEN** a browser requests a change page for a currently approved change
- **THEN** the system SHALL render a control withdrawing that approval

### Requirement: Resolve-all-and-approve control on change pages
The system SHALL render, on a change page holding comments that are `open` or `addressed`, a control that resolves all of them and approves the change in one submission, and SHALL render it instead of the plain approve control rather than alongside it. The control SHALL state, before it is submitted, how many comments it will resolve and in which statuses, because it is the one control in the interface that discards the reviewer's own outstanding feedback.

#### Scenario: The control renders only while feedback is outstanding
- **WHEN** a browser requests a change page with at least one `open` or `addressed` comment
- **THEN** the system SHALL render the resolve-all-and-approve control and SHALL NOT render the plain approve control

#### Scenario: A settled change offers plain approve only
- **WHEN** a browser requests a change page whose every comment is `resolved`, or which has none
- **THEN** the system SHALL NOT render the resolve-all-and-approve control

#### Scenario: The control names what it will resolve
- **WHEN** the resolve-all-and-approve control renders for a change with comments outstanding
- **THEN** it SHALL show how many comments it will resolve, broken down by `open` and `addressed`

#### Scenario: Submission settles the page
- **WHEN** a reviewer submits the resolve-all-and-approve control
- **THEN** the page SHALL show every comment as `resolved`, the counts updated, and the change as approved

#### Scenario: A partial failure is rendered, not swallowed
- **WHEN** the submission resolves the comments and the approval then fails
- **THEN** the system SHALL render that the comments were resolved and the change was not approved, with the reason

### Requirement: Approval state rendered on change pages
The system SHALL render a change's current approval state — approved, stale, or not approved — with its reason, on the change page.

#### Scenario: Approval state renders
- **WHEN** a browser requests a change page
- **THEN** the system SHALL render whether the change is approved, stale, or not approved, and the reason

#### Scenario: A stale approval renders as stale, not approved
- **WHEN** a change page is requested for a change whose reviewed artifacts changed after approval
- **THEN** the system SHALL render it as stale and SHALL name the artifacts that changed

#### Scenario: Approval state updates without a reload
- **WHEN** a change's approval state changes while a browser tab has its page open
- **THEN** the tab SHALL reflect the new state without the user reloading the page
