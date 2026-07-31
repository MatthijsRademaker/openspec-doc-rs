## ADDED Requirements

### Requirement: OpenSpec project discovery
The CLI SHALL discover an OpenSpec project from a target root path by locating `openspec/config.yaml`.

#### Scenario: Target root omitted
- **WHEN** a user runs an initial command without `--root`
- **THEN** the CLI uses the current working directory as the target root

#### Scenario: Target root provided
- **WHEN** a user runs an initial command with `--root <path>`
- **THEN** the CLI resolves that path and scans the OpenSpec project beneath it

#### Scenario: Config missing
- **WHEN** the target root does not contain `openspec/config.yaml`
- **THEN** the CLI reports an actionable error that identifies the missing file path

### Requirement: OpenSpec metadata scanning
The system SHALL scan current specs, active changes, and archived changes from the target project's `openspec/` directory.

#### Scenario: Current specs are present
- **WHEN** files matching `openspec/specs/**/spec.md` exist
- **THEN** the scan result includes those files as current specs with stable relative paths

#### Scenario: Active changes are present
- **WHEN** files exist under `openspec/changes/<change-name>/`
- **THEN** the scan result includes those files as active change files unless the change is under `openspec/changes/archive/`

#### Scenario: Archived changes are present
- **WHEN** files exist under `openspec/changes/archive/**/`
- **THEN** the scan result includes those files as archived change files

### Requirement: OpenSpec Markdown metadata parsing
The system SHALL parse useful OpenSpec metadata from discovered Markdown files without requiring every file to be perfectly formed.

#### Scenario: Spec purpose exists
- **WHEN** a spec file contains a `## Purpose` section
- **THEN** the parsed spec includes the purpose text for summaries

#### Scenario: Requirements and scenarios exist
- **WHEN** a spec file contains `### Requirement:` headings and `#### Scenario:` headings
- **THEN** the parsed spec includes requirement and scenario counts

#### Scenario: Change artifacts exist
- **WHEN** an active change contains proposal, design, tasks, or spec delta files
- **THEN** the parsed change includes the presence of those artifacts

### Requirement: Summary command
The CLI SHALL provide a `summary` command that prints a read-only overview of a target OpenSpec project.

#### Scenario: Human summary requested
- **WHEN** a user runs `openspec-doc summary`
- **THEN** the CLI prints counts for current specs, active changes, archived changes, requirements, scenarios, and warnings

#### Scenario: JSON summary requested
- **WHEN** a user runs `openspec-doc summary --format json`
- **THEN** the CLI prints equivalent summary data as valid JSON

#### Scenario: Summary encounters warnings
- **WHEN** parsing finds non-fatal structural issues
- **THEN** the summary command includes warning counts without mutating the target project

### Requirement: Check command
The CLI SHALL provide a `check` command that reports OpenSpec structural warnings and exits with status suitable for automation.

#### Scenario: No warnings found
- **WHEN** a user runs `openspec-doc check` against a valid target project
- **THEN** the CLI reports a passing status and exits with code `0`

#### Scenario: Warnings found
- **WHEN** a user runs `openspec-doc check` and structural warnings are found
- **THEN** the CLI reports the warnings with affected paths and exits with code `1`

#### Scenario: Fatal discovery error
- **WHEN** a user runs `openspec-doc check` against a directory without `openspec/config.yaml`
- **THEN** the CLI reports the fatal discovery error and exits with code `1`

### Requirement: Initial structural warnings
The analyzer SHALL report clear warnings for common OpenSpec structure issues detected by the initial parser.

#### Scenario: Spec missing purpose
- **WHEN** a current spec file does not contain a `## Purpose` section
- **THEN** the analyzer reports a warning for that spec path

#### Scenario: Requirement missing scenario
- **WHEN** a requirement has no `#### Scenario:` headings before the next requirement
- **THEN** the analyzer reports a warning for that requirement

#### Scenario: Scenario heading level malformed
- **WHEN** a spec contains a scenario heading that does not use exactly four hash characters
- **THEN** the analyzer reports a warning that identifies the malformed scenario heading

#### Scenario: Active change missing expected artifacts
- **WHEN** an active change is missing expected proposal, design, or tasks files
- **THEN** the analyzer reports warnings for the missing artifacts
