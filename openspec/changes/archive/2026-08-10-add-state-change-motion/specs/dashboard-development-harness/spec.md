## ADDED Requirements

### Requirement: The mock lane can originate an artifact change

The mock development lane SHALL be able to originate a change to a scope's artifact content and emit it on the live-update channel, independently of any reviewer mutation, and SHALL make that trigger reachable by a contributor working in the browser.

The mock lane's event channel currently broadcasts only as a side effect of the reviewer's own actions — posting a comment, replying, setting a status. The path it cannot exercise is the one that matters most: an agent rewriting an artifact on disk while the reviewer reads it. Without a way to originate that, the behaviour on arrival of external content can be built but not seen, and a contributor's only way to check it is to run a real agent against a real repository.

#### Scenario: An artifact rewrite can be originated without a mutation

- **WHEN** a contributor triggers an artifact change in the mock lane
- **THEN** the mock scope's artifact content SHALL change, the live-update channel SHALL emit, and no comment, reply, status, or verdict SHALL have been recorded

#### Scenario: The trigger is reachable from the browser

- **WHEN** a contributor is running the mock lane
- **THEN** the trigger SHALL be reachable without editing fixture source and restarting the server

#### Scenario: The trigger is absent from the production build

- **WHEN** the frontend production build completes
- **THEN** no mock trigger, fixture, or handler SHALL be present in the built output
