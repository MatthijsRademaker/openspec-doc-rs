## ADDED Requirements

### Requirement: The interface is served from assets embedded in the binary
The system SHALL serve the dashboard interface from static assets embedded in the binary, requiring no asset directory, checked-out repository or build toolchain on the machine running it.

#### Scenario: The interface is served with no external files
- **WHEN** the binary is run on a machine with no build toolchain and no checked-out repository
- **THEN** the system SHALL serve the complete interface from assets embedded in the binary

#### Scenario: The root serves the interface
- **WHEN** a request is made to `/`
- **THEN** the system SHALL serve the interface

### Requirement: The embedded assets are verifiably built from the checked-in source
The system's committed build output SHALL be reproducible from the checked-in frontend source, and a build that produces different output SHALL fail rather than being published.

Committing a build artifact is what removes the Node toolchain from `cargo install`. It is also what makes it possible to ship an interface that does not match its source, silently. The check is what makes the trade acceptable, so it is a property of the system rather than a convenience.

#### Scenario: Stale committed assets fail the build
- **WHEN** the frontend source is built from a clean checkout and the output differs from the committed assets
- **THEN** the build SHALL fail and SHALL report that the committed assets are stale

### Requirement: The index's data is available as JSON
The system SHALL serve the data backing the index — every discovered session and change, each with the fields the index presents — from a JSON endpoint.

#### Scenario: Index data is available as JSON
- **WHEN** the index data endpoint is requested
- **THEN** the system SHALL respond with every discovered session and change, each with its title where one exists, its identifier, its last-modified time, its open-comment count, and its standing verdict

#### Scenario: A project with no scopes returns an empty index
- **WHEN** the index data endpoint is requested for a project with no discovered sessions or changes
- **THEN** the system SHALL respond with an empty list rather than an error
