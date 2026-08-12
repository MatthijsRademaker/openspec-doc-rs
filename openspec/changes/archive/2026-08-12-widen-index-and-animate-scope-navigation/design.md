## Context

Two pages of one dashboard disagree about how much screen they are entitled to, and the navigation between them animates into a placeholder. The exploration behind this change is at `.openspec-doc/scratch/`; the findings that drive the design are:

- `.observatory-shell` is `width: min(100%, 90rem); margin: 0 auto` with padding up to `3rem`. `.scope-workbench` is `width: 100%; margin: 0` with padding up to `1rem`. The same two decisions, written twice, already drifted.
- `.index-observation` mixes proportional and fixed geometry: the panel is `min(52%, 42rem)`, the divider is pinned at `left: 52%`, the artwork is clipped at `46%`. These agree only below roughly an 80rem container.
- `view-transition-name: scope-coordinate` appears at `style.css:716` (index entry link, the origin) and `style.css:1326` (`.scope-route-acquisition__key`, the loading placeholder). `ScopeHeader.vue` declares no transition name at all.
- `.scope-entry::before` is a `--motion-rule` bar animating `scaleX(0) → scaleX(1)`, which is the treatment `add-observatory-motion-language` removed from the scope route.

The reviewer settled the two questions the exploration could not: the hero panel goes full width with the head bleeding into the background behind the change panelling, and the spec's rule that motion may never delay content is wrong and comes out.

## Goals / Non-Goals

**Goals.** One shared shell width. A hero that keeps its proportions at any width. An observation field that continues behind the panelling. An index↔scope gesture that terminates at the real scope identity, in both directions and on Back.

**Non-Goals.** No change to the index's layout structure — the registers, their proportions, their separation, the plate strip, the reading order, and the narrow-width flow are all as they are. No new artwork. No live channel on the index. No change to artifact-selection navigation within a scope, which already has its paired-coordinate gesture and keeps it.

## Decisions

### The shell width lives in one place

`--shell-width: 100%` and `--shell-padding: clamp(0.25rem, 1.5vw, 1rem)` in `:root`; `.observatory-shell` and `.scope-workbench` both consume them, and the index loses `margin: 0 auto`.

Editing the index rule directly would produce the same pixels. The tokens exist because the duplication is what caused the defect, and the repo's rules call for removing a duplicated constant rather than re-synchronising two copies of it. The scope route's values win because it is the page being matched to.

### The hero becomes proportional rather than capped

Delete the `42rem` cap so `.index-observation__content` is `width: 52%`, matching the divider already pinned at 52% and the artwork clipped at 46%. Every part of the hero is then a percentage of the container and the composition is the same picture at 1280px and at 2560px.

The rejected alternative was to keep the hero bounded inside an otherwise full-width page — a bounded masthead over an unbounded body. The reviewer chose the full-width panel.

The cost is real and worth stating: at 2560px the text panel is roughly 1330px holding a kicker, a title, and one line of description. That is sparse. It is sparse *in proportion*, which is what "the same room, larger" means, and the alternative reintroduces the mixed-units problem that opens the bare band.

### The observation field bleeds behind the panelling

Today `.index-observation__art` lives inside the hero, which is `overflow: hidden` with a bottom border — a closed band. The artwork will instead be a page-level layer beneath the hero *and* the workbench:

```
.observatory-index (position: relative, isolation: isolate)
├── .index-observation__field   position:absolute, z-index:0, aria-hidden
│      the shared observatory-field derivative, clipped at 46% as today,
│      attenuated downward by a declared mask so it reaches the
│      panelling already faded
├── .index-observation          z-index:1  (content panel, divider)
├── .index-workbench            z-index:1  (opaque registers over the field)
└── .index-plates               z-index:1  (own artwork; the field ends above it)
```

Three things make this safe rather than wallpaper:

**The registers are already opaque.** `.ruled-register` is `background: var(--surface)`. No identifier, metadata value, state label, or link is ever rendered over artwork; they sit on a panel that happens to have artwork behind it. The existing prohibition on artwork sitting beneath text is preserved in substance, and the spec is amended to say so in terms rather than leaving the change in contradiction with it.

**The fade is declared, not judged.** The visual-system requirement already permits artwork to extend behind content "only through a declared fade", and requires full strength only in free space. The bleed inherits that: the field reaches the top of the workbench already attenuated and continues decaying, so its strongest region is the hero's own free space, exactly as now.

**The gaps become load-bearing.** `.index-workbench` separates its registers with `gap: var(--space-2)`, and those gaps are where the field is seen. This is why the detached-panel grammar stays. The exploration had proposed replacing it with the scope route's hairline-over-`--border-ink` chassis; that would weld the registers into one surface and leave the bleed nowhere to show. The reviewer's instruction to keep the structure and the composition they asked for point the same way, which is a good sign about both.

