## MODIFIED Requirements

### Requirement: Local HTTP server on a discoverable port
The system SHALL bind a local HTTP server to `127.0.0.1` by default (or `--host` when given), SHALL bind port `4321` when `--port` is omitted — falling forward to the next free port through `4330` when that port is taken — SHALL fail loudly when the whole range is taken, SHALL honour an explicit `--port` exactly and fail rather than fall forward, and SHALL print the resolved local URL on successful startup.

The default stops being an OS-chosen ephemeral port because an ephemeral port cannot be found by a later process. Discovery of an already-running dashboard depends on probing a known range, so the default port has to be knowable without asking the running server where it went. An explicit `--port` does not fall forward: a user who names a port wants that port, and silently serving somewhere else is how two tabs end up disagreeing about which dashboard they are looking at.

#### Scenario: Default startup binds the base port
- **WHEN** a user runs `openspec-doc serve` with no `--port` and port `4321` is free
- **THEN** the system SHALL bind `127.0.0.1:4321` and print the resolved URL

#### Scenario: A taken base port falls forward
- **WHEN** a user runs `openspec-doc serve` with no `--port` and port `4321` is taken
- **THEN** the system SHALL bind the next free port in the range and print the resolved URL

#### Scenario: An exhausted range fails
- **WHEN** a user runs `openspec-doc serve` with no `--port` and every port in the range is taken
- **THEN** the system SHALL fail with an error naming the range

#### Scenario: Explicit host and port are honored exactly
- **WHEN** a user runs `openspec-doc serve --host <host> --port <port>`
- **THEN** the system SHALL bind to the given host and port and print the resolved URL, and SHALL fail rather than bind a different port when that port is taken

## ADDED Requirements

### Requirement: The server can be asked to exit when idle
The system SHALL accept an option that makes the server exit after a period during which no page has been subscribed to it and no external activity has been registered against it, and SHALL keep serving indefinitely when that option is absent.

The option exists so that ownership of the process is explicit. A server a person started in a terminal must not vanish under them; a server started on their behalf by a hook has no one waiting on it and needs a deadline.

#### Scenario: Idle exit is opt-in
- **WHEN** a user runs `openspec-doc serve` without the idle-exit option
- **THEN** the server SHALL keep serving regardless of how long it goes unused
