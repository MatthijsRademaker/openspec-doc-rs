---
name: openspec-doc-dashboard
description: Implement dashboard work against openspec-doc's Vue Router, API modules, Rust boundary, committed embedded dist, Bun gates, and review-workbench constraints.
license: MIT
compatibility: Requires this repository, Cargo, and its Bun-managed web workspace.
metadata:
  author: openspec-doc
  version: "1.0"
---

# openspec-doc dashboard workflow

Use this skill for any feature or bug under `web/` or at the Rust dashboard boundary.

## Architecture contract

- Vue Router owns browser navigation. `/` is implemented now. Do not register session or
  change placeholders; add `/sessions/:id` or `/changes/:name` only with its view, API
  support, and tests.
- Keep API handling in focused `web/src/lib` modules. Preserve non-2xx, malformed JSON, and
  network failures as failures; never turn them into fake empty or successful data.
- The Rust server is the runtime boundary. `cargo install --path crates/cli --locked` must
  serve the complete dashboard from committed `web/dist/` with no Node, Bun, checkout, or
  external asset request.
- `web/dist/` is generated and gitignored. Rust embeds it at compile time. A frontend
  source, CSS, font, favicon, or route change requires a fresh Bun build before Cargo.
- Pinia, generated API clients, motion libraries, and speculative global stores are not
  foundations. Add them only when the change names their state owner and behavior.

## Required reading and gates

Read `designs/dashboard-review-workbench.md`, both source boards under
`designs/visual-language/`, `web/components.json`, and relevant OpenSpec artifacts before
visual or interaction work. Follow frontend-design and shadcn-vue skills. Reject generic
SaaS/card-grid output even when stock primitives make it easy.

From `web/`:

```text
bun install --frozen-lockfile
bun run check
bun run build
bun run test:e2e
```

For local API development, start the Rust server on the documented fixed target and then
Vite:

```text
openspec-doc serve --port 8791 --no-open
bun run dev
```

Vite proxies same-origin `/api` to `http://127.0.0.1:8791`, or the explicit
`OPENSPEC_DOC_API_PROXY_TARGET`. For shipped behavior, use the embedded-browser lane, not
only Vite.

## Scope discipline

The dashboard harness establishes routing, tests, design language, and tooling. It does
not silently implement the scope-page migration: rendered markdown blocks, anchored thread
mutations, verdict controls, SSE reconciliation, and scope routes belong to
`migrate-dashboard-review-to-vue`.
