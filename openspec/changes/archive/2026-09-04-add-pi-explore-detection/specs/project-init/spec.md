## MODIFIED Requirements

### Requirement: pi.dev extension installation
The system SHALL, when pi.dev is selected, write `.pi/extensions/openspec-doc-hook.ts` from a copy of the extension embedded in the binary.

There is one copy of that file, in the binary. Every project's copy is its output, so a divergence appears as a diff rather than as a defect in a project nobody is looking at.

#### Scenario: The extension is written from the embedded copy
- **WHEN** `init --yes` configures pi.dev in a project with no `.pi/extensions/` directory
- **THEN** the system SHALL create that directory and write `openspec-doc-hook.ts` into it with the embedded contents

#### Scenario: An existing extension is replaced
- **WHEN** `init --yes` configures pi.dev in a project whose `.pi/extensions/openspec-doc-hook.ts` differs from the embedded copy
- **THEN** the system SHALL replace the file with the embedded copy and report it as modified

#### Scenario: No explore gap is reported for pi
- **WHEN** `init` plans or performs a pi.dev configuration
- **THEN** the output SHALL NOT warn that pi sessions produce no exploration note, because the installed extension captures them
