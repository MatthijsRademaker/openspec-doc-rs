## Context

Current dashboard composition already reads as an observatory: index register, exact artifact coordinates, selected-document stage, artifact-scoped conversation, semantic review-state geometry, and static celestial framing. Motion is centralized around `--motion-duration: 160ms`, `--motion-easing`, and `--motion-nudge`; reduced motion globally zeros duration and displacement. Existing effects explain rail collapse, composer expansion, drawer entry, source/thread activation, and remote-content arrival, but they do not yet form a recognizable interaction language.

This change affects several interaction owners under `web/src`: Router-owned navigation, `ScopeView` mutation and live-update orchestration, document and conversation targeting, semantic status presentation, decision drawer behavior, and centralized CSS. Rust APIs already expose every event needed. New animation data must not leak into URLs, persisted review records, or global application state.

## Goals / Non-Goals

**Goals:**

- Make review actions feel like operations performed through a precise optical instrument.
- Give acquire, triangulate, transmit, receive, resolve, and reconfigure events consistent visual grammar.
- Tie every effect to real navigation, mutation, state, or remote-update events.
- Preserve immediate content access, focus order, browser history, copied URLs, scrolling behavior, and failure reporting.
- Keep standard transitions inside 100–200ms and preserve global reduced-motion suppression.
- Verify desktop, 390px narrow, keyboard, reduced-motion, and embedded-browser behavior deterministically.

**Non-Goals:**

- Perpetual orbit rotation, parallax, pointer-following fields, cursor trails, sound, haptics, spring physics, decorative loaders, or whole-page glitch effects.
- Animating source-board raster assets or adding runtime images.
- Adding a motion library, global store, canvas renderer, WebGL, or JavaScript layout-animation framework.
- Claiming which artifact blocks changed during a remote rewrite.
- Changing Rust endpoints, review records, mutation semantics, or anchor resolution.

## Decisions

### 1. Motion is an event grammar with six verbs

Central styles and small transient view state will express:

| Verb | Trigger | Visual grammar | Settled state |
| --- | --- | --- | --- |
| Acquire | Scope, artifact, or thread selection | Registration rule, hard reveal, target ticks | Existing selected/current treatment |
| Triangulate | Bidirectional source/thread navigation | Paired origin/destination locks plus brief direction trace | Existing active thread and block |
| Transmit | Reviewer mutation in flight | One-shot carrier rule tied to submitting control | Returned thread, state, or verdict |
| Receive | Successful mutation or SSE update | Arrival edge, lock, or optical slit where content lands | Updated content and existing semantic state |
| Resolve | Review status or delivery changes | Glyph/line geometry closes or redirects | Existing visible label and token |
| Reconfigure | Rail or decision surface opens/closes | Hard mask and chassis-track movement | Existing expanded/collapsed or modal state |

Effects animate events, then disappear. Persistent state remains expressed by existing labels, glyphs, borders, and layout.

Alternative: animate celestial artwork continuously. Rejected because motion would communicate no event, compete with document reading, and violate current visual-system contract.

### 2. Use native platform and Vue transitions, not a motion dependency

Use CSS transitions, Vue `<Transition>`, and native View Transitions where they materially connect two route-owned states. A small focused navigation helper may wrap a Router mutation in `document.startViewTransition`; unsupported environments perform the same Router mutation without shared pixels. This is progressive presentation, not behavioral fallback: navigation, focus, URL, history, and selected content are identical.

Index-to-scope acquisition will share the exact scope coordinate, not mutable display title. Scope loading state can render that route coordinate truthfully before scope data arrives, allowing acquisition to terminate on data already known from the route. Artifact selection transitions two exact shared coordinates because both old and new artifacts exist in current scope data: the selected path acts as a compact carrier, while its selection glyph terminates as a registration reticle beside the destination path. A hard scan crosses only the selected-document header as the reticle settles. The native root snapshots do not crossfade; old document and conversation pixels disappear immediately so unrelated content never ghosts together. At narrow width, shared travel is disabled and the destination reports acquisition locally rather than traversing the stacked reading flow.

View-transition names must be assigned only to the initiating source and current destination, then cleared. Direct URLs, Back/Forward, invalid artifact coordinates, fetch failure, and reduced motion must not depend on an initiating element.

Alternative: preload complete scopes or add a global transition store. Rejected as unnecessary data ownership and failure complexity. Alternative: delay navigation until an outgoing animation finishes. Rejected because presentation would withhold navigation.

### 3. Keep transient motion state local to existing owners

`ScopeView` remains owner of selected artifact, active thread, mutations, SSE reconciliation, and dirty-composer deferral. It will expose short-lived event state such as current acquisition target, mutation signal, or receive target to child components. Index navigation state remains local to the index/register path. No event survives route disposal or enters persisted application data.

A shared helper may own timer cleanup and latest-event replacement for one channel, extending the current arrival-mark pattern. Concurrent channels remain independent: a remote document receive must not erase a reviewer-thread confirmation. Repeated events on one channel replace the older transient mark rather than stacking animation nodes.

### 4. Triangulation uses paired locks, not a line drawn through content

Activating a marker or thread will mark the origin before navigation and the exact destination after focus/scroll settles. A brief viewport-edge direction trace communicates travel. Desktop may orient the trace toward the adjacent plane; narrow layout uses a local vertical direction mark because source and conversation no longer coexist horizontally.

