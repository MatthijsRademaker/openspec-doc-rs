## Context

`add-cli-surface` leaves `serve` as a non-zero-exit placeholder. `add-dashboard-html-views` and `add-directive-verdict-loop` both need an actual running server with routing and live-update plumbing already in place. This change builds the server shell first, decoupled from page content, so those changes only add templates and write-wiring respectively.

## Goals / Non-Goals

**Goals:**
- `openspec-doc serve` starts a local HTTP server bound to `127.0.0.1`, selects an available port when unspecified, and prints the resolved URL, matching the TS example's `serve` UX (`--host`, `--port`, `--no-open`).
- The route table reflects the two scoping regimes already settled: session-keyed routes for pre-proposal explore tabs, change-name-keyed routes for proposal/apply tabs.
- A live-update mechanism (SSE) exists and is exercised by this change's own placeholder pages, so later changes wire real content into an already-proven push mechanism.

**Non-Goals:**
- No comment UI, no markdown-to-HTML rendering of real artifacts, no anchor engine — placeholder pages only.
- No directive read/write wiring (`add-directive-verdict-loop`).

## Decisions

- **`axum` + `tokio`** for the HTTP/SSE server — `axum`'s SSE response type and routing fit "small number of routes plus one push channel per route" directly, and it's a mainstream, well-maintained choice rather than a bespoke HTTP implementation. Alternative considered: a minimal blocking server (e.g. `tiny_http`) — rejected because SSE over a blocking single-threaded server is awkward; async fits holding open connections for live push.
- **One broadcast channel per watched scope (session or change), fed by a filesystem watcher** (`notify` crate), rather than polling — keeps "live updates" genuinely event-driven rather than a refresh-on-poll approximation.
- **Route keys mirror the two scoping regimes decided during exploration**: `/sessions/<session_id>` for pre-proposal, `/changes/<name>` for proposal/apply — no third generic key, keeping the URL space small and predictable.

## Risks / Trade-offs

- [Risk] Introducing `axum` + `tokio` pulls in a nontrivial async dependency tree for what is currently a placeholder server → Mitigation: the dependency is justified up front by the SSE requirement every later dashboard change needs; deferring it would mean re-deciding this under time pressure once real content exists.
- [Risk] Filesystem watching can be flaky or platform-dependent (inotify limits, WSL) → Mitigation: fall back to a bounded polling interval if the watcher backend fails to initialize, logged clearly rather than silently degrading to no updates at all.
