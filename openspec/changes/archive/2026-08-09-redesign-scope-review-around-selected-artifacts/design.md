## Context

The dashboard now has three successful foundations that must survive this redesign:

1. Rust supplies rendered markdown blocks with exact source ranges and resolves comments back to those blocks, including repeated-source disambiguation and fuzzy/orphaned states.
2. `ScopeView` owns route fetches, mutation refreshes, SSE reconciliation, generation guards, dirty-composer deferral, and window-position restoration. `ArtifactDocument` owns rendered blocks and source-accurate comment initiation. `DecisionInstrument` owns unanchored/orphaned comments and phase verdicts.
3. The observatory visual system supplies bundled typography, semantic review tokens, optimized offline raster derivatives, reduced-motion behavior, and embedded desktop/narrow verification.

The current composition nevertheless targets the wrong object. It presents route identity and a three-image observation band before one concatenated stream containing every artifact. Conversation is spatially correct but usually invisible, and the face/orbit source is detached from the document as a header thumbnail. At 1536×1024 it reads as a styled web page rather than the bounded review instrument in `designs/visual-language/dashboard-mockup.png`.

The source mockup cannot be copied literally without inventing repository, activity, validation, and lifecycle data. Its important transferable structure is narrower: persistent instrumentation frames one primary document; operational chrome is compact; conversation has stable presence; artwork enters unused document space asymmetrically; and decisions belong to the chassis rather than covering prose.

Session and change scopes have different artifact cardinality but the same review mechanics:

```text
session: one scratch artifact
change: proposal.md + design.md + tasks.md + zero or more specs/<capability>/spec.md
```

The redesign therefore focuses at artifact granularity. One selected artifact owns the center and renders completely. This avoids both extremes: a mockup-literal single-block viewer that hides document context, and the current all-artifact stream that prevents stable composition.

## Goals / Non-Goals

**Goals:**

- Make scope review unmistakably belong to the `dashboard-mockup.png` universe through full-bleed instrument geometry, compact operational state, asymmetric document-stage artwork, exact-path navigation, and contextual conversation.
- Render one complete selected artifact in source order and preserve normal browser scrolling, selection, keyboard navigation, exact anchors, and readable technical prose.
- Express session and change review as policies over one shared workbench rather than separate route implementations.
- Make artifact selection durable through URL state without adding a global store or nested route hierarchy.
- Keep every real thread reachable, visibly tied to its artifact and resolved block, and actionable with existing state semantics.
- Preserve all current mutation, verdict, delivery, live-update, dirty-composer, failure, offline, accessibility, responsive, and embedded-Rust behavior.
- Keep implementation boundaries simple enough that future review behavior changes have one owner.

**Non-Goals:**

- No repository identity, activity feed, validation result, lifecycle phase, agent-presence state, neighboring-scope sidebar, or route destination absent from current APIs.
- No block-at-a-time reader, document pagination, virtualized markdown, automatic scrollspy, synchronized dual scrollers, command palette, or global frontend store.
- No second session-specific document renderer or change-specific mutation coordinator.
- No new Rust endpoint, response field, SSE event shape, sidecar format, comment transition, or verdict transition.
- No compatibility mode preserving the concatenated all-artifact scope layout.
- No new raster crop, duplicated image alias, source PNG in runtime, remote asset, motion package, or frontend package-manager path.
- No artwork beneath prose, comments, controls, focus indicators, status labels, loading state, or errors.

## Decisions

### One complete selected artifact owns the document stage

`ScopeView` continues to load a complete `ScopeDetail`, but presentation derives one selected `Artifact` and the threads whose `comment.anchor.artifactPath` matches it. `ArtifactDocument` receives that artifact rather than the whole artifact array. Every block in the selected artifact remains rendered in source order; no requirement-level focus mode or excerpting is introduced.

```text
ScopeDetail
  artifacts[]
      │
      ├── selected path ──► complete Artifact ──► document stage
      │                                        └► matching threads
      └── remaining paths ─► artifact navigator
```

For a session, the sole scratch artifact is selected automatically and an artifact chooser with one fake choice is omitted. For a change, every supplied artifact is available through exact-path navigation. If a newly created change currently has only `proposal.md`, it still uses change policy but may render a compact one-item coordinate register rather than pretending more files exist.

