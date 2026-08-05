## ADDED Requirements

### Requirement: Explore detection readies the note's location without creating the note
The system SHALL provide a hook command that, when an exploration starts for a session, ensures the directory holding that session's scratch note exists, and SHALL NOT create the note file itself.

Writing the exploration down is the agent's work. An empty note left in its way turns the agent's first write into a failure, because Claude Code refuses to write over a file it has not read — observed live, where the agent's first `Write` of its exploration note failed for exactly this reason. An empty note is also worth nothing to the reviewer, so nothing is lost by leaving the file to the exploration that fills it.

#### Scenario: The note has somewhere to be written
- **WHEN** the explore hook command is invoked for a session
- **THEN** the directory holding that session's scratch note SHALL exist

#### Scenario: No empty note is left in the agent's way
- **WHEN** the explore hook command is invoked for a session that has not yet written an exploration
- **THEN** no scratch note file SHALL exist for that session

#### Scenario: An exploration already under way keeps what it has written
- **WHEN** the explore hook command is invoked for a session whose scratch note already holds content
- **THEN** the system SHALL leave that content unchanged

### Requirement: A note exists only where an exploration was written
The system SHALL NOT create a session's scratch note on any event.

Promotion treats a session with no note as having nothing to promote. If notes appeared for sessions that never explored, every open session would promote on the first new change directory, and concurrent sessions would rename their notes onto the same change-keyed path.

#### Scenario: An ordinary turn boundary creates no note
- **WHEN** `openspec-doc hook stop` is invoked for a session that has not written an exploration
- **THEN** the system SHALL NOT create a scratch note for that session

### Requirement: The agent is told where its exploration note lives
The system SHALL emit, on the explore hook command's standard output, the resolved path of the session's scratch note together with an instruction to keep the exploration written there, in the form the agent's hook contract adds to model context.

The path SHALL be emitted fully resolved rather than as a template for the agent to interpolate, so the agent cannot silently write to the wrong file.

#### Scenario: The emitted instruction names the resolved note path
- **WHEN** the explore hook command runs for a session
- **THEN** its standard output SHALL contain that session's scratch note path with the session id already substituted

#### Scenario: The instruction explains what the note is for
- **WHEN** the explore hook command runs for a session
- **THEN** its standard output SHALL state that the note is what the reviewer reads and anchors comments against

### Requirement: Explore detection matches the command by name
The system SHALL be wired to the agent's command-expansion event using a matcher on the explore command's name, and SHALL NOT detect exploration by scanning prompt text.

A prompt-text scan fires on any message that merely mentions the command, including one asking a question about it.

#### Scenario: The configured matcher fires on a real explore invocation
- **WHEN** a user types the explore command in a live session with the hook configured
- **THEN** the hook SHALL run and the session's scratch note SHALL exist afterwards

#### Scenario: A matcher that never fires is a failure, not a silent no-op
- **WHEN** the configured matcher does not match the explore command
- **THEN** that SHALL be treated as a failure of this change, since the resulting empty session page is indistinguishable from the defect being fixed
