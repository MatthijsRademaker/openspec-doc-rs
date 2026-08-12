## MODIFIED Requirements

### Requirement: Local HTTP server on the port assigned to the project
The system SHALL bind a local HTTP server to `127.0.0.1` by default (or `--host` when given), SHALL bind the port assigned to the project's canonical root when `--port` is omitted, SHALL fall forward to another port in the range and record where it landed when the assigned port is taken by something else, SHALL honour an explicit `--port` exactly and fail rather than fall forward, and SHALL print the resolved local URL on successful startup.

The default stops being an OS-chosen ephemeral port because an ephemeral port cannot be found by a later process. It is a port assigned to the project rather than the first free port, because a port that depends on which checkout started first cannot be bookmarked, printed before the server exists, or stated by anything that has not just probed for it.

An explicit `--port` neither reads nor writes the assignment: a user who names a port wants that port for this invocation, not a change to where the project lives, and silently serving somewhere else is how two tabs end up disagreeing about which dashboard they are looking at.

#### Scenario: Default startup binds the project's assigned port
- **WHEN** a user runs `openspec-doc serve` with no `--port` and the port assigned to that project's root is free
- **THEN** the system SHALL bind that port on `127.0.0.1` and print the resolved URL

#### Scenario: The same project gets the same port on a later run
- **WHEN** a user runs `openspec-doc serve` with no `--port` for a project served on some port before, after other projects have been served in between
- **THEN** the system SHALL bind the same port it bound for that project before

#### Scenario: Two projects do not collide
- **WHEN** dashboards are started with no `--port` for two different canonical project roots
- **THEN** the system SHALL bind them to different ports

#### Scenario: A squatted assigned port relocates the project
- **WHEN** a user runs `openspec-doc serve` with no `--port` and the project's assigned port is held by something that is not a dashboard for that root
- **THEN** the system SHALL bind another free port in the range, print the resolved URL, and record that port as the project's assignment from then on

#### Scenario: An exhausted range fails
- **WHEN** a user runs `openspec-doc serve` with no `--port` and every port in the range is taken
- **THEN** the system SHALL fail with an error naming the range

#### Scenario: Explicit host and port are honored exactly
- **WHEN** a user runs `openspec-doc serve --host <host> --port <port>`
- **THEN** the system SHALL bind to the given host and port and print the resolved URL, and SHALL fail rather than bind a different port when that port is taken

#### Scenario: An explicit port does not change where the project lives
- **WHEN** a user runs `openspec-doc serve --port <port>` and later runs `openspec-doc serve` with no `--port`
- **THEN** the second run SHALL bind the project's assigned port, unaffected by the explicit one

## ADDED Requirements

### Requirement: The server can be asked to exit when idle
The system SHALL accept an option that makes the server exit after a period during which no page has been subscribed to it and no external activity has been registered against it, and SHALL keep serving indefinitely when that option is absent.

The option exists so that ownership of the process is explicit. A server a person started in a terminal must not vanish under them; a server started on their behalf by a hook has no one waiting on it and needs a deadline.

#### Scenario: Idle exit is opt-in
- **WHEN** a user runs `openspec-doc serve` without the idle-exit option
- **THEN** the server SHALL keep serving regardless of how long it goes unused
