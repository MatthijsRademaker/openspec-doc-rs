## Why

Review companion feedback currently stops at browser-side sidecar comments and a later Pi prompt handoff. To feel like a smooth in-browser review experience, reviewers need to ask Pi about active change feedback directly from the companion page and see live analysis without switching to the Pi TUI.

## What Changes

- Add browser Pi review agent analysis for `serve` sessions as the default companion behavior, with an explicit opt-out for users who do not want browser-triggered Pi sessions.
- Add browser companion controls that bundle selected/open feedback and active change artifact context into a live Pi review request.
- Stream Pi review session progress and analysis back into the browser.
- Persist lightweight review agent session events so refreshes can recover completed analysis.
- Keep the first slice non-mutating: Pi may analyze and propose artifact updates, but browser-triggered sessions must not edit OpenSpec artifacts.
- Show clear disabled/unavailable states when the Pi review agent is opted out, unavailable, or cannot start, and provide retry affordances for failed starts or failed runs where safe.

## Capabilities

### New Capabilities
- `browser-pi-review-sessions`: Browser-native Pi review sessions for active OpenSpec change feedback and artifacts.

### Modified Capabilities
- `review-companion`: Adds review companion entrypoints and states for live Pi analysis while preserving existing comment behavior.

## Impact

- Affected code: CLI `serve` options, local viewer server APIs, review companion rendering/client JavaScript, review feedback digest/bundling code, and tests.
- New optional integration dependency may be needed for `@earendil-works/pi-coding-agent` SDK access, or loaded dynamically when agent mode is enabled.
- Browser-triggered Pi sessions are local-only, active-change-only, and non-mutating in this change.
- Existing default sidecar comments remain compatible and continue to be stored outside `openspec/`.
