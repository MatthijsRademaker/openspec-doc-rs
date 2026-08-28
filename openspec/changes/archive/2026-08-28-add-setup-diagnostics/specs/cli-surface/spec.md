## MODIFIED Requirements

### Requirement: Command surface and help text
The system SHALL provide an `openspec-doc` binary with a documented top-level `--help` output listing all subcommands (`summary`, `serve`, `hook`, `comment`, `scratch`, `init`, `doctor`), and each subcommand SHALL provide its own `--help` output.

`doctor` is added by this change. `add-project-init` already corrected this list to include `init` and the long-missing `scratch`, so this restatement adds one name to an otherwise current requirement.

#### Scenario: Top-level help lists all subcommands
- **WHEN** a user runs `openspec-doc --help` or `openspec-doc -h`
- **THEN** the output SHALL list `summary`, `serve`, `hook`, `comment`, `scratch`, `init`, and `doctor` with a one-line description for each

#### Scenario: Subcommand help is available
- **WHEN** a user runs `openspec-doc <subcommand> --help` for any of `summary`, `serve`, `hook`, `comment`, `scratch`, `init`, `doctor`
- **THEN** the system SHALL print that subcommand's specific usage and options without executing the subcommand's behavior
