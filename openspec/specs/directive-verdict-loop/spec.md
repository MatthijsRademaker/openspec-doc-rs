# directive-verdict-loop Specification

## Purpose
TBD - created by archiving change add-directive-verdict-loop. Update Purpose after archive.
## Requirements
### Requirement: Promotion check on every hook stop invocation
The system SHALL run the scratch-note promotion check for the session before directive lookup on every `hook stop` invocation.

#### Scenario: Promotion runs before directive lookup
- **WHEN** `openspec-doc hook stop` is invoked for a session with a pending scratch note
- **THEN** the system SHALL run the promotion check first, recording any validate outcome, before proceeding to directive lookup

### Requirement: Session registration on every hook stop invocation
The system SHALL ensure a session has a directive record on every `hook stop` invocation, so the dashboard can discover the session before any directive has been written for it.

Without this the loop has no entry point: a session is discoverable because it has a directive record, a directive is produced from a verdict, and a verdict is submitted from the session's own page.

#### Scenario: A session that has never been sent a directive is still discoverable
- **WHEN** `openspec-doc hook stop` is invoked for a session with no directive record
- **THEN** the system SHALL write an empty directive record for that session, and SHALL NOT produce a block decision from it

#### Scenario: Registration leaves an existing record untouched
- **WHEN** `openspec-doc hook stop` is invoked for a session that already has a directive record
- **THEN** the system SHALL leave that record exactly as it stands, including a directive still waiting to be injected

### Requirement: Phase-verdict to directive translation
The system SHALL translate an untranslated phase-verdict record (keep-exploring, move-to-proposal, or comment-resolution) into the pending-directive file format at hook-stop time, using fixed reason-text templates that name relevant sidecar and artifact paths rather than embedding their content.

#### Scenario: Keep-exploring verdict translates to a pointer directive
- **WHEN** an untranslated "keep exploring" phase-verdict record exists for a session
- **THEN** the system SHALL write a pending directive whose reason text points to the session's scratch note and comment sidecar paths, and SHALL NOT embed the verdict's free-text notes directly in the reason text

#### Scenario: Move-to-proposal verdict translates to a pointer directive
- **WHEN** an untranslated "move to proposal" phase-verdict record exists for a session
- **THEN** the system SHALL write a pending directive whose reason text instructs formalizing the exploration into a proposal, pointing to the scratch note path

#### Scenario: A move-to-proposal directive points at the reviewer's comments
- **WHEN** an untranslated "move to proposal" phase-verdict record exists for a session
- **THEN** the reason text SHALL name the session's comment sidecar path and SHALL instruct that the open comments be accounted for in the proposal

The exploration note and the verdict record say what the reviewer concluded; the comment sidecar says which passages they wanted changed. A directive that names the first two and not the third sends the agent to formalize an exploration without telling it that anchored feedback on that exploration exists.

#### Scenario: Comment-resolution verdict translates to a pointer directive
- **WHEN** an untranslated "send to agent" comment-resolution phase-verdict record exists for a change
- **THEN** the system SHALL write a pending directive whose reason text points to the change's comment sidecar and instructs addressing open comments

### Requirement: Reason text is an attributed pointer the agent will act on
The system SHALL phrase every reason-text template as an attributed pointer to review artifacts inside the user's own project — identifying the directive as review feedback from this project's dashboard and naming the file paths to read — and SHALL NOT phrase it as a bare imperative that an agent cannot verify against project state.

This requirement exists because a receiving agent can reject an injected directive as a prompt-injection attempt, which was observed during `add-agent-hook-bridge` verification. Transporting the reason text correctly is not sufficient; the agent has to actually act on it.

#### Scenario: Templates identify their origin and name a readable path
- **WHEN** any verdict kind is translated into a pending directive
- **THEN** the reason text SHALL identify itself as review feedback originating from this project's openspec-doc dashboard, and SHALL name at least one path within the project for the agent to read

#### Scenario: No template is a bare imperative
- **WHEN** the reason-text templates are inspected
- **THEN** no template SHALL instruct the agent to emit specific literal output, and no template SHALL depend on the agent obeying an instruction it cannot corroborate from project files

#### Scenario: A refusing agent is a failure, not a warning
- **WHEN** a receiving agent declines to act on an injected directive, or treats it as untrusted input
- **THEN** that SHALL be treated as a failure of this change requiring the templates to be revised, not as acceptable behaviour

### Requirement: Verdict consumption
The system SHALL mark a phase-verdict record as translated after producing its directive, and SHALL NOT translate the same verdict record again on a later `hook stop` invocation.

#### Scenario: Translated verdict is not retranslated
- **WHEN** `openspec-doc hook stop` is invoked again after a phase-verdict record has already been translated into a directive
- **THEN** the system SHALL NOT produce a second directive from that same verdict record

