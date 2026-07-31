## ADDED Requirements

### Requirement: Normalized hook event ingestion
The system SHALL read an agent-specific Stop-hook payload from stdin for a given `--agent` value (`claude` or `pi`) and normalize it into an internal representation containing at least the agent kind, session id, transcript path, and working directory.

#### Scenario: Claude Code payload is normalized
- **WHEN** `openspec-doc hook stop --agent claude` receives a valid Claude Code Stop-hook JSON payload on stdin
- **THEN** the system SHALL extract the session id, transcript path, and cwd into the internal event representation

#### Scenario: pi.dev payload is normalized
- **WHEN** `openspec-doc hook stop --agent pi` receives a valid turn-end payload on stdin from the pi.dev extension
- **THEN** the system SHALL extract the session id, transcript path, and cwd into the internal event representation

#### Scenario: A payload from the wrong agent fails loudly
- **WHEN** a payload in one agent's wire format is supplied with the other agent's `--agent` value
- **THEN** the system SHALL exit with a non-zero status without emitting a decision, rather than accepting the payload

#### Scenario: Malformed stdin fails loudly
- **WHEN** stdin cannot be parsed as a valid payload for the selected `--agent`
- **THEN** the system SHALL exit with a non-zero status and print a clear error to stderr, without emitting a decision

### Requirement: Session-scoped directive lookup
The system SHALL look up a pending directive file scoped by the session id extracted from the hook event.

#### Scenario: Pending directive exists
- **WHEN** a directive file exists for the session id with `pending: true`
- **THEN** the system SHALL treat it as a pending directive and use its reason text for the block decision

#### Scenario: No directive file exists
- **WHEN** no directive file exists for the session id
- **THEN** the system SHALL treat this as no pending directive and proceed to an allow decision

### Requirement: Block decision on pending directive
The system SHALL emit the agent-specific "block and continue" response, containing the pending directive's reason text, when a pending directive exists for the session.

#### Scenario: Claude Code block response
- **WHEN** a pending directive exists and `--agent claude` was selected
- **THEN** the system SHALL emit `{"decision":"block","reason":"<directive reason>"}` on stdout

#### Scenario: pi.dev block response
- **WHEN** a pending directive exists and `--agent pi` was selected
- **THEN** the system SHALL emit `{"action":"continue","message":"<directive reason>"}` on stdout, which the pi.dev extension re-injects via `sendUserMessage`

### Requirement: Allow decision with no pending directive
The system SHALL emit the agent-specific "allow stop" response, and SHALL NOT modify any files, when no pending directive exists for the session.

#### Scenario: No directive means allow
- **WHEN** no pending directive exists for the session
- **THEN** the system SHALL emit that agent's "allow stop" response and SHALL NOT write to the directive file or any other file

### Requirement: Directive consumption
The system SHALL mark a pending directive as consumed after using it for a block decision, and a consumed directive SHALL NOT be re-injected on a subsequent Stop event for the same session.

#### Scenario: Directive is consumed after use
- **WHEN** a pending directive is used to produce a block decision
- **THEN** the system SHALL update the directive file to `pending: false` while preserving the record, rather than deleting it

#### Scenario: Consumed directive does not re-fire
- **WHEN** `openspec-doc hook stop` is invoked again for the same session after a directive was already consumed
- **THEN** the system SHALL emit an allow decision, not re-inject the consumed reason
