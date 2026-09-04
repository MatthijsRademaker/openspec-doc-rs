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

## The suite is not hermetic, and the lane installs what it needs

*Added after the lane's first run, which failed on both platforms before this was understood.*

`scratch::promote` runs `openspec validate <change>` as a subprocess, and two tests assert on what it
reported — `hook_stop_promotes_the_scratch_note_and_reports_the_validate_outcome` and
`hook_stop_reports_a_failing_validate_rather_than_hiding_it`. `docs/docs/development/testing.md` has
said so for as long as it has existed, under a heading that reads "Hermetic by default", which is where
the fact went to be forgotten. On a runner with no `openspec` on `PATH`, those two fail.

Three ways out, and the choice between them is not close:

1. Gate the two tests behind the binary's presence, so they skip when it is absent.
2. Install `openspec` in the `gates` job.
3. Replace the subprocess with a fake, so nothing shells out.

**Take the second.** The first is this change's own argument turned against itself: a run that skips the
only two assertions that promotion reports a validation outcome, and reports success, is the
green-line-for-something-never-examined failure the Windows section spends four paragraphs refusing. It
would also be invisible in exactly the way that matters, because a skipped test and a passing test look
identical in a run's summary.

The third is the better long-term answer and is not this change's to make. Those two tests are the only
place anything checks that the promotion path reports a real validator's verdict rather than a verdict
it made up; replacing the validator with a fake removes the coupling and some of the value with it.
That is a decision about what the tests are for, and it belongs to whoever next touches promotion.

So the job installs `@fission-ai/openspec` at a pinned version. Pinned because the tests read its
output: an upgrade that reworded `passed` or `FAILED` breaks them, and a pin makes that a commit rather
than a Tuesday.

The cost is honest and small: the lane now depends on a package from npm, so npm being down is a red
lane. Bun is already installed in that job for `make check`, so it is one command and no new toolchain.

## Passing on a macOS machine is not passing on a macOS runner

*Added after the lane's second and third runs, which were green on Linux and red on macOS, on a
different test each time.*

The proposal says this change depends on `fix-repository-verification-gates` because that change
*"makes `cargo test --workspace` pass on macOS against a clean tree"*. That was checked on a
developer's macOS machine and it is true there. It was not true on `macos-latest`, and the difference
is load rather than platform.

Two tests in `crates/server` asserted against filesystem events **their own fixtures had caused**:

- `watch::tests::reading_a_watched_file_is_not_an_update` writes a file into the watched directory,
  *then* subscribes, then asserts that nothing arrives within two seconds. The subscription's FSEvents
  stream starts from "now" only approximately, so under load the setup's write is delivered after the
  subscription it preceded, and the test reads its own noise as a read being reported as a change.
- `routes::tests::resolve_all_and_approve_pushes_one_event_rather_than_one_per_comment` writes three
  comments, opens an SSE stream, sleeps 500ms, submits one bulk act, and asserts the stream carried
  **exactly one** event. A late fixture event makes that two. Its own comment stated the assumption it
  was relying on — *"the subscription is taken when the handler runs, and an update written before it
  exists reaches nobody"* — which is true of the subscription and says nothing about when FSEvents
  delivers.

Both now wait for the fixture's events to go quiet before asserting, rather than sleeping a fixed span.
Waiting for quiet is the point: the condition becomes the absence of events instead of the passage of
time, so a slower runner makes the test wait longer rather than fail. A longer sleep would have moved
the race rather than removed it.

Two things follow, and the second is the reason this section exists rather than a task note.

**The capability already predicted this failure.** Its scenario *"the automated run reproduces the local
verdict"* — gates pass locally on a supported platform, so the automated run for the same contents on
that platform passes too — is exactly what these two tests violated. The requirement was written before
anything could violate it and it caught them on the first opportunity. Nothing in the spec delta needs
to change; it did its job.

**"Verified on macOS" was a claim about one machine.** The lane's whole argument is that a gate whose
result nobody sees is worth nothing; the sharper version this run taught is that a gate run in one
place has only ever reported on that place. The suite had passed on macOS hundreds of times and had
never once run on a macOS machine that was busy. That is not a property of these two tests — it is a
property of every timing-sensitive assertion in the repository, and this lane is now the only thing
exercising them anywhere else.

Three of these were fixed rather than one: the audit in task 6.3 checked the remaining negative and
count assertions in `crates/server` and found them driving in-process channels and timers, where a
slower machine makes the assertion pass rather than fail. That audit is the part worth repeating if a
third one ever surfaces.

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

## The toolchain pin

`rust-toolchain.toml` pinning a specific stable version, not `stable`.

`stable` means the lane's meaning changes every six weeks without a commit, and the first symptom is usually a new clippy lint failing `--all-targets` on code nobody touched. That failure is fine to have; having it arrive on an unrelated pull request, attributed to that pull request, is not. A pinned version turns it into a one-line commit that says what it is.

The cost is that the pin is a thing to remember to bump, and nothing here bumps it. That is accepted for now and is a candidate for the same automation that opens the release pull request, once that exists.
