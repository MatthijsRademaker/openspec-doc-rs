# dashboard-html-views Specification

## Purpose

What the dashboard's pages contain: the session-scoped page rendering an exploration's scratch note, the change-scoped page rendering a change's proposal, design, tasks and spec deltas, the anchored comments shown against each, the select-to-comment interaction that creates them, and the phase-verdict controls a reviewer submits from either page. Artifacts are rendered as their own markdown source, so a browser selection is already a substring of the file the server anchors against and no offset mapping exists to get wrong. The interface is a built application — Vue and shadcn-vue under `web/`, compiled to assets embedded in the binary — and the index is served from it. The scope pages are not yet: they remain server-rendered HTML with the minimum vanilla JavaScript that selection and live updates require, until `migrate-dashboard-review-to-vue` moves them across. The server, its route table, and the event stream these pages subscribe to belong to `dashboard-server`; the comment records themselves belong to `anchored-comments`.

## Requirements

### Requirement: Session page renders scratch note and comments

The system SHALL render, at `/sessions/<session_id>`, the session's scratch note markdown and its anchored comments.

#### Scenario: Scratch note and comments render

- **WHEN** a browser requests `/sessions/<session_id>` for a session with a scratch note
- **THEN** the system SHALL render the scratch note's content and any existing anchored comments against it

### Requirement: Change page renders artifacts and comments

The system SHALL render, at `/changes/<name>`, the change's `proposal.md`, `design.md`, `tasks.md`, and spec delta files along with their anchored comments.

#### Scenario: Change artifacts and comments render

- **WHEN** a browser requests `/changes/<name>` for a discovered active change
- **THEN** the system SHALL render each present artifact's content and any existing anchored comments against it

### Requirement: Select-to-comment interaction

The system SHALL allow a reviewer to select text within a rendered artifact and submit a comment anchored to that selection, using the anchored-comments capability.

#### Scenario: Comment is created from a text selection

- **WHEN** a reviewer selects text in a rendered artifact and submits a comment
- **THEN** the system SHALL create an anchor context from the raw markdown source underlying that selection and persist the comment via the anchored-comments capability

### Requirement: Phase-verdict submission

The system SHALL allow a reviewer on a session page to submit a phase verdict of either "keep exploring" with free-text notes, or "move to proposal", and SHALL persist it as a session-scoped record distinct from per-comment status.

#### Scenario: Keep-exploring verdict is recorded with notes

- **WHEN** a reviewer submits "keep exploring" with open questions or resolutions entered as free text
- **THEN** the system SHALL persist a phase-verdict record for that session containing the verdict and the free-text content

#### Scenario: Move-to-proposal verdict is recorded

- **WHEN** a reviewer submits "move to proposal"
- **THEN** the system SHALL persist a phase-verdict record for that session containing that verdict

### Requirement: Comment-resolution verdict on change pages

The system SHALL allow a reviewer on a change page to submit a "send to agent: address open comments" verdict.

#### Scenario: Comment-resolution verdict is recorded

- **WHEN** a reviewer on a change page submits "send to agent"
- **THEN** the system SHALL persist a phase-verdict record for that change indicating open comments should be addressed

### Requirement: Live update without reload

The system SHALL update a rendered page's comment list and verdict state when the underlying SSE endpoint emits an event, without requiring a full page reload.

#### Scenario: Comment added in one tab appears in another

- **WHEN** a comment is added against an artifact while a second browser tab has the same page open
- **THEN** the second tab SHALL reflect the new comment without the user reloading the page

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

The system SHALL render the existing session and change index through the shared observatory visual system while preserving every scope field, link, and loading, empty, and failure distinction already required of the index.

#### Scenario: Real scopes render as instrument registers

- **WHEN** the index returns discovered sessions and changes
- **THEN** the system SHALL present separate session and change registers with each title, exact identifier, modified time, open-comment count, verdict, and most-recently-active marker supplied by the endpoint

#### Scenario: Index atmosphere does not invent product state

- **WHEN** the index renders celestial framing or instrument labels
- **THEN** it SHALL NOT show a fake repository, activity event, review control, scope route, or status unsupported by current data

#### Scenario: Index failure remains distinct from emptiness

- **WHEN** loading the index endpoint fails
- **THEN** the observatory interface SHALL render a failure state distinct from both loading and a successfully loaded empty project

#### Scenario: Index remains complete at narrow width

- **WHEN** the index is rendered at 390 CSS pixels wide
- **THEN** every scope identifier, modified value, open-comment count, verdict, primary link, and most-recently-active marker SHALL remain visible and reachable without horizontal page scrolling
