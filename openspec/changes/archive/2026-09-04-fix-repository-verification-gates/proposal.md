## Why

The repository's own gates do not tell the truth about the repository.

`cargo test --workspace` fails three `watch::` tests on macOS against a clean tree. The cause is not the watcher: `TempDir` resolves under `/var/folders/…`, macOS reports filesystem events at the canonical `/private/var/…`, and `classify` matches with a literal `path.starts_with`, so every event is discarded as irrelevant to the scope. Pointing `TMPDIR` at a non-symlinked directory turns all nine watch tests green. The three tests carry a `macOS follow-up (2026-08-07)` comment that blames the `notify` backend for delivering no events and instructs the reader to reproduce on Linux or CI before touching watcher code — a wrong diagnosis that has deterred a one-line fix for a month, and an instruction that cannot be followed because no CI lane runs the Rust suite. The product is not affected: `crates/cli/src/serve.rs` canonicalizes the project root before the watcher ever sees it.

Separately, `docs/` is a complete rspress site with its own lockfile and toolchain that no gate builds. The Makefile does not know it exists. It compiles today; nothing would report it if it stopped.

## What Changes

- Make the watcher tests independent of the shape of the developer's temp directory, so `cargo test --workspace` passes on macOS and Linux alike rather than depending on where `TMPDIR` points.
- Delete the three stale `macOS follow-up (2026-08-07)` comments. Their diagnosis is wrong and their instruction is unfollowable; a fixed test needs neither.
- Correct the three places in `docs/` that record these failures as having vanished unexplained. Landing a diagnosis while the documentation says the cause is unknown leaves the repository arguing with itself, and the note is the exact debris the new requirement forbids.
- Bring the documentation site under an executable gate, reachable from the Makefile alongside the Rust and frontend gates.
- Record the repository's gates as a capability: which surfaces are gated, which entry point runs them, and that a gate's result must depend on the repository rather than on the machine running it.

Out of scope: whether these gates run in continuous integration. Only `frontend-assets.yml` exists today and the Rust suite runs nowhere but a developer's machine, but that gap is being worked in a separate change, and both changes editing `.github/workflows/` would collide.

## Capabilities

### New Capabilities

- `repository-verification-gates`: the repository's executable verification gates — the surfaces they must cover, the entry point that runs them, and their independence from the environment of the machine running them.

### Modified Capabilities

None. The watcher fix is confined to test code, and no existing requirement describes the gates being extended.

## Impact

- `crates/server/src/watch.rs` — test module only; product code unchanged.
- `Makefile` — the documentation site joins the `check` target.
- `docs/` — gated, and the stale watcher-failure notes in `docs/docs/development/testing.md`, `docs/docs/roadmap.md` and the section index in `docs/rspress.config.ts` corrected to the actual cause.
- Developers on macOS, for whom `make test` currently fails on a clean checkout.
