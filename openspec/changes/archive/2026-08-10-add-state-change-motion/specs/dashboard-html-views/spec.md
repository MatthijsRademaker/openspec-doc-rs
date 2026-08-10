## ADDED Requirements

### Requirement: An update the reviewer did not cause is reported where it lands

The system SHALL report, at the point on screen where it arrives, that a scope's artifact content or comment content has been replaced by an update the reviewer did not initiate, and SHALL do so without moving the reviewer's reading position and without requiring them to act on the report.

The interface already preserves reading position across a live update, which is correct and must not change. Its consequence is that a rewritten document and an unchanged one are indistinguishable: the agent edits `proposal.md`, the frame lands, the prose is different, and nothing says so. Reporting the arrival is the only channel the interface has for the difference.

The report concerns arrival, not authorship or extent. Identifying *which* passages changed requires deciding that a block in the new content is the same block as one in the old, which is the problem the anchor resolver already owns and already gets wrong in known ways. A second, disagreeing answer to it would let the interface call a block unchanged while orphaning a comment anchored to that block.

#### Scenario: An artifact rewritten under an idle page is reported

- **WHEN** an artifact changes while its page is open with no unsent composer text
- **THEN** the system SHALL render the new content, SHALL preserve the reviewer's position, and SHALL report at the document that the content was replaced

#### Scenario: A deferred update is reported when it applies

- **WHEN** an artifact update deferred behind an unsent composer is applied after that composer is sent or dismissed
- **THEN** the system SHALL report the replacement at the moment the content changes, not at the moment the change was detected

#### Scenario: The reviewer's own submission is confirmed where it appears

- **WHEN** a comment or reply the reviewer submitted is rendered after the mutation resolves
- **THEN** the system SHALL report its arrival at the thread it created or extended

#### Scenario: The report requires no dismissal

- **WHEN** the system has reported an arrival
- **THEN** the report SHALL clear on its own, SHALL NOT require the reviewer to dismiss it, and SHALL NOT obscure or intercept interaction with the content it reports on

#### Scenario: An arrival report is not a change summary

- **WHEN** an artifact is replaced
- **THEN** the system SHALL NOT assert which blocks differ, and SHALL NOT present unchanged content as changed or changed content as unchanged
