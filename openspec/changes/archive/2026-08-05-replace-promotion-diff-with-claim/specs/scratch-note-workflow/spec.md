## REMOVED Requirements

### Requirement: Promotion detection via snapshot diff
**Reason**: The diff establishes only that a change directory appeared, never that this session created it. With concurrent sessions it credits one session's change to every session whose snapshot predates it, silently renaming the wrong exploration and relocating its comment sidecar onto the wrong change. Observed on real state: a change created from a second session left another session one turn boundary from promoting a 29KB exploration onto it. The mechanism was chosen over asking the agent to record the change name, on an assumption about agent compliance that end-to-end verification has since disproved.

**Migration**: Replaced by *Promotion detection via explicit claim* below. Existing `.openspec-doc/scratch/_session/*.changes.json` snapshots are dead and are deleted; nothing in a snapshot identifies a change as a given session's, so there is nothing to migrate. Sessions mid-exploration claim their change when they formalize it, via `openspec-doc scratch claim`.

## ADDED Requirements

### Requirement: Promotion detection via explicit claim
The system SHALL promote a session's exploration only to a change that session has explicitly claimed, and SHALL NOT infer the target from changes appearing under `openspec/changes/`.

#### Scenario: A claimed change promotes the note
- **WHEN** a session with a scratch note has claimed change `<name>` and `<name>` is an active change
- **THEN** the system SHALL treat `<name>` as that session's promoted change

#### Scenario: No claim means no promotion
- **WHEN** a session has a scratch note and no claim, and any number of new active changes exist
- **THEN** the system SHALL take no promotion action, and SHALL leave the note at its session-scoped path

#### Scenario: A claim naming a change that does not exist is refused
- **WHEN** a session has claimed a change with no active change directory of that name
- **THEN** the system SHALL take no promotion action

#### Scenario: A session without a scratch note is never promoted
- **WHEN** a session has claimed a change but has no scratch note
- **THEN** the system SHALL take no promotion action and SHALL NOT create a note at the change-name key

### Requirement: Recording a claim
The system SHALL provide a command that records a session's claim on a change, and the claim SHALL be stored as a marker in the session's scratch note.

Recording via a command rather than by hand-authoring the marker is deliberate: a near-miss on the marker's exact form parses as no claim at all, whereas running a command with resolved arguments is the shape agents have been observed to follow reliably.

#### Scenario: A claim is recorded and read back
- **WHEN** the claim command is run for a session and a change name
- **THEN** the session's scratch note SHALL carry a claim marker naming that change, and the system SHALL read that change back as the session's claim

#### Scenario: Recording a claim preserves the exploration
- **WHEN** the claim command is run for a session whose scratch note holds an exploration
- **THEN** that exploration's existing content SHALL be preserved

#### Scenario: Re-claiming replaces the previous claim
- **WHEN** the claim command is run for a session that has already claimed a different change
- **THEN** the session SHALL be recorded as claiming only the change named most recently

### Requirement: The move-to-proposal directive asks for the claim
The system SHALL include, in the move-to-proposal reason text, the claim command with the session id and an instruction to name the change once it exists.

#### Scenario: The template names the claim command
- **WHEN** a move-to-proposal verdict is translated into a directive
- **THEN** the reason text SHALL name the claim command with the session id already resolved

### Requirement: An already-promoted session is not promoted again
The system SHALL refuse to promote a session whose scratch note is a moved-pointer record left by an earlier promotion.

Promotion leaves a redirect at the vacated session path. A check that tests only for the note's existence treats that redirect as a promotable note and renames it onto a later change, destroying the note already there.

#### Scenario: A redirect is not mistaken for an exploration
- **WHEN** a session's scratch note holds a moved-pointer record and that session claims another change
- **THEN** the system SHALL take no promotion action and SHALL leave the redirect in place

## MODIFIED Requirements

### Requirement: Rename on promotion
The system SHALL rename a promoted session's scratch note from `.openspec-doc/scratch/_session/<session_id>.md` to `.openspec-doc/scratch/<name>.md` and SHALL leave a moved-pointer record at the old path referencing the new path.

#### Scenario: Scratch note is renamed to the change-name key
- **WHEN** a session's scratch note is promoted to change `<name>`
- **THEN** the system SHALL rename the scratch note file to `.openspec-doc/scratch/<name>.md`

#### Scenario: Old path leaves a moved pointer
- **WHEN** a scratch note has been renamed due to promotion
- **THEN** the system SHALL leave a record at the old session-scoped path identifying the new change-name-scoped path

#### Scenario: The promoted note records what it came from
- **WHEN** a scratch note carrying a claim marker is promoted
- **THEN** the promoted note SHALL retain that marker, so the formalized exploration states which change it became
