## MODIFIED Requirements

### Requirement: Command surface and help text
The system SHALL provide an `openspec-doc` binary with a documented top-level `--help` output listing all subcommands (`summary`, `serve`, `hook`, `comment`, `scratch`, `init`), and each subcommand SHALL provide its own `--help` output.

`init` is added by this change. `scratch` landed with the scratch-note workflow and was never added to this list; restating the requirement is what makes that omission visible. `add-setup-diagnostics` proposes the same repair alongside `doctor`, so whichever of the two lands second inherits a correct list and adds only its own subcommand.

#### Scenario: Top-level help lists all subcommands
- **WHEN** a user runs `openspec-doc --help` or `openspec-doc -h`
- **THEN** the output SHALL list `summary`, `serve`, `hook`, `comment`, `scratch`, and `init` with a one-line description for each

#### Scenario: Subcommand help is available
- **WHEN** a user runs `openspec-doc <subcommand> --help` for any of `summary`, `serve`, `hook`, `comment`, `scratch`, `init`
- **THEN** the system SHALL print that subcommand's specific usage and options without executing the subcommand's behavior
