## ADDED Requirements

### Requirement: The index identifies each scope by more than its key
The system SHALL render, for each session and change on the index, its title where one exists, its identifier, when its artifacts were last modified, how many of its comments are open, and its standing verdict.

An index of bare session ids gives a reviewer nothing to choose between. Every field named here is already on disk in the scratch, comment and verdict sidecars; the index simply does not read it.

#### Scenario: A titled session shows its title and its id
- **WHEN** the index renders a session whose scratch note has a level-one heading
- **THEN** the system SHALL show that heading as the row's primary text and the session id as secondary text on the same row

#### Scenario: An untitled session falls back to its id
- **WHEN** the index renders a session with no title
- **THEN** the system SHALL show the session id as the row's primary text and SHALL still link to that session's page

#### Scenario: Review state is visible without opening the page
- **WHEN** the index renders a scope with open comments or a recorded verdict
- **THEN** the system SHALL show the open-comment count and the standing verdict on that scope's row

#### Scenario: Links are keyed on the identifier
- **WHEN** the index renders any scope
- **THEN** the link SHALL address that scope by its identifier and SHALL NOT incorporate its title

### Requirement: A scoped page is headed by its title
The system SHALL use a scope's title as the heading and document title of its page where one exists, and SHALL show the scope's identifier on the page regardless.

The identifier is what an operator pastes into `openspec-doc comment list`, so it stays visible even once a title is available to lead with.

#### Scenario: A titled session page leads with its title
- **WHEN** a browser requests the page of a session that has a title
- **THEN** the system SHALL render that title as the page's heading and SHALL also render the session id
