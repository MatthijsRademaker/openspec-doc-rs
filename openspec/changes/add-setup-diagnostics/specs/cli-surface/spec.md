## MODIFIED Requirements

### Requirement: Command surface and help text
The system SHALL provide an `openspec-doc` binary with a documented top-level `--help` output listing all subcommands (`summary`, `serve`, `hook`, `comment`, `scratch`, `doctor`), and each subcommand SHALL provide its own `--help` output.

`doctor` is added by this change. `scratch` landed with `add-scratch-note-workflow` and was never added to this list; restating the requirement is what makes that omission visible, and leaving a known-wrong subcommand list in a requirement being edited is not a smaller change, only a worse one.

#### Scenario: Top-level help lists all subcommands
- **WHEN** a user runs `openspec-doc --help` or `openspec-doc -h`
- **THEN** the output SHALL list `summary`, `serve`, `hook`, `comment`, `scratch`, and `doctor` with a one-line description for each

#### Scenario: Subcommand help is available
- **WHEN** a user runs `openspec-doc <subcommand> --help` for any of `summary`, `serve`, `hook`, `comment`, `scratch`, `doctor`
- **THEN** the system SHALL print that subcommand's specific usage and options without executing the subcommand's behavior
