# Observatory review workbench

Status: implementation reference for `web/` dashboard work. Source boards live in
`designs/visual-language/`; they guide composition but never enter runtime builds.

## Required visual source

Read these before changing dashboard layout, typography, state, or motion:

1. `designs/visual-language/design-system.png` — palette, typography, rhythm, states,
   geometry, and motion.
2. `designs/visual-language/dashboard-mockup.png` — document-first composition and balance
   between review content and celestial framing.
3. Supporting PNGs in `designs/visual-language/` — motif vocabulary only. They are not a
   runtime asset library.

Product posture: observatory for agent-guided spec review. Reviewer observes, agent acts as
instrument, spec remains primary object. Reject generic SaaS chrome, rounded card grids,
drop shadows, bright status fills, stock dashboard sidebars, and detached chat layouts.

## Identity and typography

Dashboard has one dark identity from first paint. Canvas is near-black, content is bone,
and depth comes from stepped surfaces plus hairlines. No theme control or persisted browser
preference may alter it.

Use three bundled roles:

- **Display — Cormorant Garamond:** product wordmark, route and artifact titles,
  requirement identifiers, major document headings.
- **Prose — IBM Plex Sans Variable:** sustained artifact prose and comment bodies.
- **Instrument — IBM Plex Mono:** paths, identifiers, timestamps, state labels, commands,
  compact controls, and uppercase microcopy.

No remote font, icon, or image request. Long identifiers stay exact and wrap.

## Instrument language

Microcopy is terse, operational, and calm: `SCOPE REGISTER`, `OBSERVING INDEX`,
`3 OPEN COMMENTS`, `MOVE TO PROPOSAL`. Avoid marketing language, emoji, decorative status,
or invented repository/activity facts.

Every state combines central token, visible text, and distinct glyph or line treatment:

| Meaning | Visible treatment |
| --- | --- |
| Open | `◇ Open` plus interrupted line |
| Addressed | `↗ Addressed` plus directional line |
| Resolved | `✓ Resolved` plus closed line |
| Verdict | `◆ <verdict>` plus decision rule |
| Delivery | `→ <delivery state>` plus transport rule |
| Reviewer | `● Reviewer` plus document-side rule |
| Agent | `□ Agent` plus instrument-side rule |

Color is secondary. Components use semantic classes backed by `web/src/style.css`; no
component-local raw state colors.

## Composition

Every scope route uses one selected-artifact workbench:

```text
compact route identity + standing review state
scope/artifact rail | complete selected document | artifact conversation
                    | browser window scrolls     | loose comments + decisions
```

Change routes expose every server-supplied artifact through exact-path navigation. Exact
`Artifact.path` owns `?artifact=`; first load canonicalizes missing selection with Router
replace, deliberate selection pushes history, and invalid or removed coordinates stay
visible as unavailable state. Session routes select sole scratch artifact without fake tabs,
folders, or destinations.

Selected artifact renders completely in source order. Browser window remains sole primary
scroll owner; document stage never gains fixed-height primary scroller. Resolved exact and
fuzzy threads appear only in selected-artifact conversation. Marker and thread activation
lock same source block visibly and navigate in both directions. Missing, orphaned, and
unanchored threads remain in decision instrument. At narrow width DOM order is route state,
artifact identity/navigation, complete document, artifact conversation, loose comments,
then decisions—without duplicate interactive thread markup.

Index uses same language at lower density: instrument masthead, separate ruled session and
change registers, exact identifiers, complete metadata, explicit recent activity, and
distinct loading, empty, and failure instruments. It must not invent scope routes,
repository names, activity events, or review controls unsupported by current data.

## Motif boundary

Orbital maps, crosshairs, scan lines, nodes, and stipple may frame headings, unused margins,
rail thumbnails, and empty instruments. They are `aria-hidden`, non-interactive, and never
sit behind prose, controls, focus rings, or state labels. At narrow width atmosphere shrinks
or disappears before content.

Only named runtime slots may use raster derivatives. Shared observatory allowlist:

