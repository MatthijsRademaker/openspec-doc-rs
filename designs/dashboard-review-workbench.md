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

Every scope view follows one spine:

```text
instrument header / route state
utility rail | document spine | anchored conversation rail
persistent decision instrument
```

Document content owns reading measure. Anchored comments sit beside their block only when
space permits, then immediately follow that block in narrow flow. Unanchored/orphaned
comments remain reachable. Persistent decisions never cover content.

Index uses same language at lower density: instrument masthead, separate ruled session and
change registers, exact identifiers, complete metadata, explicit recent activity, and
distinct loading, empty, and failure instruments. It must not invent scope routes,
repository names, activity events, or review controls unsupported by current data.

## Motif boundary

Orbital maps, crosshairs, scan lines, nodes, and stipple may frame headings, unused margins,
rail thumbnails, and empty instruments. They are `aria-hidden`, non-interactive, and never
sit behind prose, controls, focus rings, or state labels. At narrow width atmosphere shrinks
or disappears before content.

Only named runtime slots may use raster derivatives. Current allowlist:

- `web/public/assets/images/index-orbit.webp` — bounded index masthead atmosphere, derived
  from `designs/visual-language/abstract-star-system.png`.

Original source boards remain in `designs/visual-language/`. Runtime raster payload is
checked after every production build and must stay at or below 6 MiB.

## Geometry, responsiveness, and motion

Use 8px rhythm, with 4px only for tight icon/text alignment. Prefer square or 2–4px corners,
hairline rules, and stepped surfaces. Pills belong only to compact state/control shapes.
Focus is visibly offset from borders.

- **Desktop, 1280px+:** primary content gets useful width; rails and atmosphere consume
  remaining space, not document measure.
- **Narrow, 390px:** one logical flow, no page-level horizontal overflow. Metadata, exact
  identifiers, state text, comments, and controls stay visible and reachable.

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

Current index proves visual system only. Artifact rendering, anchored thread mutations,
verdict controls, SSE reconciliation, and scope routes remain owned by
`migrate-dashboard-review-to-vue`.
