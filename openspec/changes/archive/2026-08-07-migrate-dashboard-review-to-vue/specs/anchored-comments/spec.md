## MODIFIED Requirements

### Requirement: Comment, reply, and status sidecar

The system SHALL persist comment, reply, and status-update events as an append-only JSONL sidecar file keyed by the artifact's current scope key, without rewriting prior records. Each reply event SHALL identify its author as `reviewer` or `agent`. A change-keyed sidecar SHALL live at `.openspec-doc/comments/<name>.jsonl` and a session-keyed one at `.openspec-doc/comments/_session/<session_id>.jsonl`, so that a session id can never collide with a change name.

#### Scenario: Reply and status updates are appended, not rewritten

- **WHEN** a reply or status update is recorded for an existing comment
- **THEN** the system SHALL append a new event referencing the comment id, and SHALL NOT modify or remove any prior event in the sidecar

#### Scenario: Reply author is preserved

- **WHEN** a reviewer or agent adds a reply to a comment thread
- **THEN** the appended reply event SHALL identify that actor as its author

#### Scenario: Existing reply without author remains agent-authored

- **WHEN** a reply event written before reply authorship existed is read
- **THEN** the system SHALL reconstruct it as agent-authored

#### Scenario: Reading back reconstructs current state

- **WHEN** the sidecar for scope key `<key>` is read
- **THEN** the system SHALL reconstruct each comment's current status and full reply/status history from the sequence of events, including each reply's author
