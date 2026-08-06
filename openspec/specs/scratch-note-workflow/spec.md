# scratch-note-workflow Specification

## Purpose

The lifecycle of an exploration's scratch note: a plain markdown file that exists from the start of a session, so explore-phase commenting has something to anchor against before any `openspec/changes/<name>/` directory does. The note starts out keyed by session id and is promoted to a change-name key — renamed, with a redirect left behind, and validated — once the session's exploration is formalized into a change. Promotion happens only for a change the session has explicitly claimed: a change directory appearing establishes nothing about which session created it, so a claim is the only thing that attributes one to the other, and a session that never claims keeps its note at the session key rather than having a target guessed for it.
## Requirements
### Requirement: Session-scoped scratch note location
The system SHALL locate a session's exploration scratch note at `.openspec-doc/scratch/_session/<session_id>.md` while no change directory exists yet for that session.

#### Scenario: Scratch note persists with no change directory
- **WHEN** a session has a scratch note at `.openspec-doc/scratch/_session/<session_id>.md` and no matching change directory exists under `openspec/changes/`
- **THEN** the system SHALL leave the scratch note at its session-scoped path, unmodified

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

### Requirement: Validate on promotion
The system SHALL run `openspec validate <name>` immediately after renaming a promoted scratch note and SHALL record the validation outcome, including failure, as part of the promotion result.

#### Scenario: Passing validation is recorded
- **WHEN** `openspec validate <name>` succeeds after promotion
- **THEN** the system SHALL record the promotion result as validated, with the validation output retained

#### Scenario: Failing validation is surfaced, not discarded
- **WHEN** `openspec validate <name>` fails after promotion
- **THEN** the system SHALL record the promotion result as validation-failed, with the failure output retained and available to the caller

#### Scenario: A failing validation does not undo the rename
- **WHEN** `openspec validate <name>` fails after promotion
- **THEN** the system SHALL leave the promoted note at its change-name key rather than reverting it to the session-scoped path

### Requirement: A session's title is read from its scratch note
The system SHALL derive a session's human-readable title from its scratch note's first heading when that heading is level one, and SHALL report no title otherwise.

The agent already writes a heading describing what it is exploring, and keeps it current as the exploration changes shape. Deriving the title from it costs no new state, no new command, and no new thing for the agent to remember. Reporting nothing when there is no such heading is deliberate: a session with nothing written has no topic, and a synthesised one is a guess the reviewer would have to learn to distrust.

The note's *first* heading, rather than the first level-one heading anywhere in it: a note that opens at level two was never given a title, and titling it after some `#` further down names it after one of its sections.

#### Scenario: A note with a heading yields a title
- **WHEN** a session's scratch note begins with a level-one heading
- **THEN** the system SHALL report that heading's text, trimmed, as the session's title

#### Scenario: A note that does not open at level one yields no title
- **WHEN** a session's scratch note is absent, is empty, contains no heading, or opens with a heading below level one
- **THEN** the system SHALL report that the session has no title, rather than reporting a placeholder or an error

#### Scenario: A promoted note titles the change it became
- **WHEN** a change's note, promoted to `.openspec-doc/scratch/<name>.md`, opens with a level-one heading
- **THEN** the system SHALL report that heading's text as the change's title, by the same rule it titles a session

#### Scenario: A heading inside a code fence is not a title
- **WHEN** a session's scratch note contains a line beginning with `#` inside a fenced code block, before any real heading
- **THEN** the system SHALL NOT report that line as the session's title

### Requirement: The session identifier remains the session's identity
The system SHALL continue to identify a session by its session id in every URL, filename, and hook payload, and SHALL treat the title as display text only.

A title read from a file the agent rewrites is mutable by design. Keying a URL or a sidecar path on it would break every existing link and orphan every recorded comment the first time the exploration's topic shifted.

#### Scenario: A title change does not change any address
- **WHEN** the first heading of a session's scratch note is rewritten
- **THEN** the session's page URL, its comment sidecar path, and its verdict sidecar path SHALL be unchanged

