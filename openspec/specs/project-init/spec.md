# project-init Specification

## Purpose

The `init` subcommand: taking a project from nothing to wired. It selects the agent harnesses to configure — from `--agent` when given, from directory presence when not — and writes three things it does not own: the hook entries merged into `.claude/settings.json`, pi's extension from a copy embedded in the binary, and a marker-delimited block of standing instructions in `AGENTS.md`. Every file it touches belongs to someone else, so everything it did not write survives byte for byte, re-running it on a correct project produces no diff, and a file it cannot parse is an error rather than something to overwrite. A dry run is the default: the plan is the same output whether or not a terminal is attached, so the path a person reads is the path the tests exercise. A problem it cannot fix is reported with the command that fixes it, rather than being silently left out of the configuration.

## Requirements
### Requirement: Init subcommand and harness selection
The system SHALL provide an `init` subcommand that configures a project's agent hooks, and SHALL determine which harnesses to configure from repeatable `--agent <claude|pi>` options when any are given, or from detection of harness directories in the project root when none are given.

Detection is directory presence — `.claude/` for Claude Code, `.pi/` for pi.dev — rather than an attempt to read what `openspec init` recorded. Upstream's marker format is upstream's to change, and a detector coupled to it fails by configuring nothing.

#### Scenario: Harnesses are detected when none are named
- **WHEN** a user runs `openspec-doc init` in a project containing a `.claude/` directory and no `.pi/` directory
- **THEN** the system SHALL report Claude Code as detected and pi.dev as not detected, and SHALL plan writes for Claude Code only

#### Scenario: Explicit agents replace detection
- **WHEN** a user runs `openspec-doc init --agent pi` in a project containing a `.claude/` directory and no `.pi/` directory
- **THEN** the system SHALL plan writes for pi.dev only, and SHALL NOT plan any write for Claude Code

#### Scenario: No harness detected and none named
- **WHEN** a user runs `openspec-doc init` in a project with neither a `.claude/` nor a `.pi/` directory
- **THEN** the system SHALL exit with a non-zero status reporting that no agent harness was detected and that `--agent` names one explicitly

### Requirement: Plan before write
The system SHALL, unless `--yes` is given, print the harnesses it selected, every file it would create or modify with a one-line description of the modification, and every warning it found, and SHALL exit zero without creating or modifying any file.

The plan is the same output whether or not a terminal is attached, so the path a person reads is the path the tests exercise.

#### Scenario: A dry run writes nothing
- **WHEN** a user runs `openspec-doc init` without `--yes` in an unconfigured project
- **THEN** the system SHALL print the planned writes and SHALL leave the project's files byte-identical to what they were before the run

#### Scenario: Confirmation performs the planned writes
- **WHEN** a user runs `openspec-doc init --yes` in an unconfigured project
- **THEN** the system SHALL perform every write named in the corresponding plan and report each one as performed

#### Scenario: Re-running on a configured project is a no-op
- **WHEN** a user runs `openspec-doc init --yes` twice in succession in the same project
- **THEN** the second run SHALL leave every file it manages byte-identical to its state after the first run

### Requirement: Claude Code hook registration
The system SHALL, when Claude Code is selected, register three hook entries in `.claude/settings.json` at the project root: a `Stop` entry running `openspec-doc hook stop --agent claude`, a `UserPromptSubmit` entry running `openspec-doc hook prompt --agent claude`, and a `UserPromptExpansion` entry matched on `opsx:explore|openspec-explore` running `openspec-doc hook explore --agent claude`.

The committed settings file is the only target. The hook command strings contain nothing machine-specific, so a project-local file that is not shared records a team-wide fact in a personal place.

#### Scenario: All three entries are registered
- **WHEN** `init --yes` configures Claude Code in a project with no `.claude/settings.json`
- **THEN** the system SHALL create that file containing the `Stop`, `UserPromptSubmit`, and `UserPromptExpansion` entries with their commands and the expansion matcher

#### Scenario: Unrelated settings content is preserved
- **WHEN** `init --yes` configures Claude Code in a project whose `.claude/settings.json` contains other keys and other hook entries
- **THEN** the system SHALL preserve every key and every hook entry it did not write

