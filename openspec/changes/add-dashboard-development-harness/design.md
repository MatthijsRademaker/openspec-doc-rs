## Context

The dashboard has two distinct distribution environments. Contributors build `web/`; users install and run a Rust binary whose committed frontend assets are embedded with `rust-embed`. `add-vue-dashboard-foundation` made that boundary explicit and proved it with npm, exact Node pinning, a clean-build asset comparison, and a Node-free `cargo install` run.

The next change, `migrate-dashboard-review-to-vue`, is interaction-heavy: route-owned scope views, rendered markdown blocks, inline comment threads, mutations, verdict controls, and SSE updates. Current repository gives an agent no frontend-specific route into that work. Only `vue-tsc` and asset freshness run; Vite cannot reach the Rust API as documented; no browser test is executable from repository state; Pi has no project MCP server; neither Pi nor Claude has dashboard guidance; and no visual reference distinguishes this product from stock grayscale shadcn.

The owner prefers Bun. This change therefore replaces npm rather than preserving two package-manager paths. Reproducibility remains load-bearing: if Bun cannot reproduce `dist/` from a clean checkout, migration stops rather than weakening the freshness check.

## Goals / Non-Goals

**Goals:**

- Make Bun the only frontend package manager, pinned exactly and proven against the committed-asset distribution model.
- Give agents one bounded command for frontend static checks and tests, plus a real-browser gate against the embedded application.
- Make Vite development work against a local Rust server without CORS workarounds or guessed proxy setup.
- Give Pi and Claude repository-specific frontend guidance and pinned shadcn-vue registry access.
- Establish a concrete review-workbench design direction before scope-page components are written.
- Put Vue Router in place before three browser paths become one SPA.
- Make the scope-page migration explicitly wait for this foundation.

**Non-Goals:**

- Not implementing the Vue scope pages, inline threads, composer, verdict bar, or SSE client; `migrate-dashboard-review-to-vue` owns them.
- Not changing any shipped HTTP API or review behavior.
- Not adding Pinia before a shared state owner exists. The migration may add it when scope data, mutations, counts, and SSE reconciliation need one store.
- Not adding Pinia Colada, generated OpenAPI clients, Knip, a motion library, Chrome DevTools MCP, or swarm machinery by analogy with another repository.
- Not requiring Bun, Node, or frontend files on the machine running the installed binary.

## Decisions

### Bun 1.3.2 is the only frontend package manager

`web/package.json` names `bun@1.3.2`, `.bun-version` contains the same version, and `bun.lock` is committed. `package-lock.json`, `.npmrc`, `.nvmrc`, the Node engine constraint, and npm commands are deleted. Clean installs use `bun install --frozen-lockfile`; scripts run through `bun run`; shadcn CLI and MCP use `bunx --bun` with an exact shadcn-vue version.

Bun is not invoked from Cargo. `web/dist/` remains committed and `rust-embed` remains the distribution boundary, so changing package manager changes contributor tooling but not installation or runtime.

Alternative: retain npm for the asset build and use Bun only for agent tools. Rejected. Two package managers and two lockfiles make generated component and dependency changes ambiguous, while preserving npm has no user-facing value once Bun is chosen.

### Reproducibility is re-proved, not inherited

CI installs Bun 1.3.2 in a clean checkout, runs a frozen install and production build, and fails on any `web/` drift including untracked files. The implementation also performs consecutive clean builds, including one after removing existing `dist/`, to confirm bytes do not depend on prior output. Existing explicit Tailwind sources remain unchanged.

A Node- and Bun-free `cargo install --path crates/cli --locked` run is repeated after migration. Package-manager preference does not weaken distribution evidence.

Alternative: trust Bun because lockfile is committed. Rejected. Locking dependency resolution does not prove deterministic bundler output.

### One layered frontend gate

Bun manages tools, but each tool keeps its own job:

- Biome formats and lints TypeScript, JSON, CSS, and Vue files where supported by the pinned release.
- `vue-tsc --noEmit` owns Vue and TypeScript semantic checking.
- Vitest plus Vue Test Utils and a DOM environment own pure logic and component behavior.
- Playwright owns browser behavior against the Rust-served embedded app.

`bun run check` is the bounded developer/agent gate: format check, lint, typecheck, and unit/component tests. `bun run build` performs typecheck and Vite build. Browser E2E remains an explicit `bun run test:e2e` lane because it builds Rust and starts processes.

Biome must be validated against current SFC files before adoption. If pinned Biome cannot parse and check them without broad ignores, implementation uses ESLint with Vue support plus a formatter instead; tasks may not mark lint complete by excluding `*.vue`.

Knip is excluded. Generated shadcn barrel exports and route-loaded views make dead-code output noisy, and this small frontend does not justify another gate yet.

### Browser tests exercise embedded assets

Playwright starts a freshly built `openspec-doc` binary against a deterministic temporary OpenSpec fixture and fixed test port. Tests verify index data, theme persistence, route navigation, narrow and desktop layouts, and zero unexpected console errors. Testing only Vite would miss the `rust-embed` boundary this repository deliberately chose.

