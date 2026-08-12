## MODIFIED Requirements

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
