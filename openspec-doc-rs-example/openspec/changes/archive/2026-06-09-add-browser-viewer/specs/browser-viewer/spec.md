## ADDED Requirements

### Requirement: Browser preview server
The CLI SHALL provide a `serve` command that starts a local read-only browser preview for a target OpenSpec project.

#### Scenario: Serve target root omitted
- **WHEN** a user runs `openspec-doc serve` without `--root`
- **THEN** the command serves the OpenSpec project from the current working directory

#### Scenario: Serve target root provided
- **WHEN** a user runs `openspec-doc serve --root <path>`
- **THEN** the command serves the OpenSpec project from the resolved target root path

#### Scenario: Serve config missing
- **WHEN** the target root does not contain `openspec/config.yaml`
- **THEN** the command reports an actionable error that identifies the missing file path and does not start the server

#### Scenario: Server starts successfully
- **WHEN** the target OpenSpec project is discovered successfully
- **THEN** the command prints the local browser URL including host and port

### Requirement: Browser project overview
The browser viewer SHALL render a project overview page summarizing the target OpenSpec project.

#### Scenario: Overview shows OpenSpec counts
- **WHEN** a user opens the viewer root page
- **THEN** the page shows counts for current specs, active changes, archived changes, requirements, scenarios, and warnings

#### Scenario: Overview links to major sections
- **WHEN** a user opens the viewer root page
- **THEN** the page provides navigation links to specs, active changes, archived changes, and warnings

### Requirement: Current spec pages
The browser viewer SHALL render current OpenSpec specs as navigable pages.

#### Scenario: Specs index lists current specs
- **WHEN** a target project contains files matching `openspec/specs/**/spec.md`
- **THEN** the specs index page lists each current spec with its stable relative path, purpose when available, requirement count, and scenario count

#### Scenario: Spec detail shows structured summary and content
- **WHEN** a user opens a spec detail page
- **THEN** the page shows the spec purpose when available, requirement and scenario summaries, warnings for that spec, and rendered Markdown content

### Requirement: Change pages
The browser viewer SHALL render active and archived OpenSpec changes as navigable pages.

#### Scenario: Active changes index lists changes
- **WHEN** active changes exist under `openspec/changes/<change-name>/`
- **THEN** the active changes page lists each active change and indicates whether proposal, design, tasks, and spec delta files are present

#### Scenario: Active change detail shows artifacts
- **WHEN** a user opens an active change detail page
- **THEN** the page shows available proposal, design, tasks, and spec delta artifacts as rendered Markdown sections

#### Scenario: Archived changes index lists archived changes
- **WHEN** archived changes exist under `openspec/changes/archive/**/`
- **THEN** the archived changes page lists archived changes separately from active changes

#### Scenario: Archived change detail shows artifacts
- **WHEN** a user opens an archived change detail page
- **THEN** the page shows available archived artifacts as rendered Markdown sections

### Requirement: Warning visibility
The browser viewer SHALL make analyzer warnings visible without preventing page rendering.

#### Scenario: Warning summary page
- **WHEN** analyzer warnings are present
- **THEN** the warnings page lists each warning with its affected OpenSpec-relative path and message

#### Scenario: Warning badges on related pages
- **WHEN** warnings apply to a spec or change shown in the viewer
- **THEN** related index and detail pages indicate that warnings exist for that item

#### Scenario: Malformed content still renders
- **WHEN** parsing finds non-fatal structural issues in OpenSpec Markdown
- **THEN** the viewer still renders available content and displays the corresponding warnings

### Requirement: Read-only viewer behavior
The browser viewer SHALL be read-only for target OpenSpec projects by default.

#### Scenario: Serve does not mutate target project
- **WHEN** a user runs `openspec-doc serve` against a target project
- **THEN** the command does not create, modify, or delete files in the target project

#### Scenario: Viewer pages do not expose editing actions
- **WHEN** a user browses the viewer
- **THEN** pages do not provide controls that write changes to the target OpenSpec project
