## Why

`add-vue-dashboard-foundation` proved that a Vue application can be built, committed, embedded, and served without putting a JavaScript toolchain on the machine that runs `cargo install`. It did not equip the agents building the rest of that application: this repository currently has no frontend linting, tests, browser gate, design source of truth, frontend skills, or MCP server, and its documented Vite development loop cannot reach the Rust API.

The scope-page migration is the wrong place to discover those omissions. It is fifty-five tasks of interaction-heavy UI; without executable browser feedback and repository-specific guidance, "use Vue and shadcn-vue" is an instruction to produce a generic component demo, not a review tool.

## What Changes

- **BREAKING for frontend contributors:** migrate `web/` from npm and an exact Node pin to Bun, pinned exactly, with `bun.lock` as the only frontend lockfile. `cargo install` remains Node- and Bun-free because built assets stay committed and embedded.
- Keep the clean-build `web/dist/` freshness guarantee, but rebuild and verify it with the pinned Bun toolchain.
- Add frontend format, lint, type, unit/component, and browser checks, wired into package scripts and CI.
- Add a working Vite development path to the Rust API through a configurable proxy and a documented fixed development port.
- Add project-local frontend skills for Pi and Claude: visual design, shadcn-vue use, browser verification, and this dashboard's architecture and constraints.
- Add a shared, pinned shadcn-vue MCP server rooted at `web/`, and verify that both supported harnesses can connect and search registry metadata; verify Bun/Vue/Vite/Tailwind and alias configuration directly from the repository files the MCP server consumes.
- Add a dashboard design source of truth for a distinct review-workbench interface: document-first composition, comment placement, reviewer/agent voice, verdict and delivery state, typography, semantic status color, responsive behavior, accessibility, and restrained motion.
- Add Vue Router as the routing foundation for `/`, `/sessions/:id`, and `/changes/:name`. Introduce Pinia only with a defined owner for shared scope/SSE state; do not add Pinia Colada, OpenAPI generation, or a motion library speculatively.
- Make `migrate-dashboard-review-to-vue` depend on this change, and update its change artifacts and the roadmap wherever they currently say the migration is ready after only the block model and Vue foundation.

## Capabilities

### New Capabilities

- `dashboard-development-harness`: reproducible Bun-based frontend development, executable quality and browser gates, working local API integration, and coding-harness guidance and tools for dashboard work.

### Modified Capabilities

None. This change strengthens how the dashboard is developed and verified without changing its shipped review behavior.

## Impact

- `web/package.json`, lockfile, package-manager pins, Vite configuration, test configuration, and frontend source organization.
- `.github/workflows/frontend-assets.yml` and new or extended verification scripts.
- `.pi/skills/`, `.claude/skills/`, `AGENTS.md`, and shared MCP configuration.
- A new dashboard design reference under a stable repository path.
- Contributor documentation and roadmap claims about frontend commands and migration order.
- `migrate-dashboard-review-to-vue` proposal, design, and prerequisite tasks.
- New development dependencies for Bun-managed formatting, linting, unit/component testing, browser testing, and routing. No runtime JavaScript toolchain dependency is added to the Rust binary.
