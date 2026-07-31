## ADDED Requirements

### Requirement: Pi-assisted browser review remains approval-gated
Pi-assisted browser review SHALL distinguish live analysis from intentional artifact mutation.

#### Scenario: Browser analysis suggests artifact changes
- **WHEN** Pi produces proposed updates during a browser review session
- **THEN** the output MUST be treated as review guidance and MUST NOT be applied to `proposal.md`, `design.md`, `tasks.md`, or spec delta files automatically

#### Scenario: Existing prompt workflow remains available
- **WHEN** browser Pi review sessions are added
- **THEN** the existing Pi prompt-template workflow for consuming sidecar feedback MUST remain available for users who prefer the Pi TUI handoff