No SVG connector will cross document prose, comments, controls, or the viewport during smooth scrolling. Origin and destination remain understandable without animation through active treatment, focus movement, accessible names, and existing `aria-controls` relationships.

Alternative: draw a continuous Bézier connector between source and thread. Rejected because scrolling, sticky rails, responsive reflow, and long documents make it fragile and visually invasive.

### 5. Transmission reflects real mutation lifecycle

Submitting comment, reply, status, or verdict sets a visible and accessible transmitting state only while its real request is unresolved. Success terminates at the actual returned destination:

- new comment or reply: created or extended thread;
- status change: affected thread and semantic state mark;
- verdict: standing verdict/delivery instrument.

Failure terminates transmission and preserves existing explicit action error. No success pulse runs before API success. Controls may remain disabled while busy as today, but their text or adjacent status must name the operation rather than presenting inert disabled chrome.

`ScopeView.mutate` should return structured outcome metadata rather than treating every mutation as an optional thread id. This keeps one mutation pipeline while allowing truthful destinations without duplicating API logic.

Alternative: optimistic completion animation. Rejected because rollback would manufacture false review state.

### 6. State metamorphosis augments, never replaces, semantic meaning

Open, addressed, resolved, verdict, and delivery components retain visible text, stable token usage, and final glyphs. Pseudo-elements and short entering/leaving glyph layers provide broken-line closure, directional traversal, or closed-lock settlement. Components must not require an observer to see intermediate frames to understand state.

Status counts update directly from source data. Motion may report changed status but must not interpolate numeric truth or queue stale intermediate values.

### 7. Remote receive marks the landing surface without inventing a diff

Existing document-replacement arrival becomes a brief optical receive: hard slit or scan edge over the document stage, combined with persistent-enough textual `Document content replaced` dwell. Overlay intercepts nothing and new content is present immediately. It marks whole-document replacement only; no block is labelled changed.

Review-state updates mark only known destinations such as added threads, changed statuses, or standing verdict. Deferred artifact updates animate when applied, not when first detected, preserving current dirty-composer contract.

Alternative: diff old and new blocks for localized flashes. Rejected because block identity and drift belong to anchor resolution and a second diff model could contradict it.

### 8. Chassis motion preserves existing geometry and modal behavior

Conversation collapse continues to interpolate compatible grid tracks so document measure does not reflow. Additional line/mask treatment may make the rail read as a retracting instrument, but it cannot introduce a second document layout.

Decision drawer keeps Teleport, backdrop, focus trap, Escape dismissal, scroll-to-start, and trigger focus restoration. Its generic soft rise becomes a hard-edged clip/mask deployment with small registration displacement. Narrow dock position and safe-area clearance remain unchanged.

### 9. Motion budget and reduced motion are centralized

Standard effects use existing 160ms duration and easing. Any departure from that token requires explicit event-specific rationale and cannot exceed 200ms. Nonblocking arrival text may retain current 2.2s dwell because it withholds nothing and self-clears.

Global `prefers-reduced-motion` inversion remains authoritative: duration and displacement become zero, native view transitions are skipped, smooth scroll becomes immediate, and every final state remains visible through text and geometry. No component-specific reduced-motion allowlist will be introduced.

### 10. Verification tests outcomes, not animation internals

Component tests will assert transient event classes/status text, cleanup, truthful success/failure destinations, and reduced-motion branch selection. Embedded Playwright coverage will exercise:

- index-to-scope and artifact navigation with correct URL/history/focus;
- marker-to-thread and thread-to-source navigation at desktop and narrow widths;
- mutation transmitting, success, and failure states;
- remote artifact receive without reading-position loss;
- decision drawer focus behavior through its new transition;
- zero-duration/reduced-motion operation;
- no console, request, asset, or horizontal-overflow failures.

Tests will not depend on pixel-perfect intermediate frames. They may inspect stable transition hooks or capture screenshots only at deliberate event endpoints.

## Risks / Trade-offs

- **Native View Transitions differ by browser** → Keep them presentational; Router mutation, focus, history, and fallback rendering remain identical and are tested independently.
- **Too many simultaneous marks create noise** → Limit one transient mark per event channel and one hero gesture per user action; newest same-channel event replaces older mark.
- **Smooth scrolling can obscure cause and effect** → Mark origin before movement and destination after movement; reduced motion uses immediate travel with same settled locks.
- **Clip and mask effects can cover focus or prose** → Render non-interactive overlays in dedicated framing planes and test focus rings plus narrow layout.
- **Busy state can become fake progress theatre** → Tie transmission strictly to request lifetime and retain explicit error text on rejection.
- **Status animation can imply delayed truth** → Update text and semantic state immediately; animate only decorative geometry around final truth.
- **Motion CSS can become scattered** → Keep vocabulary, timing, and event selectors centralized in `web/src/style.css`; component classes expose event meaning, not local timing values.

## Migration Plan

Implement vocabulary behind existing interaction owners, replacing old generic transitions as each event path gains its final treatment. No data migration or compatibility layer exists. Remove superseded transition selectors, stale arrival classes, tests, and comments in same change. Build frontend before Cargo and verify only freshly embedded output.

Rollback is source-level removal of event classes/helpers and restoration of immediate existing states; no persisted data or server contract needs rollback.

## Open Questions

None required before implementation. Exact line, slit, and mask geometry should be chosen against desktop and 390px rendered fixtures while preserving contracts above.
