# anchored-comments Specification

## Purpose

Review comments attached to a span of markdown, and the append-only sidecar the comments for one scope are recorded in. An anchor captures its selected text along with the heading path and surrounding text that bracket it, so it can be re-found after the artifact around it changes — reported honestly as `exact`, `fuzzy`, `orphaned`, or `missing` rather than presented as confidently placed. The commented artifact is any markdown file under the project root, named by its relative path, so an explore-phase scratch note and a change's `proposal.md` are handled identically. A sidecar is keyed by the same scope key as the artifact it discusses and relocates in lockstep when that key changes from a session id to a change name.

## Requirements

### Requirement: Anchor context creation
The system SHALL create an anchor context from selected text and its containing markdown, capturing the heading path, before/after text context, and character offsets, and SHALL fail loudly when the selected text is not found in the markdown.

#### Scenario: Anchor context is created for text present in the markdown
- **WHEN** selected text exists in the given markdown
- **THEN** the system SHALL produce an anchor context including heading path, before/after text, and start/end offsets

#### Scenario: Selected text not found fails loudly
- **WHEN** selected text does not exist in the given markdown
- **THEN** the system SHALL return an error rather than producing a partial or guessed anchor context

### Requirement: Anchor resolution
The system SHALL resolve a stored anchor against current markdown content by first attempting an exact offset match, then a fuzzy match using heading path and before/after text, and otherwise SHALL report the anchor as orphaned or missing.

#### Scenario: Exact offset match resolves as exact
- **WHEN** the stored offset in current markdown still contains the anchor's selected text
- **THEN** the system SHALL resolve the anchor as `exact` at that offset

#### Scenario: Fuzzy match resolves as fuzzy
- **WHEN** the exact offset no longer matches but the selected text can be located via heading path or before/after text context
- **THEN** the system SHALL resolve the anchor as `fuzzy` at the located offset

#### Scenario: No match resolves as orphaned
- **WHEN** neither an exact nor a fuzzy match can be found in the current markdown
- **THEN** the system SHALL resolve the anchor as `orphaned`

#### Scenario: Missing artifact resolves as missing
- **WHEN** the artifact referenced by the anchor is not available to resolve against
- **THEN** the system SHALL resolve the anchor as `missing`

### Requirement: Comment, reply, and status sidecar
The system SHALL persist comment, reply, and status-update events as an append-only JSONL sidecar file keyed by the artifact's current scope key, without rewriting prior records. A change-keyed sidecar SHALL live at `.openspec-doc/comments/<name>.jsonl` and a session-keyed one at `.openspec-doc/comments/_session/<session_id>.jsonl`, so that a session id can never collide with a change name.

#### Scenario: New comment is appended
- **WHEN** a comment is created against an artifact with change scope key `<name>`
- **THEN** the system SHALL append a comment event to `.openspec-doc/comments/<name>.jsonl`

#### Scenario: A session's comments are namespaced apart from a change's
- **WHEN** a comment is created against an artifact with session scope key `<session_id>`
- **THEN** the system SHALL append a comment event to `.openspec-doc/comments/_session/<session_id>.jsonl`

#### Scenario: Reply and status updates are appended, not rewritten
- **WHEN** a reply or status update is recorded for an existing comment
- **THEN** the system SHALL append a new event referencing the comment id, and SHALL NOT modify or remove any prior event in the sidecar

#### Scenario: Reading back reconstructs current state
- **WHEN** the sidecar for scope key `<key>` is read
- **THEN** the system SHALL reconstruct each comment's current status and full reply/status history from the sequence of events

### Requirement: Comment status lifecycle
The system SHALL track each comment in one of three statuses — `open`, `addressed`, or `resolved` — recorded as appended status events like any other. `addressed` asserts that work responding to the comment has been done; `resolved` asserts that the reviewer accepts it. A comment SHALL be movable back to `open` from either later status, so that a reviewer who rejects the response can reopen the thread rather than filing a duplicate.

#### Scenario: A new comment starts open
- **WHEN** a comment is created
- **THEN** the system SHALL record its status as `open`

#### Scenario: A comment is marked addressed
- **WHEN** a status update to `addressed` is recorded for an open comment
- **THEN** the system SHALL reconstruct that comment's current status as `addressed`, and SHALL retain the comment, its replies, and its prior status events

#### Scenario: A comment is resolved
- **WHEN** a status update to `resolved` is recorded for a comment
- **THEN** the system SHALL reconstruct that comment's current status as `resolved`

#### Scenario: A comment is reopened
- **WHEN** a status update to `open` is recorded for a comment currently `addressed` or `resolved`
- **THEN** the system SHALL reconstruct that comment's current status as `open`, and SHALL retain the full status history including the superseded status

### Requirement: Scope status counts
The system SHALL report, for a given scope key, how many of its comments are currently `open`, `addressed`, and `resolved`.

#### Scenario: Counts reflect current reconstructed status
- **WHEN** the counts for a scope key are requested
- **THEN** the system SHALL count each comment once under its current reconstructed status, and SHALL NOT count superseded statuses from that comment's history

#### Scenario: A scope with no sidecar counts as zero
- **WHEN** the counts are requested for a scope key with no comment sidecar
- **THEN** the system SHALL report zero in every status rather than failing

### Requirement: Sidecar relocation on scope-key rename
The system SHALL relocate a comment sidecar to a new scope key whenever its artifact's scope key changes, preserving all recorded events, and SHALL append a relocation event recording the artifact's old and new paths so that the anchors in the preserved events keep resolving against the renamed artifact.

#### Scenario: Sidecar follows promotion rename
- **WHEN** an artifact's scope key changes from a session id to a change name
- **THEN** the system SHALL relocate the session-keyed sidecar to the change-keyed one with all events intact

#### Scenario: Relocated anchors resolve against the renamed artifact
- **WHEN** a relocated sidecar is read back
- **THEN** each comment anchored to the artifact's old path SHALL report the artifact's new path, and SHALL resolve as `exact` when the anchored text is unchanged

#### Scenario: Relocation refuses to overwrite an existing sidecar
- **WHEN** a sidecar already exists at the new scope key
- **THEN** the system SHALL fail rather than relocate over it, leaving both sets of events where they are

### Requirement: Comment CLI subcommand
The system SHALL provide `comment add`, `comment list`, `comment reply`, `comment address`, `comment resolve`, and `comment reopen` operations via the CLI.

#### Scenario: Adding a comment via the CLI
- **WHEN** a user runs `openspec-doc comment add` with a target artifact, selected text, and body
- **THEN** the system SHALL create the anchor context and append a comment event to the appropriate sidecar

#### Scenario: Listing comments via the CLI
- **WHEN** a user runs `openspec-doc comment list` for a given scope key
- **THEN** the system SHALL print each comment's current status and anchor state

#### Scenario: Marking a comment addressed via the CLI
- **WHEN** a user or agent runs `openspec-doc comment address` for an existing comment
- **THEN** the system SHALL append a status event moving that comment to `addressed`

#### Scenario: Reopening a comment via the CLI
- **WHEN** a user runs `openspec-doc comment reopen` for an existing comment
- **THEN** the system SHALL append a status event moving that comment back to `open`

#### Scenario: A status operation on an unknown comment fails loudly
- **WHEN** a status operation names a comment id absent from the scope's sidecar
- **THEN** the system SHALL return an error rather than appending an event for a comment that does not exist
