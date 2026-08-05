## ADDED Requirements

### Requirement: Per-comment thread controls
The system SHALL render, for each comment on a session or change page, controls to reply to it, to mark it resolved, and to reopen it. The controls offered SHALL reflect the comment's current status, so that a resolved comment offers reopen rather than resolve.

#### Scenario: An open comment offers reply and resolve
- **WHEN** a page renders a comment whose current status is `open` or `addressed`
- **THEN** the system SHALL render a reply control and a resolve control for that comment

#### Scenario: A resolved comment offers reopen
- **WHEN** a page renders a comment whose current status is `resolved`
- **THEN** the system SHALL render a reopen control for that comment, and SHALL NOT render a resolve control

#### Scenario: Submitting a thread control persists via anchored-comments
- **WHEN** a reviewer submits a reply, resolve, or reopen control for a comment
- **THEN** the system SHALL persist the corresponding event via the anchored-comments capability, and SHALL NOT modify the commented artifact

#### Scenario: The dashboard offers no control that marks a comment addressed
- **WHEN** a page renders any comment
- **THEN** the system SHALL NOT render a control that moves that comment to `addressed`, because that status asserts work an agent did rather than a judgement the reviewer makes

### Requirement: Status counts rendered per scope
The system SHALL render, on each session and change page, the scope's count of `open`, `addressed`, and `resolved` comments.

#### Scenario: Counts render alongside the comment list
- **WHEN** a browser requests a session or change page
- **THEN** the system SHALL render the scope's current open, addressed, and resolved counts

#### Scenario: Counts update without a reload
- **WHEN** a comment's status changes while a browser tab has the page open
- **THEN** the tab SHALL reflect the updated counts without the user reloading the page

### Requirement: Thread state updates live
The system SHALL push a status or reply change to the scope's event stream so that an open page reflects it without a reload, on the same channel comment creation already uses.

#### Scenario: A status change in one tab appears in another
- **WHEN** a comment is resolved or reopened while a second browser tab has the same page open
- **THEN** the second tab SHALL reflect the comment's new status without the user reloading the page

#### Scenario: A reply in one tab appears in another
- **WHEN** a reply is added to a comment while a second browser tab has the same page open
- **THEN** the second tab SHALL reflect the new reply without the user reloading the page
