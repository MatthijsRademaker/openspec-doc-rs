## Why

A usable CLI needs a stable command surface — subcommands, `--help` text, and `--root` discovery — established before subcommand logic (`serve`, `hook`, `comment`) exists. Fixing the surface early means later MVP changes fill in one subcommand's behavior at a time without ever renegotiating the CLI's shape.

## What Changes

- New `openspec-doc` binary crate depending on the `core` crate from `add-rust-foundation`.
- `clap`-derived CLI with a global `--root <path>` option (defaults to cwd, resolved via `core`'s project-root discovery).
- Subcommand scaffolding for `summary`, `serve`, `hook`, `comment`, each documented in `--help` output.
- `summary` is fully implemented in this change: prints resolved root, active changes, archived changes, current specs.
- `serve`, `hook`, `comment` are present in the surface but exit non-zero with a "not yet implemented" message — they fail loudly rather than silently no-op, since their real behavior belongs to later changes.

## Capabilities

### New Capabilities
- `cli-surface`: the `openspec-doc` binary's command surface, help text, root resolution, and the `summary` subcommand.

### Modified Capabilities
(none)

## Impact

New binary crate (`cli`) in the workspace, depending on `core`. No existing code affected.
