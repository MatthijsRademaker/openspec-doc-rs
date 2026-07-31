## Purpose

Provide a read-only browser-based documentation viewer for inspecting OpenSpec project summaries, current specs, active changes, archived changes, and structural warnings.
## Requirements
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
The browser viewer SHALL render the root page as a dashboard that summarizes the target OpenSpec project and prioritizes the most relevant review and documentation areas with a calm, readable visual hierarchy.

#### Scenario: Dashboard shows OpenSpec counts
- **WHEN** a user opens the viewer root page
- **THEN** the page shows counts for current specs, active changes, archived changes, requirements, scenarios, and warnings

#### Scenario: Dashboard links to major sections
- **WHEN** a user opens the viewer root page
- **THEN** the page provides navigation links to specs, active changes, archived changes, and warnings

#### Scenario: Dashboard shows secondary project health
- **WHEN** a user opens the viewer root page
- **THEN** the page shows a project health summary based on available warning and OpenSpec summary data
- **AND** the health summary is presented as supporting metadata rather than as the primary hero message

#### Scenario: Dashboard shows secondary project locations
- **WHEN** a user opens the viewer root page
- **THEN** the page makes the target root and OpenSpec config locations available for verification
- **AND** those locations are visually de-emphasized relative to active changes and navigation summaries

#### Scenario: Dashboard highlights active review triage first
- **WHEN** active changes exist in the target OpenSpec project
- **THEN** the dashboard places active changes in a full-width primary section before current specs, archived changes, and secondary health or configuration details
- **AND** each listed active change shows available task progress, warning count, and artifact completeness indicators
- **AND** each listed active change provides direct links to change detail or review pages

#### Scenario: Dashboard handles no active changes
- **WHEN** no active changes exist in the target OpenSpec project
- **THEN** the dashboard clearly indicates that there is no active change review queue in the primary active changes section

#### Scenario: Dashboard shows recent spec preview
- **WHEN** current specs exist in the target OpenSpec project
- **THEN** the dashboard lists only the most recent current specs as a preview with direct links to spec detail pages
- **AND** each listed spec shows available requirement count, scenario count, and warning count
- **AND** the dashboard provides a link to the complete specs index

#### Scenario: Dashboard shows recent archive preview
- **WHEN** archived changes exist in the target OpenSpec project
- **THEN** the dashboard lists only the most recent archived changes as a preview with direct links to archived change detail pages
- **AND** the dashboard provides a link to the complete archive index

#### Scenario: Dashboard shows warning summary
- **WHEN** analyzer warnings are present
- **THEN** the dashboard shows a warning summary with a direct link to the full warnings page

#### Scenario: Dashboard remains read-only
- **WHEN** a user browses the dashboard
- **THEN** the page does not provide controls that create, modify, or delete files in the target OpenSpec project

### Requirement: Persistent viewer navigation
The browser viewer SHALL provide persistent navigation across viewer pages so users can reach the dashboard and major sections without returning through intermediate pages.

#### Scenario: Navigation appears on dashboard and section pages
- **WHEN** a user opens the dashboard, specs index, active changes index, archive index, warnings page, or supported detail page
- **THEN** the page shows navigation links for dashboard, specs, active changes, archive, and warnings

#### Scenario: Navigation includes current counts
- **WHEN** the persistent navigation is rendered
- **THEN** it includes available counts for specs, active changes, archived changes, and warnings

#### Scenario: Navigation preserves existing routes
- **WHEN** a user follows a persistent navigation link
- **THEN** the viewer routes to the existing dashboard, specs, active changes, archive, or warnings page for that section

#### Scenario: Navigation remains responsive
- **WHEN** the viewer is displayed on a narrow screen
- **THEN** the persistent navigation remains usable without hiding access to the dashboard, specs, active changes, archive, or warnings sections

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

### Requirement: Active change review navigation
The browser viewer SHALL provide a review-oriented route or section for active OpenSpec changes that helps users inspect generated proposal artifacts.

#### Scenario: Review page opens for active change
- **WHEN** a user opens the review URL for an active change
- **THEN** the viewer shows the change proposal, design, tasks, spec deltas, warnings, and artifact completion details available for that change

