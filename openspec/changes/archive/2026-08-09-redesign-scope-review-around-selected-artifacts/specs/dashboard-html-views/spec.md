## ADDED Requirements

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

## MODIFIED Requirements

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

### Requirement: Scope pages instantiate the observatory review workbench

The system SHALL compose each session and change route from one shared observatory workbench as compact route/state instrumentation, exact artifact navigation, one primary selected-document stage, artifact-scoped conversation, and persistent decision controls outside the document's reading plane. Authored celestial artwork SHALL form part of the selected document's bounded arrival composition without obscuring document text, state, focus, comments, composers, or controls.

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

#### Scenario: Desktop decision instrument leaves document unobstructed

- **WHEN** the persistent decision instrument is closed or open at desktop width
- **THEN** its trigger and drawer SHALL remain in the conversation-side instrument region and SHALL NOT cover selected-document prose, artifact navigation, block comment actions, thread actions, or focus indicators

#### Scenario: Narrow decision dock preserves final content

- **WHEN** the decision instrument is rendered at 390 CSS pixels wide
- **THEN** the final selected-document block, artifact-scoped thread action, active composer, and decision controls SHALL all be scrollable fully clear of the dock and safe-area inset

#### Scenario: Artwork yields before review content

- **WHEN** available width cannot preserve document-stage artwork or peripheral observation cells alongside review content
- **THEN** peripheral cells SHALL disappear and main artwork SHALL shrink, crop, or disappear before any artifact path, document text, state, thread, composer, or control is hidden or clipped

#### Scenario: Scope chrome reports only real state

- **WHEN** a scope page renders image-led instrument chrome inspired by the visual source
- **THEN** it SHALL NOT present repository metadata, validation results, activity events, lifecycle phases, navigation destinations, or review state absent from the scope or index APIs

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
