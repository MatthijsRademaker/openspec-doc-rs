## MODIFIED Requirements

### Requirement: Motion explains state without becoming spectacle

The system SHALL express dashboard motion as a short event-driven instrument language for acquiring coordinates, triangulating linked content, transmitting reviewer intent, receiving content, resolving semantic state, and reconfiguring interface chrome. Motion SHALL be tied to real navigation, mutation, state, or remote-update events; SHALL complete standard transitions within 100–200 milliseconds; and SHALL complete coordinate acquisition, the sole exception to that bound, within 600 milliseconds, of which the coordinate transfer and the source's rise SHALL share one shorter duration and the identity's halftone resolve MAY take the whole.

Motion MAY hold a navigation before it commits, where holding it is what lets the gesture terminate at a real destination instead of at scaffolding. A hold SHALL leave the origin fully rendered and interactive for its whole duration, SHALL end at a declared ceiling after which the navigation commits without the gesture, and SHALL commit the navigation immediately if what it waits for fails. Motion SHALL NOT hold a navigation for decoration that would look the same without the wait, SHALL NOT hold one without a declared end, and SHALL NOT leave a reviewer's action unanswered. The system SHALL NOT use decorative perpetual motion, parallax, pointer-following fields, cursor trails, sound, whole-page glitch effects, spring spectacle, or animated loading ornament.

A blanket rule that motion may never delay content reads as discipline and is not. It was written against decoration that makes a reviewer wait for nothing, but it also forbids the one case where waiting is the difference between a gesture that lands on the thing the reviewer selected and one that lands on a placeholder — and the dashboard shipped the placeholder version under it. What matters is not whether a wait exists but whether it is bounded, whether the reviewer is stranded during it, and whether anything is actually withheld: an origin page that stays live withholds nothing.

Motion MAY use opacity, slight position, registration-line growth, hard masks, clipping, halftone reveals in the plates' own texture, or glyph and line-geometry changes when those effects explain the event and leave no decorative movement after settling. A mark reporting an arrival MAY remain visible longer than a transition takes to complete, and SHALL decay on its own rather than persisting as a state the interface must later clear. The duration bound governs how long a transition takes to finish; a bounded hold before a navigation commits is governed by its own declared ceiling, and neither requires a nonblocking notice to vanish before it has been seen.

#### Scenario: Navigation acquires a real coordinate

- **WHEN** a reviewer navigates to a scope, artifact, or linked thread destination
- **THEN** any acquisition motion SHALL originate from a real selected coordinate or control and SHALL terminate at the real destination rather than at a loading placeholder, and route, history, focus, and content updates SHALL NOT be held back to let decoration finish once the navigation has committed

#### Scenario: A navigation hold is bounded and answers the action

- **WHEN** a navigation is held so its gesture can terminate at a destination that does not exist yet
- **THEN** the origin SHALL remain rendered and interactive throughout, the hold SHALL end at a declared ceiling after which the route commits without the gesture, a failed load SHALL commit the route so the destination reports its own failure, and what the hold waited for SHALL be handed to the destination rather than fetched again

#### Scenario: Transmission follows real request state

- **WHEN** a reviewer submits a comment, reply, status change, or verdict
- **THEN** transmission treatment SHALL remain in flight only while the real mutation is unresolved, SHALL terminate at the actual successful destination, and SHALL NOT present successful receipt when the mutation fails

#### Scenario: Semantic state remains immediate

- **WHEN** an open, addressed, resolved, verdict, or delivery state changes
- **THEN** visible text and final non-color geometry SHALL update from real state without waiting for its decorative transition to complete

#### Scenario: Remote content receives a bounded arrival mark

- **WHEN** content arrives through a live update the reviewer did not initiate
- **THEN** receive treatment MAY mark the surface where replacement lands but SHALL NOT obscure content, intercept interaction, move reading position, or claim which document blocks changed

#### Scenario: Standard motion is bounded

- **WHEN** a panel, route, review state, or instrument event enters or changes
- **THEN** each standard transition SHALL complete within 100–200 milliseconds and SHALL leave the interface still after the event settles

#### Scenario: Acquiring a coordinate is bounded above the standard transition

- **WHEN** a reviewer selects an artifact coordinate, the coordinate transfers from the rail to the document header, and the document reports its own arrival
- **THEN** the transfer and the source's rise SHALL run one duration so the movement reads as a single gesture, the identity's halftone resolve MAY outlast them because it is watched rather than read through, no part SHALL exceed 600 milliseconds, this SHALL be the only motion granted more than the standard bound, and the channel that gates them SHALL dwell no less than the longest of them so the reviewer sees every gesture the interface declares

#### Scenario: A suspended-input window is bounded and costs no reviewer action

