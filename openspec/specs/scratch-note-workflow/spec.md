# scratch-note-workflow Specification

## Purpose

The lifecycle of an exploration's scratch note: a plain markdown file that exists from the start of a session, so explore-phase commenting has something to anchor against before any `openspec/changes/<name>/` directory does. The note starts out keyed by session id and is promoted to a change-name key — renamed, with a redirect left behind, and validated — once the session's exploration is formalized into a change. Promotion is observed by diffing the active-change list against a per-session snapshot rather than by asking the agent to record the change name it chose.

## Requirements
### Requirement: Session-scoped scratch note location
The system SHALL locate a session's exploration scratch note at `.openspec-doc/scratch/_session/<session_id>.md` while no change directory exists yet for that session.

#### Scenario: Scratch note persists with no change directory
- **WHEN** a session has a scratch note at `.openspec-doc/scratch/_session/<session_id>.md` and no matching change directory exists under `openspec/changes/`
- **THEN** the system SHALL leave the scratch note at its session-scoped path, unmodified

### Requirement: Promotion detection via snapshot diff
The system SHALL detect promotion of a session's exploration to a formal change by comparing the active-change list at two points in time, scoped to sessions with a pending scratch note.

#### Scenario: Exactly one new active change promotes cleanly
- **WHEN** exactly one new active change directory has appeared under `openspec/changes/` since the prior snapshot, and the session has a pending scratch note
- **THEN** the system SHALL treat that change as the session's promoted change

#### Scenario: No new active change is a no-op
- **WHEN** no new active change directory has appeared since the prior snapshot
- **THEN** the system SHALL take no promotion action

#### Scenario: Multiple new active changes trigger the ambiguity guard
- **WHEN** more than one new active change directory has appeared since the prior snapshot for the same session
- **THEN** the system SHALL skip auto-promotion and record an explicit ambiguity notice rather than selecting one of them

#### Scenario: The first check only records a baseline
- **WHEN** a session has no prior snapshot to compare against
- **THEN** the system SHALL record the current active-change list as that session's snapshot and take no promotion action, regardless of whether the session has a scratch note yet

#### Scenario: An unresolved ambiguity keeps being reported
- **WHEN** a check was skipped by the ambiguity guard and the same candidates are still present at the next check
- **THEN** the system SHALL report the ambiguity again rather than advancing its snapshot past the unresolved candidates

#### Scenario: A session without a scratch note is never promoted
- **WHEN** a new active change appears for a session that has no scratch note
- **THEN** the system SHALL take no promotion action and SHALL NOT create a note at the change-name key

### Requirement: Rename on promotion
The system SHALL rename a promoted session's scratch note from `.openspec-doc/scratch/_session/<session_id>.md` to `.openspec-doc/scratch/<name>.md` and SHALL leave a moved-pointer record at the old path referencing the new path.

#### Scenario: Scratch note is renamed to the change-name key
- **WHEN** a session's scratch note is promoted to change `<name>`
- **THEN** the system SHALL rename the scratch note file to `.openspec-doc/scratch/<name>.md`

#### Scenario: Old path leaves a moved pointer
- **WHEN** a scratch note has been renamed due to promotion
- **THEN** the system SHALL leave a record at the old session-scoped path identifying the new change-name-scoped path

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