#### Scenario: Review page links from active change detail
- **WHEN** a user opens an active change detail page
- **THEN** the viewer provides a navigation link to the review-oriented view for that change

#### Scenario: Review page remains read-only
- **WHEN** a user opens the review-oriented view for an active change
- **THEN** the page does not expose controls that write comments or edits to the target project in this first implementation slice

#### Scenario: Review page explains feedback path
- **WHEN** a user opens the review-oriented view for an active change
- **THEN** the page explains that proposal feedback should be provided in Pi for now and that browser-native comments are reserved for a later review companion mode

### Requirement: Active change companion navigation
The browser viewer SHALL provide navigation to review companion mode only when companion mode is explicitly enabled.

#### Scenario: Companion link appears when enabled
- **WHEN** a user opens an active change review page while review companion mode is enabled
- **THEN** the page provides a link or control to open the same change in companion mode

#### Scenario: Companion link hidden when disabled
- **WHEN** a user opens an active change review page while review companion mode is disabled
- **THEN** the page does not provide browser-side feedback controls or companion write actions

### Requirement: Review companion preserves default read-only viewer
The browser viewer SHALL keep default `serve` behavior read-only for target OpenSpec artifacts.

#### Scenario: Default serve remains read-only
- **WHEN** a user runs `openspec-doc serve` without review companion mode
- **THEN** browser pages do not create, modify, or delete OpenSpec artifacts or review sidecar files

#### Scenario: Companion writes are scoped
- **WHEN** review companion mode is enabled and a user saves review feedback
- **THEN** the write is limited to the documented review sidecar storage path
- **AND** no file under the target project's `openspec/` directory is modified

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

### Requirement: Opt-in viewer watch mode
The browser viewer SHALL provide an explicit watch mode for `serve` that keeps the running viewer aligned with changes to OpenSpec documentation files.

#### Scenario: Serve watch option enabled
- **WHEN** a user runs `openspec-doc serve --watch` for a valid target project
- **THEN** the command starts the browser viewer without requiring a server restart for subsequent OpenSpec file changes
- **AND** the command output indicates that watch mode is enabled

#### Scenario: Serve watch option disabled by default
- **WHEN** a user runs `openspec-doc serve` without `--watch`
- **THEN** the command preserves the existing non-watch serve behavior

### Requirement: Watched OpenSpec changes refresh rendered pages
When watch mode is enabled, the viewer SHALL refresh its server-side OpenSpec model after relevant OpenSpec files change.

#### Scenario: Spec file changes while server is running
- **WHEN** watch mode is enabled and a current spec file under `openspec/specs/` changes
- **THEN** subsequent rendered viewer pages reflect the updated spec content, counts, and warnings without restarting the server

#### Scenario: Active change artifact changes while server is running
- **WHEN** watch mode is enabled and an active change artifact under `openspec/changes/<change-name>/` changes
- **THEN** subsequent rendered change and review pages reflect the updated proposal, design, tasks, spec deltas, and warnings without restarting the server

#### Scenario: Archived change artifact changes while server is running
- **WHEN** watch mode is enabled and an archived change artifact under `openspec/changes/archive/` changes
- **THEN** subsequent rendered archive pages reflect the updated archived artifact content without restarting the server

### Requirement: Browser pages detect refreshed viewer content
When watch mode is enabled, browser pages SHALL provide a way to reflect refreshed server-side content without requiring the user to know that the server model changed.

#### Scenario: Open browser page observes model refresh
- **WHEN** a browser page is open while watch mode is enabled
- **AND** the server successfully refreshes the viewer model after a watched file change
- **THEN** the browser page reloads or otherwise clearly updates to show the refreshed content

#### Scenario: Watch mode disabled browser behavior
- **WHEN** watch mode is disabled
- **THEN** viewer pages do not include hot-reload behavior or background refresh requests solely for watch mode

### Requirement: Watch refresh is resilient
The watch-mode refresh process SHALL keep the viewer usable when a transient scan, parse, or analysis failure occurs during refresh.

#### Scenario: Refresh fails after a file change
- **WHEN** watch mode is enabled and rebuilding the viewer model fails after a watched file change
- **THEN** the server keeps serving the last successfully built viewer model
- **AND** the user receives a visible or logged indication that the latest refresh failed

