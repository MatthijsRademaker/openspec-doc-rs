## 1. CLI and Server Options

- [x] 1.1 Add a `--watch` option to the `serve` command and pass it into the viewer server options.
- [x] 1.2 Update serve startup output to indicate when watch mode is enabled.
- [x] 1.3 Add command-level tests or smoke coverage for watch option parsing where existing test structure supports it.

## 2. Watch Model Refresh

- [x] 2.1 Refactor the server to read the current `ViewerModel` from mutable server state instead of closing over a startup-only model.
- [x] 2.2 Implement debounced filesystem watching for the target `openspec/` directory when watch mode is enabled.
- [x] 2.3 Include review companion sidecar feedback files in the watch scope when both watch mode and review companion mode are enabled.
- [x] 2.4 Rebuild the viewer model through `buildViewerModel()` after watched changes and retain the last successful model if rebuild fails.
- [x] 2.5 Surface refresh failures through server logging and/or rendered viewer state without crashing the running server.

## 3. Browser Refresh Behavior

- [x] 3.1 Add a lightweight model-version endpoint, polling script, or server-sent events stream that is only active for watch-mode pages.
- [x] 3.2 Ensure open browser pages reload or clearly update after a successful watched model refresh.
- [x] 3.3 Ensure non-watch pages do not include watch-mode background refresh behavior.

## 4. Validation

- [x] 4.1 Add tests for model refresh after spec/change artifact updates.
- [x] 4.2 Add tests proving failed refreshes keep serving the last successful model.
- [x] 4.3 Add or update tests for read-only behavior under watch mode.
- [x] 4.4 Run `npm run validate`.
- [x] 4.5 Run `openspec validate add-viewer-hot-reload`.
