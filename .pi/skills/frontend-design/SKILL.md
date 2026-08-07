---
name: frontend-design
description: Design or restyle openspec-doc dashboard UI using the repository review-workbench reference, accessible structure, offline assets, semantic tokens, and restrained motion.
license: MIT
compatibility: Requires this repository and its Bun-managed web workspace.
metadata:
  author: openspec-doc
  version: "1.0"
---

# openspec-doc frontend design

Use this skill for any visual change under `web/`.

## Required reading

Read `designs/dashboard-review-workbench.md` before choosing layout, type, color, status
language, or motion. Preserve long identifiers, realistic review states, and the
document-first hierarchy. Load the `openspec-visual-language` skill and ingest
`designs/visual-language/design-system.png` and `dashboard-mockup.png` for the product's
visual identity; this skill covers mechanics, that one covers essence. Reject generic SaaS
chrome, rounded card grids, stock dashboard sidebars, and detached chat layouts.

## Non-negotiable constraints

- Keep artifact/document content primary. Comments attach to their block; do not turn the
  review product into a detached chat feed.
- Distinguish Reviewer and Agent with visible labels, layout, and semantic tokens; color
  alone is insufficient.
- Use sole dark observatory identity and central semantic tokens from `web/src/style.css`
  for open, addressed, resolved, verdict, delivery, reviewer, and agent states. Do not add
  component-local raw hex/OKLCH or theme-selection behavior.
- Use bundled Cormorant Garamond for display, IBM Plex Sans Variable for sustained prose,
  and IBM Plex Mono for operational text. No Google Fonts, CDN image, remote icon, or
  network-only asset.
- Use existing shadcn-vue primitives under `web/src/components/ui` before creating a new
  primitive. Follow the shadcn-vue skill for generated components.
- Check desktop and 390px narrow behavior. No horizontal clipping, hover-only action, or
  essential content hidden to preserve a card silhouette.
- Provide landmarks, logical headings, accessible names, visible focus, text plus icon/state
  meaning, and `prefers-reduced-motion` behavior.
- Motion explains state changes and stays short; no decorative loading animation.

## Verification

From `web/` run:

```text
bun run format
bun run check
bun run build
bun run test:e2e
```

`bun run test:e2e` is completion evidence because it starts the Rust binary and checks
embedded assets. Do not claim a visual change is done from a Vite screenshot alone.
