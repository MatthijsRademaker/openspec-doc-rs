# Design

## The lane invokes the Makefile, not `cargo`

`fix-repository-verification-gates` makes the Makefile the entry point: a developer reaches any surface's gate through it *without knowing that surface's own toolchain commands*. The tempting CI lane ignores that and writes `cargo fmt --all --check`, `cargo clippy --workspace --all-targets`, `cargo test --workspace` into YAML, because that is what every Rust workflow on the internet looks like.

Doing so creates a second definition of what "checked" means, and the two copies do not drift symmetrically. If CI is the stricter copy, a developer's clean `make check` is followed by a red push, which is annoying but self-correcting. If the Makefile is the stricter copy — a gate added there and not to CI — then CI is green for a surface nobody is checking, and the failure is silent and permanent. The capability already has a scenario for the shape of this mistake: *a surface with no gate is a visible omission*. A surface gated only where nobody runs it is the same omission wearing a green check.

So the lane runs `make`. The cost is real and worth naming: `make` on a GitHub runner is one more layer between a failure and its output, and a Makefile target that bundles several commands reports the first failure and stops, so a run tells you about `fmt` or about `clippy` but not both. That is a worse developer experience than three separate steps with three separate red marks. It is accepted, because the alternative trades a diagnosis inconvenience for a correctness hole.

## Making `test: build` work in a job that already has a dist

One thing in the Makefile has to change. `test` currently depends on `build`, which runs `bun install` and `bun run build`:

```make
test: build
	cargo test --workspace
```

In CI the frontend has already been built by another job and downloaded into `web/dist`. Rebuilding it there costs a Bun install per platform leg and — the actual problem — produces a *different* `web/dist` from the one the artifact carried, which is precisely the property the fan-out exists to guarantee. So the gate targets cannot depend on a target that unconditionally rebuilds.

Two ways out. Drop the prerequisite and tell developers to run `make build` first, or make satisfying the prerequisite conditional on the artifact being absent. **Take the second**, with `web/dist` as a target in its own right:

```make
# The artifact the workspace embeds. Built when absent; `make frontend` forces it.
web/dist:
	$(MAKE) frontend

frontend:
	cd web && bun install --frozen-lockfile && bun run build

build test check: web/dist
```

A fresh clone has no `web/dist`, so `make test` builds it and the developer never learns that Bun had to run first — which is the capability's requirement, that a gate is reachable without knowing the surface's toolchain. A CI leg downloaded one, so make finds it present and does nothing, and the leg embeds the artifact it was given.

The first option was rejected for what it does to the fresh-clone case. `make test` failing with a cargo error about an unreadable embed folder, on a checkout where the developer did nothing wrong, is the same class of trap as the unwired-hooks problem this project has already been bitten by: a broken state that looks identical to an ordinary one.

The cost is a staleness hazard, and it should be stated rather than discovered: a developer who edits `web/src` and runs `make test` tests against the *previous* dist, because the target's existence is the condition and make is not tracking the frontend's sources. Listing those sources as prerequisites is possible and not worth it — it means teaching the Makefile the frontend's file layout, which is the coupling this arrangement exists to avoid.

What defuses it is specific to this repository rather than a general reassurance: **the Rust gates do not verify the frontend's contents.** They embed it and never assert on it. The frontend has its own gate — `bun run check` for its quality, `bun run test:e2e` for the embedded application in a browser — and both build fresh. So a stale dist under `cargo test` produces a stale binary that no assertion looks inside. If a Rust test ever does start asserting on embedded content, this decision is the one to revisit, and that sentence belongs in the Makefile next to the target.

## One frontend, downloaded by every leg

```text
   ┌──────────────────────────────┐
   │ frontend                     │   ubuntu, bun 1.3.2
   │   bun install --frozen       │   no cargo, no tests
   │   bun run build              │
   │   upload-artifact web/dist   │
   └───────────────┬──────────────┘
                   │
        ┌──────────┴──────────┐
        ▼                     ▼
   ┌─────────┐          ┌─────────┐
   │ ubuntu  │          │ macos   │   download-artifact → web/dist
   │ make    │          │ make    │   make check
   │ check   │          │ check   │   make test
   │ test    │          │ test    │
   └─────────┘          └─────────┘
```

Two properties, and only one of them is about speed.

The one that is not: **every leg compiles the same bytes into the binary.** Here that buys little, since nothing compares the legs' outputs. In `add-release-binaries-and-installers` it is the difference between shipping one dashboard on five platforms and shipping five dashboards that were each built from the same source, which is a weaker claim than it sounds — `vue-tsc`, Vite, and a dependency tree resolved per-runner are three places for two builds of one commit to differ. Proving the fan-out here, where a mistake costs a red lane, is why this change owns it rather than the release change.

The one that is: two Bun installs saved per run, and Bun install is the slowest step in the existing frontend lane.

The artifact is passed between jobs rather than rebuilt or cached. A cache would be checked by key and could miss, which turns "every leg has the same dist" into "every leg usually has the same dist"; an artifact is a hard dependency and a missing one fails the job.

## Why not one workflow

`frontend-assets.yml` could absorb this. Its `frontend-lane` job already builds `web/dist`, so the fan-out would come free and the repository would have one CI file instead of two.

Rejected, on the narrower of two available arguments. The broad one — that the frontend lane is deliberately uncached, *"since a cached one would verify the cache rather than the source"*, while the Rust lane wants aggressive caching — does not survive contact with the facts, because caching is per-job and one workflow can hold both policies. The narrow one holds: that lane works, it is the only automation this repository has, and rewriting it to share a job is a change to something that is not broken for a saving of roughly sixty seconds per run on free runners. `make check` already covers the frontend's own gate, so the two workflows overlap on `bun run check`; that duplication is visible, cheap, and reversible later if a third consumer of `web/dist` appears.

## Windows is not deferred out of caution

It is deferred because adding it here would require choosing between two bad options, and the second one is available and tempting.

`crates/cli/tests/cli.rs` has roughly 153 test functions, of which three and their three helpers are `#[cfg(unix)]`-gated. A Windows leg added today therefore compiles and runs about 150 tests — it is not vacuous, and that is the trap. It would be green. The three tests it silently omits are `doctor_passes_a_wired_project_and_fails_naming_a_hook_that_is_not_registered`, `doctor_leaves_a_pending_directive_and_the_projects_own_review_state_alone`, and `hook_stop_starts_a_dashboard_that_outlives_it_and_serve_url_finds_it` — which is to say, exactly the coverage of `doctor`'s two Windows defects and of the hook-start path. A green Windows badge over those three is a stronger false claim than no badge at all.

`replace-hook-shell-form-with-exec-form` fixes the defects, un-gates the tests, and adds the leg together. Until then this change states Linux and macOS as the covered platforms, and the capability requires that statement to exist so the gap is a written omission rather than a reader's inference.

## The toolchain pin

`rust-toolchain.toml` pinning a specific stable version, not `stable`.

`stable` means the lane's meaning changes every six weeks without a commit, and the first symptom is usually a new clippy lint failing `--all-targets` on code nobody touched. That failure is fine to have; having it arrive on an unrelated pull request, attributed to that pull request, is not. A pinned version turns it into a one-line commit that says what it is.

The cost is that the pin is a thing to remember to bump, and nothing here bumps it. That is accepted for now and is a candidate for the same automation that opens the release pull request, once that exists.
