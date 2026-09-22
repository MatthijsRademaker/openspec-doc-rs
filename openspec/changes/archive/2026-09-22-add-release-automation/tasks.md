## 1. Refuse the registry, in the manifests

Do this first. It is the smallest edit in the change and it closes the one way a stranger's install can be broken by a single command typed in the wrong directory.

- [x] 1.1 Set `publish = false` in `crates/core/Cargo.toml`, `crates/server/Cargo.toml`, and `crates/cli/Cargo.toml`.
- [x] 1.2 Confirm by attempting `cargo publish --dry-run -p openspec-doc-server` and seeing it refused by the manifest. This is worth doing once: the reason for the refusal — a crate that would ship without `web/dist` and not compile for its installer — is invisible in the diff, and seeing the refusal is what makes it real.
  **PASSED** `cargo publish --dry-run -p openspec-doc-server` refused with `` `package.publish` must be set to `true` or a non-empty list in Cargo.toml to publish. ``.
- [x] 1.3 Add a comment at each `publish = false` saying why, in the style the workspace already uses for its dependency decisions. A bare `publish = false` reads as a default; the reason is the point.

## 2. License

- [x] 2.1 Add an MIT `LICENSE` at the repository root, with the copyright line naming the repository owner and the current year.
- [x] 2.2 Add `license = "MIT"` to `[workspace.package]` so all three crates inherit it, matching how `version` and `edition` are already shared.
- [x] 2.3 Add the license to `README.md`, which currently states no terms at all.
- [x] 2.4 Check `docs/` for a footer or metadata that should name it, since the site is the other public face of the project.

## 3. release-plz

- [x] 3.1 Add `release-plz.toml` with `publish = false`, `git_tag_enable = true`, `git_release_enable = true`, and changelog generation on.
- [x] 3.2 Add `.github/workflows/release.yml` with the release-pull-request job on push to `main`, pinning the action by commit SHA with the version in a trailing comment, as the existing workflows do.
- [x] 3.3 Add the release job in the **same workflow**, conditioned on the release-pull-request step's own output. Do not add a second workflow keyed on `v*`: a tag pushed with `GITHUB_TOKEN` does not trigger tag-keyed workflows, and the result is a tag, an empty release, and no failed job anywhere to say so.
- [x] 3.4 Give the workflow the narrowest permissions that work — contents and pull-requests write — and do not reach for a personal access token. If something appears to need one, that is a signal about the structure, not about the credential.
- [x] 3.5 Generate the initial `CHANGELOG.md`. The existing history will produce poor entries; edit them in the first release pull request rather than pretending the generator got them right.
  **PASSED** release-plz 0.3.169 generated a 0.1.0 changelog in a clean temporary worktree; the committed file keeps the generator's header and `Unreleased` section so the first release PR remains the deliberate editing point.

## 4. Verify the interaction nothing has tested

- [x] 4.1 **Confirm on the first run that no part of release-plz's path compiles the workspace.** `web/dist` is gitignored, so the job has none, and a compile would fail in a way that reads as a tool bug rather than as the embedding decision biting. Expectation is `cargo metadata` only; confidence moderate, which is why this is a task and not a footnote.
  **PASSED** In a clean temporary worktree with `web/dist` absent, release-plz 0.3.169 ran `update --allow-dirty`; it performed version detection and generated the initial changelog without any `cargo build`, `cargo check`, `cargo test`, or `cargo package` command in its log.
- [x] 4.2 If it does compile, do not add a Bun build step to the release-plz job as a reflex. Establish which path compiles and why first — the answer changes whether the fix is a build step, a configuration flag, or a different tool.
  **PASSED** The clean-worktree run did not compile; no Bun build step was added.
- [x] 4.3 Confirm the release pull request opens with no checks reported, and that this is because pull requests opened with `GITHUB_TOKEN` do not trigger `pull_request` workflows. Write it down where a reader of the pull request would wonder. A checkless pull request in a repository that has checks looks broken.
  **PASSED** Release PR #4 opened with an empty `statusCheckRollup`; repository docs record that `GITHUB_TOKEN`-created pull requests do not trigger `pull_request` workflows. Later manual changelog editing triggered checks, confirming the distinction.

## 5. Cut the first release

- [x] 5.1 Merge the first release pull request and confirm the tag, the release, and the changelog all appear.
  **PASSED** Merged release PR #4 at `30d9ae0`; tag `v0.1.0`, GitHub release, and reviewed `CHANGELOG.md` appeared.
- [x] 5.2 Confirm the released version matches `Cargo.toml` and `Cargo.lock`, and that the lockfile was updated by the bump. A workspace whose lockfile still names the old version is a broken release that builds fine.
  **PASSED** Tag `v0.1.0` checkout reports workspace version `0.1.0`; all three workspace packages in `Cargo.lock` also report `0.1.0`.
- [x] 5.3 Confirm `openspec-doc --version` reports the released version. `doctor` compares versions between binaries and reads this string; a release that does not move it makes that comparison meaningless.
  **PASSED** Built tag `v0.1.0` from a clean checkout; `cargo run --locked -p openspec-doc-cli -- --version` reported `openspec-doc 0.1.0`.
- [x] 5.4 Confirm nothing was published to a registry.
  **PASSED** Release-plz log reports all three packages released in `git-only` mode and contains no registry upload; the GitHub release has no registry assets.
- [x] 5.5 Say in the release notes, or in the repository, that this release carries no binaries yet and `add-release-binaries-and-installers` is what makes one worth downloading. A first release that looks like a distribution and is not will otherwise generate exactly one confused issue.

## 6. The text this falsifies

- [x] 6.1 Update `README.md`'s install section, which offers `cargo install --path crates/cli` as the only route, with no version to ask for.
- [x] 6.2 Update `docs/docs/quickstart.md` for the same reason.
- [x] 6.3 Update the `docs/docs/roadmap.md` *"No release; setup is lossy"* row. Half of it closes here; the other half is the installer, and the row should say which half is which rather than being deleted.
- [x] 6.4 Document how a version is produced, where a contributor will look: that commit typing decides the bump, that nothing verifies the typing, and that the release pull request is where a bad changelog line is still fixable.
