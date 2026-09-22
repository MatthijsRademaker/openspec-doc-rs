## ADDED Requirements

### Requirement: A version is derived from commit history and lands through a release pull request

The repository SHALL derive each new version number from the conventional-commit types recorded since the previous release, SHALL propose the version bump and its generated changelog entries as a pull request rather than committing them directly, and SHALL create the release only when that pull request is merged.

The proposal step is the point of the design. Commit subjects become public release notes, so the pull request is the last moment at which a line can be corrected before anyone outside the repository reads it. Deriving the number from commit types rather than choosing it by hand is what keeps the version honest about what changed; proposing rather than committing is what keeps the prose honest.

#### Scenario: Commits since the last release produce a proposed version

- **WHEN** commits carrying releasable conventional-commit types land on the default branch
- **THEN** the system SHALL open or update a pull request containing the computed version bump and the changelog entries for those commits

#### Scenario: The proposal accumulates rather than releasing each commit

- **WHEN** several releasable commits land before the pull request is merged
- **THEN** they SHALL be collected into one proposed version, and no release SHALL be created for any of them individually

#### Scenario: Changelog text can be corrected before it is published

- **WHEN** the proposed changelog is edited in the pull request before merge
- **THEN** the released changelog SHALL contain the edited text

#### Scenario: Commits that change nothing releasable propose nothing

- **WHEN** only commits of non-releasable types land on the default branch
- **THEN** no version SHALL be proposed

#### Scenario: Merging the proposal creates the release

- **WHEN** the release pull request is merged
- **THEN** the system SHALL create a version tag and a corresponding release, and SHALL record the version in the repository's changelog

### Requirement: A release's build steps are reached without depending on a tag event

The repository SHALL condition any work that must happen for a release on the release step's own reported outcome, and SHALL NOT rely on a tag or release created by automation to trigger a separate workflow run.

A tag pushed by the automation's own credentials does not trigger tag-keyed workflows; the hosting platform suppresses it to stop workflows triggering one another without bound. A release pipeline built on that trigger does not fail — it silently omits everything after the tag, producing a tag and an empty release with no failed job to point at. The correct structure is one run whose later steps read the earlier step's output. Forcing the trigger with a longer-lived credential is not an acceptable substitute: it exchanges a structural guarantee for a secret.

#### Scenario: Work that follows a release runs in the same run that created it

- **WHEN** a release is created by the automation
- **THEN** every step that must accompany that release SHALL run in the same workflow run, conditioned on the release having been created

#### Scenario: No release step is triggered by the tag alone

- **WHEN** the repository's workflows are inspected
- **THEN** no release-producing work SHALL be triggered solely by a tag created by automation

### Requirement: The workspace is never published to a package registry

The repository SHALL configure every crate in the workspace to refuse publication to a package registry, in the crates' own manifests as well as in the release tooling's configuration.

This is not a preference about distribution channels; it is a defect that would otherwise be shippable. The server crate embeds the built dashboard at compile time, packaging for a registry excludes files the repository ignores, and the built dashboard is one of them. A published crate would therefore contain no dashboard and would fail to compile for whoever installed it. Recording the refusal in the manifests makes it a property of the workspace rather than of the release configuration, so it holds for a command typed by hand in a working directory as well as for the automation.

#### Scenario: A publish attempt is refused by the workspace itself

- **WHEN** a registry publish is attempted for any crate in the workspace, by automation or by hand
- **THEN** it SHALL be refused by the crate's own manifest

#### Scenario: A release is created without a registry publication

- **WHEN** a release is created
- **THEN** the tag, the release, and the changelog SHALL be produced, and no registry publication SHALL be attempted

### Requirement: The released software states the terms it may be used under

The repository SHALL contain a license file, and the workspace's package metadata SHALL name that license. A version SHALL NOT be released while either is absent.

A public repository with no license grants no rights: by default nobody may use, modify, or redistribute it. That has been harmless while the only way in was cloning the source. It stops being harmless the moment the project publishes a versioned artifact and invites a stranger to take it, because the invitation and the terms then contradict each other.

#### Scenario: The license is present and named in metadata

- **WHEN** the repository is inspected
- **THEN** it SHALL contain a license file, and the workspace package metadata SHALL name the same license

### Requirement: Version numbers are the product of a discipline the repository does not enforce

The repository SHALL state that conventional-commit typing is maintained by hand, SHALL NOT claim an automated check on commit message format that does not exist, and SHALL record the consequence of a mistyped commit.

An unenforced convention that is documented as enforced is worse than one documented as manual, because the reader stops checking. The exposure is specific and worth naming where someone will meet it: a commit typed as a feature when it is a fix, or as a fix when it breaks something, produces a version number that misdescribes the change and a changelog line that misdescribes it publicly, and neither is easily undone once a tag exists.

#### Scenario: The manual discipline is stated rather than implied

- **WHEN** a contributor reads how versions are produced
- **THEN** the documentation SHALL state that commit typing is not automatically verified, and SHALL name the release pull request as the point at which a mistake can still be corrected
