# explore-note-capture Specification

## Purpose
TBD - created by archiving change add-explore-note-capture. Update Purpose after archive.
## Requirements
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

### Requirement: Explore detection matches the command by name
The system SHALL be wired to explore detection on every supported coding harness, and SHALL identify an exploration by the explore command's name rather than by scanning prompt text for a mention of it.

A prompt-text scan fires on any message that merely mentions the command, including one asking a question about it.

Where the harness provides a command-expansion event, the system SHALL use a matcher on the command's name at that event. Where the harness provides no such event, the system SHALL match the command name at the start of the raw submitted input, before that input is expanded, and SHALL NOT match against expanded prompt or template text — expansion is owned by the harness or by an upstream template and can be rewritten without notice, which would stop detection silently.

The match SHALL cover every way an exploration is started on that harness, including both a prompt template and a skill where the harness offers both.

The system SHALL NOT register the explore command as an extension-provided command on a harness that dispatches such commands ahead of its raw-input event, because doing so shadows the prompt template carrying the exploration instructions.

#### Scenario: The configured matcher fires on a real explore invocation
- **WHEN** a user types the explore command in a live session with the hook configured
- **THEN** the hook SHALL run and the session's scratch note SHALL exist afterwards

#### Scenario: A matcher that never fires is a failure, not a silent no-op
- **WHEN** the configured matcher does not match the explore command
- **THEN** that SHALL be treated as a failure of this change, since the resulting empty session page is indistinguishable from the defect being fixed

#### Scenario: Every supported harness captures its explorations
- **WHEN** an exploration is started on any harness the system configures
- **THEN** that session SHALL produce a scratch note and SHALL be registered for review, on the same terms as every other supported harness

#### Scenario: Each of a harness's explore entry points is matched
- **WHEN** a harness offers both a prompt template and a skill that start an exploration
- **THEN** invoking either SHALL be detected

#### Scenario: A message about the command is not an exploration
- **WHEN** submitted input mentions the explore command without invoking it as the command
- **THEN** no exploration SHALL be started and no scratch note directory SHALL be readied

### Requirement: The agent is told where its note lives on every harness
The system SHALL deliver the explore command's emitted instruction into the model's context on every supported harness, using that harness's own mechanism for adding context, so the agent receives the resolved note path rather than constructing one.

Where a harness has no context contract for the event, the system SHALL inject the instruction as a message the way that harness's directive delivery already does.

#### Scenario: The instruction reaches the model on a harness with no hook contract
- **WHEN** an exploration is detected on a harness whose hook bridge is an in-process extension
- **THEN** the explore command's standard output SHALL be delivered to the model as a displayed message

#### Scenario: A failed explore command does not block the user's input
- **WHEN** the explore command cannot be run, or exits non-zero
- **THEN** the failure SHALL be surfaced to the user and the submitted input SHALL proceed unchanged

