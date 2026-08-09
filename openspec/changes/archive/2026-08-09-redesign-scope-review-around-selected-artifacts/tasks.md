## 1. Lock selected-artifact contract before composition changes

- [x] 1.1 Extend deterministic change fixtures with `proposal.md`, `design.md`, `tasks.md`, at least two nested capability spec paths, repeated source blocks, and comments anchored across more than one artifact
- [x] 1.2 Add `ScopeView` tests proving an unqualified change route selects the first server-provided artifact and canonicalizes `?artifact=` with Router replace rather than a new history entry
- [x] 1.3 Add route tests proving encoded nested artifact paths load directly and artifact selections participate in browser Back and Forward
- [x] 1.4 Add failure tests proving an explicitly invalid artifact query retains scope identity and available exact paths while refusing silent fallback
- [x] 1.5 Add live-update tests proving removal of the selected artifact preserves the requested query and exposes unavailable state rather than switching documents
- [x] 1.6 Add session tests proving the sole scratch artifact is selected automatically without fake tabs, folder rows, or inactive destinations
- [x] 1.7 Preserve baseline tests for successful no-artifact scopes, fetch failure, action failure, global comment counts, standing verdict, delivery state, verdict history, and decision actions

## 2. Establish URL-owned artifact selection

- [x] 2.1 Derive selected artifact in `ScopeView.vue` from loaded `ScopeDetail.artifacts` and the exact Router `artifact` query without adding a global store
- [x] 2.2 Canonicalize a missing selection to the first server-provided artifact only after successful data load and omit selection entirely for a successful empty scope
- [x] 2.3 Push exact query state when a reviewer selects another artifact and move focus/reading position to the selected document arrival heading
- [x] 2.4 Preserve explicit invalid or removed selections as unavailable state with remaining navigation reachable
- [x] 2.5 Clear or reconcile active-thread state when artifact selection changes so a hidden artifact's thread cannot remain active in visible conversation
- [x] 2.6 Keep existing load-generation guards, mutation refresh ownership, action failures, SSE ownership, and pending-artifact snapshot semantics unchanged except where selected presentation is derived

## 3. Build exact-path artifact instrumentation

- [x] 3.1 Add one focused artifact navigator component under `web/src/components/review/` that accepts supplied paths and emits exact path selection without fetching or inspecting Router state
- [x] 3.2 Present exact `proposal.md`, `design.md`, `tasks.md`, and nested `specs/<capability>/spec.md` identities with readable path grouping while retaining unambiguous full paths
- [x] 3.3 Mark selected path with `aria-current="page"`, visible text, and non-color glyph/line treatment using central observatory tokens
- [x] 3.4 Render session scratch identity as instrumentation rather than a clickable one-item navigator
- [x] 3.5 Expose real per-artifact thread state derived from loaded comments without inventing lifecycle, validation, completion, or activity metadata
- [x] 3.6 Verify long scope keys and nested paths wrap without truncation, ambiguous filename-only labels, or horizontal overflow

## 4. Render one complete selected document

- [x] 4.1 Change `ArtifactDocument.vue` to render one selected `Artifact` completely rather than iterate the entire artifact array
- [x] 4.2 Preserve rendered block order, source HTML handling, tables, code overflow, heading hierarchy, exact source ranges, and readable prose measure within selected document
- [x] 4.3 Preserve block comment initiation, free-text selection comments, repeated-source `searchFrom` targeting, and inline-markup refusal with reason
- [x] 4.4 Keep status-distinct comment markers on every selected-document block with resolved threads
- [x] 4.5 Add explicit active-thread and active-block props/events without moving anchor construction or source matching out of `ArtifactDocument`
- [x] 4.6 Give the selected source block a visible non-color lock treatment when its marker or corresponding thread is active
- [x] 4.7 Preserve successful empty-document rendering when no artifact exists and add distinct selected-artifact-unavailable rendering for an invalid exact query

## 5. Recompose artifact-scoped conversation

- [x] 5.1 Add or adapt one conversation component that receives only threads belonging to selected artifact and never fetches scope data independently
- [x] 5.2 Order selected-artifact threads by resolved source-block position and deterministically by creation order for multiple threads on one block
- [x] 5.3 Make block marker activation focus the corresponding thread and keep its exact source block visibly active
- [x] 5.4 Make thread activation scroll and focus its exact resolved block, including the second occurrence of repeated source text
- [x] 5.5 Preserve visible fuzzy-anchor movement, Reviewer/Agent labels, open/addressed/resolved semantics, replies, resolve, accept, and reopen actions
- [x] 5.6 Keep missing, orphaned, and unanchored threads in `DecisionInstrument` with original quoted text and lost-anchor state rather than assigning them to selected artifact
- [x] 5.7 Render a restrained honest empty conversation instrument when selected artifact has no threads and do not invent activity or agent state
- [x] 5.8 At narrow width, place selected-artifact conversation after complete document in normal DOM flow and provide return-to-exact-anchor controls without duplicate interactive thread markup
- [x] 5.9 Preserve immediate comment/status refresh and cross-tab SSE reconciliation for selected and non-selected artifact threads

## 6. Replace scope page composition with observatory chassis

