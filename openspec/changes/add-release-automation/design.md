# Design

## Release pull request, not release on every push

Two shapes were available and they are not close.

```text
   every push                          release pull request
   ──────────                          ────────────────────
   push feat: → CI computes version    push feat: → bot updates an open PR
             → rewrites Cargo.toml                → PR accumulates commits
             → commits back to main               → and their changelog lines
             → tags, releases          merge PR  → tag, release
   ~24 releases across this history    releases when you say so
   bot commits interleave with yours   changelog reviewable before publication
```

Cadence is not the deciding argument, and neither is cost — the repository is public, so every runner including macOS and arm64 is free, and twenty-four releases would not have been expensive.

The deciding argument is that the pull request is the only place a changelog line can be read before a stranger reads it. Given a history containing `feat: vue founddation`, that is not a theoretical benefit. Releasing on every push takes whatever was typed at commit time and publishes it under a version number with no interval in which anyone could notice.

The cost is a process change rather than a tool addition: this repository has forty-two commits, zero merge commits, and zero pull requests. Everything lands by pushing to `main`. Introducing a pull request for exactly one purpose is a real thing to have done, and it should be a deliberate act rather than something that arrives as a side effect of installing a bot.

## release-plz, and the crates.io question turned inside out

`release-plz` is Rust-native, workspace-aware, and reads conventional commits. Its centre of gravity is publishing to crates.io, which this project cannot do — and that was the reason to hesitate, since a tool kept for its side feature is a tool that will one day be upgraded along an axis you are not using.

The hesitation does not survive the details. `publish = false` skips `cargo publish` while `git_tag_enable` and `git_release_enable` keep the tag and the GitHub release, and a binary-only workspace is a documented use of the tool rather than a hack around it. The alternatives were worse for this repository specifically: `semantic-release` drags an unpinned npm plugin tree into the release path of a project that deliberately has one JavaScript toolchain and no npm metadata, and still cannot touch a Rust manifest without a second tool bolted on through `exec`.

There is a better reason to reach for `publish = false` than making the tool behave. **It makes the crates.io non-goal executable.** `crates/server` embeds `web/dist` at compile time; `cargo package` excludes gitignored files; `web/dist` is gitignored. A published `openspec-doc-server` would therefore ship with no dashboard inside it and fail to compile for whoever ran `cargo install`. Today the only thing preventing that is nobody having typed `cargo publish`. `publish = false` in each crate's own manifest turns "we have decided not to" into "the tool refuses", which is the difference this repository's conventions care about.

## The one thing to verify on the first run

`web/dist` is gitignored, so **no CI job has it unless something built it.** If any part of release-plz's path compiles the workspace, it fails in a job that has no dashboard to embed, and the failure will present as a release-plz problem rather than as the gitignored-artifact decision biting.

The expectation is that it does not: release-plz reads manifests through `cargo metadata`, which parses rather than builds, and with `publish = false` there is no `cargo publish` verification step — which is the path that would have compiled. Confidence: moderate. Not high, because "reads manifests" is a claim about an implementation rather than a documented guarantee, and the failure is cheap to observe and annoying to diagnose from a distance.

This is also the reason this change exists separately from the one that adds binaries. Watching one interaction go wrong is easy; watching it go wrong underneath a five-target build matrix that also just appeared is not.

## The tag that triggers nothing

Worth writing down before it is discovered.

**A tag pushed with `GITHUB_TOKEN` does not trigger `on: push: tags:` workflows.** GitHub suppresses it deliberately, to prevent workflows from triggering each other without limit. So the intuitive structure — release-plz cuts a tag in one workflow, a second workflow keyed on `v*` builds and uploads — silently never runs its second half. What you get is a tag and a release with nothing in them, and no failed job anywhere to indicate it.

Nothing in *this* change has assets, so nothing here breaks. It is recorded here because this is the change that establishes the trigger structure that `add-release-binaries-and-installers` will hang a build matrix off, and getting the shape right once is cheaper than discovering it with five build jobs attached. The correct structure is one workflow whose later jobs are conditioned on the release step's own output, not a second workflow keyed on the tag. Reaching for a personal access token to force the trigger is the wrong fix: it works, and it replaces a structural guarantee with a credential.

A related and smaller one: **a pull request opened with `GITHUB_TOKEN` does not trigger `pull_request` workflows either**, so the release pull request will show no checks. Its content is a version bump and a changelog, so this is tolerable — but it should be written down, because a pull request with no checks in a repository that otherwise has them looks like something is broken.

## Versions start at 0.1.0, and that number will lie

Decided: keep `0.1.0` and let conventional commits carry it forward.

The consequence to hold consciously is that below `1.0.0`, a breaking change and a new feature are both a minor bump. So the version number tells someone installing the tool nothing about whether upgrading will break them, which is most of what a version number is for. For a local-only review tool with one maintainer and no downstream consumers that is a reasonable trade. It stops being reasonable the first time someone else pins a version, and that is the signal to cut `1.0.0` rather than any particular feeling of completeness.

## The license is part of this change, not a cleanup

The repository is public and carries no `LICENSE`. Default copyright is all rights reserved: no permission to use, modify, or redistribute. Every reader so far has been someone who cloned the source, and the gap has been harmless in the way an unlocked door on an empty building is harmless.

A tagged release is the first artifact this project asks a stranger to take. Under this repository's rule that a change carries the text it falsifies, the license belongs here — a release workflow that cuts distributable versions of unlicensed software is a design that argues for something the repository does not permit.

**MIT**, chosen by the repository owner. It is the permissive default for a Rust CLI, it is what the
ecosystem this tool sits beside overwhelmingly uses, and for a local-only review tool with no patent
surface and no ambition to constrain how anyone uses it, the reasons to reach for anything stronger do
not apply. Apache-2.0's patent grant and the dual `MIT OR Apache-2.0` convention are the alternative
worth knowing was declined rather than overlooked; they matter for crates other people link against,
and `publish = false` says this workspace is not one.