#### Scenario: An existing openspec-doc entry is replaced rather than duplicated
- **WHEN** `init --yes` runs against a settings file already containing a hook entry whose command contains `openspec-doc hook `, including one prefixed by an environment assignment or an absolute path
- **THEN** the system SHALL replace that entry rather than adding a second entry for the same event

#### Scenario: A malformed settings file fails loudly
- **WHEN** `.claude/settings.json` exists and cannot be parsed as JSON
- **THEN** the system SHALL exit with a non-zero status naming the file, and SHALL NOT overwrite it

### Requirement: Missing explore command is reported
The system SHALL check whether the project provides a command or skill named `opsx:explore` or `openspec-explore`, SHALL write the `UserPromptExpansion` entry regardless, and SHALL report a warning naming the command that installs them when no such command is found.

`UserPromptExpansion` dispatches on the bare `command_name`. A matcher with no command of that name never fires and reports nothing, which is indistinguishable from an idle project — so the configuration is completed and the hole is named, rather than the hook being omitted and the hole being silent.

#### Scenario: A project without the explore command is warned
- **WHEN** `init` runs in a project with no file providing `opsx:explore` or `openspec-explore`
- **THEN** the system SHALL include a warning that the `UserPromptExpansion` hook will never fire, naming `openspec init --tools claude` as the fix, and SHALL still include the `UserPromptExpansion` entry in its plan

#### Scenario: A project with the explore command is not warned
- **WHEN** `init` runs in a project providing a command named `opsx:explore`
- **THEN** the system SHALL emit no warning about the explore command

### Requirement: pi.dev extension installation
The system SHALL, when pi.dev is selected, write `.pi/extensions/openspec-doc-hook.ts` from a copy of the extension embedded in the binary, and SHALL report that pi explorations are not captured.

There is one copy of that file, in the binary. Every project's copy is its output, so a divergence appears as a diff rather than as a defect in a project nobody is looking at.

#### Scenario: The extension is written from the embedded copy
- **WHEN** `init --yes` configures pi.dev in a project with no `.pi/extensions/` directory
- **THEN** the system SHALL create that directory and write `openspec-doc-hook.ts` into it with the embedded contents

#### Scenario: An existing extension is replaced
- **WHEN** `init --yes` configures pi.dev in a project whose `.pi/extensions/openspec-doc-hook.ts` differs from the embedded copy
- **THEN** the system SHALL replace the file with the embedded copy and report it as modified

#### Scenario: The pi explore gap is reported
- **WHEN** `init` plans or performs a pi.dev configuration
- **THEN** the output SHALL state that pi sessions produce no exploration note and are therefore not registered for review

### Requirement: Managed instruction block
The system SHALL, unless `--skip-instructions` is given, maintain a block delimited by `<!-- openspec-doc:begin -->` and `<!-- openspec-doc:end -->` in `AGENTS.md` at the project root, containing the standing instructions describing the exploration note, the review loop, and the comment commands.

The instructions are ambient rather than on demand: an agent must know what the note is for before it chooses where to write, so this cannot be a skill the model elects to load.

#### Scenario: The block is appended when absent
- **WHEN** `init --yes` runs against an `AGENTS.md` containing neither marker
- **THEN** the system SHALL append the delimited block and SHALL preserve the file's prior content byte-for-byte

#### Scenario: The block is replaced in place when present
- **WHEN** `init --yes` runs against an `AGENTS.md` already containing both markers
- **THEN** the system SHALL replace only the span between them, preserving every byte before the opening marker and after the closing marker

#### Scenario: A half-delimited file fails loudly
- **WHEN** `AGENTS.md` contains exactly one of the two markers
- **THEN** the system SHALL exit with a non-zero status reporting the file as malformed, rather than appending a second block

#### Scenario: The instruction file is created when absent
- **WHEN** `init --yes` runs in a project with no `AGENTS.md`
- **THEN** the system SHALL create it containing the delimited block

#### Scenario: Instructions can be declined
- **WHEN** a user runs `init --skip-instructions`
- **THEN** the system SHALL plan and perform no write to `AGENTS.md`
