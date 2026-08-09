## Context

The index redesign proved the desired direction with one asymmetric face/orbit field and three hard-cropped observation plates. The shared scope workbench still uses the earlier visual foundation: at 1440×1000 the requested fixture renders a 340px boxed header, an oversized scope title, a verdict box, and a clean SVG orbit; the artifact begins below a sparse utility rail. At 390×844 the same information stacks correctly, but the fixed decision bar crosses the utility/document boundary and the first artifact heading, while no authored image survives.

The behavior must not be destabilized. `ScopeView` owns route-driven fetching, SSE reconciliation, dirty-composer deferral, review-state merging, and scroll restoration. `ArtifactDocument` owns exact block anchoring, repeated-text positioning, selection comments, and spatial thread expansion. `DecisionInstrument` owns unanchored/orphaned comments and verdict submission. Art direction must compose these owners, not merge or rewrite them.

`ScopeView` is shared by session and change routes. Styling only the requested `/changes/...` fixture would create route-conditioned markup for no product reason, so the redesign applies to both kinds while using the realistic long change route as acceptance target.

The successful index currently ships four index-prefixed WebP derivatives totaling 249,596 bytes. Scope composition needs the same authored sources and crops. Copying them under new scope-prefixed names would double payload and create two files for one visual fact, so this change promotes them to shared observatory names and updates both routes atomically.

## Goals / Non-Goals

**Goals:**

- Make scope arrival feel like the same authored observatory as the index: black void, right-entering halftone face, clipped orbit geometry, hard crops, and aligned hairlines.
- Keep artifact content primary, readable, and exact.
- Preserve a persistent document-coordinate rail and spatially anchored conversation without imitating the mockup's detached thread feed.
- Prevent the decision instrument from spanning or permanently obscuring the document spine.
- Keep change and session scope routes on one composition.
- Reuse one optimized runtime derivative set across index and scope routes.
- Preserve all existing network, mutation, live-update, accessibility, responsive, and failure behavior.

**Non-Goals:**

- No new API fields, activity stream, repository identity, validation result, task completion model, lifecycle phase, or agent-presence state.
- No comment pooling, thread search, filtering, sorting, artifact tabs, scrollspy, command palette, or route-specific workbench fork.
- No rewrite of markdown block rendering, anchoring, comment transitions, verdict transitions, or SSE reconciliation.
- No new image crop or raster payload unless implementation proves the shared derivatives cannot satisfy the named slot; preference is strict reuse.
- No full-viewport nested document scroller. Window scrolling remains authoritative so live-update scroll restoration stays correct.
- No decorative perpetual motion, parallax, carousel, or image loading spectacle.

## Decisions

### Shared scope composition remains document-first

Desktop structure becomes:

```text
┌ route / scope identity / verdict ──────────────── observation field ┐
│ compact title + exact key                  face/orbits enter right  │
├ sun crop ───────── eye/face crop ───────── star-system crop ───────┤
├ utility rail ───┬ document spine ─────────────┬ conversation rail ┤
│ ← scope index   │ proposal/design/tasks/specs │ expanded anchored │
│ comment counts  │ rendered blocks             │ thread at block   │
│ artifact paths  │ comment markers/gutter      │ composer/actions  │
│                 │                              │ decision dock     │
└─────────────────┴──────────────────────────────┴───────────────────┘
```

`ScopeView` retains landmark ownership: main workbench, utility aside, artifact section, anchored conversation inside each block row, and decision aside. CSS and small presentational wrappers establish the three-region geometry. There is no new global store or review coordinator.

Alternative: copy the mockup's permanently pooled `THREADS` panel. Rejected because it detaches comments from their anchors and contradicts the shipped anchored-comment requirement. Alternative: make the entire route one cinematic image canvas. Rejected because artwork would own reading measure and controls.

### Scope header becomes compact identity plus authored field

`ScopeHeader` keeps the scope title, exact key, standing verdict, submission age, delivery state, and history. Product/route wayfinding gains a real Router link to `/`, the implemented index. The descriptive sentence may remain but cannot compete with artifact content.

The title uses a restrained display size and measure. Its purpose is route identity, not duplication of the document's first heading. The artifact H1 remains untouched because it is source content and must render exactly; reducing header gravity solves repetition without suppressing document truth.

