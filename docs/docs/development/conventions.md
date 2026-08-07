# Conventions

## Crate layout

```text
crates/core     project discovery, comments, scratch notes, verdicts, hook bridge   (no I/O framework)
crates/server   the axum dashboard: routes, pages, filesystem watcher, SSE
crates/cli      the openspec-doc binary
web             the dashboard's frontend: Vite, Vue, Tailwind, shadcn-vue
```

`core` holds **every rule and every file format**. `server` and `cli` are two front ends over it and
contain no persistence logic of their own. This is what makes the CLI a real check on the dashboard: both go
through the same code, so `comment list` shows exactly what a browser interaction recorded.

It is also what keeps two agents supportable. Decision logic lives in `core`; the agent-specific parts are
a thin adapter that translates wire formats and nothing else.

## The frontend

```text
cd web
bun install --frozen-lockfile   # the pinned Bun is in packageManager and .bun-version
bun run build                   # type-checks with vue-tsc, then writes web/dist/
bun run dev                     # Vite dev server; pair with `openspec-doc serve --port 8791 --no-open` for the API
```

**`web/dist/` is committed.** That is what keeps Bun and Node out of `cargo install`: `crates/server` compiles
the built assets in with `rust-embed`, so the binary serves the interface with no asset directory, no
checked-out repository and no toolchain. The cost is that a stale `dist/` would ship an interface that does
not match its source, silently.

**So CI rebuilds it and fails if it differs** — `.github/workflows/frontend-assets.yml` does
`bun install --frozen-lockfile && bun run build` from a clean checkout and goes red on any drift in `web/`.
Rebuild and commit `dist/` in the same commit as the source change; they are one unit.

Two things keep that check honest, and both are load-bearing:

- **Bun is pinned exactly** in `web/package.json` (`packageManager`) and `web/.bun-version`, and `bun.lock` is
  the only frontend lockfile. The package manager decides which esbuild and bundler build the assets, so a
  different one is the likeliest source of a difference that has nothing to do with the source.
- **Tailwind's sources are listed explicitly** in `src/style.css` via `source(none)` and `@source`. Left on
  automatic detection Tailwind scans everything the repository does not gitignore, and `web/dist/` is
  deliberately not gitignored — so it read class names out of the previous build's own output, and the CSS
  grew depending on whether `dist/` happened to exist when the build started.

`bun run check` is the bounded frontend gate: Biome formats and lints TypeScript, JSON, CSS, and Vue;
`vue-tsc -b` owns Vue/TypeScript semantics; Vitest and Vue Test Utils cover logic and components. The
embedded browser lane is explicit: `bun run test:e2e` builds the Rust binary and tests its committed assets.
`vue-tsc -b` also runs as the first half of `bun run build`, which is where a type error fails.

`shadcn-vue`'s scaffold imported Geist from Google Fonts. It is replaced by bundled IBM Plex Sans and
IBM Plex Mono: a dashboard that binds `127.0.0.1` and serves every asset from the binary must not need the
internet to render.

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