This choice preserves the reviewer's primary mental model—read one real document and annotate it—while giving artwork, navigation, and conversation stable desktop regions. It also minimizes code: the existing block renderer, anchor model, and scope response remain authoritative.

Alternative: retain all artifacts in one stream and only restyle it. Rejected because arbitrary combined height and repeated artifact headers keep the route visually unbounded and leave no stable conversation or art plane. Alternative: focus one block/requirement at a time. Rejected because it hides surrounding rationale, harms continuous reading, and adds substantial selection/navigation state for weaker developer experience.

### Session and change are policies over one workbench

One shared route component retains fetch, SSE, mutation, pending-update, and failure ownership. Route kind supplies only honest differences:

| Concern | Session | Change |
| --- | --- | --- |
| Artifact set | Sole scratch note when present | Proposal, design, tasks, nested spec deltas that exist |
| Artifact navigation | Hidden or compact identity | Exact-path navigator |
| Composer verdict | `keep-exploring` | `comment-resolution` |
| Advancing action | `move-to-proposal` | None here; approval remains separately owned |
| Empty copy | Scratch note not written | Change artifacts not written |

Presentation components consume explicit props; they do not inspect Router state or fetch independently. A small local policy object or computed values in `ScopeView` is sufficient; no abstract class, provider hierarchy, or Pinia store is justified.

Alternative: create `SessionReviewView` and `ChangeReviewView`. Rejected because artifact rendering, comments, live updates, failures, and responsive composition would immediately duplicate and drift.

### Exact artifact path is URL-owned

The selected change artifact is encoded in the existing route query:

```text
/changes/<name>?artifact=design.md
/changes/<name>?artifact=specs%2Fdashboard-html-views%2Fspec.md
```

`artifact` values are exact `Artifact.path` strings, not display labels or array indexes. Vue Router owns updates. Selecting an artifact pushes a history entry; initial canonicalization may replace the unqualified route so first load does not create a meaningless Back step.

Selection resolution is explicit:

1. While scope data is loading, no artifact is assumed.
2. If the route has no `artifact` query and artifacts exist, select the first server-provided artifact and canonicalize the URL with `replace`.
3. If the query exactly matches a supplied path, select it.
4. If an explicit query does not match, render an unavailable-artifact instrument that retains scope identity and navigator. Do not silently select another artifact.
5. If no artifacts exist, render the existing successful empty state and do not fabricate a query.
6. If an SSE refresh removes the selected path, preserve the URL and show unavailable state. This makes concurrent deletion visible instead of disguising it as navigation.

Artifact switching moves focus to the selected document heading and places its arrival region at the top of the viewport. Browser Back/Forward restores artifact selection through Router state. Active thread selection remains local UI state; it does not require another query parameter in this change.

Alternative: local `ref` only. Rejected because refresh, copied links, Back, and deterministic tests lose selection. Alternative: nested `/artifacts/<path>` routes. Rejected because slash-bearing paths complicate encoding and route ownership without adding behavior. Alternative: indexes. Rejected because artifact ordering changes and indexes are not identities.

### Navigator preserves exact OpenSpec coordinates

Change navigation renders exact artifact paths and may visually group path segments so nested specs are readable:

```text
proposal.md
design.md
tasks.md
specs/
  dashboard-html-views/spec.md
  dashboard-visual-system/spec.md
```

The exact full path remains available in visible or accessible text and is the query identity. Grouping is derived from slash-separated paths; the frontend does not infer fabricated Overview/Requirements/Scenarios concepts. Active selection uses `aria-current="page"`, a non-color line/glyph treatment, and existing instrument typography.

The navigator is sticky at desktop while the browser window scrolls. A session with one scratch artifact presents its identity as instrumentation, not as a clickable one-item tab. At narrow width, artifact selection precedes the selected document in normal flow and never moves behind hover-only or icon-only controls.

Alternative: hardcoded tabs for proposal/design/tasks/specs. Rejected because multiple capability specs do not fit one `SPECS` tab without another hidden selector, and absent artifacts would produce dead destinations. Alternative: filename-only labels. Rejected because different capabilities share `spec.md`.

