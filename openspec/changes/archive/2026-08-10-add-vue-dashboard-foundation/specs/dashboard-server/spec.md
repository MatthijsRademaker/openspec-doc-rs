## ADDED Requirements

### Requirement: The interface is served from assets embedded in the binary
The system SHALL serve the dashboard interface from static assets embedded in the binary, requiring no asset directory, checked-out repository or build toolchain on the machine running it.

The built frontend is a generated artifact that is not in version control; the server crate embeds whatever the build produced, at compile time. That is what removes the JavaScript toolchain from `cargo install` and from every machine that only runs the binary. How the artifact is produced and what happens when it is missing belong to `dashboard-development-harness`; this requirement is about what the running server serves.

#### Scenario: The interface is served with no external files
- **WHEN** the binary is run on a machine with no build toolchain and no checked-out repository
- **THEN** the system SHALL serve the complete interface from assets embedded in the binary

#### Scenario: The root serves the interface
- **WHEN** a request is made to `/`
- **THEN** the system SHALL serve the interface

### Requirement: The index's data is available as JSON
The system SHALL serve the data backing the index — every discovered session and change, each with the fields the index presents — from a JSON endpoint.

#### Scenario: Index data is available as JSON
- **WHEN** the index data endpoint is requested
- **THEN** the system SHALL respond with every discovered session and change, each with its title where one exists, its identifier, its last-modified time, its open-comment count, and its standing verdict

#### Scenario: A project with no scopes returns an empty index
- **WHEN** the index data endpoint is requested for a project with no discovered sessions or changes
- **THEN** the system SHALL respond with an empty list rather than an error
