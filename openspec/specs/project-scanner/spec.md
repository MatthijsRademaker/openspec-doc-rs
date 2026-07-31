## Purpose

Discover the structure of an OpenSpec project on disk: locate the project root by its `openspec/config.yaml`, and enumerate active changes, archived changes, and current capability specs.

## Requirements

### Requirement: Project root discovery
The system SHALL locate the project root containing `openspec/config.yaml` by walking up from a given starting directory, or by using an explicitly provided root path without walking.

#### Scenario: Explicit root is provided
- **WHEN** a caller provides an explicit root path containing `openspec/config.yaml`
- **THEN** the system SHALL use that path as the project root without searching parent directories

#### Scenario: No explicit root, config found by walking up
- **WHEN** no explicit root is provided and the starting directory does not contain `openspec/config.yaml` but an ancestor directory does
- **THEN** the system SHALL return the first ancestor directory, searching upward, that contains `openspec/config.yaml` as the project root

#### Scenario: No config.yaml found
- **WHEN** neither the starting directory nor any ancestor directory contains `openspec/config.yaml`
- **THEN** the system SHALL return an error identifying that no OpenSpec project was found, without guessing a root

### Requirement: Active and archived change discovery
The system SHALL enumerate active change directories under `openspec/changes/` excluding the `archive` directory, and separately enumerate archived change directories under `openspec/changes/archive/`.

#### Scenario: Active changes exclude archive directory
- **WHEN** `openspec/changes/` contains both change directories and an `archive` directory
- **THEN** the system SHALL list every non-`archive` directory as an active change and SHALL NOT list `archive` itself as an active change

#### Scenario: Archived changes are discovered separately
- **WHEN** `openspec/changes/archive/` contains one or more change directories
- **THEN** the system SHALL list each of them as an archived change, distinct from the active change list

#### Scenario: Missing changes directory
- **WHEN** `openspec/changes/` does not exist
- **THEN** the system SHALL return an empty active-changes list and an empty archived-changes list rather than an error

### Requirement: Current spec discovery
The system SHALL enumerate current specification directories under `openspec/specs/`.

#### Scenario: Specs directory has entries
- **WHEN** `openspec/specs/` contains one or more capability directories
- **THEN** the system SHALL list each as a current spec by its directory name

#### Scenario: Specs directory is missing or empty
- **WHEN** `openspec/specs/` does not exist or contains no entries
- **THEN** the system SHALL return an empty current-specs list rather than an error
