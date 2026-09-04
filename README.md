# openspec-doc

A local review dashboard for [OpenSpec](https://github.com/Fission-AI/OpenSpec) projects, plus an agent
hook bridge.

You read a coding agent's exploration or proposal in a browser **while it is happening**, select text,
leave anchored comments, and submit a phase verdict. The hook bridge feeds that verdict back to the agent as
a directive it acts on — with your next prompt, or at the turn boundary if no prompt is coming.

Everything is a plain file under `.openspec-doc/`. No database, no daemon, no cloud.

```text
   agent explores                              reviewer reads
        │                                             │
        ▼                                             ▼
  hook explore ──► .openspec-doc/scratch/…  ──►  127.0.0.1 dashboard
                                                      │
                                        comment + phase verdict
                                                      │
                                                      ▼
   agent acts  ◄─────── directive ◄─────────────  verdicts/…
        │                  ▲   ▲
        │     hook prompt ─┘   └─ hook stop blocks
        │     (start of your      (turn boundary,
        │      next turn)          when no prompt comes)
        │
        └──► creates the change, runs `scratch claim` ──► note promoted
```

Delivered exactly once across the two.

## Install

```bash
cargo install --path crates/cli --locked
openspec-doc serve          # discovers the project root by walking up from the cwd
openspec-doc serve url      # this project's dashboard URL, and whether one is serving it
```

The `openspec` CLI must also be on `PATH` — promotion runs `openspec validate`.

Claude Code hook registration requires version 2.1.139 or newer because `init` writes exec-form argument
vectors rather than shell-form command strings.

Once the hooks are wired, `serve` is not something you run: the turn-end hook brings a dashboard up at
the first turn boundary of a session that has something to review, and keeps it up at every later one.

**Each project gets its own port and keeps it.** The first time a project root is seen it is assigned the
lowest free port in `4321`–`4352`, recorded in the machine's state directory — `$XDG_STATE_HOME/openspec-doc/ports.json`,
defaulting to `~/.local/state`, and the local data directory on macOS and Windows. Machine-global rather
than under `.openspec-doc/`, because ports are global to the machine and `.openspec-doc/` is gitignored:
`git clean -xdf` would delete a record whose server is still running. So a project's URL is the same today
as it was last week whatever order your checkouts started in, which is what makes it bookmarkable and what
lets `serve url` answer with nothing running.

That record is a hint and never an authority. Whether a dashboard is *running* is asked of the network: the
whole range is swept for this project's root before anything is started, so losing the record costs
stability and never correctness.

Passing `--port` explicitly — as the frontend loop below does — gets you exactly that port, fails rather
than falling back to another, and changes nothing about the project's assignment.

## Documentation

```bash
cd docs && bun install && bun dev
```

| | |
| --- | --- |
| **Quickstart** | `docs/docs/quickstart.md` — install, wire hooks, drive one loop |
| **Vision & MVP scope** | `docs/docs/vision.md` — what is in scope, what is not, when it is done |
| **Roadmap** | `docs/docs/roadmap.md` — shipped capabilities and open changes |
| **Concepts** | `docs/docs/concepts/` — the review loop, scoping, anchoring, pointer-not-embed |
| **Reference** | `docs/docs/reference/` — CLI, on-disk state, routes, agent hooks |
| **Development** | `docs/docs/development/` — testing, manual verification, conventions |

The site also emits `llms.txt` and per-section `llms-*.txt` indexes, since this tool's audience is
substantially agents.

## Layout

```text
crates/core     project discovery, comments, scratch notes, verdicts, hook bridge   (no I/O framework)
crates/server   the axum dashboard boundary: JSON routes, embedded assets, watcher, SSE
crates/cli      the openspec-doc binary
web             the dashboard's frontend: Vite, Vue, Tailwind, shadcn-vue
```

`core` holds every rule and every file format; `server` and `cli` are two front ends over it with no
persistence logic of their own.

`web/dist/` is a gitignored build artifact, compiled into the binary via rust-embed. `make build`
builds the frontend first, then the binary — a fresh clone has no dist and cargo will not compile
without one. Distribution will move to prebuilt binaries (release workflow + install script), so
user machines need neither Bun nor cargo — see
[conventions](docs/docs/development/conventions.md#the-frontend).

## Build and test

The Makefile is the entry point: every target builds `web/dist` first if it is absent, so none of
these needs Bun to have been run by hand on a fresh clone.

```bash
make build                                             # web/dist if absent, then the binary
make test                                              # cargo test --workspace, 409 tests
make check                                             # clippy, rustfmt, the frontend gate, the docs build
make frontend                                          # force a web/dist rebuild after editing web/src
cargo test -p openspec-doc-core -p openspec-doc-cli    # 346 of them; needs `openspec` on PATH for two
```

The Rust toolchain is pinned in `rust-toolchain.toml`, so rustup uses that version rather than the
machine's default channel.

`make check` and `make test` also run automatically on every push to `main` and every pull request,
on **Linux, macOS, and Windows**, via `.github/workflows/rust.yml`. The fixtures are portable and the
full suite runs on each leg. See
[testing](docs/docs/development/testing.md#what-the-automated-run-covers) for coverage details.

## Dashboard frontend development

`web/dist/` is generated, not committed. Use Bun 1.3.2 from `web/.bun-version`; do not use npm or
regenerate assets with a different package manager.

```bash
cd web
bun install --frozen-lockfile
bun run check
bun run build
bun run test:e2e
```

For the live Vite loop, run the Rust API on fixed port 8791 in one shell and Vite in another:

```bash
openspec-doc serve --port 8791 --no-open
cd web && bun run dev
```

Vite proxies same-origin `/api` to `http://127.0.0.1:8791`; set
`OPENSPEC_DOC_API_PROXY_TARGET` for another local target. `bun run test:e2e` builds and serves the
embedded Rust application on its own fixed test port, so a Vite-only check is not enough.

For frontend-only work, run `bun run dev:mock`. This starts the MSW browser worker with deterministic
index, scope, comment, reply, status, verdict, and SSE handlers; no Rust server is needed. Handlers,
fixture data, the worker script, and a "rewrite artifact" trigger for simulating an agent editing the
open artifact all live under `web/src/mocks/`, which the `import.meta.env.DEV` gate keeps out of the
production bundle — `bun run check:mocks` fails the build if any of it reaches `dist/`. Vitest uses
the same handlers through `setupServer`.

No network needed, but `scratch::promote`'s validation tests shell out to `openspec validate`, so that
binary must be on `PATH`. See `docs/docs/development/testing.md`.

## Status

The loop is closed and verified on live sessions across Claude Code and pi.dev, at both delivery points,
including the one thing no test can establish: that an agent acts on an injected directive rather than
refusing it as prompt injection.

The MVP criterion has been run end to end — anchored comments on an exploration, a move-to-proposal verdict,
and a proposal that accounts for those comments because they arrived before it was written rather than
after.

Comment threads now support reply, resolve, reopen, and addressed-claim judgement in browser. Remembering
to start `serve` still needs operator knowledge. Setup remains another gap: hook config lives in gitignored
local settings, so fresh clone has no hooks and nothing says so. See roadmap.

Rust rewrite of a TypeScript prototype, kept under `openspec-doc-rs-example/` for reference.
`AGENTS.md` holds the working rules for this repo.
