---
name: openspec-visual-language
description: Ingest the observatory visual language from web/public/assets/images before any web/ frontend work; push past conventional dashboard UI toward the abstract, weird, dev-fluent design system in the mockups.
license: MIT
compatibility: Requires this repository and its Bun-managed web workspace.
metadata:
  author: openspec-doc
  version: "1.0"
---

# openspec-doc visual language

Load this skill alongside `frontend-design`, `shadcn-vue`, and `openspec-doc-dashboard` for
any task under `web/`. That trio defines the mechanics (tokens, primitives, gates); this
skill defines the *soul*. Mechanics without soul produce a generic SaaS dashboard, which is
the failure mode this skill exists to prevent.

## Required viewing — before designing anything

Read these images. Actually read them, every frontend task, before choosing layout, type,
or motif:

1. `web/public/assets/images/design-system.png` — the source of truth. Color system,
   typography pairing, spacing scale, states, iconography, motion principles, example
   fragment.
2. `web/public/assets/images/dashboard-mockup.png` — the target experience. Study how the
   system feels: density, hierarchy, atmosphere, the relationship between document content
   and the celestial artwork.

Then skim the supporting assets for the motif vocabulary: `abstract-star-system.png`,
`abstract-sun.png`, `abstract-face.png`, `main-panel-background.png`, `logo.png`, and the
activity thumbnails (`agent-updated.png`, `comment-resolved.png`, `comment-updated.png`,
`task-updated.png`).

## The essence

The product is an **observatory for agent-guided spec review**, not an admin panel. The
reviewer is an astronomer; the agent is an instrument; the spec is the sky. Every design
decision should pass one test: does this feel like it belongs in the mockup's universe?

Pillars visible in the assets:

- **Near-black canvas, bone ink.** `#0B0C0E` canvas, parchment/bone text (`#E7E2D6`,
  `#9A968F`), hairline borders (`#2A2D31`). Depth comes from layering and hairlines, not
  from shadows or saturation.
- **Serif display, mono meta.** Large serif (REQ-07, "Persistent memory across sessions")
  for document gravity; uppercase microtracked monospace (`spec.md § 2.4 · updated 2h ago`)
  for everything operational. Never a generic geometric sans as the voice of the product.
- **Celestial/cartographic motifs.** Concentric orbital maps, crosshairs, star charts,
  scan lines, stippled halftone faces. These are atmosphere and wayfinding — rails,
  backgrounds, empty states, activity icons — not decoration pasted over content.
- **Instrument microcopy.** The UI speaks like an instrument: "the system is observing",
  "OBSERVE / PLAN / EXECUTE / VERIFY", "SEND 4 COMMENTS TO AGENT ⌘↵". Terse, uppercase,
  declarative. No marketing voice, no emoji, no exclamation marks.
- **Dense but calm.** High information density held together by hairline rules, generous
  margins around document content, and an 8px grid. Weird in atmosphere, never in
  legibility.

## Push boundaries — with a ratchet

Do not reach for conventional dashboard patterns (card grids with drop shadows, rounded
blue buttons, left-nav-with-icons SaaS chrome, generic empty-state illustrations). Ask
instead: how would this screen work as a star chart, a rail of instruments, a crosshair
lock? Unusual layout, orbital progress, glyph-and-hairline states, halftone imagery are
all in bounds.

The ratchet that keeps "weird" fluent for a developer:

- **Interactions stay conventional even when appearance doesn't.** Keyboard-first, command
  palette, visible focus, readable body text, predictable navigation. Weirdness lives in
  atmosphere, motifs, and microcopy — never in relearning how to click.
- **Content stays primary.** Artwork frames the document; it never competes with it. The
  mockup's stippled face sits beside the requirement text, not under it.
- **Accessibility and gates still apply.** Everything in the `frontend-design` skill —
  semantic tokens from `web/src/style.css`, bundled IBM Plex fonts, offline assets,
  reduced-motion, `bun run test:e2e` — is unchanged. Express the visual language *through*
  the semantic tokens; extend the token set centrally rather than hardcoding locally.
- **Motion is instrument-grade.** 100–200ms, `cubic-bezier(0.2, 0, 0.2, 1)`, fades and
  slight lateral shifts. No parallax theatrics, no decorative loaders.

## Using the image assets

Assets live under `web/public/assets/images/` and are large PNGs (1.7–2.8 MB). Use them
deliberately: CSS backgrounds, decorative layers, activity thumbnails — never as
render-blocking critical content, and only where the mockup's composition calls for that
motif. If a screen needs a motif that doesn't exist as an asset yet, recreate it with CSS/
SVG hairlines and stipple-style treatment in the same spirit rather than importing a new
stock style.