- [x] 6.1 Recompose `ScopeHeader.vue` as compact route/state instrumentation retaining title, exact key, standing verdict, submission age, delivery state, and verdict history
- [x] 6.2 Replace centered scope-page proportions with a full-bleed hairline-connected desktop workbench for instrumentation rail, selected document, artifact conversation, and decisions
- [x] 6.3 Ensure selected artifact identity and first meaningful source block appear in the initial 1536×1024 viewport without an oversized duplicate route hero
- [x] 6.4 Move `observatory-field.webp` into a bounded asymmetric selected-document arrival plane with protected opaque text region and pointer-inert decorative semantics
- [x] 6.5 Remove the three-equal-image pre-document scope gallery and delete markup/styles/tests made obsolete by that composition
- [x] 6.6 Reuse shared plate derivatives only in subordinate hard-edged rail/footer chassis cells where they do not delay document arrival or imply product state
- [x] 6.7 Use CSS/SVG hairlines, nodes, crosshairs, and scan geometry to articulate empty instrument space without generic cards, shadows, or component-local state colors
- [x] 6.8 Keep all artwork outside prose, links, markers, comments, composers, errors, state labels, and visible focus planes with empty alt text and `aria-hidden`
- [x] 6.9 Verify scope selectors remain isolated from successful index composition and shared primitive defaults

## 7. Preserve decisions, scrolling, and responsive behavior

- [x] 7.1 Keep browser window scrolling authoritative and reject a nested primary document scroller or synchronized dual-scroll implementation
- [x] 7.2 Keep desktop `DecisionInstrument` constrained to conversation-side chassis so closed and open states cannot cover selected document, navigation, thread actions, or focus rings
- [x] 7.3 Preserve session `keep-exploring` and `move-to-proposal` behavior, change `comment-resolution` behavior, optional unanchored comment creation, and all existing emitted mutation contracts
- [x] 7.4 Preserve scope-level dirty-composer deferral for artifact snapshots and immediate review-state reconciliation because current SSE events do not identify changed artifact paths
- [x] 7.5 Preserve same-artifact reading position across applied live updates and move to selected-document arrival when reviewer deliberately changes artifact
- [x] 7.6 At intermediate width, remove peripheral plate cells and move instrumentation/conversation into flow before reducing document measure or hiding state
- [x] 7.7 At 390×844, order scope state, artifact selection, complete document, artifact conversation, loose comments, and decisions in one logical flow
- [x] 7.8 Keep narrow decision controls safe-area-aware with enough page clearance for final block, thread action, active composer, and close/submission controls
- [x] 7.9 Preserve visible non-hover block comment actions, keyboard traversal, focus reveal, and long identifier/path wrapping at narrow width
- [x] 7.10 Preserve 100–200ms instrument motion and make reduced-motion artifact changes, thread activation, focus navigation, and decision state immediate

## 8. Update executable and written acceptance together

- [x] 8.1 Update `designs/dashboard-review-workbench.md` to replace concatenated artifact spine, same-row-only conversation, route hero, and pre-document plate assumptions with selected-artifact workbench rules
- [x] 8.2 Update `designs/observatory-runtime-budget.md` only if rendered roles of existing shared derivatives change; retain exact files, measured bytes, allowlist, and 6 MiB ceiling
- [x] 8.3 Update visual-system and component tests to assert document-stage field placement, absence of pre-document equal plate gallery, decorative semantics, compact route state, and no fake repository/activity/validation claims
- [x] 8.4 Extend embedded desktop E2E for canonical/default selection, direct nested selection, Back/Forward, invalid selection, cross-artifact switching, and selected-artifact-only content
- [x] 8.5 Extend embedded desktop E2E for bidirectional thread/anchor navigation, exact repeated occurrence, reply/status actions, verdict controls, and decision/document non-overlap
- [x] 8.6 Preserve embedded E2E for exact comment creation, inline-markup refusal, session comments, empty verdict submission, cross-tab updates, dirty-composer deferral, and latest live update winning
- [x] 8.7 Extend embedded narrow E2E for artifact selection order, complete document, conversation return-to-anchor, final-content clearance, visible actions, and zero horizontal overflow
- [x] 8.8 Capture deterministic 1536×1024 design evidence plus project-standard desktop and 390×844 embedded screenshots and inspect them side by side with `dashboard-mockup.png`
- [x] 8.9 Assert zero unexpected console errors, page errors, failed requests, external requests, missing fonts, and missing runtime images across session and change variants

## 9. Run completion gates and boundary review

- [x] 9.1 Run `bun run format` from `web/` and review generated formatting rather than accepting unrelated churn
- [x] 9.2 Run `bun run check` from `web/`, including type, unit, and runtime-image checks
- [x] 9.3 Run `bun run build` from `web/` and confirm exactly four allowlisted observatory rasters remain within the recorded runtime budget
- [x] 9.4 Run `bun run test:e2e` against freshly built Rust-embedded dashboard for desktop and narrow projects
- [x] 9.5 Manually verify keyboard navigation through index link, artifact paths, block markers, conversation actions, verdict history, and decisions with visible focus
- [x] 9.6 Manually emulate reduced motion and verify artifact switching, thread navigation, live updates, and decision controls remain immediate and understandable
- [x] 9.7 Review final diff for duplicated session/change workbenches, global store introduction, nested document scroll, API or sidecar drift, fake state, duplicated raster payload, stale concatenated-layout rules, package-manager drift, or unrelated refactors
