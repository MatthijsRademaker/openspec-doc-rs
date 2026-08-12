# dashboard-development-harness Specification

## Purpose

The repository's frontend development and verification lane: one pinned Bun toolchain and lockfile for dependency installation and builds, one executable quality gate over formatting, linting, Vue and TypeScript semantics and unit behaviour, browser verification against a freshly built binary and its embedded assets, the Vite development proxy to the local Rust API, Vue Router's ownership of dashboard browser paths, and the guidance and design sources a coding harness is pointed at before it touches `web/`.
## Requirements
### Requirement: The frontend is built with one pinned Bun toolchain and embedded at compile time

The repository SHALL use one exactly pinned Bun toolchain and one committed Bun lockfile for all dashboard frontend dependency installation and build commands, and SHALL reject lockfile drift during a frozen install. The built frontend SHALL be a generated, uncommitted artifact that the server crate embeds at compile time, a build directory that is absent SHALL fail the compile rather than produce a binary with no interface to serve, and a clean checkout SHALL be provably able to install, check, build and serve it.

`web/dist/` was committed when this capability was written, guarded by a CI check that rebuilt it and compared bytes. It is now gitignored, and the guarantee moved with it: there is no committed baseline to drift from, so what has to hold is that the artifact cannot be silently absent and that a clean checkout can produce it. Committing a build artifact bought `cargo install` with no JavaScript toolchain; embedding at compile time buys the same thing without the diff noise, at the cost of one build step before Cargo on a fresh clone.

Known gap, stated rather than implied: only an *absent* `web/dist/` is a compile error. A `web/dist/` that exists but is empty compiles, and the binary then panics on the first request for the shell. Closing it needs a compile-time assertion that the shell document is embedded, not a change to this requirement.

#### Scenario: Frozen installation uses the pinned Bun version

- **WHEN** frontend dependencies are installed in a clean checkout
- **THEN** the pinned Bun version SHALL install them from the committed Bun lockfile without modifying it

#### Scenario: An absent build fails the compile

- **WHEN** the server crate is compiled with no `web/dist/` directory present
- **THEN** compilation SHALL fail rather than produce a binary with no interface to serve

#### Scenario: A clean checkout can build and serve the frontend it embeds

- **WHEN** the frontend lane runs on a checkout with no dependency cache and no prior build output
- **THEN** the pinned frozen install, the frontend gate, the production build, and browser verification against the freshly built binary SHALL all pass

#### Scenario: Installed binary needs no JavaScript toolchain

- **WHEN** `openspec-doc` is installed and run on a machine with neither Bun nor Node available
- **THEN** the binary SHALL serve the complete embedded dashboard

### Requirement: Frontend changes have one executable quality gate

The repository SHALL provide a bounded frontend check that verifies formatting, linting, Vue and TypeScript semantics, and unit and component behavior, and SHALL include Vue single-file components in those checks rather than excluding them.

#### Scenario: Valid frontend passes every static and unit check

- **WHEN** the frontend check is run on a conforming checkout
- **THEN** formatting, linting, typechecking, unit tests, and component tests SHALL all pass

#### Scenario: Invalid Vue code fails the gate

- **WHEN** a Vue single-file component contains a lint or type error covered by the configured tools
- **THEN** the frontend check SHALL fail against that component

#### Scenario: Component regression fails the gate

- **WHEN** a tested dashboard interaction no longer produces its required state or accessible output
- **THEN** the component test lane SHALL fail

### Requirement: Browser verification exercises the embedded application

The repository SHALL provide browser tests that start a freshly built Rust binary against deterministic project data and exercise the assets embedded in that binary, including dashboard loading, theme persistence, routing, responsive layout, and browser console health.

#### Scenario: Embedded dashboard passes browser smoke

- **WHEN** the browser suite runs against the freshly built binary
- **THEN** the dashboard SHALL load fixture data, persist a selected theme across reload, navigate through registered routes, and produce no unexpected console errors

#### Scenario: Responsive layout remains usable

- **WHEN** the browser suite opens the dashboard at desktop and narrow viewport sizes
- **THEN** primary content and controls SHALL remain reachable without being clipped by the layout

#### Scenario: Missing embedded asset fails browser verification

- **WHEN** the shell references an asset not served by the binary
- **THEN** browser verification SHALL fail rather than accepting a partially rendered page

### Requirement: Vite development reaches the local Rust API

The Vite development server SHALL proxy same-origin `/api` requests to a documented local Rust server target, SHALL provide a fixed default development target, and SHALL allow that target to be overridden explicitly.

#### Scenario: Default development commands connect

- **WHEN** a contributor starts the Rust server on the documented development port and starts Vite with no proxy override
- **THEN** a frontend request to `/api/index` SHALL reach the Rust server and return index JSON

#### Scenario: Proxy target is overridden

- **WHEN** a contributor supplies a different supported API proxy target
- **THEN** Vite SHALL forward `/api` requests to that target

### Requirement: Dashboard browser paths are owned by Vue Router

The frontend SHALL use Vue Router to own its browser navigation, with the existing index registered as a route-owned view and future scope paths added only when their views and server support exist.

#### Scenario: Index is rendered through the router

- **WHEN** the embedded application opens at `/`
- **THEN** Vue Router SHALL render the index view without changing its required index behavior

#### Scenario: Unsupported scope routes are not faked