The Vite development path receives a smaller proxy configuration test, but it is not substituted for embedded-app E2E.

### Development uses an explicit proxy contract

Vite proxies `/api` to `OPENSPEC_DOC_API_PROXY_TARGET`, defaulting to `http://127.0.0.1:8791`. Contributor docs pair it with:

```text
openspec-doc serve --port 8791 --no-open
bun run dev
```

The target resolver is a pure exported function with a unit test. Runtime `serve` keeps its current random-port default; fixed port is a development convention, not a product behavior change.

Alternative: add CORS and configure an absolute API URL in browser code. Rejected. Production is same-origin, so development should preserve that contract.

### Vue Router lands with the index route; scope routes remain migration work

The app creates a router and moves the existing index screen into a route-owned view. This change registers `/` only. `migrate-dashboard-review-to-vue` adds `/sessions/:id` and `/changes/:name` when their views and JSON endpoints exist. History mode is used because Rust owns those browser paths and the migration already requires serving the shell there.

Pinia does not land here. Router has an immediate owner; a global store does not.

### Skills are repository-specific and mirrored deliberately

Pi and Claude each receive four routable skills: frontend design, shadcn-vue, browser verification, and `openspec-doc` dashboard work. Dashboard and shadcn skills name actual `web/` paths, Bun commands, semantic-token rules, offline assets, explicit Tailwind sources, API boundaries, Router ownership, embedded `dist/`, and required gates. Generic reference skills are constrained by this product's accessibility and restrained-motion requirements.

Files visible to both harnesses are kept byte-identical where content is shared, and a repository check fails on drift. `AGENTS.md` routes every `web/` task through dashboard and shadcn guidance. No `.pi/package.json` is added: trusted project `.pi` resources are already auto-discovered.

### One shared, pinned MCP server

Project `.mcp.json` configures shadcn-vue using `bunx --bun shadcn-vue@2.8.1 mcp --cwd web`. Shared config is preferred over `.pi/mcp.json` because both supported harnesses need the same registry view; Pi-only files remain override territory. Server stays lazy through the Pi MCP adapter.

Verification must connect and search registry metadata from fresh Pi and Claude sessions. The pinned shadcn-vue 2.8.1 MCP is a registry bridge and exposes no project-info method, so project facts are verified directly from the files it consumes: `web/.bun-version`, `web/package.json`, `web/vite.config.ts`, and `web/components.json`. Committing JSON without starting the server and completing both checks is not completion.

A browser MCP is excluded. Playwright already supplies browser automation and tests without adding a second overlapping control surface.

### Design source is a review workbench, not either reference product

`designs/dashboard-review-workbench.md` and committed desktop/mobile reference frames define product intent before migration UI starts. Direction is document-first and information-dense but calm: artifact content dominates; comments read as a spatial conversation; reviewer and agent voices are unmistakable; open/addressed/resolved and verdict delivery use semantic tokens, not arbitrary colors; motion explains state changes and never decorates waiting.

Typography is self-hosted and embedded. The implementation either bundles the selected family (recommended: IBM Plex Sans with IBM Plex Mono) or changes `components.json` to match executable CSS. Remote font requests and a declared-but-unused font are both invalid.

The design reference is required input to frontend skills and browser review. It is not an excuse to freeze exact pixels: content density and long identifiers still require responsive behavior.

## Risks / Trade-offs

- **Bun output differs by platform or prior state** → pin Bun exactly, retain explicit Tailwind sources, run clean repeated builds, and stop migration if byte reproducibility fails.
- **Biome's Vue support is incomplete** → validate against every current SFC; switch to Vue-aware ESLint plus formatter rather than hiding files.
- **Playwright E2E becomes slow** → keep component tests in fast gate and browser suite small, but retain embedded-binary boundary in CI.
- **Mirrored Pi/Claude skills drift** → compare shared files in automated check and fail on mismatch.
- **Design skill encourages decorative excess** → project design reference overrides generic advice with document readability, accessibility, offline assets, and restrained motion.
- **Fixed development port collides locally** → allow `OPENSPEC_DOC_API_PROXY_TARGET` override; keep product server's random default unchanged.
- **Router lands before scope routes** → register only existing `/`; no placeholder views or fake routes.

## Migration Plan

1. Pin Bun and replace npm metadata and lockfile.
2. Rebuild twice from clean states; commit Bun-built `dist/`; verify stale-output check and Node/Bun-free Cargo install.
3. Add format, lint, type, unit/component, and browser tools; make gates pass on current index.
4. Add Vite proxy and Router without changing shipped index behavior.
5. Add design source, skills, MCP, and agent routing; verify skill discovery plus direct frontend configuration and MCP registry search in fresh Pi and Claude sessions.
6. Update contributor docs, CI, roadmap, and all affected dependency statements in `migrate-dashboard-review-to-vue`.
7. Only then unblock scope-page migration.

Rollback is one commit before migration starts: restore npm metadata and npm-built `dist/`. No persisted user data or runtime schema changes occur.

## Open Questions

None. Bun preference is settled; Pinia remains deliberately deferred to the change that can name its state boundary.
