## ADDED Requirements

### Requirement: Review agent serve configuration
The browser preview server SHALL report whether live review agent analysis is enabled, explicitly disabled, or unavailable for the current session.

#### Scenario: Serve reports review agent enabled by default
- **WHEN** the server starts with default companion behavior
- **THEN** CLI output or server-rendered state MUST indicate that browser Pi review analysis is enabled and non-mutating

#### Scenario: Serve reports review agent explicitly disabled
- **WHEN** the server starts with live review agent analysis explicitly disabled
- **THEN** CLI output or server-rendered state MUST indicate that browser Pi review analysis is disabled

#### Scenario: Serve reports review agent unavailable
- **WHEN** the server cannot provide live Pi analysis because SDK setup, credentials, or model availability are missing
- **THEN** CLI output or server-rendered state MUST indicate that browser Pi review analysis is unavailable when relevant
