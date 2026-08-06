## ADDED Requirements

### Requirement: A standing directive is delivered as prompt context
The system SHALL provide a hook command that, for the session named in the payload, translates any standing untranslated verdict into a directive and emits that directive's reason text on standard output, in the form the agent's prompt-submission contract adds to model context.

Delivering at prompt time puts review feedback in the turn the reviewer's own prompt starts, rather than at the end of that turn.

#### Scenario: A verdict submitted before a prompt is in context for that prompt
- **WHEN** the prompt hook command is invoked for a session with an untranslated verdict
- **THEN** the system SHALL emit that verdict's directive reason text on standard output

#### Scenario: A directive already waiting is delivered rather than re-translated
- **WHEN** the prompt hook command is invoked for a session whose directive is still pending
- **THEN** the system SHALL emit that pending directive's reason text on standard output

#### Scenario: A session with nothing outstanding emits nothing
- **WHEN** the prompt hook command is invoked for a session with no untranslated verdict and no pending directive
- **THEN** the system SHALL emit nothing on standard output

### Requirement: A directive is delivered exactly once across both delivery points
The system SHALL mark a directive consumed when it is delivered as prompt context, and SHALL NOT deliver the same directive again at the following turn boundary.

#### Scenario: Prompt-time delivery leaves nothing for the turn boundary
- **WHEN** a directive has been delivered as prompt context and `openspec-doc hook stop` is then invoked for that session
- **THEN** the system SHALL allow the turn to end rather than blocking with the directive a second time

#### Scenario: Turn-end delivery leaves nothing for the next prompt
- **WHEN** a directive has been delivered at a turn boundary and the prompt hook command is then invoked for that session
- **THEN** the system SHALL emit nothing on standard output

### Requirement: A failure to deliver never refuses the reviewer's prompt
The system SHALL report any failure of the prompt hook command on standard error and SHALL exit zero, emitting no directive text.

This diverges deliberately from `hook stop`, which propagates errors and exits non-zero. That is safe at a turn boundary, where the worst outcome is a turn ending normally. This hook sits in front of the human's own input, where the worst outcome is a reviewer who cannot submit a prompt at all — strictly worse than the delayed feedback this change exists to fix.

#### Scenario: A malformed sidecar does not block the prompt
- **WHEN** the prompt hook command cannot read the session's verdict or directive state
- **THEN** the system SHALL exit zero, SHALL emit nothing on standard output, and SHALL report the failure on standard error

### Requirement: Promotion is not checked at prompt time
The system SHALL NOT run the scratch-note promotion check in the prompt hook command.

Promotion detects change directories that appear as a result of agent work, so a turn boundary is when there is something new to find. Running it per prompt would add an `openspec validate` subprocess to the latency of every prompt for a check that is almost never satisfiable at that moment.

#### Scenario: No validate subprocess runs for a prompt
- **WHEN** the prompt hook command is invoked for a session whose exploration has just been formalized into a change
- **THEN** the system SHALL NOT promote the scratch note and SHALL NOT run the change validator
