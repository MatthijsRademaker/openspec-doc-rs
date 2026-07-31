## Context

The review companion currently renders active-change artifacts in tabs with a sticky companion panel that contains guidance, existing feedback, and a permanent feedback form. Selection changes pre-populate that form, and the right-click flow focuses the textarea and scrolls the side panel into view.

That model preserves sidecar feedback correctly, but it makes document review feel cramped and form-driven. The default viewer shell also caps the page width for dashboard-like pages, which leaves too little room for comfortable reading and navigation in the companion.

## Goals / Non-Goals

**Goals:**

- Make the companion document-first: the primary action is reading rendered OpenSpec artifacts.
- Replace permanent comment creation UI with selected-text popovers that appear only when useful.
- Render existing anchored comments as subtle in-text or margin markers attached to their resolved artifact context.
- Let users inspect, reply to, edit, and update comment status from marker popovers and/or the comments overview.
- Move the comments overview out of the dominant sticky side panel and into a separate page section.
- Widen the companion canvas while preserving a comfortable Markdown reading measure.
- Avoid nested scrolling and avoid forced focus/scroll behavior during comment creation.
- Preserve existing sidecar review event storage and API semantics where possible.

**Non-Goals:**

- No browser editing of OpenSpec proposal, design, tasks, or spec delta Markdown artifacts.
- No collaborative realtime annotation model.
- No rich text editor dependency.
- No new persistent sidecar schema unless implementation discovers marker placement cannot be derived from existing anchor resolution.
- No redesign of non-companion dashboard, spec, archive, or warning pages beyond shared styling necessary for harmony.

## Decisions

- **Use a dedicated companion shell rather than the default page width.** The companion route will use a wider main container for review context, while the Markdown document column remains capped to a readable width. This avoids both the current half-screen feeling and the opposite failure mode of overly long lines.

- **Prefer document popovers over a permanent creation form.** Selecting text in a companion artifact will expose a small annotation affordance near the selection. Activating it opens a compact composer popover with artifact path, selected text context, category/severity fields, and the comment body. The editor is not focused merely because text was selected or right-clicked; focus moves only after the user intentionally opens the composer.

- **Render anchored feedback as markers derived from resolved comments.** Existing feedback from `/api/review-feedback/:change` will be mapped to artifact paths and resolved anchor states. Exact and fuzzy comments will be represented as unobtrusive numbered markers near matching text when possible. Orphaned and missing comments remain visible in the comments overview with their original selected text.

- **Use marker popovers for thread inspection.** Clicking a marker opens a popover containing the comment body, selected text, anchor state, status, replies, edit controls, reply controls, and status controls. These controls continue posting to the existing review feedback APIs.

- **Keep a separate comments overview for triage.** The overview will be a normal page section rather than a full-height sticky side panel. It can group feedback by open/addressed/resolved state and provide jump actions to document markers. Jumping to a comment scrolls the primary page to the marker and briefly highlights it.

- **Use one primary vertical scroll surface.** The companion page should avoid full-height nested scroll containers for the main review flow. Stage tabs or a compact review toolbar may be sticky, but feedback lists and document content should participate in the page scroll.

- **Keep implementation dependency-light.** Use plain server-rendered HTML, CSS, and browser JavaScript as the current viewer does. Popovers can be implemented with positioned elements and accessible buttons/dialog semantics without adding a UI framework.

## Risks / Trade-offs

- **Anchor marker placement may be imprecise with simple Markdown rendering** → Start with markers next to rendered blocks or artifact sections when exact inline wrapping is complex; keep the comments overview authoritative for all comments.
- **Popover positioning can be brittle across viewport sizes** → Clamp popovers to the viewport and fall back to an inline composer below the active artifact on narrow screens.
- **Numbered markers may clutter dense documents** → Use subtle styling, collapse resolved markers visually, and show details only on hover/focus/click.
- **Removing the permanent form may hide how to comment** → Provide concise review guidance and a visible empty-state hint explaining “select text to add a comment.”
- **Maintaining existing tests while changing UX may require broad render assertions updates** → Add targeted tests for new markup states and avoid over-specifying generated JavaScript strings.