- **WHEN** a coordinate transfer runs, replacing the live document with a snapshot so that the interface cannot be clicked while it does
- **THEN** that window SHALL last no longer than the acquisition duration, SHALL never exceed it by growing a treatment the snapshot does not need, and SHALL cost only pointer targeting: by the time it opens, the reviewer's originating action SHALL have taken effect, and route, history, focus, semantic text, and the selected content SHALL already have updated — for a navigation this means the window opens after the route has committed, never during a hold that precedes it

#### Scenario: Acquired identity resolves in the plate's own texture

- **WHEN** the acquisition gesture reports an artifact or scope whose source has replaced the previous one
- **THEN** the identity SHALL resolve through the halftone the observatory plates are built from, advancing in discrete passes rather than as a continuous fade, the dot grid SHALL re-register between passes so the name breaks up differently each time rather than one fixed screen filling in, the resolve SHALL start deep enough that the name reads as texture rather than as text behind a screen, and it SHALL NOT be reported instead by lit bars travelling across the header or across an index entry, which describe nothing about the surface they cross

#### Scenario: Acquired source rises into a fixed frame

- **WHEN** the identity resolves
- **THEN** the source blocks MAY rise into place from a displacement bounded by the shared displacement token, the identity and its plate SHALL stay fixed as the frame that displacement resolves against, the source SHALL reach full legibility within the standard transition bound rather than the gesture's, and neither mask nor offset SHALL survive the gesture

#### Scenario: A resolving surface is eaten, never withheld

- **WHEN** either surface is part-way through the gesture
- **THEN** it SHALL already be rendered at partial coverage rather than absent, its legibility SHALL only improve as the gesture runs, and the reviewer SHALL be able to read, select, and comment on it throughout

#### Scenario: An arrival mark decays without blocking

- **WHEN** the interface marks content that arrived without the reviewer acting
- **THEN** the mark SHALL fade out on its own, SHALL leave no residual state once it has, and SHALL NOT intercept interaction or delay access to the content it marks

#### Scenario: Loading is not decorated

- **WHEN** the interface is waiting for a scope, an index, or a mutation to resolve
- **THEN** it SHALL report the wait in text and SHALL NOT present an animated loading ornament

#### Scenario: Narrow motion preserves content order

- **WHEN** an event occurs at 390 CSS pixels wide and its desktop source and destination no longer occupy adjacent planes
- **THEN** the system SHALL use local or reading-direction treatment and SHALL NOT draw a connector across document content, reorder content, create horizontal overflow, or hide controls

#### Scenario: Reduced motion is immediate

- **WHEN** the reviewer requests reduced motion
- **THEN** native view transitions, smooth scrolling, transforms, masks, clipping transitions, decorative displacement, and navigation holds SHALL be removed or made immediate, while text, focus, semantic state, and final geometry remain understandable

#### Scenario: Reduced motion covers effects added later

- **WHEN** a new transition or animation is added to the interface
- **THEN** it SHALL be suppressed under reduced motion without requiring that it be listed individually, and any effect that must survive reduced motion SHALL opt out explicitly

#### Scenario: Celestial artwork remains still

- **WHEN** orbital, face, stipple, scan-line, or star-system artwork frames an interaction
- **THEN** the artwork SHALL remain static unless a bounded non-interactive overlay reports a real event, SHALL NOT be displaced by pointer position, and SHALL NOT animate perpetually

## ADDED Requirements

### Requirement: Observation field artwork remains static

The shared observation field and its artifact-arrival placement SHALL render only static decorative artwork. Neither placement SHALL contain a pointer-responsive eyelid, iris, pupil, gaze overlay, plate drift, or script-written bearing. Artwork SHALL remain outside the accessibility tree, reject pointer interaction, and yield to review content and responsive cropping.

#### Scenario: Pointer movement leaves observation artwork unchanged

- **WHEN** a reviewer moves a fine pointer across the index or a wide artifact route
- **THEN** the observation image and its crop SHALL remain at their rest geometry, no `--field-gaze-*` value or equivalent bearing SHALL be written, and no gaze overlay SHALL be rendered

#### Scenario: Both placements use the static source image

- **WHEN** the index field or artifact arrival artwork is visible
- **THEN** it SHALL render the shared static observation image with its existing responsive crop and SHALL contain no `ObservationGaze` or equivalent pointer-following component

#### Scenario: Static artwork stays decorative and subordinate

- **WHEN** observation artwork appears beside review content
- **THEN** it SHALL be `aria-hidden`, have empty alternative text, reject pointer interaction, make no external request, and SHALL NOT obscure text, controls, focus indicators, paths, comments, or semantic state

## REMOVED Requirements

### Requirement: The observation field carries a bounded human gaze

**Reason**: The pointer-responsive eyelid/iris concept adds visual distraction without communicating review state and is being removed before it becomes a dashboard contract.

**Migration**: Delete the `ObservationGaze` component, pointer tracker, plate drift, gaze-specific styles, and gaze-specific browser/unit coverage. Keep the underlying static observation image and its responsive accessibility behavior.
