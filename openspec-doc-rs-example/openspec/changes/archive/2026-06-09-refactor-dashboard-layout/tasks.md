## 1. Dashboard Data and Ordering

- [x] 1.1 Identify the current dashboard renderer, style rules, and tests that cover dashboard output.
- [x] 1.2 Add or reuse helper logic to select recent current specs for the dashboard preview with deterministic fallback ordering.
- [x] 1.3 Add or reuse helper logic to select recent archived changes for the dashboard preview with deterministic fallback ordering.
- [x] 1.4 Ensure full specs and archive index pages still list all items.

## 2. Dashboard Layout Refactor

- [x] 2.1 Move active changes into a full-width primary dashboard section near the top of the root page.
- [x] 2.2 Render current specs and archived changes as limited recent preview sections with clear "View all" links and counts where useful.
- [x] 2.3 Demote project health into compact supporting metadata while preserving warning count and warnings-page navigation.
- [x] 2.4 Demote root and config paths into low-emphasis verification metadata.
- [x] 2.5 Adjust spacing, typography, colors, and responsive layout so the dashboard is calmer and readable on narrow screens.

## 3. Verification

- [x] 3.1 Add or update renderer tests/snapshots for active-change-first layout, recent specs preview, recent archive preview, and de-emphasized health/config metadata.
- [x] 3.2 Run relevant unit tests and type checks for the viewer.
- [x] 3.3 Run `openspec validate refactor-dashboard-layout` and fix any proposal/spec issues.
