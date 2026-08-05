## MODIFIED Requirements

### Requirement: Phase-verdict to directive translation
The system SHALL translate an untranslated phase-verdict record (keep-exploring, move-to-proposal, or comment-resolution) into the pending-directive file format at hook-stop time, using fixed reason-text templates that name relevant sidecar and artifact paths rather than embedding their content.

#### Scenario: Keep-exploring verdict translates to a pointer directive
- **WHEN** an untranslated "keep exploring" phase-verdict record exists for a session
- **THEN** the system SHALL write a pending directive whose reason text points to the session's scratch note and comment sidecar paths, and SHALL NOT embed the verdict's free-text notes directly in the reason text

#### Scenario: Move-to-proposal verdict translates to a pointer directive
- **WHEN** an untranslated "move to proposal" phase-verdict record exists for a session
- **THEN** the system SHALL write a pending directive whose reason text instructs formalizing the exploration into a proposal, pointing to the scratch note path

#### Scenario: A move-to-proposal directive points at the reviewer's comments
- **WHEN** an untranslated "move to proposal" phase-verdict record exists for a session
- **THEN** the reason text SHALL name the session's comment sidecar path and SHALL instruct that the open comments be accounted for in the proposal

The exploration note and the verdict record say what the reviewer concluded; the comment sidecar says which passages they wanted changed. A directive that names the first two and not the third sends the agent to formalize an exploration without telling it that anchored feedback on that exploration exists.

#### Scenario: Comment-resolution verdict translates to a pointer directive
- **WHEN** an untranslated "send to agent" comment-resolution phase-verdict record exists for a change
- **THEN** the system SHALL write a pending directive whose reason text points to the change's comment sidecar and instructs addressing open comments
