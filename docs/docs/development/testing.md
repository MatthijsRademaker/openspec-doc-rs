# Testing

```bash
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --all --check
```

## Current state

| Crate | Tests | Status |
|---|---|---|
| `openspec-doc-core` | 123 | pass |
| `openspec-doc-cli` | 22 | pass |
| `openspec-doc-server` | 33 | 30 pass, **3 fail** |

The three failures are all in `watch.rs` — the filesystem-watcher tests. They fail on a stashed tree too,
so they are pre-existing and not caused by recent work. They look like timing or platform sensitivity in
the tests rather than evidence about the watcher, but nobody has proven that. Do not let them mask a real
regression: check `-p openspec-doc-core -p openspec-doc-cli` for a clean signal.

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
