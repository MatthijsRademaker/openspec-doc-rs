## Context

`add-vue-dashboard-foundation` delivered a working Vue index with stock shadcn neutral tokens, rounded cards, a table, IBM Plex Sans headings/body, IBM Plex Mono identifiers, and selectable light/dark themes. `add-dashboard-development-harness` then froze that generic direction in `designs/dashboard-review-workbench.md`, four SVG frames, frontend skills, and browser assertions.

The later source images define a different product: an observatory for agent-guided review. `design-system.png` supplies near-black/bone colors, serif/mono hierarchy, an 8px grid, hairline depth, restrained states, cartographic motifs, and 100–200ms motion. `dashboard-mockup.png` shows those decisions composing a document-first review surface. Supporting PNGs provide motif vocabulary, not permission to paste every image into every screen.

Current `web/public/assets/images/` is 24 MB. Vite copies `public/` wholesale, so a normal build embeds source references and every supporting image into the Rust binary whether the UI uses them or not. Reference ingestion and runtime delivery therefore need separate paths.

## Goals / Non-Goals

**Goals:**

- Make observatory identity executable through central tokens, bundled typography, shared primitives, and one proven real screen.
- Make the current index unmistakably part of the same product as the scope mockup without inventing activity data or review controls its API does not provide.
- Give `migrate-dashboard-review-to-vue` stable visual primitives and composition rules before it builds interaction-heavy scope views.
- Keep review state, focus, narrow behavior, and reduced motion accessible.
- Keep design references available to agents without shipping them in the binary.

**Non-Goals:**

- Not building session/change scope views, thread actions, verdict controls, or SSE behavior; migration and live-update changes own them.
- Not reproducing the mockup pixel for pixel or adding fake navigation, activity, repository metadata, or decorative controls.
- Not creating a broad component library. A primitive needs the current index plus a named migration owner, or it stays local.
- Not redesigning Rust APIs or sidecar formats.
- Not reconciling the separate committed-versus-generated `dist/` policy mismatch.

## Decisions

### One dark product theme replaces theme selection

The visual source has one coherent palette: near-black canvas, subtly stepped surfaces, bone text, and neutral instrument states. Keeping the current light theme would require inventing a second visual system with no source and would preserve a theme toggle whose main purpose was proving shadcn theming before art direction existed.

Remove the toggle and dark-class persistence. Put the observatory values in the root theme so first paint, tests, and embedded output all have one identity without script-dependent theme initialization.

Alternative: retain dark and build a parchment-light counterpart. Rejected because it doubles token, contrast, screenshot, and component-state work while weakening source fidelity.

### Three typography roles, not one font everywhere

Bundle Cormorant Garamond as the display serif. Use it for product wordmark text, route/document titles, requirement identifiers, and major document headings. Keep IBM Plex Mono for uppercase instrument labels, paths, identifiers, timestamps, commands, state metadata, and compact controls. Keep IBM Plex Sans only for sustained prose and comment bodies where mono or high-contrast serif would reduce reading comfort.

The CSS theme exposes explicit `display`, `sans`, and `mono` roles. Components choose by semantic role; changing `--font-heading` alone is insufficient because operational headings and document headings intentionally differ.

Alternative: system serif. Rejected because platform substitution makes screenshots and hierarchy nondeterministic. Alternative: Cormorant for all prose. Rejected because long technical documents and comments need sturdier text at small sizes.

### Exact source palette becomes central semantic tokens

Base tokens begin from source values: canvas `#0B0C0E`, surface `#111317`, raised `#181A1E`, border `#2A2D31`, primary bone `#E7E2D6`, muted ink `#9A968F`, and accent ink `#C7C3B7`. Focus, destructive, reviewer/agent, open/addressed/resolved, verdict, and delivery tokens extend that set centrally.

State tokens stay restrained. Meaning comes from visible labels and glyph/line treatments first, with hue as a secondary cue. Existing saturated badge fills are removed. Hairlines and surface steps create depth; shadows and stacked rounded cards do not.

Alternative: component-local sampling from the PNG. Rejected because near-identical raw colors drift immediately and semantic state becomes impossible to audit.

### 8px rhythm and instrument-grade geometry

Use an 8px base rhythm with half-step 4px only for compact icon/text alignment. Major layout gaps, rail widths, control heights, and document spacing use named spacing variables. Radius is restrained: square or 2–4px for instrument panels, pills only for state/control shapes that are pills in the source. Focus remains visibly offset from hairline borders.

