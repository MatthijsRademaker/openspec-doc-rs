# dashboard-server Specification

## Purpose

The `serve` subcommand's local HTTP server: binding to a host and an available port, printing the resolved URL and opening a browser at it unless suppressed, and routing over the project's two scoping regimes — session-keyed routes for pre-proposal explore tabs, change-name-keyed routes for proposal and apply tabs. It also owns the live-update mechanism those routes share: a filesystem watcher per scope feeding a server-sent-events endpoint, so an open page updates without a reload. What a page *contains* belongs to other capabilities; this one owns the server, its route table, and the push channel they render into.

## Requirements
### Requirement: Serve subcommand startup
The system SHALL bind a local HTTP server to `127.0.0.1` by default (or `--host` when given), SHALL select an available port when `--port` is omitted or `0`, and SHALL print the resolved local URL on successful startup.

#### Scenario: Default startup selects an available port
- **WHEN** a user runs `openspec-doc serve` with no `--port`
- **THEN** the system SHALL bind to an available port on `127.0.0.1` and print the resolved URL

#### Scenario: Explicit host and port are honored
- **WHEN** a user runs `openspec-doc serve --host <host> --port <port>`
- **THEN** the system SHALL bind to the given host and port and print the resolved URL

### Requirement: Browser open control
The system SHALL open a browser to the resolved URL by default, and SHALL print the URL without opening a browser when `--no-open` is passed.

#### Scenario: Default opens a browser
- **WHEN** a user runs `openspec-doc serve` without `--no-open`
- **THEN** the system SHALL attempt to open a browser at the resolved URL

#### Scenario: --no-open suppresses browser launch
- **WHEN** a user runs `openspec-doc serve --no-open`
- **THEN** the system SHALL print the resolved URL and SHALL NOT attempt to open a browser

### Requirement: Session and change scoped routes
The system SHALL serve a page at `/sessions/<session_id>` for a discovered pre-proposal session and at `/changes/<name>` for a discovered active change.

#### Scenario: Known session route resolves
- **WHEN** a request is made to `/sessions/<session_id>` for a session with a known scratch note or directive record
- **THEN** the system SHALL serve a page for that session

#### Scenario: Known change route resolves
- **WHEN** a request is made to `/changes/<name>` for a change discovered by the project scanner
- **THEN** the system SHALL serve a page for that change

#### Scenario: Unknown route returns 404
- **WHEN** a request is made to `/sessions/<session_id>` or `/changes/<name>` for an id or name not discovered by the system
- **THEN** the system SHALL respond with a 404 status rather than serving a generic or empty page

### Requirement: Live update push
The system SHALL provide a server-sent-events endpoint per session or change route that emits an event when files relevant to that route change on disk, without requiring the client to poll or reload.

#### Scenario: File change triggers a push event
- **WHEN** a file relevant to an open session or change route changes on disk
- **THEN** the system SHALL emit a server-sent event on that route's SSE endpoint without the client issuing a new request

#### Scenario: Watcher initialization failure falls back to polling
- **WHEN** the filesystem watcher backend fails to initialize for a route
- **THEN** the system SHALL fall back to a bounded polling interval for that route and SHALL log the fallback rather than silently serving no updates

