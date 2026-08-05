# openspec-doc

A local review dashboard for [OpenSpec](https://github.com/Fission-AI/OpenSpec) projects, plus an agent
hook bridge.

You read a coding agent's exploration or proposal in a browser **while it is happening**, select text,
leave anchored comments, and submit a phase verdict. The hook bridge feeds that verdict back to the agent
at its next turn boundary, as a directive it acts on.

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
   agent acts  ◄── hook stop blocks ◄── directive ◄── verdicts/…
        │
        └──► creates the change, runs `scratch claim` ──► note promoted
```

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
```

`core` holds every rule and every file format; `server` and `cli` are two front ends over it with no
persistence logic of their own.

## Build and test

```bash
cargo test -p openspec-doc-core -p openspec-doc-cli    # 145 tests, hermetic
cargo clippy --workspace --all-targets
cargo fmt --all --check
```

Three `watch.rs` tests in `openspec-doc-server` fail on a clean tree — pre-existing, see
`docs/docs/development/testing.md`.

## Status

The loop is closed and verified on live sessions across Claude Code and pi.dev, including the one thing no
test can establish: that an agent acts on an injected directive rather than refusing it as prompt
injection.

Two things still need a terminal or insider knowledge — closing out a comment, and remembering to start
`serve`. Whether those sit inside the MVP boundary is an open scope question; see the roadmap.

Rust rewrite of a TypeScript prototype, kept under `openspec-doc-rs-example/` for reference.
`AGENTS.md` holds the working rules for this repo.
