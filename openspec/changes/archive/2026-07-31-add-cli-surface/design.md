## Context

`add-rust-foundation` provides root discovery and change/spec enumeration in a `core` library crate. This change wraps it in the actual `openspec-doc` binary that end users and hooks invoke, establishing argument-parsing conventions before `serve`, `hook`, and `comment` have real logic.

## Goals / Non-Goals

**Goals:**
- Establish the full MVP command surface (all subcommands present) with accurate `--help` from this change onward, so later changes fill in one subcommand's behavior without changing the CLI's public shape.
- Root resolution failures produce a clear, actionable CLI error — never a panic or stack trace.
- Placeholder subcommands fail loudly (non-zero exit, explicit "not yet implemented" message) rather than silently no-op.

**Non-Goals:**
- No real `serve`, `hook`, or `comment` behavior — those belong to `add-dashboard-server`, `add-agent-hook-bridge`, and `add-anchored-comments`.
- No JSON output modes for `summary` yet.

## Decisions

- **Use `clap` with derive macros** for the CLI surface — mature, standard in the Rust ecosystem, generates `--help` and error messages without hand-rolled parsing. Alternative considered: hand-written arg parsing — rejected as reinventing help/error generation for no benefit.
- **Global `--root` lives on the top-level `Cli` struct**, threaded to every subcommand, rather than each subcommand redefining it — keeps root-resolution logic (calling into `core`) in one place.
- **Placeholder subcommands are real `clap` enum variants**, not omitted from the surface — `--help` accurately reflects the target MVP surface from this change onward, and later changes only fill in match-arm bodies without touching the CLI's shape or its tests.

## Risks / Trade-offs

- [Risk] Committing to a command surface this early risks having to break it once hook/comment/server requirements are better understood → Mitigation: keep subcommand argument lists minimal in this change (root option only); add subcommand-specific flags in the change that implements each one.
- [Risk] "Not yet implemented" exits could be mistaken for real bugs by a user running the tool between MVP changes → Mitigation: the placeholder message explicitly names which change enables the subcommand.
