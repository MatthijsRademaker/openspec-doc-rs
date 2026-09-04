## 1. Watcher tests stop depending on the temp directory's shape

- [x] 1.1 Add a test helper in `crates/server/src/watch.rs` that creates a `TempDir` and returns it alongside its canonicalized path, with the reason for canonicalizing stated once on that helper
- [x] 1.2 Route every watcher-driven test through the helper, joining subpaths onto the canonical root rather than canonicalizing subpaths that do not exist yet
- [x] 1.3 Delete the three `macOS follow-up (2026-08-07)` comments
- [x] 1.4 Confirm `cargo test -p openspec-doc-server --lib watch::` passes on macOS with `TMPDIR` unset and left at its default
- [x] 1.5 Confirm the same suite still fails when the assertion it makes is genuinely broken, by temporarily inverting one `assert!` — the tests must not have been made to pass by no longer testing anything

## 2. The documentation site joins the check gate

- [x] 2.1 Add the documentation site's frozen install and build to the Makefile `check` target
- [x] 2.2 Confirm `make check` fails when `docs/` fails to build, by temporarily breaking a page reference
- [x] 2.3 Confirm `make check` fails when `docs/bun.lock` no longer matches `docs/package.json`

## 3. The documentation stops calling the failures unexplained

- [x] 3.1 Replace the "nobody knows why" note in `docs/docs/development/testing.md` with the symlinked-temp-directory diagnosis
- [x] 3.2 Correct the `watch.rs` row in `docs/docs/roadmap.md`'s known-defects table from "gone, unexplained" to fixed, with the cause
- [x] 3.3 Update the testing page's description in `docs/rspress.config.ts`, which still advertises known-failing watcher tests

## 4. Verification

- [x] 4.1 Run `make test` from a clean checkout on macOS and confirm it is green with no environment configuration
- [x] 4.2 Run `make check` and confirm every gate passes, including the new documentation lane
- [x] 4.3 Confirm no `TODO`, commented-out command, or stale platform note is left behind in the files touched
