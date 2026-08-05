## MODIFIED Requirements

### Requirement: A note exists only where an exploration was written
The system SHALL NOT create a session's scratch note on any event.

A session's claim on a change is recorded inside its scratch note, so a session with no note has no claim and nothing to promote. Notes appearing for sessions that never explored would put an empty, unclaimable note on the dashboard for every open session.

#### Scenario: An ordinary turn boundary creates no note
- **WHEN** `openspec-doc hook stop` is invoked for a session that has not written an exploration
- **THEN** the system SHALL NOT create a scratch note for that session

### Requirement: The agent is told where its exploration note lives
The system SHALL emit, on the explore hook command's standard output, the resolved path of the session's scratch note together with an instruction to keep the exploration written there, in the form the agent's hook contract adds to model context.

The path SHALL be emitted fully resolved rather than as a template for the agent to interpolate, so the agent cannot silently write to the wrong file.

The instruction SHALL name the claim command with this session's id already resolved, and SHALL NOT describe promotion as following from a change directory appearing. Nothing but a claim promotes the note, so an instruction that promises otherwise describes a mechanism the tool does not have.

#### Scenario: The emitted instruction names the resolved note path
- **WHEN** the explore hook command runs for a session
- **THEN** its standard output SHALL contain that session's scratch note path with the session id already substituted

#### Scenario: The instruction explains what the note is for
- **WHEN** the explore hook command runs for a session
- **THEN** its standard output SHALL state that the note is what the reviewer reads and anchors comments against

#### Scenario: The instruction names the claim command
- **WHEN** the explore hook command runs for a session
- **THEN** its standard output SHALL name the claim command with that session's id already resolved