| Runtime slot | Source and crop | Output | Index and scope roles |
| --- | --- | --- | --- |
| `web/public/assets/images/observatory-field.webp` | `main-panel-background.png`; crop `x=128, y=75, w=1408, h=845` excludes far-left pseudo-control rail while retaining black safe space and right-side face/orbit field | 1200×720 WebP, quality 84, 95,710 bytes | Decorative asymmetric field. Index: right-entering masthead art beside protected text, then bounded crop at 390px. Scope: bounded right-entering selected-document arrival plane beside opaque artifact identity; hidden at 390px before document measure or controls yield. |
| `web/public/assets/images/observatory-plate-sun.webp` | `abstract-sun.png`; crop `x=196, y=120, w=1280, h=640` centers dark solar aperture and radiating line field | 640×320 WebP, quality 82, 65,168 bytes | Decorative first hard-edged plate in index lower strip. Scope has no pre-document gallery; any future scope use is limited to subordinate peripheral chassis cells. |
| `web/public/assets/images/observatory-plate-face.webp` | `abstract-face.png`; crop `x=196, y=150, w=1280, h=640` isolates eye and halftone texture rather than repeating masthead face composition | 640×320 WebP, quality 82, 73,302 bytes | Decorative second hard-edged plate in index lower strip. Scope has no pre-document gallery; any future scope use is limited to subordinate peripheral chassis cells. |
| `web/public/assets/images/observatory-plate-star-system.webp` | `abstract-star-system.png`; crop `x=168, y=230, w=1280, h=640` places orbital intersections and nodes off center | 640×320 WebP, quality 82, 15,416 bytes | Decorative third hard-edged plate in index lower strip. Scope has no pre-document gallery; any future scope use is limited to subordinate peripheral chassis cells. |

These four shared slots replace `index-orbit.webp` and every index- or scope-prefixed alias;
retaining aliases would create competing art systems and duplicate payload. Every image and
plate group remains non-interactive, uses empty alt text, is `aria-hidden`, and stays outside
text and focus planes. Original source boards remain in `designs/visual-language/`. Runtime
raster payload is checked after every production build and must stay at or below 6 MiB.

## Geometry, responsiveness, and motion

Use 8px rhythm, with 4px only for tight icon/text alignment. Prefer square or 2–4px corners,
hairline rules, and stepped surfaces. Pills belong only to compact state/control shapes.
Focus is visibly offset from borders.

- **Desktop, 1280px+:** full-bleed hairline chassis reserves roughly 14–18rem for scope and
  artifact instrumentation, flexible readable document measure, and 20–26rem for selected
  artifact conversation plus decisions. Route chrome stays compact enough for artifact
  identity and first meaningful source block to enter 1536×1024 initial viewport.
- **Intermediate:** instrumentation enters flow first, then conversation follows document;
  artwork and peripheral cells yield before paths, prose, state, comments, or controls.
- **Narrow, 390px:** one logical flow, no page-level horizontal overflow. Exact paths wrap,
  complete document precedes conversation, comment actions stay visible without hover, and
  final content clears decision controls and safe-area inset.

Motion explains entry or state change with opacity or slight lateral movement in 100–200ms
using `cubic-bezier(0.2, 0, 0.2, 1)`. No perpetual orbit, parallax, animated ornament, or
spectacle loader. `prefers-reduced-motion: reduce` removes transforms and makes transitions
immediate.

## Interaction and accessibility

Appearance may be strange; interaction stays conventional. Use landmarks, logical headings,
real buttons and links, visible focus, accessible names, reading-order focus, and live
regions for loading/failure state. Reviewer and Agent always have visible role labels.
Target WCAG AA. Verify keyboard traversal, reduced motion, network-offline assets, desktop,
and 390px against embedded Rust output.

## Implementation boundary

Reuse shadcn-vue behavior from `web/src/components/ui`; adapt variants through central
observatory tokens instead of cloning interaction logic. Vue Router owns implemented SPA
routes. API modules preserve failures. `web/dist/` is generated, gitignored, and embedded by
Rust, so build it before Cargo.

`ScopeView` owns route fetches, exact query selection, active thread, mutations, SSE,
generation guards, and dirty-composer artifact deferral. `ArtifactNavigator` emits exact
paths only. `ArtifactDocument` owns rendered blocks and source-accurate anchor construction.
`ArtifactConversation` owns selected-artifact thread presentation without fetching.
`DecisionInstrument` owns loose/lost comments and verdict actions. No global store, nested
primary document scroller, duplicate session/change workbench, or compatibility layout.
