## Why

The shared scope route is behaviorally strong but visually stops where the successful index redesign began: the requested change fixture renders as a large boxed header, a clean generic orbit, a sparse utility column, and a mostly unart-directed document field. It does not capture the image-led face, halftone, clipped-orbit, rail, and observation-plate composition of `dashboard-mockup.png`, and its fixed decision bar visually crosses document content at both desktop and 390px.

This change carries the same authored observatory direction into the actual review workbench without weakening anchored comments, live updates, document primacy, or verdict behavior.

## What Changes

- Recompose the shared session/change scope shell around a compact route instrument, a readable document spine, a persistent utility rail, a spatially aligned conversation rail, and a decision instrument that does not cover document content.
- Use the long change fixture at `/changes/implement-observatory-design-system-with-a-realistically-long-identifier` as the primary desktop and narrow acceptance target; apply the same shared shell to `/sessions/:id` rather than forking `ScopeView` by route kind.
- Replace the clean placeholder orbit in the scope header with an image-led observation field derived from `main-panel-background.png`: black safe space and title/state remain left/center, while halftone face and clipped orbital geometry enter from the right.
- Add a restrained desktop observation band using the approved sun, face/eye, and star-system plate crops so the art direction remains visible before a reviewer reaches the end of a long artifact stack.
- Promote the successful index derivatives from index-specific filenames to shared observatory runtime slots and reuse the same optimized files across index and scope routes; do not duplicate equivalent raster payload under scope-specific names.
- Compact the scope header so the artifact's own document heading retains gravity instead of competing with a second oversized copy of the same title.
- Make the existing artifact path list an intentional document-coordinate rail with a real Router link back to the implemented index, visible comment counts, exact paths, and anchor navigation based only on existing data.
- Keep anchored conversation beside its block at desktop and immediately after its block at narrow width. Do not replace spatial anchoring with the mockup's detached pooled thread feed.
- Strengthen the conversation rail's visual presence with ruled geometry and bounded atmosphere when no thread is expanded, while never inventing comments, activity, or agent state.
- Recompose the persistent decision instrument as part of the workbench grid on desktop and a compact safe-area-aware command dock at narrow width. Preserve all existing composer, verdict, orphaned-comment, reply, resolve, reopen, and addressed-claim behavior.
- Preserve live artifact/review reconciliation, dirty-composer deferral, reading-position restoration, exact repeated-block anchoring, failure semantics, keyboard focus, reduced motion, and offline embedding.
- Update design reference, runtime-image contract, component tests, embedded Playwright coverage, and deterministic desktop/narrow source-board screenshots in the same change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dashboard-html-views`: Strengthen the shared scope-workbench composition, spatial conversation rail, decision-instrument placement, narrow flow, and route wayfinding while preserving every existing review interaction.
- `dashboard-visual-system`: Extend the image-led observation composition to scope routes and replace index-specific runtime artwork filenames with shared optimized observatory slots reused across dashboard routes.

## Impact

- `web/src/views/ScopeView.vue`: shared workbench landmarks, route/index wayfinding, review-state placement, and safe decision-instrument integration.
- `web/src/components/review/ScopeHeader.vue`, `ArtifactDocument.vue`, `CommentThread.vue`, and `DecisionInstrument.vue`: composition and semantics without changing mutation ownership.
- `web/src/style.css`: scope header imagery, desktop three-region grid, conversation atmosphere, observation band, document measure, and narrow command-dock behavior.
- `web/src/views/IndexView.vue` and index asset references only for atomic rename from `index-*` to shared `observatory-*` runtime slots; successful index layout and behavior remain unchanged.
- `web/public/assets/images/`, `web/scripts/check-runtime-images.mjs`, `designs/dashboard-review-workbench.md`, and `designs/observatory-runtime-budget.md`: shared names, exact allowlist, route roles, and unchanged 6 MiB aggregate ceiling.
- Scope component/unit tests plus `web/e2e/dashboard.spec.ts` and fixture launcher: desktop/narrow art, anchored thread alignment, decision safety, live-update behavior, focus, overflow, and browser health.
- No Rust API, sidecar format, SSE protocol, Router path, comment transition, verdict transition, package dependency, remote request, or source-art location change.
