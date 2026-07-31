## MODIFIED Requirements

### Requirement: OpenSpec metadata scanning
The system SHALL scan current specs, active changes, and archived changes from the target project's `openspec/` directory for CLI summaries, checks, exports, and browser viewer rendering.

#### Scenario: Current specs are present
- **WHEN** files matching `openspec/specs/**/spec.md` exist
- **THEN** the scan result includes those files as current specs with stable relative paths

#### Scenario: Active changes are present
- **WHEN** files exist under `openspec/changes/<change-name>/`
- **THEN** the scan result includes those files as active change files unless the change is under `openspec/changes/archive/`

#### Scenario: Archived changes are present
- **WHEN** files exist under `openspec/changes/archive/**/`
- **THEN** the scan result includes those files as archived change files

#### Scenario: Viewer consumes scan results
- **WHEN** the browser viewer renders a target OpenSpec project
- **THEN** it uses the same scanned current spec, active change, and archived change paths as the summary and check commands

### Requirement: OpenSpec Markdown metadata parsing
The system SHALL parse useful OpenSpec metadata from discovered Markdown files for CLI summaries, checks, exports, and browser viewer rendering without requiring every file to be perfectly formed.

#### Scenario: Spec purpose exists
- **WHEN** a spec file contains a `## Purpose` section
- **THEN** the parsed spec includes the purpose text for summaries

#### Scenario: Requirements and scenarios exist
- **WHEN** a spec file contains `### Requirement:` headings and `#### Scenario:` headings
- **THEN** the parsed spec includes requirement and scenario counts

#### Scenario: Change artifacts exist
- **WHEN** an active change contains proposal, design, tasks, or spec delta files
- **THEN** the parsed change includes the presence of those artifacts

#### Scenario: Viewer consumes parsed metadata
- **WHEN** the browser viewer renders specs or changes
- **THEN** it uses parsed purpose, requirement, scenario, and artifact metadata rather than deriving those details from rendered HTML

### Requirement: Initial structural warnings
The analyzer SHALL report clear warnings for common OpenSpec structure issues detected by the initial parser and make those warnings available to CLI commands and the browser viewer.

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

#### Scenario: Viewer consumes analyzer warnings
- **WHEN** the browser viewer renders overview, index, detail, or warning pages
- **THEN** it uses analyzer warnings from the same analysis result as the summary and check commands