Motion uses opacity and slight lateral movement over 100–200ms with `cubic-bezier(0.2, 0, 0.2, 1)`. `prefers-reduced-motion` removes transforms and makes transitions immediate. No perpetual scan, orbit, parallax, or loading animation is introduced.

### Motifs are framing and wayfinding, never content backgrounds

Create small CSS/SVG orbital, crosshair, node, and rail treatments where geometry is sufficient. Raster artwork may occupy unused margins, rail thumbnails, empty states, or a bounded panel beside a document heading. It may not sit beneath prose, controls, focus indicators, or state labels.

Opacity, crop, and responsive disappearance are controlled centrally. Narrow layout drops atmosphere before dropping content or controls.

Alternative: use `main-panel-background.png` behind every route. Rejected because it turns a composition source into wallpaper and makes document contrast dependent on the image crop.

### References and runtime assets use separate trees

Move original source PNGs to `designs/visual-language/`. Frontend skills read them there. `web/public/assets/images/` contains only runtime derivatives named by code or CSS. Derivatives use WebP where transparency and browser support permit, retain dimensions appropriate to their rendered slot, and have descriptive filenames.

Add an explicit runtime-image allowlist/check. Built runtime raster payload must remain at or below 6 MiB, excluding fonts and JavaScript/CSS, and neither `design-system.png` nor `dashboard-mockup.png` may appear in `dist/`. Record before/after `dist/` and binary sizes.

Alternative: leave originals under `public/` and trust pages not to request them. Rejected because Vite still copies and Rust still embeds them.

### Index proves system without pretending scope features exist

Replace rounded card/table composition with a calm instrument shell and ruled scope registers. Keep existing API and every field: title, exact identifier, modified time, open-comment count, verdict, and most-recently-active marker. Loading, empty, and failure states remain distinct.

Desktop uses product/instrument header, restrained celestial framing, and separate session/change registers. Narrow view becomes one column; identifiers wrap; metadata stays visible; atmosphere yields first. No activity feed, fake repository name, scope tabs, or thread rail appears until real data and routes exist.

### Migration consumes, not forks, visual language

Update `migrate-dashboard-review-to-vue` in the same proposal commit. Its scope composition is:

```text
instrument header / route state
utility rail | document spine | anchored conversation rail
persistent decision instrument
```

At narrow width rails enter document flow: metadata first, each comment immediately after its anchor, persistent controls reachable without overlaying content. Celestial artwork frames major document moments and empty margin; it never owns review behavior. Existing shadcn primitives remain behavioral/accessibility bases but receive observatory variants through central tokens rather than local restyling.

## Risks / Trade-offs

- **[Risk] High-contrast display serif harms technical readability at small sizes.** → Restrict Cormorant to display roles; test long identifiers and markdown headings at desktop and 390px.
- **[Risk] Dark-only removes a shipped preference.** → Treat removal as explicit breaking behavior, delete persistence code/tests, and verify WCAG AA instead of retaining dead compatibility.
- **[Risk] Motifs become decorative noise or generic “space” wallpaper.** → Require every motif to frame hierarchy or wayfinding, disappear before content at narrow widths, and pass screenshot review against both source images.
- **[Risk] Raster derivatives still bloat embedded binaries.** → Separate source/runtime trees, enforce allowlist and 6 MiB budget, and record binary impact.
- **[Risk] Shared primitives anticipate scope pages incorrectly.** → Add only primitives with one index owner and one concrete migration owner; leave one-use composition local.
- **[Risk] Old reference frames keep steering agents toward obsolete UI.** → Replace or delete them in same change and update mirrored skills plus drift check.

## Migration Plan

1. Move source images to design-only location and update mirrored skill paths before removing anything from `public/`.
2. Bundle display serif; establish tokens, typography roles, spacing, focus, state glyphs, and reduced-motion base.
3. Add bounded shared motifs/primitives and restyle index without changing API behavior.
4. Replace obsolete design reference/frames and update migration artifacts as one unit.
5. Run format, check, build, component tests, embedded Playwright at desktop and 390px, keyboard/reduced-motion checks, runtime-asset allowlist, and size comparison.
6. Land before scope migration resumes.

Rollback reverts visual source, tokens, components, and runtime derivatives together. No persisted data migration exists.

## Open Questions

None. Dark-only direction, bundled matching serif, and optimized runtime derivatives are approved.
