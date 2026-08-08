## 1. Promote runtime artwork to shared observatory slots

- [ ] 1.1 Rename the four committed `index-*` WebP derivatives to `observatory-field.webp` and `observatory-plate-{sun,face,star-system}.webp` without recompressing or changing bytes
- [ ] 1.2 Update `IndexView.vue` to shared asset paths only; preserve successful index markup, crop behavior, semantic order, and responsive composition
- [ ] 1.3 Update `web/scripts/check-runtime-images.mjs` to allow exactly four shared observatory files and reject old index names, duplicate scope aliases, source PNGs, and payload above 6 MiB
- [ ] 1.4 Update `designs/dashboard-review-workbench.md` with both index and scope roles, crop behavior, decorative semantics, and responsive treatment for each shared file
- [ ] 1.5 Update `designs/observatory-runtime-budget.md` with shared names and measured bytes; confirm aggregate remains 249,596 bytes unless a later explicitly approved crop change updates every artifact
- [ ] 1.6 Update visual-system and asset tests for exact shared paths, then build once to prove no missing index request before changing scope composition

## 2. Lock scope semantics before visual changes

- [ ] 2.1 Extend `ScopeView` tests to assert both change and session routes use the same workbench landmarks and provide a Router-owned link back to `/`
- [ ] 2.2 Add header tests for compact route title, exact identifier, standing verdict, submission age, delivery state, and verdict history without suppressing repeated artifact H1 source content
- [ ] 2.3 Add decorative-semantics tests proving scope observation field and plate images use empty alt text, remain `aria-hidden`, and introduce no activity, lifecycle, repository, or validation claims
- [ ] 2.4 Preserve existing tests for successful empty artifacts, loading/failure distinction, comment counts, orphaned comments, addressed claims, reply/resolve/reopen controls, and verdict actions
- [ ] 2.5 Preserve existing live-update tests for clean replacement, immediate review-state refresh, dirty-composer deferral, dismissal/send application, and latest deferred update winning

## 3. Recompose route header and authored atmosphere

- [ ] 3.1 Add a real Vue Router link from scope instrumentation to `/`, with visible instrument copy and keyboard focus matching the observatory system
- [ ] 3.2 Compact `ScopeHeader` title, description, key, verdict, delivery, and history into protected identity/state planes so artifact heading retains primary document gravity
- [ ] 3.3 Replace header `OrbitalFrame` usage with `observatory-field.webp` as non-interactive right-entering face/orbit artwork; stop painted pixels before every text and focus region
- [ ] 3.4 Add a restrained pre-document observation band using the three shared plate derivatives with hard edges, aligned hairlines, no labels implying product state, and no interaction
- [ ] 3.5 Remove any scope-only SVG/image branch made dead by the header replacement while preserving `OrbitalFrame` behavior still used elsewhere
- [ ] 3.6 Keep loading, empty, action-failure, and deferred-artifact notices on opaque readable planes independent from decorative artwork

## 4. Strengthen utility, document, and conversation geometry

- [ ] 4.1 Recompose desktop scope layout as sticky utility rail, readable document spine, and reserved conversation region without introducing a nested document scroller
- [ ] 4.2 Order utility content as index route link, visible open/addressed/resolved counts, then exact artifact anchor paths; retain native fragment navigation and long-path wrapping
- [ ] 4.3 Keep `ArtifactDocument` as sole owner of block rendering, exact source positions, selection comments, repeated occurrence anchoring, and thread grouping
- [ ] 4.4 Tune artifact heading and block measure so proposal/design/tasks/spec prose remains readable while conversation retains useful width at 1280px and above
- [ ] 4.5 Add decorative CSS/SVG chassis to empty conversation space using hairlines, nodes, or crosshairs only; keep it `aria-hidden`, pointer-inert, and free of fake thread labels or counts
- [ ] 4.6 Preserve collapsed markers beside their blocks and render each expanded thread or anchored composer only in that block's conversation cell
- [ ] 4.7 Verify reviewer/agent labels, fuzzy-anchor notice, lost-anchor quote, status glyphs, and thread actions still use existing semantic tokens and visible non-color distinctions

