## Context

The current browser viewer is implemented as server-rendered HTML in `src/viewer/render.ts`, backed by `ViewerModel` data assembled in `src/viewer/model.ts`. It already has routes for the root overview, specs, active changes, archive, warnings, active change review pages, and optional review companion pages.

The root page currently presents count cards and links. That is useful, but it does not organize the project around the dashboard questions users need answered first: what exists, what is changing, and what needs attention. The shared layout also uses a compact header navigation, which means deeper pages do not provide a strong app-like navigation structure.

## Goals / Non-Goals

**Goals:**

- Make `/` feel like a dashboard rather than a plain overview.
- Preserve all existing routes while making them easier to navigate.
- Surface active changes, warnings, missing artifacts, and task progress as review triage signals.
- Keep the first implementation compatible with the existing server-rendered renderer and no new browser framework.
- Keep default viewer behavior read-only for target OpenSpec artifacts.

**Non-Goals:**

- Add browser-side editing of OpenSpec proposal, design, task, or spec files.
- Replace the server-rendered HTML architecture with a client-side app framework.
- Add search, filtering, or persistence beyond the existing optional review companion sidecar behavior.
- Add new analyzer rules unless directly needed to display already available warning or artifact data.

## Decisions

### Use the existing server-rendered viewer shell

Implement the dashboard and navigation shell in the current `render.ts` HTML renderer rather than introducing a UI framework.

Rationale: the desired dashboard is mostly a presentation and navigation improvement over data already available in `ViewerModel`. Avoiding a framework keeps the first slice small, dependency-free, and consistent with the current CLI/server architecture.

Alternative considered: introduce a bundled client-side UI. That may become useful later for search, collapsible trees, or richer interaction, but it is unnecessary for a read-only dashboard slice.

### Treat the dashboard as review triage plus navigation

The root dashboard should prioritize active changes and warnings before broad browsing lists. It should still include spec and archive summaries, but the strongest value is helping users decide where to look next.

Rationale: OpenSpec files are already documentation. `openspec-doc` adds value by making project health and review state visible.

Alternative considered: make the dashboard primarily a documentation index. That would improve browsing but would underuse existing task, artifact, and warning metadata.

### Add a persistent navigation shell to all pages

Update the shared layout so the major routes are visible throughout the viewer, with dashboard, specs, active changes, archive, and warnings represented as first-class navigation destinations.

Rationale: an easily navigable dashboard experience depends on consistent orientation, not just a better root page. The shell can be implemented with semantic HTML and responsive CSS.

Alternative considered: only improve the root page. That would help first impressions but would not solve navigation friction on deeper pages.

### Derive dashboard sections from existing model data first

Use currently available summary counts, spec metadata, change artifact flags, task counts, and warnings. Add small model helpers only if they simplify rendering; avoid changing parser/analyzer behavior for this slice.

Rationale: this keeps the change focused on experience and reduces risk. Future changes can add richer health scoring, search indexes, or timeline data.

## Risks / Trade-offs

- Dashboard may become visually dense → Mitigate by grouping into clear sections: health, review queue, specs, archive, and warnings.
- Server-rendered HTML can become harder to maintain as UI grows → Mitigate with small rendering helper functions and avoid large inline blocks where possible.
- Existing tests may assert old overview wording → Mitigate by updating viewer tests to assert the new dashboard behavior while preserving route compatibility.
- Review triage ordering may feel subjective → Mitigate with simple transparent priority: warnings first, missing artifacts next, incomplete tasks next.
