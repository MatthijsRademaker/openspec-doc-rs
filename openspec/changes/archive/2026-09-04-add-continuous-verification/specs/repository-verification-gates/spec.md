## ADDED Requirements

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
