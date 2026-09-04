## Why

Installing this tool requires a Rust toolchain, a Bun toolchain, and knowing that the second one has to run before the first or `cargo` will not compile. `README.md` already admits where this is going: *"Distribution will move to prebuilt binaries (release workflow + install script), so user machines need neither Bun nor cargo."*

`add-release-automation` produces versions — a tag, a changelog, a GitHub release. It deliberately puts nothing in them. This change is what makes one worth downloading: a binary for each supported platform, and a one-line installer that picks the right one.

The embedding decision is what makes this more than a build matrix. `crates/server` compiles `web/dist` into the binary, and `web/dist` is a gitignored artifact of a separate toolchain. So a released binary is a Bun build and a cargo build fused together, and the naive matrix — Bun and cargo on each of five runners — produces **five independently built dashboards** with nothing asserting they are the same. `add-continuous-verification` established the alternative shape for the gate run: build the frontend once, hand the artifact to every platform. This change carries that shape into the artifacts users actually receive, where the difference stops being about build minutes and becomes a claim the project can either make or not make.

## What Changes

- Build a release binary for five targets on each release: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, and `x86_64-pc-windows-msvc`.
- **One frontend build per release, fanned out to all five.** Every platform's binary embeds the same bytes, and the project can say so.
- **Linux is musl-only, statically linked.** A `curl`-installed binary has to work on a machine you know nothing about, and a glibc-linked build carries its build runner's glibc floor into every user's environment — the failure where it runs on the maintainer's Ubuntu and dies on someone's Debian. Every workspace dependency is pure Rust, so a fully static build costs nothing here. Shipping both gnu and musl would only move the choice into the installer, where it is harder to get right and impossible to test.
- Attach checksums, and **create the release as a draft until its binaries are present**. A release that becomes current before its assets exist is a window in which every install resolves to a release with nothing in it. This change is why drafting exists; `add-release-automation` deliberately did not add it, because a draft protecting nothing is complexity with no argument behind it.
- Ship `install.sh` and `install.ps1` **as release assets**, served from the release they install. Serving them from the default branch instead would pin every future install to the tip of an unreviewed branch, where one bad push breaks installs for everyone until it is reverted.
- **Report the missing runtime dependency rather than reporting success.** `crates/core/src/scratch/promote.rs:110` shells out to `openspec validate`; the `openspec` CLI is a hard runtime requirement for promotion, and promotion is where an exploration becomes a change rather than an edge feature. An installer that prints success over a half-satisfied dependency graph is this repository's *"do not hide errors with silent defaults"* rule broken at the front door.

Deliberately **not** in scope:

- **Installing `openspec`.** It is a Node tool with its own opinions, and an installer that begins installing other people's package managers has stopped being one-shot. Detect it and say so; do not fetch it.
- **macOS code signing and notarization.** Recorded as a stated non-goal with its consequence rather than left to surprise someone: a binary fetched by `curl` carries no quarantine attribute and runs, while the same file downloaded through a browser is blocked by Gatekeeper. So the install script works and the releases-page download does not, on the platform where people are most likely to try it. Notarization requires a paid Apple developer account.
- **A Homebrew tap, a Scoop manifest, an AUR package, or a Nix flake.** Each is a distribution channel with its own update obligation. One channel that works is the deliverable.
- **Cross-compiling to targets nobody has asked for**, including 32-bit anything, Windows on ARM, and the BSDs.
- **Self-update.** The installer is idempotent; re-running it is the update mechanism.

## Capabilities

### Modified Capabilities

- `release-distribution`: a release currently means a tag, a changelog, and a GitHub release. It gains the artifacts — what platforms are covered, the guarantee that they embed one dashboard build, that a release is not current until they exist, and how the installer that fetches them is itself distributed and what it refuses to lie about.

## Impact

- `.github/workflows/release.yml` — the build matrix and the upload, in the same run as the release step from `add-release-automation`, because a tag created by automation does not trigger a tag-keyed workflow.
- `install.sh`, `install.ps1` — new, at the repository root, and published as release assets.
- `Cargo.toml` — a release profile decision. The binary embeds a frontend and links statically; size and build time are both worth one deliberate look rather than inheriting whatever `--release` gives.
- `README.md`, `docs/docs/quickstart.md` — the install instructions become one line, and `cargo install --path` moves to being the contributor's path rather than the user's.
- `docs/docs/roadmap.md` — the remaining half of the *"No release; setup is lossy"* row.
- Users on platforms not in the five, who go from "build it yourself" to "build it yourself, and the documentation now implies you were not considered". The unsupported case should be named rather than left as an absence.

**Depends on `add-release-automation`** for the release to attach assets to, and on `replace-hook-shell-form-with-exec-form` for the Windows binary to be worth shipping. Building an `openspec-doc.exe` whose `doctor` cannot find itself on `PATH` would be shipping the appearance of Windows support rather than the thing.
