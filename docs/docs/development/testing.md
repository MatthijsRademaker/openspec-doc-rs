# Testing

```bash
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --all --check
```

## Current state

| Crate | Tests | Status |
|---|---|---|
| `openspec-doc-core` | 124 | pass |
| `openspec-doc-cli` | 33 | pass |
| `openspec-doc-server` | 33 | pass |

All 190 pass.

:::note The three `watch.rs` failures are gone, and nobody knows why
This table previously recorded three failing filesystem-watcher tests, described as pre-existing and
probably timing or platform sensitivity. They pass now — checked repeatedly on a clean tree.

Nothing was done to fix them, so treat this as unexplained rather than resolved. The most likely reading is
that they were always environmental, which would mean the "pre-existing, not caused by recent work" note
was doing real harm: it stood for weeks as a known defect and discouraged anyone from looking. If they
return, that is a timing bug worth chasing, not a familiar nuisance.
:::

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
