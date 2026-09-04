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

### Requirement: Resolve-all-and-approve control on change pages
The system SHALL render, on a change page holding comments that are `open` or `addressed`, a control that resolves all of them and approves the change in one submission, and SHALL render it instead of the plain approve control rather than alongside it. The control SHALL state, before it is submitted, how many comments it will resolve and in which statuses, because it is the one control in the interface that discards the reviewer's own outstanding feedback.

#### Scenario: The control renders only while feedback is outstanding
- **WHEN** a browser requests a change page with at least one `open` or `addressed` comment
- **THEN** the system SHALL render the resolve-all-and-approve control and SHALL NOT render the plain approve control

#### Scenario: A settled change offers plain approve only
- **WHEN** a browser requests a change page whose every comment is `resolved`, or which has none
- **THEN** the system SHALL NOT render the resolve-all-and-approve control

#### Scenario: The control names what it will resolve
- **WHEN** the resolve-all-and-approve control renders for a change with comments outstanding
- **THEN** it SHALL show how many comments it will resolve, broken down by `open` and `addressed`

#### Scenario: Submission settles the page
- **WHEN** a reviewer submits the resolve-all-and-approve control
- **THEN** the page SHALL show every comment as `resolved`, the counts updated, and the change as approved

#### Scenario: A partial failure is rendered, not swallowed
- **WHEN** the submission resolves the comments and the approval then fails
- **THEN** the system SHALL render that the comments were resolved and the change was not approved, with the reason

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
