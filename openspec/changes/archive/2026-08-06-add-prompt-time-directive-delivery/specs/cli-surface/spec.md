## MODIFIED Requirements

### Requirement: Command surface and help text
The system SHALL provide an `openspec-doc` binary with a documented top-level `--help` output listing all subcommands (`summary`, `serve`, `hook`, `comment`, `scratch`), and each subcommand SHALL provide its own `--help` output.

#### Scenario: Top-level help lists all subcommands
- **WHEN** a user runs `openspec-doc --help` or `openspec-doc -h`
- **THEN** the output SHALL list `summary`, `serve`, `hook`, `comment`, and `scratch` with a one-line description for each

#### Scenario: Subcommand help is available
- **WHEN** a user runs `openspec-doc <subcommand> --help` for any of `summary`, `serve`, `hook`, `comment`, `scratch`
- **THEN** the system SHALL print that subcommand's specific usage and options without executing the subcommand's behavior

### Requirement: Root resolution
The system SHALL accept a global `--root <path>` option on every subcommand and, when omitted, SHALL resolve the project root from the current working directory using the same discovery rules as the `project-scanner` capability.

#### Scenario: Explicit root override
- **WHEN** a user runs any subcommand with `--root <path>`
- **THEN** the system SHALL use `<path>` as the project root without walking up from the current directory

#### Scenario: Root resolution failure is reported clearly
- **WHEN** no `--root` is given and no ancestor of the current working directory contains `openspec/config.yaml`
- **THEN** the system SHALL exit with a non-zero status and print an error message identifying that no OpenSpec project was found, without a stack trace or panic

#### Scenario: Root resolution failure does not refuse a prompt
- **WHEN** the invoked subcommand is `hook prompt` and the project root cannot be resolved
- **THEN** the system SHALL report the failure on standard error and exit zero, emitting nothing on standard output

Root resolution happens before subcommand dispatch, so without this exception it sits outside `hook prompt`'s fail-soft guarantee and can refuse the reviewer's prompt — the one outcome that command exists to avoid.

#### Scenario: An unparseable invocation of the prompt hook does not refuse a prompt
- **WHEN** arguments naming `hook prompt` fail to parse — an unknown flag, a missing `--agent`, an invalid agent value
- **THEN** the system SHALL exit zero and emit nothing on standard output, rather than exiting with the usual usage-error status

The usual status for a usage error is 2, and Claude Code treats a `UserPromptSubmit` hook exiting 2 as a block. A typo in a hook command string would otherwise refuse every prompt in the session and return usage text in place of an answer.

## REMOVED Requirements

### Requirement: Placeholder subcommands fail loudly
**Reason**: Scaffolding for a phase that ended. `serve`, `hook`, and `comment` are all implemented, so the requirement describes behaviour the system no longer has and must not have — a "not yet implemented" message from any of them would now be a defect. It survived because nothing prompts a spec to drop a requirement once the thing it guarded is built.

**Migration**: None. Each subcommand's behaviour is specified by its own capability: `dashboard-server`, `agent-hook-bridge`, `anchored-comments`, `scratch-note-workflow`.
