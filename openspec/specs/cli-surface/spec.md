## Purpose

The `openspec-doc` binary's command surface: its subcommands and help text, the global `--root` option that resolves a project root for every subcommand, and the `summary` subcommand that reports a project's discovered structure.

## Requirements

### Requirement: Command surface and help text
The system SHALL provide an `openspec-doc` binary with a documented top-level `--help` output listing all MVP subcommands (`summary`, `serve`, `hook`, `comment`), and each subcommand SHALL provide its own `--help` output.

#### Scenario: Top-level help lists all subcommands
- **WHEN** a user runs `openspec-doc --help` or `openspec-doc -h`
- **THEN** the output SHALL list `summary`, `serve`, `hook`, and `comment` with a one-line description for each

#### Scenario: Subcommand help is available
- **WHEN** a user runs `openspec-doc <subcommand> --help` for any of `summary`, `serve`, `hook`, `comment`
- **THEN** the system SHALL print that subcommand's specific usage and options without executing the subcommand's behavior

### Requirement: Root resolution
The system SHALL accept a global `--root <path>` option on every subcommand and, when omitted, SHALL resolve the project root from the current working directory using the same discovery rules as the `project-scanner` capability.

#### Scenario: Explicit root override
- **WHEN** a user runs any subcommand with `--root <path>`
- **THEN** the system SHALL use `<path>` as the project root without walking up from the current directory

#### Scenario: Root resolution failure is reported clearly
- **WHEN** no `--root` is given and no ancestor of the current working directory contains `openspec/config.yaml`
- **THEN** the system SHALL exit with a non-zero status and print an error message identifying that no OpenSpec project was found, without a stack trace or panic

### Requirement: Summary subcommand
The system SHALL provide a `summary` subcommand that prints the resolved project root, active change names, archived change names, and current spec names using the `project-scanner` capability.

#### Scenario: Summary reports discovered project structure
- **WHEN** a user runs `openspec-doc summary --root <path>` against a valid OpenSpec project
- **THEN** the system SHALL print the resolved root path, the list of active change names, the list of archived change names, and the list of current spec names

### Requirement: Placeholder subcommands fail loudly
The system SHALL expose `serve`, `hook`, and `comment` subcommands in the CLI surface and help output, and SHALL exit with a non-zero status and a clear "not yet implemented" message when invoked, rather than exiting successfully with no effect.

#### Scenario: Unimplemented subcommand exits non-zero with a clear message
- **WHEN** a user runs `openspec-doc serve`, `openspec-doc hook`, or `openspec-doc comment` before their implementing change lands
- **THEN** the system SHALL exit with a non-zero status and print a message stating the subcommand is not yet implemented

