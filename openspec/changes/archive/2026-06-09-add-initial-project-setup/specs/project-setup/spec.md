## ADDED Requirements

### Requirement: npm package metadata
The project SHALL define npm package metadata for an installable CLI package named `openspec-doc`.

#### Scenario: Package identifies CLI entrypoint
- **WHEN** a developer inspects package metadata
- **THEN** the metadata declares a `bin` entry named `openspec-doc` that points to the built CLI entrypoint

#### Scenario: Package limits published files
- **WHEN** the package is prepared for publication
- **THEN** package metadata limits the published runtime files to the built output and essential documentation

### Requirement: TypeScript build setup
The project SHALL compile TypeScript source files into distributable JavaScript output.

#### Scenario: Build script emits distribution files
- **WHEN** a developer runs the package build script
- **THEN** TypeScript source files are compiled into `dist/`

#### Scenario: Type check script validates source
- **WHEN** a developer runs the type-check script
- **THEN** the project validates TypeScript types without emitting runtime output

### Requirement: Layered source boundaries
The project SHALL organize initial implementation source into separate CLI and core layers.

#### Scenario: CLI delegates to core modules
- **WHEN** command handlers need OpenSpec project metadata
- **THEN** they call scanner, parser, and analyzer modules instead of reading project files inline

#### Scenario: Core modules avoid command output concerns
- **WHEN** scanner, parser, or analyzer modules return results
- **THEN** they return typed data structures without printing user-facing CLI output

### Requirement: Development validation scripts
The project SHALL provide repository scripts for repeatable build, type-check, test, lint, format-check, and aggregate validation workflows.

#### Scenario: Contributor validates the project
- **WHEN** a contributor runs the aggregate validation script
- **THEN** the project runs the relevant build, type-check, lint or format-check, and test steps

### Requirement: Automated test foundation
The project SHALL include automated tests and fixtures for initial scanner, parser, analyzer, and CLI behavior.

#### Scenario: Tests use fixture projects
- **WHEN** tests validate OpenSpec parsing or analysis behavior
- **THEN** they use fixture directories rather than relying only on this repository's own `openspec/` directory

### Requirement: Development documentation
The project SHALL document how to install dependencies, run validation, build the CLI, and execute initial commands locally.

#### Scenario: New contributor follows setup instructions
- **WHEN** a new contributor reads the repository documentation
- **THEN** they can identify the commands needed to install dependencies, build, test, and run the CLI against a target project
