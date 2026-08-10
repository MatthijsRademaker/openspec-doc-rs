## ADDED Requirements

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

## MODIFIED Requirements

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
