## MODIFIED Requirements

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

## ADDED Requirements

### Requirement: The agent is told where its note lives on every harness
The system SHALL deliver the explore command's emitted instruction into the model's context on every supported harness, using that harness's own mechanism for adding context, so the agent receives the resolved note path rather than constructing one.

Where a harness has no context contract for the event, the system SHALL inject the instruction as a message the way that harness's directive delivery already does.

#### Scenario: The instruction reaches the model on a harness with no hook contract
- **WHEN** an exploration is detected on a harness whose hook bridge is an in-process extension
- **THEN** the explore command's standard output SHALL be delivered to the model as a displayed message

#### Scenario: A failed explore command does not block the user's input
- **WHEN** the explore command cannot be run, or exits non-zero
- **THEN** the failure SHALL be surfaced to the user and the submitted input SHALL proceed unchanged
