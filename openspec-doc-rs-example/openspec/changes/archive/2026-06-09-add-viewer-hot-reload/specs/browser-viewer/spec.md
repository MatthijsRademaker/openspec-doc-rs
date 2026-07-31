## ADDED Requirements

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
