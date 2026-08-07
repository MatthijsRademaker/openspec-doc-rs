# Conventions

## Crate layout

```text
crates/core     project discovery, comments, scratch notes, verdicts, hook bridge   (no I/O framework)
crates/server   the axum dashboard boundary: JSON routes, embedded assets, watcher, SSE
crates/cli      the openspec-doc binary
web             the dashboard's frontend: Vite, Vue, Tailwind, shadcn-vue
```

`core` holds **every rule and every file format**. `server` and `cli` are two front ends over it and
contain no persistence logic of their own; Vue is browser rendering and interaction over server JSON. This
makes CLI a real check on dashboard: both mutation paths use same core writers, so `comment list` shows
exactly what browser interaction recorded.

It is also what keeps two agents supportable. Decision logic lives in `core`; the agent-specific parts are
a thin adapter that translates wire formats and nothing else.

## The frontend

```text
cd web
bun install --frozen-lockfile   # the pinned Bun is in packageManager and .bun-version
bun run build                   # type-checks with vue-tsc, then writes web/dist/
bun run dev                     # Vite dev server; pair with `openspec-doc serve --port 8791 --no-open` for the API
```

**`web/dist/` is a build artifact, not committed.** `crates/server` compiles the built assets in with
`rust-embed`, so the binary serves the interface with no asset directory and no checked-out repository.
The consequence for local development: rust-embed needs a dist present at compile time, a fresh clone
has none, and cargo will not compile without one — `make build` builds the frontend first, then the
binary. Distribution is headed for prebuilt binaries (a release workflow that builds the frontend,
then the embedding binary, plus an install script), so user machines will need neither Bun nor cargo.

**CI proves a clean checkout builds and embeds** — `.github/workflows/frontend-assets.yml` does
`bun install --frozen-lockfile && bun run build` from a clean checkout, then drives the embedded app in
Chromium.

Two things keep that lane honest, and both are load-bearing:

- **Bun is pinned exactly** in `web/package.json` (`packageManager`) and `web/.bun-version`, and `bun.lock` is
  the only frontend lockfile. The package manager decides which esbuild and bundler build the assets, so a
  different one is the likeliest source of a difference that has nothing to do with the source.
- **Tailwind's sources are listed explicitly** in `src/style.css` via `source(none)` and `@source`. Left on
  automatic detection Tailwind scans everything the repository does not gitignore, so the CSS could grow
  from class names found in files that merely happen to exist on disk. Explicit sources keep the CSS a
  function of the source tree alone.

`bun run check` is the bounded frontend gate: Biome formats and lints TypeScript, JSON, CSS, and Vue;
`vue-tsc -b` owns Vue/TypeScript semantics; Vitest and Vue Test Utils cover logic and components. The
embedded browser lane is explicit: `bun run test:e2e` builds the frontend and the Rust binary, then tests
the embedded assets it just built.
`vue-tsc -b` also runs as the first half of `bun run build`, which is where a type error fails.

`shadcn-vue`'s scaffold imported Geist from Google Fonts. Dashboard typography is bundled:
Cormorant Garamond gives product and document headings their display role, IBM Plex Sans is reserved for
sustained prose and comments, and IBM Plex Mono carries operational labels, identifiers, paths, timestamps,
and compact controls. A dashboard that binds `127.0.0.1` and serves every asset from the binary must not
need the internet to render.

Dashboard has one dark observatory identity; no theme toggle, persisted appearance preference, or alternate
light token set exists. `designs/dashboard-review-workbench.md` owns implementation guidance, and its source
boards live under `designs/visual-language/`. Those original PNGs are design inputs, never runtime payload.
`web/public/assets/images/` contains only named, optimized derivatives used by frontend code. Production
build runs `bun run check:assets`, which rejects unlisted rasters, source-board copies, and payload above the
6 MiB budget.

## Working rules

`AGENTS.md` holds the full set. The ones that show up most in the code:

**Invalid states fail loudly.** No silent defaults, swallowed errors, or fake success values. A malformed
sidecar line is an error, not a skipped record.

**Errors propagate with their source chain intact.** The CLI prints `caused by:` lines.

One deliberate exception, and it is written into a spec so it cannot be mistaken for sloppiness: a hook
standing between the reviewer and their own prompt fails **soft**. Refusing the human's input is worse than
the problem being reported. Everywhere else, fail fast.

**Prefer small feature-named files over large ones.** Navigation beats file count.

**Prefer executable truth in `src/` when docs disagree.** Including these docs.

**No backwards compatibility.** Old paths, flags, and schemas get replaced and deleted, not preserved. When
snapshot-based promotion was replaced, `snapshot.rs`, `Promotion::Ambiguous` and `Error::Snapshot` all went
with it — while the variant exists, so does the concept it encodes.

**Remove dead code.** Twice now, a function has been found with no non-test caller
(`scratch::ensure_session`, and the whole snapshot API). Both times that absence *was* the bug: the note was
never being created, and the mechanism was never being invoked.

## Documenting decisions in code

Comments explain **why**, especially where the obvious thing is wrong. Some examples worth imitating:

- `translate.rs` opens by explaining that a bare imperative directive was refused as prompt injection, so
  the templates below are attributed pointers.
- `explore.rs` explains that only the *directory* is created, because an empty placeholder makes the
  agent's first write fail.
- `note.rs` explains that a note exists exactly when an exploration was written, and what promotion leans
  on that for.

Each of those encodes a live failure. Without the comment the next person deletes the guard.

## OpenSpec workflow

Changes go through `openspec/changes/<name>/` with proposal, design, spec deltas, and tasks, then archive
into `openspec/changes/archive/` with their specs synced into `openspec/specs/`.

Two habits worth keeping:

**Record what a live run found, in the task.** Several tasks now carry a `**PASSED**` note with the session
id and what the agent actually did. That is the only durable record of behaviour no test captures.

**When implementation reveals a defect, add the task rather than absorbing it silently.** Tasks like
`1.4`–`1.7` and `3.3a` exist because something was found mid-implementation. The alternative — fixing it
quietly — is how a change's scope runs away from whoever is reviewing it.
