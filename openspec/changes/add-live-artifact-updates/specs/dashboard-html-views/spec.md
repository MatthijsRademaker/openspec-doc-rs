## MODIFIED Requirements

### Requirement: Live update without reload
The system SHALL update a rendered page's comment list, verdict state and rendered artifacts when the underlying SSE endpoint emits an event, without requiring a full page reload. It SHALL defer an artifact update while the reviewer has unsent text in a composer, indicating that the artifact has changed and applying the update once the composer is sent or dismissed. It SHALL NOT defer a review-state update.

The filesystem watcher already reports artifact changes and the event already reaches the browser; only the review state is refreshed in response. An exploration that is readable "while it is happening" cannot leave the exploration itself stale. Replacing the document under an open composer is why that refresh was limited in the first place, so the limit is replaced with a rule rather than removed.

#### Scenario: Comment added in one tab appears in another
- **WHEN** a comment is added against an artifact while a second browser tab has the same page open
- **THEN** the second tab SHALL reflect the new comment without the user reloading the page

#### Scenario: An artifact rewritten under an idle page updates
- **WHEN** an artifact changes while its page is open with no unsent composer text
- **THEN** the system SHALL render the new content without the reviewer acting

#### Scenario: An artifact rewritten under a page with unsent text offers a refresh
- **WHEN** an artifact changes while a composer holds unsent text
- **THEN** the system SHALL indicate that the artifact has changed and SHALL NOT replace the content until the composer is sent or dismissed

#### Scenario: A deferred update applies once the composer closes
- **WHEN** a composer holding unsent text is sent or dismissed after an artifact change was deferred
- **THEN** the system SHALL apply the deferred update

#### Scenario: Reading position survives an update
- **WHEN** an artifact update is applied to a page the reviewer has scrolled
- **THEN** the system SHALL preserve the reviewer's position in the document

#### Scenario: An artifact appearing for the first time is rendered
- **WHEN** a scope's page is open with no artifact on disk and an artifact is then written
- **THEN** the system SHALL render it without the reviewer reloading
