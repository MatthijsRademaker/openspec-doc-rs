## ADDED Requirements

### Requirement: A doctor subcommand reports whether the project is wired up
The system SHALL provide a `doctor` subcommand that runs each setup check, prints one line per check identifying the check and its outcome, and exits non-zero when any check fails.

Without it, an unwired project is indistinguishable from an idle one: the dashboard serves, the pages render, and they are empty in both cases.

#### Scenario: A fully wired project passes
- **WHEN** a user runs `openspec-doc doctor` in a project whose hooks are registered and whose binary responds
- **THEN** the system SHALL print a passing line for each check and SHALL exit zero

#### Scenario: A missing hook fails the command
- **WHEN** a user runs `openspec-doc doctor` in a project with no `openspec-doc` hook entry in any settings file
- **THEN** the system SHALL print a failing line naming each unregistered hook and SHALL exit non-zero

#### Scenario: Every check is reported, not only the failing ones
- **WHEN** any check fails
- **THEN** the system SHALL still print a line for every other check, so the report is a complete picture of the setup rather than the first problem found

### Requirement: A hook check is settled by running the hook
The system SHALL check each registered hook by executing the command string recorded in settings and requiring a response specific to that hook, and SHALL NOT report a hook as working on the basis of its presence in a settings file alone.

A settings entry proves the operator typed something. It does not prove `openspec-doc` resolves on `PATH`, that the resolved binary is recent enough to have the named subcommand, or that the command exits with a response an agent can use. Each of those leaves the configuration text exactly right and the loop dead.

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

### Requirement: A hook is reported with the settings file it came from
The system SHALL search the user-level, project, and local Claude Code settings files, and SHALL report each occurrence of an `openspec-doc` hook with the path of the file it was found in.

Whether a hook lives only in a gitignored file is the finding, not a detail: that is the state a fresh clone arrives in, and it is what makes the setup lossy. Occurrences are listed rather than resolved into a single winner, because reproducing settings precedence exactly would produce a confidently wrong report when it is modelled wrong.

#### Scenario: The origin of each hook is named
- **WHEN** a hook is found in a settings file
- **THEN** the system SHALL name that file's path alongside the hook in the report

#### Scenario: A hook registered twice is reported twice
- **WHEN** the same hook appears in more than one settings file
- **THEN** the system SHALL report each occurrence with its own file path rather than reporting one

#### Scenario: All three hooks are enumerated
- **WHEN** `doctor` runs
- **THEN** the report SHALL cover `Stop`, `UserPromptSubmit`, and `UserPromptExpansion`, naming any of them that is not registered

### Requirement: The explore matcher is checked as text and labelled as such
The system SHALL compare the `UserPromptExpansion` entry's matcher against the value the explore hook requires, and SHALL mark that finding in the report as a text comparison rather than an executed check.

Running the command directly bypasses matcher dispatch, so no probe can reach the failure this check exists for: a matcher that is present and valid and never matches, whose hook then never runs and reports nothing. An operator who reads the result as "verified" concludes the loop is sound while the explore hook is dead — the original silent failure with a passing line in front of it.

#### Scenario: A wrong matcher is reported as failing
- **WHEN** the `UserPromptExpansion` entry's matcher does not match the required value
- **THEN** the system SHALL report the matcher as failing, SHALL print both the configured and the required value, and SHALL exit non-zero

#### Scenario: A correct matcher is not reported as executed
- **WHEN** the matcher matches the required value
- **THEN** the system SHALL report it as a text comparison and SHALL state that a matcher can only be confirmed to fire from a live session

### Requirement: The binary under check is identified
The system SHALL report the path and version of the `openspec-doc` resolved on `PATH` and of the running executable, and SHALL report a failing check when they differ.

An operator working in this repository routinely has an installed copy on `PATH` and a newer one under `target/`. The agent runs whichever `PATH` resolves, so a check run from the newer binary that reports on the older one's behalf is answering the wrong question.

#### Scenario: A different binary on PATH is reported
- **WHEN** the `openspec-doc` on `PATH` is not the running executable
- **THEN** the system SHALL print both paths with both versions and SHALL exit non-zero

#### Scenario: No binary on PATH is reported
- **WHEN** no `openspec-doc` resolves on `PATH`
- **THEN** the system SHALL report that as a failing check, since every hook command depends on it

### Requirement: Project root resolution is checked against the real project
The system SHALL resolve the project root using the same discovery rules as every other subcommand and SHALL print the resolved path.

#### Scenario: The resolved root is printed
- **WHEN** `doctor` runs inside an OpenSpec project
- **THEN** the system SHALL print the resolved project root

#### Scenario: An unresolvable root fails the command
- **WHEN** `doctor` runs where no ancestor directory contains `openspec/config.yaml` and no `--root` is given
- **THEN** the system SHALL report the failure and exit non-zero

### Requirement: Anything not checked is reported as not checked
The system SHALL print a line for each part of the setup it does not examine, stating that it was not checked, and SHALL NOT omit it or report it as passing.

A green line for something never examined is the exact failure this capability exists to remove.

#### Scenario: pi.dev is reported as unchecked
- **WHEN** `doctor` runs
- **THEN** the system SHALL report pi.dev's hook as not checked, stating that its turn boundary is an in-process extension rather than a settings entry

#### Scenario: The dashboard server is reported as unchecked
- **WHEN** `doctor` runs
- **THEN** the system SHALL report that it does not check whether `serve` is running
