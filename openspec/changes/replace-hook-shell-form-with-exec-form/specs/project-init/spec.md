## MODIFIED Requirements

### Requirement: Claude Code hook registration
The system SHALL, when Claude Code is selected, register three hook entries in `.claude/settings.json` at the project root, each naming the `openspec-doc` executable and carrying its arguments as a vector rather than as a single string a shell would have to split: a `Stop` entry running the turn-boundary hook for the Claude agent, a `UserPromptSubmit` entry running the prompt hook for the Claude agent, and a `UserPromptExpansion` entry matched on `opsx:explore|openspec-explore` running the explore hook for the Claude agent.

The committed settings file is the only target. The hook command strings contain nothing machine-specific, so a project-local file that is not shared records a team-wide fact in a personal place.

An argument vector is registered rather than a command line because the agent executes the two forms differently: a single string is handed to a shell, and which shell that is varies by platform — `sh` on macOS and Linux, Git Bash on Windows, PowerShell where Git Bash is absent — while a vector is spawned directly with no shell on any platform. The hooks need no shell feature, so the form that involves one buys nothing and costs a platform-dependent execution path, one that is additionally known to be broken on Windows in the agent's own current release.

#### Scenario: All three entries are registered
- **WHEN** `init --yes` configures Claude Code in a project with no `.claude/settings.json`
- **THEN** the system SHALL create that file containing the `Stop`, `UserPromptSubmit`, and `UserPromptExpansion` entries, each naming the executable and its arguments separately, with the expansion matcher

#### Scenario: No registered entry requires a shell to interpret it
- **WHEN** the system writes a hook entry
- **THEN** the entry SHALL be executable by spawning the named executable with the given arguments, and SHALL NOT depend on any shell to split, expand, or interpret it

#### Scenario: Unrelated settings content is preserved
- **WHEN** `init --yes` configures Claude Code in a project whose `.claude/settings.json` contains other keys and other hook entries
- **THEN** the system SHALL preserve every key and every hook entry it did not write

#### Scenario: An existing openspec-doc entry is replaced rather than duplicated
- **WHEN** `init --yes` runs against a settings file already containing a hook entry invoking this tool's hook subcommand, whether that entry names the executable by bare name or by absolute path, and whether it carries its arguments as a vector or as a single string written by an earlier version
- **THEN** the system SHALL replace that entry rather than adding a second entry for the same event

#### Scenario: A project wired by an earlier version is rewritten, not doubled
- **WHEN** `init --yes` runs against a settings file whose entries carry their arguments as one string
- **THEN** the resulting file SHALL contain exactly one entry per event for this tool, in the form that needs no shell

#### Scenario: A malformed settings file fails loudly
- **WHEN** `.claude/settings.json` exists and cannot be parsed as JSON
- **THEN** the system SHALL exit with a non-zero status naming the file, and SHALL NOT overwrite it

## ADDED Requirements

### Requirement: The agent must be new enough for the form the hooks are registered in
The system SHALL require a Claude Code version that executes a hook entry's arguments as a vector, SHALL state that minimum where a user configuring the tool will read it, and SHALL report an agent below it as a failing setup check naming the required version.

An agent that does not recognise the argument vector does not reject the entry — it ignores the vector and runs the executable with no arguments, which is a usage error, which exits with the status the agent reads as a refusal of the user's prompt. The result is a session in which every prompt is blocked, with nothing on screen naming the cause. This repository has already had that failure once from a different cause and fixed it; registering a form an old agent cannot read reintroduces it for every user on one.

The check is honest about what it buys. It makes the cause discoverable after a user has been blocked; it does not prevent the block, because the hook has already run by then. Preventing it would mean writing the older form for older agents, which is the shell-dependent path this capability no longer has.

#### Scenario: An agent below the minimum is reported
- **WHEN** the setup check runs with an agent older than the version that executes argument vectors
- **THEN** the system SHALL report a failing check naming the installed version and the required one

#### Scenario: The minimum is stated to the user
- **WHEN** a user reads the instructions for configuring this tool with Claude Code
- **THEN** those instructions SHALL state the minimum agent version
