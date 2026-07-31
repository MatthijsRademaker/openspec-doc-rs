## Why

The dashboard currently gives too much visual weight to secondary metadata, making the page feel busy and harder to scan. Users need the most review-relevant work—especially active changes—to stand out first, while current specs, archive, health, and config information should support navigation without dominating the page.

## What Changes

- Refactor the dashboard visual hierarchy to be calmer, more spacious, and easier on the eyes.
- Move active changes into a full-width primary section at the top of the dashboard.
- Limit dashboard previews for current specs and archived changes to the most recent items, with links to the complete section pages.
- Demote project health and config location details into less prominent secondary presentation.
- Preserve existing dashboard navigation to full specs, active changes, archive, and warnings pages.
- No breaking changes to CLI commands, routes, or read-only behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `browser-viewer`: Update dashboard layout and visual-priority requirements for active changes, recent specs/archive previews, and secondary health/config metadata.

## Impact

- Affects browser dashboard rendering, layout, styling, and related tests/snapshots.
- May require scanner/parser metadata already available in the viewer model to sort specs and archived changes by recency.
- Does not affect CLI command syntax, exported data contracts, target project mutation behavior, or OpenSpec artifact parsing semantics.
