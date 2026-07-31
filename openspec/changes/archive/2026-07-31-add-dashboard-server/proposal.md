## Why

The dashboard needs a running local process serving pages and pushing live updates. Building the HTTP/SSE server shell — routing over discovered sessions and changes, with live-update plumbing already proven — before any real page content exists lets `add-dashboard-html-views` focus purely on markup and lets `add-directive-verdict-loop` focus purely on wiring writes, instead of every later change also having to stand up a server.

## What Changes

- Fills the `serve` placeholder subcommand from `add-cli-surface`.
- New HTTP server (`axum` + `tokio`) bound to `127.0.0.1` (or `--host`), selecting an available port when `--port` is omitted or `0`, printing the resolved URL, and opening a browser unless `--no-open` is passed — mirroring the TS example's `serve` UX.
- Route table reflecting the two scoping regimes already settled during exploration: `/sessions/<session_id>` for pre-proposal explore tabs, `/changes/<name>` for proposal/apply tabs.
- Live-update plumbing: a filesystem watcher feeding a per-route SSE endpoint, so a page updates without a manual reload.
- Placeholder page content only — no comment UI, no markdown rendering, no directive wiring. Those belong to `add-dashboard-html-views` and `add-directive-verdict-loop`.

## Capabilities

### New Capabilities
- `dashboard-server`: the `serve` subcommand's HTTP server, session/change routing, and SSE live-update mechanism.

### Modified Capabilities
(none)

## Impact

New `server` module/crate depending on `core`, new dependencies (`axum`, `tokio`, `notify`). Fills the `serve` subcommand in the `cli` crate.
