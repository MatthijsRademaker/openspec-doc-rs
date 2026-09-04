# Testing

```bash
make test    # cargo test --workspace, building web/dist first if it is absent
make check   # cargo clippy --workspace --all-targets, cargo fmt --all --check, the frontend gate, the docs build
```

Reach the gates through the Makefile rather than the `cargo` lines underneath them. The automated run
invokes the same two targets, so a command added to a target is a command the automation starts
running with no separate edit — which is the property that keeps "checked locally" and "checked in CI"
from becoming two different claims.

The compiler is pinned in `rust-toolchain.toml`, so both paths use the same version.

## Current state

| Crate | Tests | Status |
|---|---|---|
| `openspec-doc-core` | 201 | pass |
| `openspec-doc-cli` | 137 | pass |
| `openspec-doc-server` | 63 | pass |

All 401 pass. `openspec-doc-cli`'s figure is 77 unit tests beside the code plus the 60 in
`tests/cli.rs` that drive the built binary.

:::note The three `watch.rs` failures were a symlinked temp directory
This table recorded three failing filesystem-watcher tests for weeks, first as timing or platform
sensitivity, then as having vanished unexplained. Both readings were wrong, and the second was wrong
because the tests fail only where `TMPDIR` points through a symlink — which is the macOS default and
not a Linux one.

`std::env::temp_dir()` resolves under `/var/folders/…`, `/var` is a symlink to `/private/var`, FSEvents
reports the canonical path, and `classify` compares with `path.starts_with` — a literal prefix match that
resolves nothing. Every event was discarded as outside the scope. The tests now canonicalize their
temporary root before handing it to a `Target`, and pass on either platform with nothing configured.

The product was never exposed: `crates/cli/src/serve.rs` canonicalizes the project root before any
`Target` is built. The lesson is the one the note itself is evidence for — an annotation deferring a
diagnosis to a lane that does not run costs more than the bug.
:::

## What the automated run covers

`.github/workflows/rust.yml` runs `make check` and `make test` on every push to `main` and every pull
request, on **Linux (`ubuntu-latest`) and macOS (`macos-latest`)**. One job builds `web/dist` and both
platform legs download that single artifact, so each embeds the same frontend bytes rather than one it
built itself; a leg whose artifact is missing fails instead of compiling.

**Windows is not covered, and it is an omission rather than an oversight.** `doctor`'s binary
resolution and its hook-registration probe are both broken there. Three tests cover exactly that code
and are `#[cfg(unix)]`-gated, so a Windows leg would run the other 398 and report success:

- `doctor_passes_a_wired_project_and_fails_naming_a_hook_that_is_not_registered`
- `doctor_leaves_a_pending_directive_and_the_projects_own_review_state_alone`
- `hook_stop_starts_a_dashboard_that_outlives_it_and_serve_url_finds_it`

A green Windows badge over those three is a stronger false claim than no badge at all, which is why
`replace-hook-shell-form-with-exec-form` fixes the defects, un-gates the tests, and adds the leg in one
change: supported and verified have to move together.

`.github/workflows/frontend-assets.yml` remains separate and uncached, because what it proves is that a
clean checkout builds the frontend, embeds it, and serves it in a real browser. The two lanes overlap on
`bun run check`, which `make check` also runs; that duplication costs about a minute per run and is
cheaper than rewriting the only automation this repository had.

## Hermetic by default

Every test builds its own temp project. No test needs a network or a browser.

One exception: `scratch::promote`'s validation tests shell out to `openspec validate` and will fail if
`openspec` is not on `PATH`.

## Layout

Unit tests live beside the code in `#[cfg(test)]` modules. `crates/cli/tests/cli.rs` drives the built
binary end to end via `CARGO_BIN_EXE_openspec-doc`, including piping real hook payloads on stdin.

Hook payload fixtures in `crates/core/src/hook/adapter.rs` are **captured from real agent runs**, with only
session ids and paths rewritten. Keep every field, including the ones we do not read — that is the
regression test that an added upstream field does not become a parse error.

## What tests here cannot establish

This matters more than usual in this project, and it is why
[Manual verification](/development/manual-verification.md) exists as a first-class document rather than a
nicety.

**Whether an agent complies with an injected directive.** Every assertion about delivery can pass while the
loop fails, because the agent receives the text and refuses it. That happened once. Tests assert templates
are not *shaped* like the refused one; only a live run shows compliance. See
[Pointer, not embed](/concepts/pointer-not-embed.md).

**Whether a hook matcher fires.** A wrong matcher produces no error — the hook simply never runs. Unit
tests exercise the command; nothing exercises the wiring.

**Whether the instructions we emit are true.** Nothing asserts that the text handed to an agent describes
the mechanism the code actually has. The explore instruction once told agents that promotion happens "once
a change directory exists" for a while after that mechanism was deleted. Caught by a live run, not a test.

**Concurrency across sessions.** Two of the three defects that motivated claim-based promotion were found
by inspecting live state, not by any test. The tests added afterwards are regressions, not the thing that
found them.

The pattern: **the interesting failures here are in the seams between the tool and the agent**, and those
seams have no unit-test surface.
