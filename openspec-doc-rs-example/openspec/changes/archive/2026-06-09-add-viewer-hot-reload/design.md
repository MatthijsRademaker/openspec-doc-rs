## Context

`openspec-doc serve` currently calls `buildViewerModel()` once when the HTTP server starts and passes that immutable model to every route render and feedback validation path. This keeps server behavior simple, but it means edits made by Pi or the user during an OpenSpec review session are invisible until the process is restarted.

The viewer is server-rendered HTML, so hot reload can be implemented without introducing a frontend bundler. The core requirement is to refresh the server-side OpenSpec model when relevant files change and give browser pages a way to update.

## Goals / Non-Goals

**Goals:**
- Provide an explicit watch/hot-reload mode for `openspec-doc serve`.
- Rebuild the viewer model after relevant OpenSpec files change while the server process remains running.
- Let browser pages reflect the rebuilt model without requiring a server restart.
- Keep default serve behavior read-only and safe for target OpenSpec artifacts.
- Keep implementation small and aligned with the existing scanner/parser/analyzer/viewer layers.

**Non-Goals:**
- Replacing the server-rendered UI with a client-side app or bundler.
- Editing OpenSpec artifacts from the browser.
- Watching arbitrary project source files outside OpenSpec and review sidecar paths.
- Guaranteeing millisecond-level HMR for CSS or frontend modules.

## Decisions

### Add explicit watch mode

Add a documented serve option such as `--watch` instead of changing the default server semantics. This avoids surprising users who expect the initial read-only viewer to be stable and keeps the feature opt-in while it matures.

Alternative considered: always rebuild on every request. That is simpler and always fresh, but it can make every page load pay the scanner/parser/analyzer cost and may hide performance problems. Explicit watch mode provides a clearer contract.

### Rebuild the server-side viewer model with debounced filesystem events

In watch mode, the server should watch the target project's `openspec/` directory and, when review companion mode is enabled, the `.pi/openspec-review/comments/` sidecar directory. Changes should trigger a debounced rebuild of the `ViewerModel` using the existing `buildViewerModel()` function. If a rebuild fails, the server should retain the last good model and expose/log the latest refresh error rather than crashing.

Alternative considered: mutate individual model sections incrementally. That adds complexity and risks divergence from the existing scanner/parser/analyzer behavior. Full rebuild keeps the implementation correct and consistent.

### Use lightweight browser refresh signaling

Because pages are server-rendered, the browser can update by reloading the current page when the server reports that the model version changed. A small built-in endpoint, server-sent events stream, or minimal polling endpoint can expose a monotonically increasing model version. The first implementation should prefer the smallest reliable approach.

Alternative considered: introduce a frontend dev server with HMR. That is unnecessary for the current HTML renderer and would add tooling complexity before the UI requires it.

### Preserve read-only guarantees

Watching files and reloading pages must not create, modify, or delete files under `openspec/`. Review companion feedback writes remain governed by the existing explicit `--review-companion` mode and sidecar write validation.

## Risks / Trade-offs

- **Filesystem watcher portability**: Node watcher behavior varies across platforms. Mitigation: keep watch scope narrow, debounce events, and allow manual browser refresh/server restart as fallback.
- **Transient parse errors during edits**: Files may be observed while partially written. Mitigation: debounce rebuilds and keep the last successful model if a rebuild fails.
- **Performance on large OpenSpec trees**: Full model rebuilds may be expensive. Mitigation: start with debounced full rebuilds and add incremental indexing only if profiling justifies it.
- **Browser reload disrupting review**: Auto-reload may reset scroll/form state. Mitigation: keep reload behavior limited to watch mode and avoid auto-submitting or mutating feedback forms.
