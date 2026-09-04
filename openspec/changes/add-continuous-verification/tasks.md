## 1. Make the Makefile invokable from CI

- [ ] 1.1 Make `web/dist` a Makefile target satisfied only when it is absent, and point the gate targets at it instead of at `build`. In CI the frontend arrives as a downloaded artifact, and a gate target that rebuilds it both costs a Bun install per leg and produces a *different* `web/dist` from the one the artifact carried — which defeats the single-build guarantee this change is adding.
- [ ] 1.2 Add a `frontend` target that forces the rebuild, since the conditional one deliberately will not. A developer working on `web/` needs a way to say so.
- [ ] 1.3 Write the staleness caveat into the Makefile beside the target: editing `web/src` and running `make test` tests the previous dist, and that is tolerable only because the Rust gates embed the frontend without asserting on it while the frontend's own gates build fresh. If a Rust test ever asserts on embedded content, this decision is the one to revisit.
- [ ] 1.4 Do not add a CI-only Makefile target. A target that exists for the lane is a second definition of the gates by another route, which is the thing the design rejects.

## 2. The lane

- [ ] 2.1 Add `.github/workflows/rust.yml`, triggered on push to `main` and on pull requests, matching the existing workflow's convention of pinning actions by commit SHA with the version in a trailing comment.
- [ ] 2.2 Add a `frontend` job: Bun 1.3.2, `bun install --frozen-lockfile`, `bun run build`, upload `web/dist` as an artifact. No cargo, no tests, no Playwright — this job exists to produce one artifact.
- [ ] 2.3 Add a `gates` job matrixed over `ubuntu-latest` and `macos-latest`, depending on `frontend`, downloading the artifact into `web/dist` before any cargo command runs.
- [ ] 2.4 Run `make check` and `make test` in that job. Not `cargo` commands. If a `cargo` invocation appears in this workflow, the design decision has been reversed and the design needs revisiting rather than the YAML.
- [ ] 2.5 Fail the leg if `web/dist` is absent after the download step, rather than letting cargo report a confusing embed error. The artifact is a hard dependency and its absence should say so.
- [ ] 2.6 Cache the cargo registry and the `target` directory per platform. Unlike the frontend lane, this one is not proving a clean build; it is proving the code compiles and passes, so a cache verifies nothing it should not.
- [ ] 2.7 Do not add a Windows leg. `replace-hook-shell-form-with-exec-form` adds it together with the fixes that make it honest.

## 3. Pin the toolchain

- [ ] 3.1 Add `rust-toolchain.toml` at the repository root pinning a specific stable version — not the `stable` channel — with `rustfmt` and `clippy` components, and the edition the workspace already declares in mind.
- [ ] 3.2 Confirm the pinned version builds the workspace and passes clippy `--all-targets` before landing. A pin that arrives with new lint failures makes its own introduction the noisy commit it exists to prevent.
- [ ] 3.3 Note in `docs/docs/development/conventions.md` that the toolchain is pinned and that bumping it is a deliberate commit, since nothing bumps it automatically.

## 4. Say what is covered and what is not

- [ ] 4.1 Record the covered platforms — Linux and macOS — where a reader looks for them, and record Windows as an omission with its reason: `doctor`'s binary resolution and hook probe are broken there, three tests are `#[cfg(unix)]`-gated, and a run that skipped exactly those tests would be green and wrong.
- [ ] 4.2 Name the three excluded tests explicitly rather than saying "some tests". A count is not an omission report; the reader needs to know that what is missing is the `doctor` coverage.
- [ ] 4.3 Update `README.md`'s "Build and test" section, which presents running the suite by hand as the only way it is run. Check the test count it quotes while there; it says 228 and nothing verifies it.
- [ ] 4.4 Update `docs/docs/development/testing.md` for the same reason.
- [ ] 4.5 Update the `docs/docs/roadmap.md` "Known defects and debt" row that rests on no lane existing. Under this repository's rule that a change carries the text it falsifies, this belongs here rather than in a later documentation pass.

## 5. Verification

- [ ] 5.1 Push a commit that fails `cargo fmt --check` and confirm the lane fails. A lane nobody has seen fail is a lane nobody has seen work.
- [ ] 5.2 Push a commit that fails a test on one platform only, and confirm the matrix reports which platform. This is the case the whole matrix exists for and it is worth seeing once.
- [ ] 5.3 Confirm the `gates` job fails, rather than silently compiling, when the `frontend` job's artifact is unavailable.
- [ ] 5.4 Confirm on a run that both platform legs used the same artifact — the run's single `frontend` job — rather than each having produced one.
- [ ] 5.5 Confirm total wall-clock time for the lane is acceptable on a pull request. If it is not, the answer is caching or a smaller matrix, not dropping `make`.
