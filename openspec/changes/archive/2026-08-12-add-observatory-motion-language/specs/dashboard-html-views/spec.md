## ADDED Requirements

### Requirement: Route and artifact navigation acquire exact coordinates

The system SHALL present index-to-scope and artifact-selection navigation as acquisition of the exact identifier or artifact path being selected, while Vue Router remains owner of navigation and final behavior remains independent of motion support.

#### Scenario: Index selection acquires a scope coordinate

- **WHEN** a reviewer follows a scope link from the index
- **THEN** the acquisition response SHALL use that scope's exact identifier, the Router SHALL navigate to its identifier-keyed route immediately, and the destination SHALL remain correct when motion is unavailable

#### Scenario: Artifact selection acquires an exact path

- **WHEN** a reviewer selects another artifact coordinate
- **THEN** the acquisition response SHALL connect that exact path to the selected-document stage while preserving the route's `artifact` query, complete document replacement, focus movement, and browser history behavior

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
