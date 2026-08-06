## ADDED Requirements

### Requirement: The server exposes review state as JSON and serves the interface as static assets
The system SHALL serve the dashboard's data as JSON endpoints and the interface itself as static assets embedded in the binary, rather than rendering HTML pages per request.

#### Scenario: A scope's artifacts and comments are retrievable as data
- **WHEN** a client requests a scope's detail endpoint
- **THEN** the system SHALL return its artifacts decomposed into blocks, its comments with their resolved anchor states, and its verdict state, as JSON

#### Scenario: The interface is served without an external asset directory
- **WHEN** the binary is run on a machine with no build toolchain and no checked-out repository
- **THEN** the system SHALL serve the complete interface from assets embedded in the binary

### Requirement: The update event distinguishes what changed
The system SHALL indicate, in each event it emits on a scope's update stream, whether the scope's artifacts changed, its review state changed, or both.

A client that cannot tell the two apart must either refetch everything on every event or refetch the wrong thing. The watcher already distinguishes them: `scope::target` records the artifact paths and the sidecar paths separately.

#### Scenario: An artifact change is reported as one
- **WHEN** a scope's artifact file changes
- **THEN** the emitted event SHALL indicate that the artifacts changed

#### Scenario: A comment change is reported as one
- **WHEN** a comment is recorded against a scope
- **THEN** the emitted event SHALL indicate that the review state changed

## MODIFIED Requirements

### Requirement: Session and change scoped routes
The system SHALL serve the dashboard interface at `/sessions/<session_id>` for a discovered pre-proposal session and at `/changes/<name>` for a discovered active change, and SHALL serve that scope's data from a corresponding JSON endpoint.

Comment and verdict submission move to JSON endpoints with the frontend replacement. Form-post handling is not retained alongside them: two submission paths against one sidecar is two things to keep correct, and the browser is the only client.

#### Scenario: Known session route resolves
- **WHEN** a request is made to `/sessions/<session_id>` for a session with a known scratch note or directive record
- **THEN** the system SHALL serve the interface for that session

#### Scenario: Known change route resolves
- **WHEN** a request is made to `/changes/<name>` for a change discovered by the project scanner
- **THEN** the system SHALL serve the interface for that change

#### Scenario: Unknown scope is reported as unknown
- **WHEN** a scope's data endpoint is requested for an id or name not discovered by the system
- **THEN** the system SHALL respond with a 404 status rather than an empty scope

#### Scenario: A submission is accepted as JSON
- **WHEN** a comment or verdict is submitted for a scope
- **THEN** the system SHALL accept it as a JSON request and SHALL record it through the same core writers the CLI uses
