# setup-diagnostics Specification

## Purpose

The `doctor` subcommand: whether this project's wiring actually runs. `init` writes the configuration and reports whether it is written; nothing else reports whether the agent can run it, and a written configuration that does not run fails the way everything here fails — by the absence of an event, which is indistinguishable from an idle project. So every hook check executes the command string recorded in the settings file, verbatim, and requires the answer that hook exists to give. The hooks to check and the predicate that recognises them come from the same definition `init` writes from, because two definitions drift and the drift presents as `doctor` passing a project `init` would rewrite. Every probe runs against a throwaway project, never this one: the turn-boundary hook consumes a pending directive, and a check that ate the reviewer's outstanding feedback would cause the failure it exists to diagnose. Anything not examined gets a line saying so, because a green line for something never looked at is the failure mode this capability exists to remove.

## Requirements

### Requirement: A doctor subcommand reports whether the configured hooks run
The system SHALL provide a `doctor` subcommand that runs each configured hook, prints one line per check identifying the check and its outcome, and exits non-zero when any check fails.

`init` reports whether the configuration is written. Nothing else reports whether it runs, and a written configuration that does not run fails the way everything here fails — by the absence of an event, which is indistinguishable from an idle project.

#### Scenario: A working install passes
- **WHEN** a user runs `openspec-doc doctor` in a project whose hooks are registered and whose binary responds
- **THEN** the system SHALL print a passing line for each check and SHALL exit zero

#### Scenario: An unregistered hook fails the command
- **WHEN** a user runs `openspec-doc doctor` in a project whose `.claude/settings.json` has no entry for one of the three hooks
- **THEN** the system SHALL print a failing line naming that hook, stating that `openspec-doc init` registers it, and SHALL exit non-zero

#### Scenario: Every check is reported, not only the failing ones
- **WHEN** any check fails
- **THEN** the system SHALL still print a line for every other check, so the report is a complete picture rather than the first problem found

### Requirement: A hook check is settled by running the hook
The system SHALL check each registered hook by executing the command string recorded in settings and requiring a response specific to that hook, and SHALL NOT report a hook as working on the basis of its presence in a settings file alone.

A settings entry proves something was written. It does not prove `openspec-doc` resolves on `PATH`, that the resolved binary is recent enough to have the named subcommand, or that the command exits with a response an agent can use. Each of those leaves the configuration text exactly right and the loop dead, and each is the part `init` cannot reach.

#### Scenario: A registered hook whose binary is absent fails
- **WHEN** a hook is registered in settings and its command cannot be executed or exits non-zero
- **THEN** the system SHALL report that hook as failing, SHALL include the command's exit status and its standard error in the report, and SHALL exit non-zero

#### Scenario: A registered hook whose binary is too old fails
- **WHEN** a hook is registered and its command resolves to a binary that does not recognize the subcommand named in it
- **THEN** the system SHALL report that hook as failing rather than as registered

#### Scenario: The turn-boundary hook must return a decision
- **WHEN** the `Stop` hook's command is executed with a synthetic turn-end payload
- **THEN** the system SHALL require a decision payload on standard output and SHALL report the hook as failing if standard output is empty or unparseable

#### Scenario: The explore hook must return a note path
- **WHEN** the `UserPromptExpansion` hook's command is executed with a synthetic payload
- **THEN** the system SHALL require a non-empty instruction on standard output and SHALL report the hook as failing otherwise

#### Scenario: The prompt hook is probed with a directive it must not stay silent about
- **WHEN** the `UserPromptSubmit` hook's command is executed
- **THEN** the system SHALL have seeded a standing verdict for the probe session beforehand and SHALL require the directive's text on standard output

This hook exits zero and prints nothing when it fails, so that a broken hook can never refuse the reviewer's prompt. Silence therefore means both "healthy, nothing outstanding" and "entirely broken", and probing it with an empty session would pass on an install with no binary at all.

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

### Requirement: The probe never touches the project's own state
The system SHALL run every hook probe against a throwaway project root created for the check, with the probe's working directory set to that root, and SHALL NOT run a probe that resolves to the project being checked.

The turn-boundary hook registers sessions, runs the promotion check, and consumes any pending directive. A probe against the real project would silently consume the reviewer's outstanding feedback, making `doctor` a cause of the failure it exists to diagnose.

#### Scenario: A pending directive survives the check
- **WHEN** `doctor` runs in a project with an undelivered directive for a live session
- **THEN** that directive SHALL still be pending after the command completes

#### Scenario: The check leaves no session behind
- **WHEN** `doctor` completes
- **THEN** the project's `.openspec-doc/` directory SHALL contain no session, note, or directive record created by the check

#### Scenario: A command with an explicit root is not probed
- **WHEN** a registered hook's command string contains a `--root` option
- **THEN** the system SHALL report that hook as unchecked, SHALL state that probing it would run against the named project and consume a pending directive, and SHALL exit non-zero

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

### Requirement: Anything not checked is reported as not checked
The system SHALL print a line for each part of the setup it does not examine, stating that it was not checked, and SHALL NOT omit it or report it as passing.

A green line for something never examined is the exact failure this capability exists to remove.

#### Scenario: pi.dev is reported as unchecked
- **WHEN** `doctor` runs
- **THEN** the system SHALL report pi.dev's extension as not checked, stating that its delivery points are an in-process extension rather than a settings entry

#### Scenario: The dashboard server is reported as unchecked
- **WHEN** `doctor` runs
- **THEN** the system SHALL report that it does not check whether `serve` is running

#### Scenario: The explore matcher is reported as unchecked
- **WHEN** `doctor` runs
- **THEN** the system SHALL report that the `UserPromptExpansion` matcher was not exercised, stating that executing the command bypasses matcher dispatch and that `openspec-doc init` is what keeps the matcher correct

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