- **WHEN** this development-harness change is complete before Vue scope views exist
- **THEN** it SHALL NOT register placeholder session or change views that present fake success or empty scope data

### Requirement: Supported coding harnesses receive dashboard-specific guidance

The repository SHALL expose routable frontend-design, shadcn-vue, browser-verification, and dashboard-workflow skills to both Pi and Claude, SHALL route `web/` work to the repository-specific guidance, and SHALL fail an automated check when shared harness guidance drifts between copies.

#### Scenario: Pi discovers dashboard guidance

- **WHEN** a fresh trusted Pi session starts in the repository
- **THEN** it SHALL list skills covering dashboard work, shadcn-vue, frontend design, and browser verification

#### Scenario: Claude discovers equivalent dashboard guidance

- **WHEN** a fresh Claude session starts in the repository
- **THEN** it SHALL expose equivalent guidance for the same frontend task shapes

#### Scenario: Frontend task loads repository constraints

- **WHEN** an agent starts a task that adds or changes files under `web/`
- **THEN** its available instructions SHALL direct it to the dashboard design reference, existing shadcn primitives, Bun commands, offline-asset rule, semantic tokens, the embedded `dist/` boundary, and required verification lanes

#### Scenario: Shared skill copies drift

- **WHEN** guidance intended to be identical across Pi and Claude differs
- **THEN** the harness-resource check SHALL fail and SHALL name the mismatched files

### Requirement: Shadcn registry tooling is available through shared MCP configuration

The repository SHALL configure one project-shared shadcn-vue MCP server, pinned to an explicit version and rooted at `web/`, so supported coding harnesses use the actual component configuration and registry rather than guessing registry APIs.

#### Scenario: MCP exposes configured registry access

- **WHEN** an agent connects to the project shadcn-vue MCP server and requests configured registries
- **THEN** the server SHALL expose the registry configuration consumed from `web/components.json`, while retaining standard shadcn registry access without an explicit custom registry

#### Scenario: Harness verifies frontend configuration alongside MCP

- **WHEN** a fresh supported harness connects to the project server
- **THEN** it SHALL verify Bun from `web/.bun-version` and `web/package.json`, Vue/Vite/Tailwind from `web/package.json` and `web/vite.config.ts`, and aliases/component/style paths from `web/components.json`; the verification SHALL NOT assume the MCP server exposes a project-info method

#### Scenario: MCP searches the registry

- **WHEN** an agent searches for a shadcn-vue component through the configured MCP server
- **THEN** the server SHALL return registry metadata using the pinned CLI version

#### Scenario: MCP does not depend on caller working directory

- **WHEN** a supported harness starts from the repository root
- **THEN** the MCP server SHALL still resolve `web/` as its project directory

### Requirement: Dashboard visual decisions have a repository source of truth

The repository SHALL maintain a dashboard design reference defining the review-workbench direction, desktop and narrow layouts, typography and offline font delivery, semantic review-state tokens, reviewer-versus-agent presentation, accessibility constraints, and motion policy, and frontend guidance SHALL require agents to consult it before styling product UI.

#### Scenario: Agent begins product UI work

- **WHEN** an agent is asked to create or restyle a dashboard view or component
- **THEN** its repository guidance SHALL direct it to inspect the dashboard design reference before choosing layout, type, color, or motion

#### Scenario: Motion policy is carried by a worked example

- **WHEN** the design reference states the motion policy
- **THEN** it SHALL name one shipped gesture as the reference implementation and state the rules that gesture demonstrates against its actual tokens, durations, and mechanisms, rather than stating the policy only as a list of permitted geometry and rejected effects, and frontend guidance SHALL point agents at that example before they design new motion

#### Scenario: Typography matches executable assets

- **WHEN** the dashboard declares a font family in component or design configuration
- **THEN** that family SHALL be bundled into embedded assets or the declaration SHALL match the local fallback actually rendered, without a network font request

#### Scenario: Review states use semantic design tokens

- **WHEN** open, addressed, resolved, verdict, or delivery state is styled
- **THEN** the frontend SHALL use documented semantic tokens whose light and dark values are defined centrally

### Requirement: The mock lane can originate an artifact change

The mock development lane SHALL be able to originate a change to a scope's artifact content and emit it on the live-update channel, independently of any reviewer mutation, and SHALL make that trigger reachable by a contributor working in the browser.

The mock lane's event channel currently broadcasts only as a side effect of the reviewer's own actions — posting a comment, replying, setting a status. The path it cannot exercise is the one that matters most: an agent rewriting an artifact on disk while the reviewer reads it. Without a way to originate that, the behaviour on arrival of external content can be built but not seen, and a contributor's only way to check it is to run a real agent against a real repository.

#### Scenario: An artifact rewrite can be originated without a mutation

- **WHEN** a contributor triggers an artifact change in the mock lane
- **THEN** the mock scope's artifact content SHALL change, the live-update channel SHALL emit, and no comment, reply, status, or verdict SHALL have been recorded

#### Scenario: The trigger is reachable from the browser

- **WHEN** a contributor is running the mock lane
- **THEN** the trigger SHALL be reachable without editing fixture source and restarting the server

#### Scenario: The trigger is absent from the production build

- **WHEN** the frontend production build completes
- **THEN** no mock trigger, fixture, or handler SHALL be present in the built output
