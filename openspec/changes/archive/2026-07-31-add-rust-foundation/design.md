## Context

This is the first Rust code in the repository, which is being rewritten from the TypeScript `openspec-doc-rs-example` (see `openspec-doc-rs-example/src/core/scanner.ts`, `scanner-discovery.ts`, `file-system.ts`, `path-utils.ts`). The TS scanner already establishes the filesystem discovery pattern this design ports: find `openspec/config.yaml` by walking up from a starting directory, then enumerate changes/specs/archive. This change ports that discovery pattern only — not the markdown parsing layered on top of it in the TS example.

## Goals / Non-Goals

**Goals:**
- Establish a Cargo workspace layout that later changes (CLI surface, hook bridge, dashboard server, comments) can add crates to without restructuring.
- Provide a typed scan result (project root, active changes, archived changes, current specs) usable by both a future CLI binary and a future hook binary.
- Match the TS example's root-resolution semantics (explicit override, walk-up search, cwd default) so behavior is a drop-in mental model for anyone who used the TS tool.

**Non-Goals:**
- No markdown parsing of proposal/design/tasks/spec content.
- No CLI argument parsing (`add-cli-surface`).
- No network/server code.
- No hook or comment logic.

## Decisions

- **Single Cargo workspace with one library crate** (`core`) rather than a monolithic binary crate. The future CLI crate and hook-bridge crate both need this scanning logic; a shared lib crate is the natural boundary and avoids duplicating discovery logic across binaries. Alternative considered: a single binary crate with internal modules — rejected because `add-cli-surface` and `add-agent-hook-bridge` are separate proposals producing separate binaries (or binary targets) that both depend on this logic.
- **Root discovery walks up from a starting directory** looking for `openspec/config.yaml`, capped at the filesystem root, mirroring the TS example's behavior. Alternative considered: always require an explicit root — rejected, it breaks the "cd into project and run" zero-config UX the TS README establishes as a goal.
- **Change enumeration excludes a literal `archive` entry** from the active list and separately walks `openspec/changes/archive/**` for archived changes, matching TS `scanner.ts` exactly — this keeps behavior consistent with the tool this rewrite replaces.

## Risks / Trade-offs

- [Risk] Filesystem walk-up may behave inconsistently across platforms (symlinks, case sensitivity) → Mitigation: canonicalize paths before comparison, stop the walk at the filesystem root, and cover walk-up edge cases with fixture-based tests using temp directories.
- [Risk] Shipping "just scanning" with no CLI leaves no way to manually exercise the crate → Mitigation: include a throwaway debug binary/example for manual verification only; it is not the public CLI surface.