The geometry-only `OrbitalFrame` is removed from `ScopeHeader`. A shared `observatory-field.webp` image fills an isolated right-hand plane. The same derivative's crop already excludes source pseudo-controls and retains black safe space plus face/orbit/globe forms. Scope-specific `object-position`, clipping, and masks may differ from index use, but text and state occupy opaque planes and painted pixels stop before their bounds.

At intermediate width, verdict state moves beneath identity before document measure shrinks. At 390px the authored field becomes one shallow bounded eye/orbit crop below route state or disappears if it collides with title/key/state. No art sits behind the title, key, verdict, history, error, or focus ring.

Alternative: generate another near-identical scope hero. Rejected unless shared crop demonstrably fails; duplicate source semantics and bytes are not justified by a CSS crop difference.

### Observation band keeps source art visible without delaying the document

A desktop-only band directly under the compact header reuses the sun, face/eye, and star-system plates. Height stays restrained, roughly 88–120px, so first artifact heading remains visible within a 1000px-tall desktop viewport. Hard edges and shared rules make it part of the instrument chassis, not a gallery.

No plate receives activity labels, lifecycle labels, counts, links, hover effects, or click behavior. Group and images are `aria-hidden`, use empty alt text, and cannot receive pointer events. At intermediate widths plates reduce from three to one or two; at 390px the entire band disappears before utility or artifact content.

Alternative: place plates only after all artifacts. Rejected because real changes contain long proposal/design/task/spec stacks and the art direction would be effectively absent. Alternative: insert plates between artifacts. Rejected because decorative interruptions would fragment document review.

### Utility rail becomes explicit route and document instrumentation

The desktop utility aside remains sticky against window scrolling. It contains, in order:

1. real link back to index;
2. open/addressed/resolved counts;
3. exact artifact anchor links.

It does not add fake Overview/Requirements/Scenarios tabs because the API exposes artifact paths, not those guaranteed categories. Exact long paths wrap; they are not truncated into ambiguous filenames. Existing fragment links remain native, preserving copy/open-in-new-tab behavior where applicable.

At 390px utility enters normal flow before artifacts, as today. Index link, counts, and artifact paths remain visible. No collapsible drawer or hamburger hides them.

Alternative: implement active artifact scrollspy. Rejected as interaction scope unrelated to art direction and unnecessary for anchor navigation.

### Conversation rail remains spatial and gains visual chassis

`ArtifactDocument` keeps each `review-block-row` as document block plus optional conversation. Desktop reserves useful conversation width instead of allowing the right region to read as accidental emptiness. A central ruled backdrop, node line, or crosshair geometry can articulate the empty rail but must be CSS/SVG, `aria-hidden`, and free of fake labels or event counts.

Collapsed thread markers remain beside their block. Expanding one renders `CommentThread` in that block's conversation cell; a new anchored composer occupies the same cell. Reviewer/Agent labels, anchor-moved/lost signals, reply/resolve/reopen/accept controls, and status tokens remain unchanged.

At widths below the desktop threshold, conversation follows its block. No sticky or detached thread panel survives narrow mode. This preserves reading order and the exact anchor relationship.

Alternative: show compact previews of every thread in a permanent right feed. Rejected because it duplicates comment bodies and creates two reachability surfaces to reconcile.

### Decision instrument moves off the document spine

Desktop fixed/sticky decision chrome is constrained to the conversation-side instrument region rather than spanning the center document. Its closed trigger remains persistently reachable. The rail reserves enough bottom inset that an expanded anchored thread can be scrolled clear of the dock.

Opening the drawer may expand within or over the conversation-side plane because the reviewer invoked it, but it must not cover the document spine. Orphaned and unanchored threads, optional comment textarea, and submission controls remain in the drawer. Focus remains within normal document order; this change does not add modal trapping.

At 390px the command dock becomes a compact full-width bottom instrument with safe-area inset and a matching reserved page tail. The last artifact block, comment action, expanded thread, and composer must all scroll fully above it; no control or text may remain unreachable beneath the dock. When the drawer opens, it may occupy bounded viewport height and scroll internally, while the close control, loose threads, composer, and verdict action remain reachable.

Alternative: move the decision composer to document end. Rejected because phase decisions stop being persistent. Alternative: introduce a full-screen modal. Rejected because detached interaction and focus-management complexity are unnecessary.

### Runtime artwork is promoted, not duplicated

Rename files and all references atomically:

```text
index-observation-field.webp  → observatory-field.webp
index-plate-sun.webp          → observatory-plate-sun.webp
index-plate-face.webp         → observatory-plate-face.webp
index-plate-star-system.webp  → observatory-plate-star-system.webp
```

