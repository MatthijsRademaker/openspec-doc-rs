## Why

The browser viewer currently builds its OpenSpec model once at server startup, so users must restart `openspec-doc serve` after changing specs, proposals, tasks, or review feedback files. This slows down Pi-assisted proposal review and makes the browser view feel stale during normal OpenSpec iteration.

## What Changes

- Add an opt-in viewer refresh/watch capability for `openspec-doc serve` so rendered pages can reflect OpenSpec file changes without restarting the server.
- Ensure refreshed pages rebuild the same scanner/parser/analyzer-derived model used by `summary`, `check`, and the initial viewer render.
- Provide a browser update behavior for users, such as automatic reload or clear stale-state handling, when watched OpenSpec content changes.
- Preserve default read-only behavior for target OpenSpec artifacts; hot reload must not edit files under `openspec/`.
- Keep review companion sidecar feedback behavior scoped to documented `.pi/openspec-review/comments/*.jsonl` records.

## Capabilities

### New Capabilities

### Modified Capabilities
- `browser-viewer`: Add serve/watch behavior so browser pages can reflect OpenSpec changes while the viewer process remains running.

## Impact

- Affected code: `src/cli/commands/serve.ts`, `src/server/dev-server.ts`, `src/viewer/model.ts`, and possibly `src/viewer/render.ts` if client-side reload support is added.
- Affected tests: server/viewer tests should cover model refresh behavior and read-only guarantees.
- CLI/API impact: likely adds a `serve` option such as `--watch` or an equivalent documented refresh mode.
- Dependencies: may use Node.js filesystem watching APIs; avoid adding persistent dependencies unless implementation proves they are needed.