The field ends above `.index-plates`, which carry their own hard-cropped artwork and would otherwise collide with it.

### Navigation holds until the destination exists

The gesture cannot terminate at the real scope header while the header does not exist when the browser captures the destination snapshot. Two ways out were considered.

*Render the identity early.* The scope key is in the route and the title is in the index payload, so the loading state could render the real `ScopeHeader` identity immediately and carry the coordinate name. No hold, no spec change. It was rejected because it lands the gesture on a half-built header — identity present, standing verdict and delivery state still empty — and because on a cold load or a copied URL the title is not known at all, so the same navigation resolves to a different destination depending on how the reviewer arrived.

*Hold the navigation until the scope resolves.* Prefetch the scope, then run the route change inside the view transition, so the coordinate transfers onto a complete header. This is what the change does. The reviewer's removal of the no-delay rule is what makes it available, and on a local dashboard reading local files the wait is a disk read.

The hold is bounded, because "delay is allowed" is not a design:

- The origin page stays fully rendered and interactive for the whole hold. Nothing is withheld; the reviewer is on a working index.
- The hold ends at a declared ceiling. Past it the route commits immediately without the gesture, and the existing loading state does its job.
- A failed prefetch commits the route too, so the scope's own failure state reports the error. A load that fails must never present as a click that did nothing.

The prefetched detail is handed to `ScopeView` rather than refetched, so the hold buys the gesture and not a second round trip.

### Route transitions are owned by the Router

Wrapping the two link sites individually leaves Back unhooked, which is half the reviewer's request. Index↔scope route changes are therefore wrapped once at the Router, which covers the entry link, the return link, and popstate in both directions uniformly.

The wrap is keyed on the route *name* changing. Artifact selection changes only the `artifact` query within one route and keeps its existing bespoke `withViewTransition` call and its paired `artifact-coordinate` / `artifact-lock` names, untouched.

### The root suppression is scoped to what it was written for

`::view-transition-old(root) { opacity: 0; animation: none }` exists so that two complete documents never ghost over each other during artifact replacement — the reason is documented above the rule and it is a good one. But it is declared globally, so a route change between two entirely different pages inherits a suppression that was not reasoning about it, and that is the largest single cause of the hard cut.

A `route` marker is set on the document element for the duration of a route transition and the suppression becomes conditional on its absence. Artifact replacement behaves exactly as today. Route changes get a page-level treatment, which per the reference-gesture requirement must be drawn from what the surface is and must advance in discrete passes — so a halftone pass consistent with the plates, not a continuous crossfade.

### The index acquisition becomes the reference gesture

`.scope-entry::before` — a lit bar growing across the entry — is replaced by the change identity resolving through the plate halftone, the same treatment the scope route uses for artifact identity, reusing `--motion-duration-resolve` and the existing `steps()` machinery rather than adding a parallel vocabulary.

This is a spec-conformance fix as much as a visual one. The reference-gesture requirement says a treatment that merely travels across a surface is to be rejected in favour of one that resolves it, and records that acquisition "previously reported itself with a lit bar travelling across the artwork […] which broke no prohibition and still had to go". The index kept the bar.

## Risks / Trade-offs

**The hold makes a click feel slow if the ceiling is set badly.** The ceiling is the whole mitigation, and it should be set from the measured local fetch rather than from taste. If scope loads turn out not to be reliably fast, the correct response is to abandon the hold and render the identity early, not to raise the ceiling.

**A native view transition suspends hit testing while it runs.** The existing artifact transfer already spends ~440ms this way and the stylesheet calls that out as a latency budget. Route transitions add another window, and the hold sits in front of it. Total time from click to interactive destination is the sum, and the acquisition bound of 600ms governs the transition; the hold is additional and bounded separately.

**Bleed behind panelling is one step from wallpaper.** The declared fade and the opaque registers are what keep it on the right side of that line, and both are testable — the fade is a stated distance, and no text is over artwork because the panels are opaque. If the field reads as busy behind the registers in practice, the fade is the lever.

**Removing the no-delay rule removes a real guard.** The replacement is narrower than the thing it replaces: a hold is permitted only where it lets a gesture reach a real destination, only with the origin live, only to a declared ceiling. Delaying content for decoration's sake is still out.

## Migration Plan

Single change, no data or API surface. The index and scope routes render from unchanged payloads throughout.

## Open Questions

None blocking. The ceiling value for the navigation hold is an implementation measurement rather than a design decision, and is recorded in tasks as such.
