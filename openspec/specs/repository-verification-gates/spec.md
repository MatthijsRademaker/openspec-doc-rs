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

### Requirement: The gates run on every change, not only when a developer runs them

The repository SHALL run its verification gates automatically on every push to the default branch and on every pull request, and SHALL run them by invoking the repository's own gate entry point rather than by restating the commands that entry point runs.

A gate that only runs when someone remembers to run it reports on the commits that person remembered, which is not the same set as the commits that were pushed. Invoking the entry point rather than a copy of its contents is what keeps the two definitions from becoming two: a gate added to the Makefile and not to the automation would otherwise leave the automation green for a surface nobody is checking, which is the same omission this capability already forbids, wearing a passing badge.

#### Scenario: A change that fails a gate fails automatically

- **WHEN** a commit that fails the workspace's format, lint, or test gate is pushed or proposed
- **THEN** the automated run SHALL fail without a developer having run anything

#### Scenario: A gate added to the entry point is run automatically without editing the automation

- **WHEN** a new command is added to a Makefile gate target
- **THEN** the automated run SHALL execute it, and the automation SHALL NOT require a separate edit naming that command

#### Scenario: The automated run reproduces the local verdict

- **WHEN** the gates pass on a supported platform on a clean checkout
- **THEN** the automated run for the same contents on that platform SHALL also pass

### Requirement: The platforms the gates are run on are stated, and an uncovered platform is a stated omission

The repository SHALL record which platforms its automated gate runs cover. A platform the repository distributes a binary for and does not run its gates on SHALL be recorded as an omission, with the reason, and SHALL NOT be represented as covered.

The gap this closes is specific. The workspace's test suite is very nearly platform-agnostic — a handful of tests are compiled out on non-Unix targets — so an automated run on an unprepared platform passes while silently omitting exactly the tests covering the code that platform breaks. A passing run that omitted the relevant tests is a stronger false claim than no run at all, and the only defence is that the covered set is written down rather than inferred from a workflow file.

#### Scenario: A platform whose tests are partly compiled out is not claimed as covered

- **WHEN** a platform would run the suite with some tests excluded by conditional compilation
- **THEN** the repository SHALL record it as not covered, naming the excluded tests, rather than adding a run that reports success

#### Scenario: Coverage and support move together

- **WHEN** a platform is added to the set the repository distributes a binary for
- **THEN** the automated gate run SHALL cover that platform in the same change, or the change SHALL record why it does not

### Requirement: The embedded frontend artifact is built once per automated run

The automated gate run SHALL build the frontend artifact the Rust workspace embeds exactly once and SHALL provide that same artifact to every platform running the workspace's gates. A platform leg SHALL NOT build its own copy.

`web/dist/` is gitignored and embedded at compile time, so every leg needs one and the naive lane builds one per leg. Two builds of one commit can differ — a type-checker, a bundler, and a dependency tree resolved per runner are three places for that to happen — so per-leg building quietly replaces "these platforms embed the same dashboard" with "these platforms each embedded something built from the same source". The distinction is nearly free to keep here and load-bearing once the same shape carries released binaries.

#### Scenario: Every platform leg embeds the same frontend build

- **WHEN** the gates run on more than one platform for one commit
- **THEN** each platform SHALL have embedded the artifact produced by the single frontend build for that run

#### Scenario: A missing frontend artifact fails the leg

- **WHEN** a platform leg starts without the frontend artifact available
- **THEN** that leg SHALL fail rather than building a replacement or compiling without one

### Requirement: The compiler version is pinned by the repository

The repository SHALL pin the Rust toolchain version it is built and verified with, in a file at the repository root, and SHALL NOT resolve it to whatever the running machine's default channel currently points at.

An unpinned toolchain changes what the gates mean roughly every six weeks with no commit to attribute the change to. The usual symptom is a new lint failing an unrelated pull request. Pinning does not prevent that failure; it moves it into a one-line commit that says what it is.

#### Scenario: The pinned toolchain is used by the automated run

- **WHEN** the gates run automatically
- **THEN** they SHALL use the toolchain version the repository pins

#### Scenario: A toolchain upgrade is a change to the repository

- **WHEN** the toolchain version changes
- **THEN** it SHALL change by an edit to the pinning file, and the gates SHALL run against the new version as part of that change

