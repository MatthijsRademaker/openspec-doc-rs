## ADDED Requirements

### Requirement: Every release carries a binary for each supported platform

Each release SHALL include a prebuilt executable for every platform the repository states it supports, published as a downloadable asset with a checksum, and SHALL name the platforms it does not support rather than leaving them as an absence.

Installing this tool otherwise requires two toolchains and the knowledge that one has to run before the other, because the binary embeds an artifact built by the second. That is a reasonable ask of a contributor and an unreasonable one of a user. Naming the unsupported platforms matters for a smaller reason with a real cost: a reader on a platform that is not listed cannot otherwise tell whether they were considered and excluded or simply not thought of, and the two call for different responses from them.

#### Scenario: A release includes an executable per supported platform

- **WHEN** a release is published
- **THEN** it SHALL carry one executable asset for each supported platform, and a checksum for each

#### Scenario: The supported set is stated where a user chooses

- **WHEN** a user reads the installation instructions
- **THEN** the supported platforms SHALL be listed, and platforms outside the set SHALL be named as requiring a build from source

#### Scenario: Linux executables do not depend on the build machine's system C library

- **WHEN** a Linux executable from a release is run on a distribution older than the one that built it
- **THEN** it SHALL run, without depending on a system C library version present on the build machine

### Requirement: Every platform's executable embeds the same dashboard build

The release process SHALL build the embedded dashboard exactly once per release and SHALL use that single build for every platform's executable. A platform's build SHALL NOT produce its own copy.

The dashboard is a compile-time embedded artifact of a separate toolchain, so the obvious build matrix runs that toolchain once per platform and produces one dashboard per platform. Whether those would differ is unmeasured, and the point of building once is that it does not have to be measured: "every platform ships the same dashboard" becomes a property of how the artifacts were produced rather than an assumption about two build runs agreeing.

#### Scenario: All platform executables in a release embed one dashboard build

- **WHEN** a release's executables are produced
- **THEN** each SHALL have embedded the artifact from that release's single dashboard build

#### Scenario: A platform build without the dashboard artifact fails

- **WHEN** a platform's build starts without the dashboard artifact available
- **THEN** that build SHALL fail rather than producing its own copy or an executable without one

### Requirement: A release becomes current only once its executables are present

The release process SHALL withhold a release from being resolvable as the current release until every platform's executable and checksum has been attached, and SHALL make it current as the final step.

The installer resolves the current release rather than a version number, which is what lets a user install without knowing one. A release that becomes current when it is created is therefore current and empty for as long as its builds take, and every installation attempted in that window resolves to a release with nothing to download. Publishing last collapses "this release exists" and "this release can be installed" into one event, and has the useful side effect that a failed platform build leaves something nobody can install rather than a published release missing a platform.

#### Scenario: A release under construction is not resolved by the installer

- **WHEN** a release has been created and its executables have not yet been attached
- **THEN** it SHALL NOT be resolvable as the current release

#### Scenario: A release with a failed platform build does not become current

- **WHEN** any platform's build fails during a release
- **THEN** the release SHALL NOT become current

#### Scenario: The current release always has its executables

- **WHEN** the current release is resolved
- **THEN** every supported platform's executable and checksum SHALL be present on it

### Requirement: A one-line installer is distributed with the release it installs

The repository SHALL provide an installation script for POSIX shells and one for Windows, SHALL publish both as assets of each release, and SHALL document fetching them from the current release rather than from a branch. The script SHALL verify the checksum of what it downloaded, SHALL install without requiring administrative privileges, and SHALL report when the install location is not on the user's `PATH`.

Serving the installer from a branch pins every future installation to that branch's most recent commit, so a mistake in the script breaks installation for everyone from the moment it is pushed until someone reverts it — with no release boundary in between and no review the user benefits from. Publishing it as a release asset makes the installer move only when a release moves. The checksum check is not about the threat model, which a `curl`-piped script does not improve on; it is about truncated downloads, which are common, and about the difference between reporting a corrupt download and letting the user discover it as an unexplained failure to execute.

#### Scenario: The installer places a working executable

- **WHEN** the installation script is run on a supported platform
- **THEN** it SHALL download the executable for that platform from the current release, verify its checksum, and place it where the user can run it without administrative privileges

#### Scenario: A corrupt download is reported rather than installed

- **WHEN** the downloaded executable does not match its published checksum
- **THEN** the script SHALL report the mismatch and SHALL NOT install it

#### Scenario: An install location outside PATH is reported

- **WHEN** the script installs to a location that is not on the user's `PATH`
- **THEN** it SHALL say so and SHALL state how to add it

#### Scenario: An unsupported platform is refused with a reason

- **WHEN** the script runs on a platform with no published executable
- **THEN** it SHALL report which platforms are published and SHALL NOT install anything

#### Scenario: Re-running the installer upgrades in place

- **WHEN** the script is run on a machine that already has an installation
- **THEN** it SHALL replace it with the current release's executable

### Requirement: The installer reports an unmet runtime dependency rather than reporting success

The installation script SHALL check for the external commands the tool requires at run time, and SHALL report any that are missing as part of its result, naming what will not work. It SHALL NOT report a successful installation without qualification when a required command is absent, and SHALL NOT install that command itself.

Promotion shells out to the `openspec` command, and promotion is the step where an exploration becomes a change — the middle of the loop, not a peripheral feature. An installer that prints success and exits has installed something whose central workflow fails on first use, which is this repository's rule against silent defaults and fake success values broken at the first interaction a user has with the project. Installing the missing command is a different failure: it belongs to another ecosystem with its own package manager, and a script that starts bootstrapping those has stopped being the one-shot install it was described as.

#### Scenario: A missing runtime dependency is named in the result

- **WHEN** the script completes on a machine without the required external command
- **THEN** it SHALL name the missing command and what will not work, as part of its result rather than after it

#### Scenario: The installer does not install other ecosystems' tooling

- **WHEN** a required external command is missing
- **THEN** the script SHALL NOT attempt to install it or its package manager

#### Scenario: A satisfied dependency is not reported as a problem

- **WHEN** every required external command is present
- **THEN** the script SHALL report a successful installation without qualification

### Requirement: The platform-dependent limits of an unsigned executable are stated

The repository SHALL state, where a user chooses how to install, that executables are not code-signed and what that means on each affected platform.

The consequence is asymmetric in a way nobody expects: on macOS a binary fetched by the installation script runs normally, while the identical file downloaded through a browser is refused by the operating system. A user who meets that refusal without warning concludes the software is broken rather than unsigned, and the two conclusions lead to very different next actions.

#### Scenario: The browser-download caveat is stated next to the download

- **WHEN** a user reads where releases can be downloaded
- **THEN** the documentation SHALL state that executables are unsigned and that a browser download may be refused on macOS, naming the installation script as the path that works
