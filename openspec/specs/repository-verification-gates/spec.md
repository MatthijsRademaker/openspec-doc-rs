# repository-verification-gates Specification

## Purpose

The repository's own executable verification gates: which of its buildable surfaces must be covered, the Makefile entry point that reaches every one of them without a developer knowing each surface's toolchain, and the rule that a gate's verdict is a fact about the repository rather than about the machine running it. A surface that builds but is gated by nothing is recorded here as a deliberate omission or it is a defect; a test that passes only where the operating system hands it a convenient path shape is the same defect wearing a green tick, and is fixed by the test resolving the path rather than by the developer configuring an environment variable.

## Requirements
### Requirement: Every buildable surface is covered by an executable gate

The repository SHALL cover each of its buildable surfaces — the Rust workspace, the dashboard frontend, and the documentation site — with an executable verification gate, and each gate SHALL be reachable from the repository's Makefile without the developer knowing the surface's own toolchain commands.

#### Scenario: The documentation site is verified by the repository's check gate

- **WHEN** the repository's check gate is run
- **THEN** the documentation site SHALL be installed from its committed lockfile and built, and a build failure SHALL fail the gate

#### Scenario: A drifted documentation lockfile fails the gate

- **WHEN** the documentation site's lockfile no longer matches its manifest
- **THEN** the frozen install SHALL fail the gate rather than resolving a different dependency set

#### Scenario: A surface with no gate is a visible omission

- **WHEN** a buildable surface is added to the repository
- **THEN** it SHALL either be reachable from a Makefile gate or be recorded here as deliberately ungated

### Requirement: A gate's verdict comes from the repository, not the machine

The repository's verification gates SHALL produce the same verdict on every supported developer platform for the same repository contents. A test SHALL NOT depend on the shape of a path the operating system supplies, including a temporary directory reached through a symbolic link, and SHALL resolve such a path itself rather than requiring the developer to configure the environment.

#### Scenario: The workspace test suite passes on a clean checkout on any supported platform

- **WHEN** the workspace test suite is run on macOS or on Linux against an unmodified checkout
- **THEN** every test SHALL pass without the developer setting an environment variable

#### Scenario: A fixture path supplied through a symbolic link is resolved by the test

- **WHEN** a test hands a filesystem path to code that compares it against paths reported by the operating system
- **THEN** the test SHALL resolve that path to its canonical form first, so that a platform reporting the canonical path does not fail the comparison

#### Scenario: A platform-specific failure is diagnosed rather than annotated

- **WHEN** a test fails on one platform and passes on another
- **THEN** the cause SHALL be established and fixed, and the repository SHALL NOT carry a comment deferring the diagnosis to a lane that does not run

