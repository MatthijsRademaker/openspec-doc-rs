## Why

The current scope route uses the correct observatory palette, typography, and raster sources but composes them as a large route hero, a separate three-image gallery, and one continuous multi-artifact page with a mostly empty conversation region. That implementation follows the archived redesign exactly, but it does not capture `dashboard-mockup.png`'s defining experience: one bounded document instrument, asymmetric artwork integrated into its unused plane, persistent scope navigation, and dense contextual conversation.

A second cosmetic pass would repeat the failure. Scope review needs one explicit interaction model that preserves straightforward implementation and anchor truth: one complete selected artifact in the center, normal document scrolling, comments contextualized to that artifact, and session/change behavior expressed as variants of the same workbench.

## What Changes

- **BREAKING:** replace the change route's concatenated proposal/design/tasks/spec document stream with one selected artifact at a time. The selected artifact renders completely and scrolls in source order; switching artifacts changes the central document.
- Keep session review on the same workbench. A session automatically selects its sole scratch artifact and does not render fake tabs or an empty artifact chooser.
- Add exact artifact navigation for changes, preserving paths such as `proposal.md`, `design.md`, `tasks.md`, and `specs/<capability>/spec.md` rather than translating them into invented product categories.
- Make selected artifact URL-owned through an encoded query value so refresh, browser history, copied links, comment navigation, and browser tests restore the same document without a global store.
- Show a clear unavailable-artifact instrument when an explicitly requested artifact does not exist. Do not silently substitute another document. A route with no artifact selection chooses the first server-provided artifact.
- Replace the route-level hero and pre-document plate gallery with a full-bleed workbench composition derived from `dashboard-mockup.png`: compact route/state instrumentation, persistent scope/artifact rail, a primary document stage, contextual thread rail, and decision controls integrated into the instrument chassis.
- Integrate the shared halftone face/orbit field into a bounded portion of the selected document's arrival plane. Artwork frames heading and unused margin rather than occupying a detached gallery or sitting beneath prose.
- Scope the desktop conversation rail to the selected artifact. Selecting a block marker or thread makes the exact source block visibly active; selecting a thread navigates the document to its resolved block. Unanchored and unresolved comments remain in the scope-level decision surface because no artifact can honestly own them.
- Preserve complete source rendering within the selected artifact, exact repeated-block anchoring, free-text selection comments, reviewer/agent distinction, reply/resolve/reopen/accept behavior, verdict submission, delivery state, live updates, dirty-composer deferral, and explicit failures.
- Keep browser/window scrolling authoritative. Sticky instrumentation may frame the document, but the route SHALL NOT introduce a nested primary document scroller or a speculative global state store.
- At narrow width, collapse the same data and actions into one logical flow. Artifact selection remains reachable, artwork yields first, the selected document remains complete, active conversation stays reachable, and persistent decisions do not obscure final content.
- Update design reference, component tests, deterministic fixtures, and embedded desktop/narrow Playwright evidence so acceptance checks composition against the source mockup, not merely presence of observatory assets.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dashboard-html-views`: Change scope review from a concatenated multi-artifact/spatial-row layout to a shared selected-artifact workbench with URL-owned artifact navigation, complete selected-document scrolling, contextual thread inspection, session/change variants, and preserved review behavior.
- `dashboard-visual-system`: Replace detached scope hero/gallery artwork with a mockup-faithful full-bleed instrument composition and bounded image-led document stage while retaining central tokens, accessibility, responsive yielding, offline assets, and runtime-image limits.

## Impact

- `web/src/views/ScopeView.vue`: selected-artifact ownership, URL synchronization, artifact filtering, shared session/change policy, SSE reconciliation against current selection, and workbench landmarks.
- `web/src/components/review/ArtifactDocument.vue`, `ScopeHeader.vue`, `CommentThread.vue`, and `DecisionInstrument.vue`: single-artifact rendering, active-anchor/thread coordination, compact state presentation, and decision placement without changing mutation semantics.
- New or adapted focused components under `web/src/components/review/` only where ownership is stable, likely an exact-path artifact navigator and selected-artifact thread inspector. No global store or speculative component framework.
- `web/src/style.css`: full-bleed scope geometry, compact instrument header, document-stage artwork, sticky rails, selected-anchor treatment, narrow ordering, focus, and reduced-motion behavior through existing semantic tokens.
- `web/src/lib/scope-review.ts`: no server contract expansion expected. Existing scope artifacts, blocks, comments, anchor states, verdicts, and counts are sufficient for initial selected-artifact behavior.
- Vue Router route remains `/sessions/:id` and `/changes/:name`; artifact selection is query-owned rather than a new nested route or compatibility alias.
- Existing shared WebP derivative set remains the only runtime raster set. No source PNG, remote asset, duplicated crop, package manager, or new frontend state dependency.
- `designs/dashboard-review-workbench.md`, scope unit tests, `web/e2e/dashboard.spec.ts`, and deterministic screenshots: replace superseded concatenated-workbench acceptance with selected-artifact, source-fidelity, deep-link, anchor, live-update, desktop, and 390px evidence.
- No Rust mutation endpoint, SSE protocol, sidecar format, comment-state transition, verdict-state transition, or embedded-asset boundary change.
