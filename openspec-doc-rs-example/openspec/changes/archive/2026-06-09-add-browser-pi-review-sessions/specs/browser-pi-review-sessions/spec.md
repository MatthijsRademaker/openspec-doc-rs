## ADDED Requirements

### Requirement: Default browser Pi review agent mode
The system SHALL expose browser Pi review sessions by default for companion-enabled serve sessions and SHALL provide an explicit opt-out for live analysis.

#### Scenario: Review agent enabled by default
- **WHEN** the viewer server is running with default companion behavior
- **THEN** active change companion pages MUST expose an action to ask Pi to analyze the active change and selected or open feedback

#### Scenario: Review agent explicitly disabled
- **WHEN** the viewer server is running with live review agent analysis explicitly disabled
- **THEN** active change companion pages MUST show that Pi review analysis is disabled and MUST NOT expose a working browser action that starts a Pi session

#### Scenario: Review agent unavailable
- **WHEN** the viewer server cannot start Pi analysis because SDK setup, credentials, or model availability are missing
- **THEN** active change companion pages MUST show that Pi review analysis is unavailable without disabling existing comment creation and thread review behavior

### Requirement: Active-change-only session validation
The system SHALL create browser Pi review sessions only for safe, active OpenSpec changes in the served target project.

#### Scenario: Session requested for active change
- **WHEN** the browser requests a Pi review session for an active change with a safe change name
- **THEN** the server MUST validate the change and create a session request using artifacts and feedback from that active change

#### Scenario: Session requested for archived or missing change
- **WHEN** the browser requests a Pi review session for an archived, missing, or unsafe change name
- **THEN** the server MUST reject the request and MUST NOT start a Pi session

### Requirement: Review bundle creation
The system SHALL bundle active change context before sending a browser-initiated review request to Pi.

#### Scenario: Bundle includes review context
- **WHEN** a Pi review session is created from the browser
- **THEN** the bundle MUST include the active change name, included artifact paths and markdown snapshots, analyzer warnings related to the change, included feedback threads, anchor states, and the reviewer instruction

#### Scenario: Bundle records selected feedback scope
- **WHEN** the browser requests analysis for selected feedback records
- **THEN** the bundle MUST include the selected comment identifiers and MUST exclude unselected feedback unless the request explicitly asks for all open feedback

### Requirement: Live browser session streaming
The system SHALL stream normalized Pi review session events to the browser.

#### Scenario: Agent analysis streams text
- **WHEN** Pi emits assistant text while analyzing a browser review request
- **THEN** the browser MUST receive incremental text updates without waiting for the full run to complete

#### Scenario: Agent lifecycle is visible
- **WHEN** a Pi review session starts, uses tools, completes, fails, or is aborted
- **THEN** the browser MUST receive normalized lifecycle events suitable for rendering review-agent state

#### Scenario: Failed session can be retried
- **WHEN** a Pi review session fails before completion and retry is safe
- **THEN** the browser MUST show the failure status with a retry action that starts a new session from the current review context

#### Scenario: Retry not safe or unavailable
- **WHEN** a Pi review session fails and retry is not safe or cannot be offered
- **THEN** the browser MUST explain the failure and MUST NOT show a misleading retry action

### Requirement: Non-mutating browser analysis sessions
The system SHALL keep browser-triggered Pi review analysis non-mutating in this change.

#### Scenario: Pi analysis proposes updates
- **WHEN** Pi analyzes browser feedback and active change artifacts
- **THEN** Pi MAY describe proposed artifact updates, unresolved questions, and next actions in its response

#### Scenario: Pi analysis does not apply updates
- **WHEN** a browser-triggered Pi review analysis session runs
- **THEN** the system MUST NOT expose an apply endpoint and MUST NOT modify OpenSpec artifacts or feedback statuses as a result of the session

### Requirement: Session recovery after browser refresh
The system SHALL retain enough review agent session information for the browser to recover recent analysis after a refresh.

#### Scenario: Completed session is listed
- **WHEN** the browser reloads an active change companion page after a Pi review session completed
- **THEN** the page MUST be able to show recent session status and persisted response content for that change

#### Scenario: Running session cannot be recovered
- **WHEN** the server no longer has an in-memory running Pi session after a browser refresh or server restart
- **THEN** the page MUST show a clear unavailable or stale state instead of pretending the session is still streaming
