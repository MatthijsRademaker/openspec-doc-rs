## 1. CLI and Server Foundation

- [x] 1.1 Add a `serve` command to the CLI command registry with `--root`, host, and port option handling.
- [x] 1.2 Implement a local HTTP server module that starts, reports its URL, handles startup failures, and can be stopped in tests.
- [x] 1.3 Wire `serve` to existing target root discovery so missing `openspec/config.yaml` reports the actionable discovery error and does not start the server.
- [x] 1.4 Ensure `serve` performs read-only startup without creating, modifying, or deleting files in the target project.

## 2. Viewer Data Model

- [x] 2.1 Create a viewer model builder that consumes existing scanner, parser, and analyzer outputs instead of duplicating OpenSpec file parsing.
- [x] 2.2 Include overview counts for current specs, active changes, archived changes, requirements, scenarios, and warnings.
- [x] 2.3 Include spec detail data with stable relative paths, purpose, requirement summaries, scenario summaries, raw Markdown, and related warnings.
- [x] 2.4 Include active and archived change detail data with artifact presence and rendered artifact source content.
- [x] 2.5 Add tests proving viewer data uses the same scan, parse, and warning results as summary/check behavior.

## 3. HTML Rendering

- [x] 3.1 Add a rendering layer that turns the viewer model into escaped, navigable HTML pages.
- [x] 3.2 Render the project overview page with counts and links to specs, active changes, archived changes, and warnings.
- [x] 3.3 Render specs index and spec detail pages with purpose, counts, requirements, scenarios, warnings, and Markdown content.
- [x] 3.4 Render active changes index/detail pages showing proposal, design, tasks, and spec delta artifacts when present.
- [x] 3.5 Render archived changes index/detail pages separately from active changes.
- [x] 3.6 Render a warnings page listing warning messages and affected OpenSpec-relative paths.
- [x] 3.7 Add tests or snapshots for representative rendered overview, spec, change, archive, and warning pages.

## 4. Routing and Browser Behavior

- [x] 4.1 Add routes for `/`, `/specs`, `/specs/<name>`, `/changes`, `/changes/<name>`, `/archive`, and `/warnings`.
- [x] 4.2 Return helpful not-found pages for unknown viewer routes without crashing the server.
- [x] 4.3 Print the local browser URL with resolved host and port when the server starts successfully.
- [x] 4.4 Keep browser-opening behavior absent or opt-in so automation is not surprised.

## 5. Validation and Documentation

- [x] 5.1 Add fixture coverage for minimal, warning-producing, active-change, and archived-change OpenSpec projects if not already present.
- [x] 5.2 Add command-level tests or smoke checks for successful `serve` startup and missing-config failure.
- [x] 5.3 Document `openspec-doc serve` usage, options, read-only behavior, and expected local URL output.
- [x] 5.4 Run project validation scripts and `openspec validate add-browser-viewer`.
