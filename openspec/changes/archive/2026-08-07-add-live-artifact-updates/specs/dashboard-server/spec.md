## MODIFIED Requirements

### Requirement: Live update push
The system SHALL provide a server-sent-events endpoint per session or change route that emits an event when files relevant to that route change on disk, without requiring the client to poll or reload, and SHALL indicate in each event whether the scope's artifacts changed, its review state changed, or both.

A client that cannot tell the two apart must either refetch everything on every event or refetch the wrong thing. The watcher already distinguishes them: `scope::target` records the artifact paths and the sidecar paths separately, so the distinction exists at the point of emission and is simply not carried.

#### Scenario: File change triggers a push event
- **WHEN** a file relevant to an open session or change route changes on disk
- **THEN** the system SHALL emit a server-sent event on that route's SSE endpoint without the client issuing a new request

#### Scenario: An artifact change is reported as one
- **WHEN** a scope's artifact file changes
- **THEN** the emitted event SHALL indicate that the artifacts changed

#### Scenario: A comment change is reported as one
- **WHEN** a comment is recorded against a scope
- **THEN** the emitted event SHALL indicate that the review state changed

#### Scenario: One logical write emits one event
- **WHEN** a single rewrite of an artifact causes the watcher backend to report several filesystem events
- **THEN** the system SHALL emit one event rather than one per underlying report

#### Scenario: Watcher initialization failure falls back to polling
- **WHEN** the filesystem watcher backend fails to initialize for a route
- **THEN** the system SHALL fall back to a bounded polling interval for that route and SHALL log the fallback rather than silently serving no updates
