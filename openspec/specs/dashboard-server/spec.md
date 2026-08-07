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
The system SHALL serve the dashboard interface at `/sessions/<session_id>` for a discovered pre-proposal session and at `/changes/<name>` for a discovered active change, and SHALL serve that scope's data — its artifacts decomposed into blocks, its comments with their resolved anchor states, and its verdict state — from a corresponding JSON endpoint.

Comment and verdict submission move to JSON endpoints with the frontend replacement. Form-post handling is not retained alongside them: two submission paths against one sidecar is two things to keep correct, and the browser is the only client.

#### Scenario: Known session route resolves
- **WHEN** a request is made to `/sessions/<session_id>` for a session with a known scratch note or directive record
- **THEN** the system SHALL serve the interface for that session

#### Scenario: Known change route resolves
- **WHEN** a request is made to `/changes/<name>` for a change discovered by the project scanner
- **THEN** the system SHALL serve the interface for that change

#### Scenario: A scope's artifacts and comments are retrievable as data
- **WHEN** a client requests a scope's data endpoint
- **THEN** the system SHALL return its artifacts decomposed into blocks, its comments with their resolved anchor states and the block each resolves to, and its verdict state including whether its directive has been delivered

#### Scenario: Unknown scope is reported as unknown
- **WHEN** a scope's data endpoint is requested for an id or name not discovered by the system
- **THEN** the system SHALL respond with a 404 status rather than an empty scope

#### Scenario: A submission is accepted as JSON
- **WHEN** a comment, a reply, a comment status change or a verdict is submitted for a scope
- **THEN** the system SHALL accept it as a JSON request and SHALL record it through the same core writers the CLI uses

#### Scenario: A block comment names the position it came from
- **WHEN** a comment is submitted against a rendered block
- **THEN** the system SHALL anchor it using that block's position in the artifact, so a block whose text is repeated anchors to the occurrence the reviewer pointed at

### Requirement: Live update push
The system SHALL provide a server-sent-events endpoint per session or change route that emits an event when files relevant to that route change on disk, without requiring the client to poll or reload.

#### Scenario: File change triggers a push event
- **WHEN** a file relevant to an open session or change route changes on disk
- **THEN** the system SHALL emit a server-sent event on that route's SSE endpoint without the client issuing a new request

#### Scenario: Watcher initialization failure falls back to polling
- **WHEN** the filesystem watcher backend fails to initialize for a route
- **THEN** the system SHALL fall back to a bounded polling interval for that route and SHALL log the fallback rather than silently serving no updates

### Requirement: An artifact is decomposed into anchorable blocks
The system SHALL decompose an artifact's markdown into blocks, each carrying a stable identifier, its rendered form, the exact source text it was sliced from, and that text's byte range in the artifact.

Blocks are paragraphs, headings, list items, table rows, and lines within a code fence. A list item is a block rather than the whole list, because a tasks file is one list and commenting on it as a single unit says nothing.

The source text is sliced from the artifact by the reported range and is never reconstructed from the parse events. Reconstruction produces text that differs from the file wherever the parser normalised anything, and that difference is invisible until an anchor's substring search fails in front of a reviewer.

#### Scenario: Every block's source occurs in its artifact
- **WHEN** an artifact is decomposed into blocks
- **THEN** each block's source text SHALL occur in that artifact's markdown

#### Scenario: A block reports where its source is
- **WHEN** an artifact is decomposed into blocks
- **THEN** each block SHALL report the byte range its source text occupies in the artifact

### Requirement: Markup in an artifact is not markup on the page
The system SHALL render raw HTML found in an artifact as text rather than emitting it as markup.

Artifacts are written by an agent and rendered into the reviewer's browser. Passing agent-written markup through to the page is a stored-injection path, and "it is only localhost" is the reasoning that keeps one alive until the day it matters.

#### Scenario: A script element in an artifact arrives as text
- **WHEN** an artifact contains a `<script>` element
- **THEN** the block containing it SHALL carry that element as escaped text and SHALL NOT carry it as markup