#### Scenario: Later refresh succeeds
- **WHEN** a previous watch refresh failed
- **AND** a later watched file change can be scanned, parsed, and analyzed successfully
- **THEN** the server updates to the latest successful viewer model

### Requirement: Watch mode remains non-mutating
Watch mode SHALL NOT modify target OpenSpec artifacts.

#### Scenario: Watched file change triggers refresh only
- **WHEN** watch mode observes a change under `openspec/`
- **THEN** it rebuilds viewer state without creating, modifying, or deleting files under `openspec/`

#### Scenario: Review companion sidecar scope preserved
- **WHEN** watch mode and review companion mode are both enabled
- **THEN** browser feedback writes remain limited to the documented `.pi/openspec-review/comments/` sidecar path
- **AND** watch-mode refreshes do not write to OpenSpec artifacts

### Requirement: Review agent serve configuration
The browser preview server SHALL report whether live review agent analysis is enabled, explicitly disabled, or unavailable for the current session.

#### Scenario: Serve reports review agent enabled by default
- **WHEN** the server starts with default companion behavior
- **THEN** CLI output or server-rendered state MUST indicate that browser Pi review analysis is enabled and non-mutating

#### Scenario: Serve reports review agent explicitly disabled
- **WHEN** the server starts with live review agent analysis explicitly disabled
- **THEN** CLI output or server-rendered state MUST indicate that browser Pi review analysis is disabled

#### Scenario: Serve reports review agent unavailable
- **WHEN** the server cannot provide live Pi analysis because SDK setup, credentials, or model availability are missing
- **THEN** CLI output or server-rendered state MUST indicate that browser Pi review analysis is unavailable when relevant

### Requirement: Dashboard lifecycle overview
The browser viewer SHALL show project-level OpenSpec lifecycle guidance on the dashboard.

#### Scenario: Dashboard shows lifecycle overview
- **WHEN** a user opens the viewer root page
- **THEN** the dashboard shows the Explore, Propose, Review, Apply, and Archive lifecycle phases
- **AND** the lifecycle overview is visible near active change triage

#### Scenario: Dashboard highlights active change phases
- **WHEN** active changes exist in the target OpenSpec project
- **THEN** the dashboard shows each active change with its inferred lifecycle phase
- **AND** each listed change includes a concise next-action summary

#### Scenario: Dashboard lifecycle links to active change pages
- **WHEN** the dashboard shows an active change in the lifecycle queue
- **THEN** it links to the active change review page
- **AND** it links to the review companion page when companion mode is enabled

### Requirement: Active changes lifecycle badges
The browser viewer SHALL show lifecycle phase badges on active change lists and detail pages.

#### Scenario: Active changes index shows lifecycle phase
- **WHEN** a user opens the active changes index
- **THEN** each active change item shows its inferred lifecycle phase badge
- **AND** each item includes the primary recommended next action for that phase

#### Scenario: Active change detail shows lifecycle panel
- **WHEN** a user opens an active change detail page
- **THEN** the page shows a lifecycle guidance panel for that change
- **AND** the panel includes the inferred phase, reasons, and next actions

#### Scenario: Archived change detail does not suggest active lifecycle actions
- **WHEN** a user opens an archived change detail page
- **THEN** the page does not suggest active Propose, Review, Apply, or Archive next actions
- **AND** it may indicate that the lifecycle is complete for that archived change

### Requirement: Active change review lifecycle guidance
The browser viewer SHALL include lifecycle guidance on active change review pages.

#### Scenario: Review page shows current lifecycle phase
- **WHEN** a user opens the review page for an active change
- **THEN** the page shows the inferred lifecycle phase near the review header
- **AND** it summarizes why that phase was inferred

#### Scenario: Review page provides phase-specific commands
- **WHEN** a user opens the review page for an active change
- **THEN** the page provides copyable command or Pi guidance for the recommended next phase action
- **AND** the commands are presented as user-run actions rather than browser-executed actions

#### Scenario: Review page explains archive readiness
- **WHEN** a reviewed active change has completed tasks and no change warnings
- **THEN** the review page identifies archive as the recommended next phase
- **AND** it recommends validation before running archive commands

