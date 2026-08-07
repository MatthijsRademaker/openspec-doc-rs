---
name: shadcn-vue
description: Add or adapt shadcn-vue components in web using the actual components.json, existing primitives, semantic tokens, Bun commands, and generated-file review.
license: MIT
compatibility: Requires this repository and its Bun-managed web workspace.
metadata:
  author: openspec-doc
  version: "1.0"
---

# openspec-doc shadcn-vue

Use this skill before adding or changing a shadcn-vue component under `web/`.

## Project contract

- Read `web/components.json`; it is the registry project configuration. It currently names
  `reka-nova`, TypeScript, IBM Plex Sans, `src/style.css`, `@/*`, and `@/components/ui`.
- Existing primitives live in `web/src/components/ui`. Reuse them before adding a duplicate
  Card, Table, Badge, Button, or interaction primitive.
- Tailwind v4 sources are explicit in `web/src/style.css` with `source(none)` and `@source`
  entries for `index.html` and `src/**/*.{vue,ts}`. Keep those sources explicit; do not add a
  broad repository scan or put `dist/` into the source set.
- Put reusable state colors in central semantic tokens (`open`, `addressed`, `resolved`,
  `verdict`, `delivery`, `reviewer`, `agent`) in `:root`. Dashboard has no alternate theme.
  Do not paste raw status colors into a component.
- Generated files are code. Review imports, accessibility, responsive classes, and unused
  exports after every registry add; do not blindly accept a generated diff.

## Commands

Use Bun, never a second package-manager path:

```text
bunx --bun shadcn-vue@2.8.1 add <component>
bunx --bun shadcn-vue@2.8.1 diff
bun run format
bun run check
```

The pinned project MCP server is rooted at `web/`; use it to inspect registry metadata
instead of guessing component APIs. Do not add a browser MCP or a Pi-local duplicate.

## Product constraints

Consult `designs/dashboard-review-workbench.md` and source boards under
`designs/visual-language/` before styling. Preserve document-first review hierarchy,
visible Reviewer/Agent labels, keyboard/focus behavior, narrow layouts, offline assets, and
restrained motion. Generic stock component or SaaS card-grid output is failure.
