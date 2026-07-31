## ADDED Requirements

### Requirement: Browser Pi review entrypoint
The review companion SHALL provide browser-native entrypoints for live Pi analysis by default unless review agent mode is explicitly disabled or unavailable.

#### Scenario: Ask Pi action appears by default
- **WHEN** a user opens an active change companion page during a default companion serve session
- **THEN** the page MUST show an Ask Pi action near the review guidance or comments overview

#### Scenario: Ask Pi action explains included context
- **WHEN** the user prepares to start Pi analysis from the companion page
- **THEN** the page MUST summarize whether open feedback, selected feedback, artifact snapshots, and warnings will be sent to Pi

#### Scenario: Disabled state appears when opted out
- **WHEN** a user opens an active change companion page while review agent analysis mode is explicitly disabled
- **THEN** the page MUST show a concise disabled state or omit the active Ask Pi control without disrupting existing comment creation and thread review behavior

#### Scenario: Unavailable state appears when Pi cannot start
- **WHEN** a user opens an active change companion page and Pi review analysis is unavailable because setup, credentials, or model availability are missing
- **THEN** the page MUST show an actionable unavailable state without disrupting existing comment creation and thread review behavior

### Requirement: In-browser Pi review transcript
The review companion SHALL render live Pi analysis as an in-browser review transcript.

#### Scenario: Transcript displays streaming response
- **WHEN** the browser receives Pi review text deltas
- **THEN** the companion page MUST append them to the active review transcript in reading order

#### Scenario: Transcript displays completion state
- **WHEN** a Pi review session completes successfully
- **THEN** the companion page MUST show that the analysis is complete and keep the response visible

#### Scenario: Transcript displays failure state
- **WHEN** a Pi review session cannot start or fails while running
- **THEN** the companion page MUST show an actionable error message without losing existing review comments

#### Scenario: Transcript offers retry after safe failure
- **WHEN** a Pi review session fails and retry is safe
- **THEN** the companion page MUST provide a retry action that starts a new session from the current review context and preserves the failed transcript for reference

### Requirement: Browser follow-up messages
The review companion SHALL allow the user to continue an active browser Pi review session with follow-up instructions when analysis mode is enabled.

#### Scenario: Follow-up sent after initial response
- **WHEN** a completed or idle Pi review session exists for the active change and the user submits a follow-up instruction
- **THEN** the companion page MUST send the message to the same review session and render the resulting streamed response

#### Scenario: Follow-up during running response
- **WHEN** the user submits a follow-up while the Pi review session is still streaming
- **THEN** the system MUST either queue the follow-up using Pi's supported streaming behavior or show a clear message that the user must wait