### Conversation rail is artifact-scoped and anchor-linked

Desktop conversation gains stable presence by listing real threads belonging to the selected artifact. It is not a scope-wide chat feed: every entry exposes anchor state and can navigate to its resolved source block. Markers remain on document blocks. Selecting either marker or rail entry establishes one active thread, gives its block a visible lock treatment, scrolls the block into view when needed, and exposes existing reply/status actions.

Thread ordering follows document block/source order, with deterministic creation order for multiple threads on one block. Fuzzy threads remain associated with the block where they resolved and visibly report movement. Orphaned, missing, and unanchored comments remain in `DecisionInstrument`; they cannot be assigned to an artifact rail as though their current position were known.

The rail uses only real comments. An artifact with no threads renders a restrained empty instrument, not fake activity. The existing open/addressed/resolved distinctions and Reviewer/Agent labels remain visible and non-color-dependent.

At narrow width, the conversation instrument follows the selected document in normal DOM flow. Marker activation scrolls/focuses the corresponding rail thread and each thread provides a return-to-anchor action when it has a resolved block. This avoids a new modal/sheet primitive, duplicate interactive thread DOM, responsive JavaScript branching, and collisions with the persistent decision dock. The selected document and all thread actions remain reachable in one page.

Alternative: retain only same-row expanded threads. Rejected because conversation remains visually absent until expansion and the mockup's stable instrument density is lost. Alternative: pool every scope comment. Rejected because comments from hidden artifacts would be detached from unseen source. Alternative: render separate desktop and mobile copies. Rejected because duplicate IDs, focusable controls, and state synchronization damage accessibility and developer experience.

### Block commenting remains owned by source-rendering code

`ArtifactDocument` remains sole owner of block DOM, rendered source ranges, free-text selection, repeated-block targeting, comment markers, and exact anchor payload construction. Artifact selection changes only its input cardinality. The workbench may coordinate active thread identity and block focus through explicit events/props, but it does not reimplement source matching.

Selection comment and block comment composers remain near the source operation that created them. Submitted comments refresh scope state through existing mutation ownership, after which the new thread appears in the selected artifact rail. Invalid inline-markup selections continue to fail with their exact reason.

Alternative: move comment creation entirely into the rail. Rejected because selected source text and block position originate in document DOM and should not be reconstructed elsewhere.

### Browser window remains sole primary scroll owner

The document stage grows with the selected artifact and the browser window scrolls. Scope/artifact and conversation instruments may use CSS `position: sticky`, but there is no fixed-height center pane and no nested primary document scrollbar.

This preserves native anchor behavior, keyboard page navigation, mobile momentum scrolling, focus reveal, current live-update position restoration, and simple Playwright assertions. Artifact selection deliberately starts at the new document's arrival region; an update to the same selected artifact preserves reading position under existing reconciliation rules.

The API event only reports whether artifacts changed, not which path changed. Dirty-composer deferral therefore remains scope-level: any artifact snapshot replacement waits while unsent text exists. The redesign does not guess which file changed or partially merge snapshots. Review-state updates remain immediate.

Alternative: recreate the source mockup as a fixed viewport with independently scrolling center and right rails. Rejected because dual scroll ownership makes focus, restoration, narrow behavior, and SSE reconciliation materially harder for negligible semantic benefit.

### Full-bleed instrument composition replaces hero plus gallery

Desktop composition follows source hierarchy without copying unsupported data:

```text
┌ scope / artifact rail ┬ compact route identity + standing state ─────────────┐
│ real scope state      ├───────────────────────────────┬───────────────────────┤
│ exact artifact paths  │ selected document             │ artifact conversation │
│                       │ protected reading plane        │ real threads only     │
│                       │ asymmetric face/orbit framing  │ decision instrument   │
└───────────────────────┴───────────────────────────────┴───────────────────────┘
```

At 1280px and above, route chrome uses useful viewport width instead of a centered page-card margin. Target proportions are approximately 14–18rem for scope/artifact instrumentation, a flexible document measure that never falls below readable width, and 20–26rem for conversation. These are CSS constraints, not fixed screenshot pixels. Rails consume surplus width before document measure.

