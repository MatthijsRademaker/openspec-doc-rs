# anchored-comments Specification

## Purpose

Review comments attached to a span of markdown, and the append-only sidecar the comments for one scope are recorded in. An anchor captures its selected text along with the heading path and surrounding text that bracket it, so it can be re-found after the artifact around it changes — reported honestly as `exact`, `fuzzy`, `orphaned`, or `missing` rather than presented as confidently placed, or as `unanchored` for a comment that was never attached to a passage at all. The commented artifact is any markdown file under the project root, named by its relative path, so an explore-phase scratch note and a change's `proposal.md` are handled identically. A sidecar is keyed by the same scope key as the artifact it discusses and relocates in lockstep when that key changes from a session id to a change name.

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

### Requirement: A comment may be scoped to a session or change rather than anchored to text
The system SHALL allow a comment to be recorded against a session or a change with no anchor, and SHALL store it in the same sidecar as anchored comments.

Not all feedback is about a passage. Before this, unanchored feedback could only be carried in a phase verdict's notes field, which made it a third kind of text — neither attached to anything nor delivered through the comment sidecar that directives point the agent at.

#### Scenario: An unanchored comment is recorded
- **WHEN** a comment is submitted for a scope with no selected text
- **THEN** the system SHALL record it in that scope's comment sidecar with no anchor

#### Scenario: An unanchored comment needs no artifact
- **WHEN** an unanchored comment is submitted for a scope that has no artifacts on disk
- **THEN** the system SHALL record it, rather than failing for want of something to anchor against

#### Scenario: An unanchored comment is not reported as drifted
- **WHEN** the anchor state of an unanchored comment is resolved
- **THEN** the system SHALL report it as `unanchored`, distinctly from the `orphaned` state that means an anchor was lost

#### Scenario: An unanchored comment survives its sidecar's relocation
- **WHEN** a sidecar holding an unanchored comment is relocated to a new scope key
- **THEN** the system SHALL carry that comment across with the rest, having no artifact path in it to rewrite

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

### Requirement: A comment's body can be edited
The system SHALL allow the body of an existing comment to be replaced, SHALL record the replacement as an appended event, and SHALL report the most recent body when the sidecar is read.

A reviewer who mistypes otherwise has to add a second comment correcting the first, leaving the agent two instructions and no way to tell which one stands.

Only a comment is editable, never a reply: a reply is the agent's report of what it did, and rewriting it would remove the record the reviewer is judging.

#### Scenario: An edited comment reads back with its new body
- **WHEN** a comment's body is edited
- **THEN** reading the sidecar SHALL report the new body for that comment

#### Scenario: The latest edit is the one that stands
- **WHEN** a comment's body is edited more than once
- **THEN** reading the sidecar SHALL report the body from the last edit

#### Scenario: The earlier body is not removed from the file
- **WHEN** a comment's body is edited
- **THEN** the sidecar SHALL still contain the record that created it, unmodified

#### Scenario: Editing an unknown comment fails
- **WHEN** an edit names a comment id the sidecar does not contain, including the id of a reply
- **THEN** the system SHALL fail with an error naming that id, rather than appending an event nothing will fold

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
The system SHALL provide `comment add`, `comment list`, `comment reply`, `comment edit`, `comment address`, `comment resolve`, and `comment reopen` operations via the CLI, and `comment add` SHALL accept a body with no selected text.

The CLI is how the dashboard's behaviour is verified, so every operation the dashboard performs has to be reachable without it.

#### Scenario: Adding a comment via the CLI
- **WHEN** a user runs `openspec-doc comment add` with a target artifact, selected text, and body
- **THEN** the system SHALL create the anchor context and append a comment event to the appropriate sidecar

#### Scenario: Listing comments via the CLI
- **WHEN** a user runs `openspec-doc comment list` for a given scope key
- **THEN** the system SHALL print each comment's current status and anchor state

#### Scenario: A comment is created without a selection
- **WHEN** a user runs `openspec-doc comment add` with a body and neither an artifact nor selected text
- **THEN** the system SHALL record an unanchored comment against the named scope

#### Scenario: Half a selection is refused
- **WHEN** a user runs `openspec-doc comment add` with an artifact but no selected text, or the reverse
- **THEN** the system SHALL reject the invocation, rather than treating an incomplete selection as scope-level feedback

#### Scenario: An unanchored comment is listed as such
- **WHEN** a user lists a scope containing an unanchored comment
- **THEN** the system SHALL show it as unanchored rather than printing an empty artifact path or an empty quotation

#### Scenario: Editing a comment via the CLI
- **WHEN** a user runs `openspec-doc comment edit` for an existing comment with a new body
- **THEN** the system SHALL append an edit event, and the next `comment list` SHALL report the new body

#### Scenario: Marking a comment addressed via the CLI
- **WHEN** a user or agent runs `openspec-doc comment address` for an existing comment
- **THEN** the system SHALL append a status event moving that comment to `addressed`

#### Scenario: Reopening a comment via the CLI
- **WHEN** a user runs `openspec-doc comment reopen` for an existing comment
- **THEN** the system SHALL append a status event moving that comment back to `open`

#### Scenario: A status operation on an unknown comment fails loudly
- **WHEN** a status operation names a comment id absent from the scope's sidecar
- **THEN** the system SHALL return an error rather than appending an event for a comment that does not exist
