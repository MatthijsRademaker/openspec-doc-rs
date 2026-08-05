## ADDED Requirements

### Requirement: Approval control on change pages
The system SHALL render, on a change page, a control submitting the `approved` verdict, and a control withdrawing approval when the change is currently approved.

#### Scenario: Approve control renders when feedback is settled
- **WHEN** a browser requests a change page whose every comment is `resolved`, or which has no comments, and which is not currently approved
- **THEN** the system SHALL render an approve control

#### Scenario: Approve control is refused while feedback is outstanding
- **WHEN** a reviewer submits the approve control for a change with `open` or `addressed` comments
- **THEN** the system SHALL refuse the verdict and SHALL render why, rather than recording an approval

#### Scenario: Withdraw control renders for an approved change
- **WHEN** a browser requests a change page for a currently approved change
- **THEN** the system SHALL render a control withdrawing that approval

### Requirement: Approval state rendered on change pages
The system SHALL render a change's current approval state — approved, stale, or not approved — with its reason, on the change page.

#### Scenario: Approval state renders
- **WHEN** a browser requests a change page
- **THEN** the system SHALL render whether the change is approved, stale, or not approved, and the reason

#### Scenario: A stale approval renders as stale, not approved
- **WHEN** a change page is requested for a change whose reviewed artifacts changed after approval
- **THEN** the system SHALL render it as stale and SHALL name the artifacts that changed

#### Scenario: Approval state updates without a reload
- **WHEN** a change's approval state changes while a browser tab has its page open
- **THEN** the tab SHALL reflect the new state without the user reloading the page