Bytes and source crops remain unchanged unless visual verification rejects a slot. `IndexView` updates paths only; its successful composition must not change. `designs/dashboard-review-workbench.md` records both index and scope rendered roles for each shared derivative. `designs/observatory-runtime-budget.md` records shared names and same measured total. Runtime allowlist contains exactly the four shared paths and rejects old index names, duplicate scope aliases, and source PNGs.

Alternative: keep index names and reference them from scopes. Rejected because runtime names would encode false ownership. Alternative: copy files under scope names. Rejected because it doubles payload and violates careful DRY.

### Live behavior remains outside visual refactor

`ScopeView` fetch, mutation, event parsing, generation guards, dirty composer set, deferred artifact state, review-state merge, and window scroll restoration are untouched except where markup placement passes existing events/props through. No silent default or fallback is added.

Tests that exercise clean live updates, immediate review-state refresh, dirty-composer deferral, latest-update wins, exact repeated occurrence, inline-markup refusal, cross-tab updates, and server refusal remain required. Visual restructuring is not evidence those paths still work.

### Verification targets source direction and operational truth

The primary desktop review uses 1440×1000 against the exact realistic change fixture. Acceptance checks:

- compact route identity and real state remain legible;
- face/orbit field enters from right without boxed-thumbnail treatment;
- plate band appears before document and remains subordinate;
- Proposal header and document content remain visible in first viewport;
- utility, document, and conversation regions read as one ruled instrument;
- decision trigger does not cross document bounds;
- expanding repeated-block comment opens beside second occurrence.

At 390×844, acceptance checks:

- identity, verdict, delivery, history, counts, exact key, paths, artifact content, comment markers, and decision controls remain reachable;
- plate band is absent and hero crop yields as needed;
- conversation follows anchored block;
- long identifier/path wraps without horizontal overflow;
- final content and active composer scroll above command dock.

Embedded Rust output remains completion evidence. Vite mock screenshots are design-inspection input only.

## Risks / Trade-offs

- **[Risk] Shared index crop lacks enough scope-specific detail.** → First tune `object-position`, clip, and mask per route; add a new derivative only after screenshot evidence proves crop reuse inadequate and then update proposal/spec/reference together.
- **[Risk] Art band pushes document below fold.** → Cap band at restrained desktop height and require Proposal heading in 1440×1000 first viewport.
- **[Risk] Header and artifact still feel duplicative.** → Reduce route title gravity; never hide or mutate source H1.
- **[Risk] Right-side decision dock collides with expanded conversation.** → Reserve rail bottom inset and test expanded open, addressed, resolved, and composer states at realistic content lengths.
- **[Risk] Narrow fixed dock obscures content.** → Use safe-area-aware height plus explicit page tail; assert final block/composer can scroll fully above dock.
- **[Risk] Asset rename breaks the successful index.** → Rename paths, allowlist, reference, budget, tests, and both route references atomically; embedded browser health rejects missing requests.
- **[Risk] Scope CSS changes leak into index or existing primitives.** → Keep selectors under scope workbench blocks and reuse central tokens without changing shared primitive defaults.
- **[Risk] Visual work accidentally changes review state.** → Preserve component owners and existing mutation/live-update tests; review diff for script changes not required by markup wiring.

## Migration Plan

1. Rename four runtime derivatives to shared observatory paths and update allowlist, budget record, design reference, index references, and asset tests as one atomic step.
2. Recompose `ScopeHeader` with index route link, compact identity/state planes, shared observation field, and responsive image treatment.
3. Add the desktop observation band from shared plates.
4. Refine scope utility/document/conversation grid while preserving `ArtifactDocument` anchoring ownership.
5. Reposition `DecisionInstrument` into conversation-side desktop geometry and narrow safe dock without changing emitted operations.
6. Update component and route tests, then extend embedded desktop/narrow fixtures and screenshot evidence.
7. Run Bun format/check/build/e2e, runtime asset checks, keyboard/reduced-motion review, and source-board visual comparison.

No persisted-data or server migration exists. Rollback must restore old asset names, index references, allowlist, scope markup, styles, reference, and tests together; partial rollback leaves embedded asset requests broken.

## Open Questions

None. Shared scope application, shared derivative rename, compact header, pre-document observation band, spatial conversation, and no API expansion are settled.