## 5. Move persistent decisions off document spine

- [ ] 5.1 Constrain closed desktop `DecisionInstrument` to conversation-side geometry so it never spans artifact text, path links, comment markers, or block actions
- [ ] 5.2 Reserve conversation-rail bottom inset so expanded anchored threads and actions can scroll fully clear of persistent decision trigger
- [ ] 5.3 Constrain open decision drawer to bounded conversation-side space at desktop while keeping orphaned/unanchored comments, close control, textarea, reply/status actions, and verdict submission reachable
- [ ] 5.4 Preserve existing emit contracts and mutation ownership for unanchored comment creation, keep-exploring, move-to-proposal, comment-resolution, reply, resolve, accept, and reopen
- [ ] 5.5 Test closed and open decision states against document bounding boxes at desktop so visual non-overlap is executable evidence rather than screenshot inference

## 6. Preserve intermediate and 390px review flow

- [ ] 6.1 At intermediate widths, move header state below identity and reduce observation plates before reducing artifact measure or hiding review metadata
- [ ] 6.2 At 390px, place route identity/state, utility counts/paths, artifact blocks, and each expanded conversation in one logical DOM flow without horizontal page overflow
- [ ] 6.3 Hide observation band at 390px and keep only a shallow bounded hero crop when it does not compete with title, key, verdict, delivery, or history
- [ ] 6.4 Recompose narrow decision trigger as compact safe-area-aware bottom dock with explicit page tail so final artifact block, marker, expanded thread, and composer scroll completely above it
- [ ] 6.5 Bound open narrow decision drawer to viewport-safe height with internal scrolling while keeping close control, loose threads, composer, and submission action keyboard-reachable
- [ ] 6.6 Verify realistic long change identifiers, UUID sessions, artifact paths, status labels, quoted anchors, and button copy wrap without clipping or truncation
- [ ] 6.7 Preserve immediate comment actions at narrow width rather than hiding them behind hover behavior

## 7. Extend embedded acceptance coverage

- [ ] 7.1 Keep deterministic fixture route `/changes/implement-observatory-design-system-with-a-realistically-long-identifier` populated with repeated blocks, anchored/open comment, addressed claim, orphaned resolved comment, verdict, and delivery state
- [ ] 7.2 Add embedded desktop assertions for shared artwork requests, compact route header, visible Proposal content in 1440×1000 first viewport, three-region geometry, index link, and decision/document non-overlap
- [ ] 7.3 Expand second repeated-block comment at desktop and assert thread, Agent/Reviewer labels, reply, and resolve controls remain beside the second occurrence
- [ ] 7.4 Add embedded narrow assertions for hidden plate band, complete route state, utility-before-document flow, adjacent expanded thread, safe final-content clearance, and no horizontal overflow
- [ ] 7.5 Preserve exact-anchor creation, inline-markup refusal, cross-tab reply/status reconciliation, empty-verdict submission, session comment creation, and live artifact update E2E coverage
- [ ] 7.6 Preserve browser-health assertions for zero console errors, page errors, failed requests, and external requests on both index and scope routes
- [ ] 7.7 Capture deterministic 1440×1000 and 390×844 embedded scope screenshots and inspect them against `dashboard-mockup.png`, `main-panel-background.png`, and all three abstract plate sources

## 8. Run completion gates and review boundaries

- [ ] 8.1 Run `bun run format`, `bun run check`, and `bun run build` from `web/`
- [ ] 8.2 Confirm runtime-image check reports exactly four shared observatory rasters, no old aliases or source PNGs, and aggregate payload at or below 6 MiB
- [ ] 8.3 Run `bun run test:e2e` against freshly built Rust-embedded dashboard for desktop and narrow projects
- [ ] 8.4 Manually verify keyboard traversal through index link, artifact links, repeated comment marker, thread actions, verdict history, and decision drawer with visible focus
- [ ] 8.5 Emulate reduced motion and confirm scope art, drawers, comment actions, and state updates use no decorative transform or delayed feedback
- [ ] 8.6 Review final diff for mutation/live-update logic changes, duplicated raster files, stale index asset names, component-local colors, fake data, package-manager drift, or unrelated scope/API edits
