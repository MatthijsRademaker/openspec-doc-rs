## MODIFIED Requirements

### Requirement: The commands to probe come from the shared definition of a wired project
The system SHALL enumerate the hooks to check from the same module `init` writes them from, SHALL recognise this tool's entries in `.claude/settings.json` by the same predicate `init` uses, and SHALL execute each hook as recorded there — spawning the executable it names with the arguments it carries, without a shell — rather than the canonical definition.

One definition, read by both commands. Two copies drift, and the drift presents as `doctor` passing a project `init` would rewrite. The recorded entry is what runs, not the canonical one: a developer pointing an entry at a local build is running that binary, and probing the canonical form would check a binary the agent never invokes.

Spawning directly is what makes the probe faithful on every platform rather than on Unix only. The agent hands a single-string command to a shell whose identity depends on the platform and on what is installed; an argument vector it spawns itself. A probe that spawned a shell would be reproducing an execution path the agent no longer takes, and on Windows would reproduce one the agent is currently unable to take correctly at all.

#### Scenario: A locally built binary is probed as recorded
- **WHEN** a registered hook's entry names a binary by absolute path
- **THEN** the system SHALL spawn that binary with the recorded arguments and report on it

#### Scenario: The probe spawns no shell
- **WHEN** any hook is probed
- **THEN** the system SHALL spawn the recorded executable directly, and SHALL NOT invoke a shell to interpret the command

#### Scenario: An entry that cannot be executed without a shell is reported as unchecked
- **WHEN** a registered entry carries its arguments as a single string, as an earlier version wrote them
- **THEN** the system SHALL report that hook as failing, naming the command that rewrites it, rather than attempting to split the string itself

#### Scenario: Every hook in the shared definition is enumerated
- **WHEN** `doctor` runs
- **THEN** the report SHALL cover every hook the shared definition contains, naming any of them that is not registered

### Requirement: The binary under check is identified
The system SHALL report the path and version of the `openspec-doc` resolved on `PATH` and of the running executable, SHALL resolve it using the executable file naming the running platform uses, and SHALL report a failing check when the two differ.

An operator working in this repository routinely has an installed copy on `PATH` and a newer one under `target/`. The agent runs whichever `PATH` resolves, so a check run from the newer binary that reports on the older one's behalf is answering the wrong question.

Resolution has to account for the platform's executable suffix. Searching each `PATH` entry for a file named exactly `openspec-doc` finds nothing on a platform whose executables end in `.exe`, and the resulting report — no binary on `PATH`, therefore a failing check, therefore every hook depends on something absent — is wrong about a correct installation, in the command whose entire purpose is to be trusted about that.

#### Scenario: A different binary on PATH is reported
- **WHEN** the `openspec-doc` on `PATH` is not the running executable
- **THEN** the system SHALL print both paths with both versions and SHALL exit non-zero

#### Scenario: An installed binary is found on a platform whose executables carry a suffix
- **WHEN** the check runs on a platform where executables carry a filename suffix and an `openspec-doc` executable is present on `PATH`
- **THEN** the system SHALL resolve it and report its path and version, and SHALL NOT report it as absent

#### Scenario: No binary on PATH is reported
- **WHEN** no `openspec-doc` resolves on `PATH`
- **THEN** the system SHALL report that as a failing check, since every hook command depends on it

## ADDED Requirements

### Requirement: The agent's version is checked against the form the hooks are registered in
The system SHALL resolve the coding agent's own executable, SHALL report its version, and SHALL report a failing check when that version is older than the one required to execute a hook entry's arguments as a vector.

Every other check in this capability asks whether this tool's side of the wiring works. This one asks whether the other side can read what was written, which became a real question the moment registration stopped being a plain command line. An agent below the floor does not refuse the entry — it runs the executable with no arguments, exits on a usage error, and the agent reads that exit as a refusal of the user's prompt. Every prompt in the session is blocked and nothing on screen says why. A silent, total failure caused by a version mismatch is exactly what a diagnostic command is for.

#### Scenario: An agent below the required version fails the check
- **WHEN** the resolved agent's version is older than the one that executes argument vectors
- **THEN** the system SHALL report a failing check naming the installed version, the required version, and how to upgrade, and SHALL exit non-zero

#### Scenario: An agent that cannot be resolved is reported as unchecked
- **WHEN** no agent executable resolves on `PATH`
- **THEN** the system SHALL report the version check as not checked, stating that the agent was not found, rather than passing or failing it

#### Scenario: A supported agent version passes
- **WHEN** the resolved agent is at or above the required version
- **THEN** the system SHALL report the check as passing and SHALL name the version it found
