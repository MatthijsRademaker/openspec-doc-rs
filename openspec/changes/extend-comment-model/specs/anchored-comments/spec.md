## ADDED Requirements

### Requirement: A comment may be scoped to a session or change rather than anchored to text
The system SHALL allow a comment to be recorded against a session or a change with no anchor, and SHALL store it in the same sidecar as anchored comments.

Not all feedback is about a passage. Before this, unanchored feedback could only be carried in a phase verdict's notes field, which made it a third kind of text — neither attached to anything nor delivered through the comment sidecar that directives point the agent at.

#### Scenario: An unanchored comment is recorded
- **WHEN** a comment is submitted for a scope with no selected text
- **THEN** the system SHALL record it in that scope's comment sidecar with no anchor

#### Scenario: An unanchored comment needs no artifact
- **WHEN** an unanchored comment is submitted for a scope that has no artifacts on disk
- **THEN** the system SHALL record it, rather than failing for want of something to anchor against

#### Scenario: An unanchored comment is not reported as drifted
- **WHEN** the anchor state of an unanchored comment is resolved
- **THEN** the system SHALL report it as unanchored, distinctly from the orphaned state that means an anchor was lost

### Requirement: A comment's body can be edited
The system SHALL allow the body of an existing comment to be replaced, SHALL record the replacement as an appended event, and SHALL report the most recent body when the sidecar is read.

A reviewer who mistypes otherwise has to add a second comment correcting the first, leaving the agent two instructions and no way to tell which one stands.

#### Scenario: An edited comment reads back with its new body
- **WHEN** a comment's body is edited
- **THEN** reading the sidecar SHALL report the new body for that comment

#### Scenario: The earlier body is not removed from the file
- **WHEN** a comment's body is edited
- **THEN** the sidecar SHALL still contain the record that created it, unmodified

#### Scenario: Editing an unknown comment fails
- **WHEN** an edit names a comment id the sidecar does not contain
- **THEN** the system SHALL fail with an error naming that id, rather than appending an event nothing will fold

## MODIFIED Requirements

### Requirement: Comment CLI subcommand
The system SHALL provide `comment add`, `comment list`, `comment reply`, `comment edit`, and `comment resolve` operations via the CLI, and `comment add` SHALL accept a body with no selected text.

The CLI is how the dashboard's behaviour is verified, so every operation the dashboard performs has to be reachable without it.

#### Scenario: Adding a comment via the CLI
- **WHEN** a user runs `openspec-doc comment add` with a target artifact, selected text, and body
- **THEN** the system SHALL create the anchor context and append a comment event to the appropriate sidecar

#### Scenario: Listing comments via the CLI
- **WHEN** a user runs `openspec-doc comment list` for a given scope key
- **THEN** the system SHALL print each comment's current status and anchor state

#### Scenario: A comment is created without a selection
- **WHEN** a user runs the comment creation subcommand with no selected text
- **THEN** the system SHALL record an unanchored comment against the named scope

#### Scenario: An unanchored comment is listed as such
- **WHEN** a user lists a scope containing an unanchored comment
- **THEN** the system SHALL show it as unanchored rather than printing an empty quotation
