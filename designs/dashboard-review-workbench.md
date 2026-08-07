# Dashboard review workbench

Status: implementation reference for `web/` dashboard work. This document owns visual
hierarchy, voice, state meaning, responsive behavior, accessibility, and motion. It does
not freeze exact pixels; real document length and review state must remain visible.

## Product posture

openspec-doc is a review workbench, not a generic project dashboard. The artifact being
reviewed is primary. Controls, metadata, and conversation explain that artifact instead
of competing with it. Dense information is acceptable; noise is not.

Use IBM Plex Sans Variable for interface and document prose, and IBM Plex Mono for
identifiers, paths, timestamps, and code-shaped values. Both are bundled locally under
`web/` and embedded in `web/dist/`. No remote font or icon request is allowed.

## Hierarchy

Every view has one predictable spine:

1. **Workspace header** — product mark, scope identity, theme control, and delivery state.
2. **Artifact header** — human title, exact identifier, review state, modified time, and
   the smallest useful action set.
3. **Document spine** — proposal, design, tasks, or spec content in reading order. Headings
   and block boundaries are more important than cards.
4. **Inline conversation** — comments attach beside the block they discuss. A thread shows
   author, role, timestamp, body, and open/addressed/resolved state at its anchor.
5. **Decision bar** — verdict controls and delivery status remain visible without pinning
   a large decorative footer over content.

The index uses the same vocabulary at lower density: two clearly named collections for
sessions and changes, long identifiers retained, recent activity explicit, and empty,
loading, and failure states distinct.

## Review conversation and voice

Comments belong to document coordinates, not a detached chat stream. On wide screens an
anchored thread sits in a gutter beside its block; on narrow screens it follows that block
in normal document flow. A connector or subtle anchor marker may relate thread to text,
but the comment body must never be the only indication of its location.

Reviewer and agent messages are visually distinct through a combination of role label,
layout, border treatment, and semantic token:

- **Reviewer**: warm `reviewer` accent, label `Reviewer`, left/document-side alignment.
- **Agent**: cool `agent` accent, label `Agent`, right/workbench-side alignment where space
  permits; normal flow on narrow screens.

Never communicate role with color alone. The label and accessible text are mandatory.
Agent output is evidence or proposed work, not an authoritative verdict.

## State language

State text and icon shape accompany every state color. Central tokens live in
`web/src/style.css`; components do not invent raw status colors.

| Meaning | Token | Visible language |
| --- | --- | --- |
| comment needs work | `open` | Open |
| agent responded or work is proposed | `addressed` | Addressed |
| reviewer accepted resolution | `resolved` | Resolved |
| reviewer decision exists | `verdict` | Keep exploring, Move to proposal, or Comment resolution |
| directive/review outcome transport | `delivery` | Pending, delivered, or failed |
| human voice | `reviewer` | Reviewer |
| tool/agent voice | `agent` | Agent |

The light and dark values are paired in `:root` and `.dark`, and exposed as Tailwind
semantic colors. Amber/blue/green/violet/teal/warm/cool provide redundant hue cues; do
not reduce a state to hue alone.

## Responsive composition

The committed reference frames are in `designs/frames/`.

- **Desktop (at least 1100px)**: document column stays readable at roughly 70ch; comment
  gutter and metadata rail may use remaining space. Long identifiers wrap in their own
  region and never push verdict controls off-screen.
- **Narrow (under 768px)**: one column, no horizontal page scroll. Metadata wraps below the
  title. Anchored comments become full-width blocks immediately after their anchor. Actions
  remain reachable in normal flow; no essential control exists only in a hover state.
- **Intermediate widths**: collapse the comment gutter before shrinking document type below
  readable size. Preserve block order and focus order during every layout transition.

Index tables may become stacked rows on narrow screens. Every row keeps its identifier,
status, modified value, and primary link; hiding data to preserve a table silhouette is
not acceptable.

## Accessibility contract

Use landmarks (`header`, `main`, `nav`, `aside` where meaningful), one logical `h1`, and
ordered heading levels for document blocks. Use actual buttons and links, not clickable
containers. Every icon-only control has an accessible name. Status badges include visible
text and are not the sole source of meaning.

Keyboard focus is visible against both themes. Focus order follows reading order. Anchored
comments expose their relationship to the referenced block with a label or description.
Dynamic loading, mutation, and delivery failures use an appropriate live region without
stealing focus. Dialogs trap focus only while open and return it to their trigger.

Target WCAG AA contrast for text and controls. Test at narrow and desktop viewports, with
keyboard navigation, reduced motion, and a screen reader-friendly accessibility tree.

## Motion policy

Motion explains a state transition: a thread opening, a status changing, or a route/view
entering. Use short opacity/position transitions (normally 120–180ms) and avoid bouncing,
parallax, decorative gradients, and perpetual activity indicators. Loading may use one
quiet progress treatment; it must not imply data exists.

Respect `prefers-reduced-motion: reduce` by removing transforms and reducing transitions
to immediate state changes. Never delay access to review content for animation.

## Implementation constraints

- `web/components.json`, CSS, and generated component output must agree on IBM Plex Sans;
  `web/src/style.css` is the central token source.
- Existing shadcn-vue primitives remain the base for controls, tables, badges, cards, and
  future review controls. Extend them with semantic tokens instead of local hex/OKLCH
  values.
- Vue Router owns browser navigation. Register a route only when its view and server/API
  support exist; no placeholder session or change views.
- API calls live in focused modules and preserve malformed/failed responses as failures.
- `web/dist/` is committed and embedded. Any asset, font, favicon, or CSS change requires
  the Bun build and freshness gate.
- Browser review uses the embedded Rust binary, not only a Vite page. The repository
  Playwright lane is the completion gate.

## Current index review boundary

The current index is a foundation screen, not the scope-page rewrite. It already conforms
by preserving exact identifiers, separating loading/empty/failure states, providing a
persistent theme toggle, using Router-owned `/`, and using bundled typography and semantic
status infrastructure. It intentionally defers document rendering, anchored threads,
reviewer/agent conversation, verdict controls, delivery events, and scope routes to
`migrate-dashboard-review-to-vue`. Reference frames show that intended shell without
pretending those interactions exist today.
