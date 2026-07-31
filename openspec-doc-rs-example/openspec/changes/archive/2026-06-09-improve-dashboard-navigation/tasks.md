## 1. Dashboard Rendering

- [x] 1.1 Replace the root overview renderer with a dashboard page that includes health, counts, review triage, specs, archive, and warnings sections.
- [x] 1.2 Add review triage rendering for active changes with links, task progress, warning counts, and artifact completeness indicators.
- [x] 1.3 Add dashboard spec overview rendering with links, requirement counts, scenario counts, and warning counts.
- [x] 1.4 Add dashboard warning and archive summary rendering with clear empty states.

## 2. Navigation Shell

- [x] 2.1 Update the shared layout to provide persistent navigation links for dashboard, specs, active changes, archive, and warnings.
- [x] 2.2 Include available section counts in persistent navigation.
- [x] 2.3 Add responsive styling so the navigation and dashboard remain usable on narrow screens.

## 3. Tests and Validation

- [x] 3.1 Update or add viewer tests for the dashboard root page counts, health summary, review triage, spec overview, warning summary, and empty states.
- [x] 3.2 Update or add viewer tests confirming persistent navigation appears on dashboard, index, detail, review, archive, and warnings pages.
- [x] 3.3 Run OpenSpec validation for `improve-dashboard-navigation`.
- [x] 3.4 Run the relevant npm validation commands for viewer changes.