Header height is compact enough that selected artifact title and first meaningful source block appear in the initial 1536×1024 review viewport. Scope title, exact key, standing verdict, submission age, delivery state, and history remain visible, but route identity does not become a second oversized document hero.

Hairlines connect header, rails, document stage, conversation, and decision controls into one chassis. Corners remain square or restrained. Empty regions use line, node, and crosshair geometry rather than generic cards, shadows, or equal thumbnail panels.

Alternative: preserve centered outer padding and make components more ornate. Rejected because source atmosphere comes from whole-screen proportion and connected geometry, not decorative density inside isolated boxes.

### Artwork is integrated into document arrival, not detached from it

`observatory-field.webp` becomes the selected document's bounded arrival field. Its face/orbit/globe detail enters asymmetrically from the right while document identity and first source heading occupy a protected opaque left plane. Artwork can extend into unused margin, but painted pixels stop before prose, markers, composers, state labels, and focus rings.

The current three-equal-image pre-document band is removed. Shared plate derivatives may appear as subordinate hard-cropped observation cells in desktop rail/footer chassis, matching the source mockup's peripheral vocabulary, but they do not delay document arrival, receive labels implying activity, or become clickable cards. At intermediate width these cells disappear before rails or content shrink. At 390px they are absent.

Artwork remains `aria-hidden`, uses empty alternative text, is pointer-inert, and keeps existing offline WebP paths and runtime budget. New derivatives require separate evidence that CSS cropping cannot satisfy a named slot; none are expected.

Alternative: use the field as a route-wide background. Rejected because readability would depend on crop and artwork would become wallpaper. Alternative: keep the top gallery. Rejected because equal image panels read as portfolio thumbnails and physically separate atmosphere from review.

### Decision controls belong to conversation-side chassis

Desktop decision controls occupy the conversation-side instrument and remain persistent without crossing the document reading plane. Opening unanchored/orphaned comments and verdict composer may use bounded conversation width. The selected document reserves enough trailing space that its final marker and rail actions can scroll above persistent controls.

At narrow width, decision controls use existing safe-area-aware bottom behavior and page clearance. They remain separate from artifact selection and thread navigation. Session copy/actions and change copy/actions continue to come from route kind; no lifecycle state is inferred from artifact filenames.

### Responsive behavior changes placement, not ownership

At intermediate widths, decorative plate cells disappear first, then left instrumentation enters a compact horizontal/normal-flow form, then conversation follows document. Exact paths, state, comments, and decisions remain visible.

At 390×844, DOM reading order is:

```text
scope identity and standing state
artifact selection or scratch identity
selected document
selected-artifact conversation
scope-level loose comments and decisions
```

Long scope keys and artifact paths wrap. Document tables and code may scroll within their own semantic element, but the page itself has no horizontal overflow. Artwork reduces to a shallow bounded arrival crop or disappears. No essential action depends on hover.

### State ownership remains local and explicit

`ScopeView` owns:

- loaded/pending `ScopeDetail`;
- selected artifact derived from Router query plus loaded artifacts;
- active thread id;
- mutation busy/error state;
- dirty composers and deferred artifact snapshot;
- SSE connection and generation guards.

`ArtifactNavigator` emits exact path selection. `ArtifactDocument` emits comment operations and active-thread intent. `ArtifactConversation` emits reply/status and anchor-navigation intent. `DecisionInstrument` retains loose-thread and verdict operations. No child fetches scope data, mutates Router independently, or silently repairs invalid state.

This boundary keeps executable truth near current owners and avoids a single-use abstraction layer.

### Verification measures behavior and source fidelity

Deterministic desktop fixture remains `/changes/implement-observatory-design-system-with-a-realistically-long-identifier`, expanded to contain multiple exact artifact paths and comments across at least two artifacts. Desktop source review uses 1536×1024, matching `dashboard-mockup.png`; embedded acceptance may retain project-standard 1440×1000 where required, but screenshot evidence records both target and gate viewports.

Desktop acceptance checks:

