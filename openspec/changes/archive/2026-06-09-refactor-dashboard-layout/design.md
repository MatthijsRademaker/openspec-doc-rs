## Context

The browser viewer already renders a dashboard with project counts, active change triage, current specs, warnings, archive summary, project health text, and root/config paths. The current layout places health and filesystem locations in the hero and renders active changes as one card within a multi-column dashboard grid. This makes secondary metadata visually compete with the primary review queue.

The refactor should improve information hierarchy without changing the underlying scanner/parser/analyzer contracts or read-only viewer routes.

## Goals / Non-Goals

**Goals:**

- Make the dashboard calmer and easier to scan through spacing, grouping, and reduced prominence for secondary details.
- Make active changes the first full-width dashboard section after the high-level header/counts because they are the most time-sensitive review target.
- Show only recent current specs and recent archived changes on the dashboard, while preserving complete index pages for each section.
- Demote project health and root/config locations into compact supporting metadata instead of hero-level content.
- Keep the implementation compatible with existing routes and viewer model data where practical.

**Non-Goals:**

- Redesigning section/detail pages beyond any shared style adjustments needed for consistency.
- Adding browser editing, comments, or write actions.
- Changing CLI behavior, exported JSON, or OpenSpec parsing rules.
- Introducing a client-side framework or new runtime dependency solely for this layout change.

## Decisions

1. **Use a dashboard-specific information hierarchy instead of changing route structure.**
   - Active changes remain linked through `/changes` and `/changes/<name>/review`, but the dashboard renders their triage section as full-width and above the secondary content grid.
   - Alternative considered: making `/changes` the default landing page. Rejected because users still need a project dashboard and route stability.

2. **Keep health visible but make it secondary.**
   - Health should appear as a compact status chip/card near counts or in a secondary details region, with a link to warnings when warnings exist.
   - Root/config locations should be collapsed into small muted metadata, a details element, or another low-emphasis area.
   - Alternative considered: removing root/config from the dashboard entirely. Rejected because those paths help verify the served target project, but they do not deserve top visual priority.

3. **Limit dashboard previews by recency and link to full indexes.**
   - Dashboard current specs and archive sections should render a small recent subset and include clear links to `/specs` and `/archive` for full lists.
   - Recency should use available file metadata if the viewer model exposes it; otherwise implementation may use a deterministic fallback such as scanned order or name ordering until metadata is added.
   - Alternative considered: keeping all items visible with denser styling. Rejected because long lists are a major source of dashboard visual clutter.

4. **Prefer CSS/layout changes and small helper functions over model-wide redesign.**
   - Implement the layout primarily in the existing viewer renderer and stylesheet. Add minimal helper functions for selecting recent specs/archive entries and rendering secondary metadata.
   - If file modification time is needed for true recency, add optional timestamp fields to the viewer model without changing CLI/export contracts.
   - Alternative considered: introducing a separate dashboard view model. Deferred until dashboard-specific data needs grow.

## Risks / Trade-offs

- **Risk: "Most recent" can be ambiguous for specs because current specs may not have an OpenSpec date.** → Mitigate by documenting and testing the chosen recency source, preferring filesystem modification time when available and deterministic fallback ordering when unavailable.
- **Risk: Limiting dashboard previews may hide items users expect to see immediately.** → Mitigate with explicit "View all" links and counts showing that more items exist.
- **Risk: Visual refactor may regress responsive behavior.** → Mitigate with responsive layout tests or smoke checks for narrow widths where available, and CSS that stacks sections on small screens.
- **Risk: Demoting health too much could hide important warnings.** → Mitigate by retaining warning counts and direct warnings links, with warning styling when warnings are present.
