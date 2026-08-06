## MODIFIED Requirements

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
