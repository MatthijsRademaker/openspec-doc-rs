## Context

Two gates are broken in opposite ways. One runs and lies; one does not run at all.

`crates/server/src/watch.rs` tests the filesystem watcher by writing into a `TempDir` and asserting an update arrives. On macOS `std::env::temp_dir()` resolves to `/var/folders/…`, and `/var` is a symlink to `/private/var`. The watcher is handed the symlinked path, the FSEvents backend reports the canonical one, and `classify` matches with `path.starts_with(relevant)` — a component-wise prefix comparison that does no resolution. Nothing matches, no update is ever published, and three tests time out. All nine pass with `TMPDIR` pointed at a real directory.

The product is not exposed to this. `crates/cli/src/serve.rs` canonicalizes the project root before constructing any `Target`, so the paths the watcher holds and the paths the backend reports are already the same shape. The failure is a property of the fixture, not of the code under test.

Three of the tests carry a `macOS follow-up (2026-08-07)` comment attributing the timeout to `notify` delivering no events for temporary directories, and directing the reader to reproduce on Linux or CI before changing watcher code. Both halves are wrong: the backend delivers the events, and no CI lane runs the Rust suite to reproduce anything on.

`docs/` is an rspress site with its own `bun.lock`, its own dependency tree, and eight built pages. The Makefile's `build`, `test` and `check` targets do not mention it.

## Goals / Non-Goals

**Goals:**

- `cargo test --workspace` passes on macOS and Linux from a clean checkout, without the developer configuring anything.
- The documentation site is built by a gate a developer already runs.
- The repository states which surfaces are gated, so the next ungated surface is a visible omission rather than an unasked question.

**Non-Goals:**

- Continuous integration. Only `frontend-assets.yml` exists and the Rust suite runs on no machine but a developer's, but that is a separate change already in flight; two changes editing `.github/workflows/` would collide.
- Any change to watcher behaviour. `classify`, `watcher` and the polling fallback are correct as written and are not touched.
- Gating `openspec-doc-rs-example/`, `designs/` or `scripts/`.

## Decisions

### Fix the fixture, not the watcher

The tests canonicalize their temporary root before handing it to `Target`, through one helper the watcher-driven tests share. Subpaths are joined onto the canonical root rather than canonicalized themselves, because `a_sidecar_directory_that_does_not_exist_yet_is_still_watched` deliberately names a directory that does not exist yet and `canonicalize` on a missing path fails.

The alternative is to canonicalize inside `watcher()` or `Target`. Rejected: `serve.rs` already does it at the only place a `Target` is built from user input, so a second resolution guards a case that cannot arise, and it would put a fallible I/O call into a constructor that currently cannot fail. It also inverts the diagnosis — it would make the product absorb a defect that lives in the test.

The trade-off is accepted with eyes open: if `serve.rs` ever stops canonicalizing, no watch test would notice, because the tests now hand the watcher paths that are canonical by construction. That regression would surface as a live dashboard that stops updating, and only for a project reached through a symlink.

### Delete the stale comments rather than correct them

Once the fixture is canonical the tests do not fail on macOS, so there is no follow-up to describe. A corrected comment explaining a symlink problem the test no longer has is the same kind of debris that made this take a month to diagnose. The reasoning that survives — why the tests canonicalize — belongs on the helper that does it, in one place, not repeated at three call sites.

### The documentation site joins `make check`, not a target of its own

`make check` is the gate a developer runs before committing. A `make docs` target that nobody runs is not a gate; it is a note. Installation uses `bun install --frozen-lockfile`, matching how `web/` is installed, so a drifted `docs/bun.lock` fails the gate rather than being silently resolved.

`check` gains a bun install and an rspress build. That is the cost of the site being verified at all, and it is paid by the person who can act on the failure.

### One capability, describing the gates rather than restating them

`repository-verification-gates` names the surfaces that must be gated, the entry point that reaches them, and the rule that a gate's verdict must come from the repository rather than the machine. It does not restate what the frontend gate checks — `dashboard-development-harness` owns that, and duplicating it would produce two requirements to keep in step.

## Risks / Trade-offs

- **A canonical-by-construction fixture can no longer catch a product path that stops being canonical** → Accepted, and recorded in the decision above. The `serve.rs` canonicalization is the load-bearing line; if it moves, this trade-off should be revisited.
- **`make check` gets slower and needs a second `bun install`** → The install is frozen and cached between runs; the site is small and builds in seconds. A gate that is slightly slower beats a surface nobody verifies.
- **Naming gated surfaces in a spec means the list can go stale as the repository grows** → That is the point: an ungated surface becomes a spec omission someone can see, instead of a question nobody thought to ask.
