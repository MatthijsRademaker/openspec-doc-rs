# openspec-doc

A local review dashboard for [OpenSpec](https://github.com/Fission-AI/OpenSpec) projects, plus an agent
hook bridge.

You read a coding agent's exploration or proposal in a browser **while it is happening**, select text,
leave anchored comments, and submit a phase verdict. The hook bridge feeds that verdict back to the agent as
a directive it acts on — with your next prompt, or at the turn boundary if no prompt is coming.

Everything is a plain file under `.openspec-doc/`. No database, no daemon, no cloud.

```
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
cargo install --path crates/cli
openspec-doc serve          # discovers the project root by walking up from the cwd
```

The `openspec` CLI must also be on `PATH` — promotion runs `openspec validate`.

## Documentation

```bash
cd docs && bun install && bun dev
```

| | |
|---|---|
| **Quickstart** | `docs/docs/quickstart.md` — install, wire hooks, drive one loop |
| **Vision & MVP scope** | `docs/docs/vision.md` — what is in scope, what is not, when it is done |
| **Roadmap** | `docs/docs/roadmap.md` — shipped capabilities and open changes |
| **Concepts** | `docs/docs/concepts/` — the review loop, scoping, anchoring, pointer-not-embed |
| **Reference** | `docs/docs/reference/` — CLI, on-disk state, routes, agent hooks |
| **Development** | `docs/docs/development/` — testing, manual verification, conventions |

The site also emits `llms.txt` and per-section `llms-*.txt` indexes, since this tool's audience is
substantially agents.

## Layout

```
crates/core     project discovery, comments, scratch notes, verdicts, hook bridge   (no I/O framework)
crates/server   the axum dashboard: routes, pages, filesystem watcher, SSE
crates/cli      the openspec-doc binary
web             the dashboard's frontend: Vite, Vue, Tailwind, shadcn-vue
```

`core` holds every rule and every file format; `server` and `cli` are two front ends over it with no
persistence logic of their own.

`web/dist/` is committed and compiled into the binary, so `cargo install` needs no Node toolchain. CI
rebuilds it from a clean checkout and fails if it differs — see
[conventions](docs/docs/development/conventions.md#the-frontend).

## Build and test

```bash
cargo test --workspace                                 # 228 tests
cargo test -p openspec-doc-core -p openspec-doc-cli    # 182 of them, hermetic
cargo clippy --workspace --all-targets
cargo fmt --all --check
```

No network needed, but `scratch::promote`'s validation tests shell out to `openspec validate`, so that
binary must be on `PATH`. See `docs/docs/development/testing.md`.

## Status

The loop is closed and verified on live sessions across Claude Code and pi.dev, at both delivery points,
including the one thing no test can establish: that an agent acts on an injected directive rather than
refusing it as prompt injection.

The MVP criterion has been run end to end — anchored comments on an exploration, a move-to-proposal verdict,
and a proposal that accounts for those comments because they arrived before it was written rather than
after.

Two things still need a terminal or insider knowledge — closing out a comment, and remembering to start
`serve`. Setup is a third: the hook config lives in gitignored local settings, so a fresh clone has no hooks
and nothing says so. Whether those sit inside the MVP boundary is an open scope question; see the roadmap.

Rust rewrite of a TypeScript prototype, kept under `openspec-doc-rs-example/` for reference.
`AGENTS.md` holds the working rules for this repo.