- workbench uses full-bleed connected chassis rather than centered hero/gallery sections;
- selected artifact and first meaningful block appear in initial viewport;
- exact artifact rail visibly selects one path;
- face/orbit field is integrated into selected document arrival and does not underlay prose;
- pre-document three-equal-plate gallery is absent;
- conversation rail contains only real selected-artifact threads and thread navigation reaches exact repeated occurrence;
- decision controls do not overlap document or conversation actions;
- no unsupported repository/activity/validation state appears.

Behavioral acceptance checks direct `?artifact=` loading, canonical default selection, Back/Forward, invalid selection, session auto-selection, cross-artifact switching, exact repeated anchors, comment creation, all thread transitions, verdict submission, clean and dirty live updates, and current failure distinctions.

Narrow acceptance checks one logical flow, selected-path reachability, complete document rendering, thread/anchor navigation, decision clearance, long path wrapping, no horizontal overflow, no plate band, and no hidden hover-only action.

Completion remains `bun run format`, `bun run check`, `bun run build`, and embedded `bun run test:e2e`; Vite screenshots are design input only.

## Risks / Trade-offs

- **[Risk] Artifact query makes previously simple URLs longer.** → Canonicalize only when artifacts exist, use exact paths, and keep scope identity in unchanged route path.
- **[Risk] Artifact switching hides unresolved comments in other documents.** → Scope counts remain global and navigator shows per-artifact thread state derived from loaded comments; decision submission still represents whole scope.
- **[Risk] Artifact-scoped rail weakens immediate spatial adjacency.** → Keep block markers, active block lock treatment, bidirectional thread/anchor navigation, quoted anchor text, and visible fuzzy/lost state.
- **[Risk] Rail becomes crowded on comment-heavy documents.** → Let rail participate in window scroll or use bounded sticky content only where actions remain reachable; do not introduce a second hidden thread copy.
- **[Risk] Integrating artwork into arrival region competes with source H1.** → Protect source content on opaque plane, tune crop in unused right margin, and require artwork to disappear before reducing document measure.
- **[Risk] Session with no scratch and change with no artifacts have no default selection.** → Preserve explicit successful empty instruments and omit fabricated navigation/query state.
- **[Risk] SSE removes selected artifact.** → Keep requested URL and show explicit unavailable state with remaining exact paths; never silently switch.
- **[Risk] Current tests encode all-artifact rendering and same-row expansion.** → Replace only superseded composition assertions while retaining mutation, anchor, live-update, and failure coverage.
- **[Risk] Shared scope CSS leaks into successful index.** → Keep scope selectors under workbench roots and preserve central token semantics without changing shared primitive defaults.
- **[Risk] Mockup fidelity tempts fake shell data.** → Every visible state/navigation label must map to scope/index response or static instrument language; browser fixtures assert unsupported claims are absent.

## Migration Plan

1. Lock selected-artifact URL, empty, invalid, session, and change behavior in route/component tests before changing layout.
2. Introduce exact-path navigator and derive selected artifact/threads in `ScopeView` without changing fetch, mutation, or SSE ownership.
3. Change `ArtifactDocument` input to one selected artifact and wire active thread plus bidirectional anchor navigation.
4. Recompose conversation as selected-artifact instrument while preserving loose/orphaned comments in `DecisionInstrument`.
5. Replace scope hero/gallery markup and CSS with compact full-bleed chassis and bounded document-arrival observation field; relocate or remove scope plate usage atomically with reference/spec updates.
6. Implement intermediate and 390px flow without duplicate interactive DOM or nested primary scrollers.
7. Update deterministic fixtures and unit/E2E coverage for session auto-selection, multi-artifact changes, URL history, invalid selection, comments, live updates, screenshot geometry, accessibility, and browser health.
8. Run frontend and embedded completion gates, inspect source comparison at 1536×1024 and narrow output, then review diff for API drift, duplicated state, fake data, old concatenated layout, stale plate assumptions, and unrelated refactors.

Rollback reverts selected-artifact URL behavior, workbench components, scope styles, specs, reference, and tests together. No persisted data or server migration exists. Old concatenated layout is not retained as a fallback.

## Open Questions

None. Artifact-level focus, shared session/change workbench, exact query identity, normal window scrolling, selected-artifact conversation, integrated document artwork, current API boundary, and no compatibility layout are settled.
