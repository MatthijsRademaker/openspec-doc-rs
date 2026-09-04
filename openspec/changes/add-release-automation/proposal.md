## Why

There is no version of this software. There is `main`.

`workspace.package.version` has said `0.1.0` across all forty-two commits. There are no git tags, no GitHub releases, and no changelog. The only documented way to install the tool is `cargo install --path crates/cli` from a clone — which means every user is on whatever `main` was the day they cloned, nobody can say which version they are running, and a bug report cannot name one. The roadmap carries this as *"No release; setup is lossy"*, rated moderate and deliberately outside the MVP boundary. That boundary has been met.

There is also a licensing problem that a release makes urgent. The repository is public and has **no `LICENSE` file**, which under copyright default means all rights reserved: legally, nobody may use or redistribute it. That is tolerable while the only way in is `git clone` and a local `cargo install`, because those users are already reading the source. It stops being tolerable the moment the project publishes a versioned artifact and invites people to download it.

This change does not produce binaries. It produces versions: a changelog, a tag, and a GitHub release, derived from commit history and landed through a pull request. `add-release-binaries-and-installers` puts something worth downloading inside them. The two are separated because release automation against this workspace has one unverified interaction — whether any part of it tries to *compile* a workspace that cannot compile without `web/dist` — and finding that out with a five-target build matrix already attached makes the failure harder to read, not easier.

## What Changes

- Adopt **release-plz** in release-pull-request mode. It reads conventional commits since the last release, opens and maintains a pull request carrying the version bump and the generated changelog, and on merge creates the tag and the GitHub release.
- **Never publish to a package registry.** Set `publish = false` in `release-plz.toml` *and* in each crate's `Cargo.toml`. The second is the one that matters: `crates/server` embeds `web/dist` at compile time and `cargo package` excludes gitignored files, so a published `openspec-doc-server` would carry no dashboard and would not compile for whoever installed it. Making that an executable refusal rather than a documented intention is this repository's own stated preference, and it removes a way to break a stranger's install by running one command in the wrong directory.
- Add a `CHANGELOG.md`, generated and then maintained by the tool.
- Add an **MIT** `LICENSE`, and `license = "MIT"` in the workspace package metadata.
- Keep semantic versioning a **human discipline**. No commitlint, no commit hook, no CI gate on commit messages. This is a decision with a known cost, recorded below rather than left implicit.
- Start from `0.1.0` and let the history carry it forward.

Deliberately **not** in scope:

- **Binaries, checksums, installers, and the build matrix.** `add-release-binaries-and-installers` owns all of it, and owns the change from a plain release to a drafted one, because drafting only earns its complexity once a release has assets whose absence would matter.
- **Publishing to crates.io**, which is blocked by the gitignored `web/dist` and is being closed off rather than worked around.
- **Enforcing commit message format.** See below; this is a choice, not an oversight.
- **Automating the toolchain pin bump** that `add-continuous-verification` introduces. It is a candidate for the same machinery later and is not worth coupling now.

## The accepted risk, stated plainly

Automated releases make commit subjects load-bearing: they decide the version bump and they become the public changelog. This repository's history is not written that way. Of forty-two commits, twenty-three are typed `feat` and one `fix`, so nearly every commit reads as a feature — including `feat: revert eye tracking`, which is a revert. Two are typed `wip`, which is not a conventional-commits type at all, and one carries no type. The subjects themselves are notes to self: `feat: vue founddation`, with the typo, and `feat: improved ui and ux`.

Run that history through a changelog generator and the result is private shorthand published under a version number. The tooling does not clean this up; it publishes it.

With no pull requests in the repository's history there is also no natural gate. Commitlint conventionally runs on a pull request title and there are no pull request titles. A check on `main` can only go red *after* the commit has landed. A local commit hook is machine-local configuration — the exact thing this project has already been burned by, per the roadmap's record that its own agent hooks were *"absent for an unknown period without anyone noticing"*.

The decision is to write the messages carefully by hand and accept the exposure. The mitigation that does exist is the release pull request itself: it is the last point at which a bad changelog line can be edited before anyone reads it. That is the strongest argument for the pull-request model over releasing on every push, and it is why this change buys the model rather than the speed.

## Capabilities

### New Capabilities

- `release-distribution`: how a version of this software comes to exist — where the version number comes from, what a release is, what is deliberately never published, and under what terms the result may be used.

## Impact

- `release-plz.toml` — new.
- `.github/workflows/release.yml` — new.
- `CHANGELOG.md` — new.
- `LICENSE` — new, MIT.
- `Cargo.toml` at the workspace root and in all three crates — `publish = false`, and the `license` field.
- `README.md`, `docs/docs/quickstart.md` — both present `cargo install --path` as the way in, with no version to ask for.
- `docs/docs/roadmap.md` — the *"No release; setup is lossy"* debt row, half of which this closes.

**Depends on `add-continuous-verification`.** Cutting versioned releases out of a repository whose test suite no automation has ever run is the ordering this whole sequence exists to avoid.
