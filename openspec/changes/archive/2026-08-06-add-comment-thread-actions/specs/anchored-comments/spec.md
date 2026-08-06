## ADDED Requirements

### Requirement: Comment status lifecycle
The system SHALL track each comment in one of three statuses — `open`, `addressed`, or `resolved` — recorded as appended status events like any other. `addressed` asserts that work responding to the comment has been done; `resolved` asserts that the reviewer accepts it. A comment SHALL be movable back to `open` from either later status, so that a reviewer who rejects the response can reopen the thread rather than filing a duplicate.

#### Scenario: A new comment starts open
- **WHEN** a comment is created
- **THEN** the system SHALL record its status as `open`

#### Scenario: A comment is marked addressed
- **WHEN** a status update to `addressed` is recorded for an open comment
- **THEN** the system SHALL reconstruct that comment's current status as `addressed`, and SHALL retain the comment, its replies, and its prior status events

#### Scenario: A comment is resolved
- **WHEN** a status update to `resolved` is recorded for a comment
- **THEN** the system SHALL reconstruct that comment's current status as `resolved`

#### Scenario: A comment is reopened
- **WHEN** a status update to `open` is recorded for a comment currently `addressed` or `resolved`
- **THEN** the system SHALL reconstruct that comment's current status as `open`, and SHALL retain the full status history including the superseded status

### Requirement: Scope status counts
The system SHALL report, for a given scope key, how many of its comments are currently `open`, `addressed`, and `resolved`.

#### Scenario: Counts reflect current reconstructed status
- **WHEN** the counts for a scope key are requested
- **THEN** the system SHALL count each comment once under its current reconstructed status, and SHALL NOT count superseded statuses from that comment's history

#### Scenario: A scope with no sidecar counts as zero
- **WHEN** the counts are requested for a scope key with no comment sidecar
- **THEN** the system SHALL report zero in every status rather than failing

## MODIFIED Requirements

### Requirement: Comment CLI subcommand
The system SHALL provide `comment add`, `comment list`, `comment reply`, `comment address`, `comment resolve`, and `comment reopen` operations via the CLI.

#### Scenario: Adding a comment via the CLI
- **WHEN** a user runs `openspec-doc comment add` with a target artifact, selected text, and body
- **THEN** the system SHALL create the anchor context and append a comment event to the appropriate sidecar

#### Scenario: Listing comments via the CLI
- **WHEN** a user runs `openspec-doc comment list` for a given scope key
- **THEN** the system SHALL print each comment's current status and anchor state

#### Scenario: Marking a comment addressed via the CLI
- **WHEN** a user or agent runs `openspec-doc comment address` for an existing comment
- **THEN** the system SHALL append a status event moving that comment to `addressed`

#### Scenario: Reopening a comment via the CLI
- **WHEN** a user runs `openspec-doc comment reopen` for an existing comment
- **THEN** the system SHALL append a status event moving that comment back to `open`

#### Scenario: A status operation on an unknown comment fails loudly
- **WHEN** a status operation names a comment id absent from the scope's sidecar
- **THEN** the system SHALL return an error rather than appending an event for a comment that does not exist
