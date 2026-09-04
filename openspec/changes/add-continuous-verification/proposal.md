## Why

The repository's gates run nowhere but a developer's machine, and one of them does not run at all.

`.github/workflows/frontend-assets.yml` is the only workflow. It installs Bun, runs `bun run check`, builds `web/dist`, drives the embedded application in Chromium, and checks the mirrored harness resources. It never runs `cargo fmt`, `cargo clippy`, or `cargo test`. So across forty-two commits, no automation has compiled this workspace — the frontend lane compiles the Rust binary only as a step on the way to launching a browser at it, on one platform, without running a single test.

`fix-repository-verification-gates` is making the gates tell the truth about the repository. This change makes them run without being asked. Those are different failures with the same shape: a gate whose result nobody sees is worth what a gate that reports the wrong result is worth, which is nothing. That change says so directly — *"an instruction that cannot be followed because no CI lane runs the Rust suite"* — and defers the lane to here rather than editing `.github/workflows/` twice.

The lane is also what every later piece of release work stands on. A pipeline that publishes binaries for five targets out of a repository where no automation has ever run `cargo test` is a faster way to ship whatever is on `main`, not a better one.

## What Changes

- Add a Rust lane on push to `main` and on pull requests: format, lint, and the workspace test suite, on Linux and macOS.
- **Run the Makefile's gates rather than a second copy of their command list.** `fix-repository-verification-gates` establishes the Makefile as the entry point a developer reaches a surface's gate through; a CI lane that spells the commands out again is a second definition, and it drifts in one of two directions — green locally and red in CI, or the reverse, which is worse.
- Handle the `web/dist` prerequisite once, in a job of its own, and hand the built artifact to each platform leg. The workspace does not compile without it, so every leg needs one, and building it per-leg would produce a separate frontend per platform. That matters far more in the release matrix than it does here, which is the argument for proving the shape here first, where the blast radius is a red lane rather than a shipped binary.
- Pin the compiler. There is no `rust-toolchain.toml`, so "it built last week" is currently a complete explanation for a build that has stopped working.
- Record in the capability which platforms the lane covers, so a platform that is *not* covered is a stated omission rather than something inferred by reading a workflow file.

Deliberately **not** in scope:

- **Windows.** The workspace has two known Windows defects in `doctor` and three test helpers that only compile on Unix. A Windows leg here is either red, or green because it skipped the tests covering the broken code — the second being exactly the green-line-for-something-never-examined failure this repository names as the thing its diagnostics exist to remove. `replace-hook-shell-form-with-exec-form` fixes the defects and adds the leg in the same change, because "supported" and "verified" have to move together.
- **Cross-compilation, release artifacts, and installers.** `add-release-binaries-and-installers` owns those. This lane builds for the host it runs on.
- **Merging with `frontend-assets.yml`.** It works, its stated value is being an uncached clean build, and rewriting a working lane to save one Bun install is not worth the diff.
- Coverage measurement, an MSRV policy beyond the pin, and branch protection, which is a repository setting rather than a file in it.

## Capabilities

### Modified Capabilities

- `repository-verification-gates`: the capability says which surfaces must be gated and that a gate's verdict must come from the repository rather than the machine. It says nothing about *when* the gates run, which leaves "every gate passes on the maintainer's laptop, on the commits the maintainer remembered to check" as a conforming implementation. This adds the requirement that they run on every change, and that the platforms they run on are named rather than inferred.

## Impact

- `.github/workflows/rust.yml` — new.
- `rust-toolchain.toml` — new, at the repository root.
- `Makefile` — the gates become an interface once CI invokes them. Expect to want `check` and `test` reachable without `build` rebuilding a frontend the job already downloaded.
- `docs/docs/development/testing.md`, `README.md` — both describe running the suite by hand as the only way it is run.
- `docs/docs/roadmap.md` — the "Known defects and debt" table's *"No release; setup is lossy"* row rests partly on there being no lane; the half this change closes moves out of it.

**Depends on `fix-repository-verification-gates`** and must be sequenced after it. That change makes `cargo test --workspace` pass on macOS against a clean tree. Without it the macOS leg is red on arrival, and a lane that is red on its first run teaches everyone to ignore it.
