## Why

The Rust rewrite of `openspec-doc` needs a shared foundation — project-root discovery and an OpenSpec directory scanner — before any CLI, hook bridge, server, or comment logic can exist. Every later MVP change depends on this data model; building it standalone first avoids re-deriving filesystem discovery logic in each subsequent change.

## What Changes

- New Rust Cargo workspace at the repo root.
- New `core` library crate providing project-root discovery: locate `openspec/config.yaml` from an explicit root or by walking up from a starting directory.
- Scan model enumerating active change directories (`openspec/changes/*`, excluding `archive/`), archived change directories (`openspec/changes/archive/**`), and current spec directories (`openspec/specs/*`).
- Explicitly excludes markdown content parsing (Purpose/Requirement/Scenario extraction), CLI argument parsing, and any network/server code — those are scoped to later changes.

## Capabilities

### New Capabilities
- `project-scanner`: locating the OpenSpec project root and enumerating active changes, archived changes, and current specs by directory structure alone.

### Modified Capabilities
(none — greenfield)

## Impact

New Rust workspace (`Cargo.toml`, `crates/core`) at the repo root. No existing code is affected.
