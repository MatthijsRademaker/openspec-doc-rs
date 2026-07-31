## 1. Server crate setup

- [x] 1.1 Create the `server` module/crate depending on `core`; add `axum`, `tokio`, `notify` dependencies
- [x] 1.2 Implement `serve` subcommand argument handling (`--host`, `--port`, `--no-open`), filling the placeholder from `add-cli-surface`

## 2. Startup and routing

- [x] 2.1 Implement port selection (explicit port, or first available when omitted/0) and bind to the resolved host
- [x] 2.2 Print the resolved local URL on successful startup
- [x] 2.3 Implement browser-open behavior, honoring `--no-open`
- [x] 2.4 Implement `/sessions/<session_id>` and `/changes/<name>` routes serving placeholder pages, backed by `core` discovery
- [x] 2.5 Implement 404 handling for unknown session ids or change names

## 3. Live updates

- [x] 3.1 Add a filesystem watcher on relevant sidecar/change paths per route scope
- [x] 3.2 Implement an SSE endpoint per route, pushing an event on watched-file change
- [x] 3.3 Implement the bounded-polling fallback if the watcher backend fails to initialize
- [x] 3.4 Add a test or manual check confirming a file change is observed as a pushed SSE event without client polling

## 4. Manual verification

- [x] 4.1 Run `openspec-doc serve` against this repo; confirm the URL prints and the browser opens/doesn't per `--no-open`
- [x] 4.2 Touch a file under a discovered change directory and confirm the corresponding route's SSE stream emits an event
