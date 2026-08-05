## ADDED Requirements

### Requirement: Approval is an explicit reviewer act
The system SHALL record approval of a change only from an explicit `approved` verdict submitted for that change, and SHALL NOT infer approval from the absence of comments, from artifact completeness, or from task progress.

#### Scenario: Approval is recorded from an explicit verdict
- **WHEN** a reviewer submits an `approved` verdict for a change
- **THEN** the system SHALL append an approval record for that change

#### Scenario: A change nobody commented on is not approved
- **WHEN** a change's approval state is queried and it has no comments and no approval record
- **THEN** the system SHALL report it as not approved

#### Scenario: Completed artifacts do not imply approval
- **WHEN** a change has every expected artifact present and no approval record
- **THEN** the system SHALL report it as not approved

### Requirement: All feedback resolved before approval
The system SHALL refuse an `approved` verdict for a change with any comment whose current status is `open` or `addressed`, and SHALL report which statuses blocked it. `addressed` does not satisfy the precondition, because it records the agent's claim that work was done rather than the reviewer's acceptance of it.

#### Scenario: Approval with open comments is refused
- **WHEN** an `approved` verdict is submitted for a change with at least one `open` comment
- **THEN** the system SHALL refuse the verdict, SHALL NOT append an approval record, and SHALL report the outstanding count

#### Scenario: Approval with addressed but unresolved comments is refused
- **WHEN** an `approved` verdict is submitted for a change whose comments are all `addressed` and none `resolved`
- **THEN** the system SHALL refuse the verdict and SHALL report that addressed comments still require the reviewer's resolution

#### Scenario: Approval with all comments resolved succeeds
- **WHEN** an `approved` verdict is submitted for a change whose every comment is `resolved`
- **THEN** the system SHALL append the approval record

#### Scenario: Approval of a change with no comments succeeds
- **WHEN** an `approved` verdict is submitted for a change with no comments at all
- **THEN** the system SHALL append the approval record, because the precondition is that no feedback is outstanding rather than that feedback exists

### Requirement: Approval binds to the artifacts it approved
The system SHALL record, with each approval, a fingerprint of the change's reviewed artifact content, and SHALL report an approval as stale once the current fingerprint differs. The fingerprint SHALL cover `proposal.md`, `design.md`, and every spec delta, and SHALL NOT cover `tasks.md`, whose checkbox churn during implementation is expected progress rather than a change to what was reviewed.

#### Scenario: An unchanged change stays approved
- **WHEN** the approval state is queried and no reviewed artifact has changed since approval
- **THEN** the system SHALL report the change as approved

#### Scenario: Editing a reviewed artifact makes the approval stale
- **WHEN** `proposal.md`, `design.md`, or any spec delta changes after an approval was recorded
- **THEN** the system SHALL report the approval as stale, naming which artifacts changed, and SHALL NOT report the change as approved

#### Scenario: Ticking a task does not make the approval stale
- **WHEN** only `tasks.md` changes after an approval was recorded
- **THEN** the system SHALL continue to report the change as approved

#### Scenario: Re-approving after a change clears staleness
- **WHEN** an `approved` verdict is submitted for a change whose prior approval was stale
- **THEN** the system SHALL append a new approval record carrying the current fingerprint, and SHALL report the change as approved

### Requirement: Approval can be withdrawn
The system SHALL accept an explicit withdrawal of approval for a change, so that a reviewer need not edit an artifact to force staleness.

#### Scenario: Withdrawal returns the change to not approved
- **WHEN** approval is withdrawn for an approved change
- **THEN** the system SHALL report the change as not approved

#### Scenario: Withdrawal preserves the record
- **WHEN** approval is withdrawn
- **THEN** the system SHALL append the withdrawal rather than removing the prior approval record

### Requirement: Approval state is queryable
The system SHALL report a change's approval state as one of approved, stale, or not approved, together with the reason, via the CLI, so that an apply workflow can run it as a precheck.

#### Scenario: CLI reports approval state
- **WHEN** a user runs the approval-state subcommand for a change
- **THEN** the system SHALL print whether it is approved, stale, or not approved, and the reason

#### Scenario: A non-approved state exits non-zero
- **WHEN** the approval-state subcommand runs for a change that is stale or not approved
- **THEN** the system SHALL exit with a non-zero status, so that a precheck fails loudly rather than being read as success

#### Scenario: An unknown change fails loudly
- **WHEN** the approval-state subcommand names a change that does not exist
- **THEN** the system SHALL exit with a non-zero status and an error, rather than reporting it as not approved

### Requirement: Unapproved implementation is detected at turn end
The system SHALL detect, at `hook stop`, that a change has task progress recorded while its approval state is stale or not approved, and SHALL raise this through the pending-directive mechanism. This detection is after the fact by construction: a Stop hook runs at turn end and can only block-and-continue or allow-stop, so it cannot prevent implementation that has already occurred.

#### Scenario: Task progress without approval raises a directive
- **WHEN** `hook stop` runs for a session whose change has at least one completed task and no current approval
- **THEN** the system SHALL write a pending directive reporting that implementation has progressed on an unapproved change, naming the change and the approval-state command to run

#### Scenario: Task progress with a current approval raises nothing
- **WHEN** `hook stop` runs for a session whose change has completed tasks and a current, non-stale approval
- **THEN** the system SHALL NOT write a directive on account of approval state

#### Scenario: Detection does not repeat for the same unapproved state
- **WHEN** `hook stop` runs again for a change already reported as implemented without approval, with no intervening approval or further task progress
- **THEN** the system SHALL NOT write a second directive for the same state

#### Scenario: The directive is a pointer, not an embed
- **WHEN** the unapproved-implementation directive is written
- **THEN** its reason text SHALL identify itself as originating from this project's dashboard and SHALL name the change and the command to query approval state, rather than embedding comment or artifact content
