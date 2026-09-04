## 1. Refuse the registry, in the manifests

Do this first. It is the smallest edit in the change and it closes the one way a stranger's install can be broken by a single command typed in the wrong directory.

- [ ] 1.1 Set `publish = false` in `crates/core/Cargo.toml`, `crates/server/Cargo.toml`, and `crates/cli/Cargo.toml`.
- [ ] 1.2 Confirm by attempting `cargo publish --dry-run -p openspec-doc-server` and seeing it refused by the manifest. This is worth doing once: the reason for the refusal — a crate that would ship without `web/dist` and not compile for its installer — is invisible in the diff, and seeing the refusal is what makes it real.
- [ ] 1.3 Add a comment at each `publish = false` saying why, in the style the workspace already uses for its dependency decisions. A bare `publish = false` reads as a default; the reason is the point.

## 2. License

- [ ] 2.1 Add an MIT `LICENSE` at the repository root, with the copyright line naming the repository owner and the current year.
- [ ] 2.2 Add `license = "MIT"` to `[workspace.package]` so all three crates inherit it, matching how `version` and `edition` are already shared.
- [ ] 2.3 Add the license to `README.md`, which currently states no terms at all.
- [ ] 2.4 Check `docs/` for a footer or metadata that should name it, since the site is the other public face of the project.

## 3. release-plz

- [ ] 3.1 Add `release-plz.toml` with `publish = false`, `git_tag_enable = true`, `git_release_enable = true`, and changelog generation on.
- [ ] 3.2 Add `.github/workflows/release.yml` with the release-pull-request job on push to `main`, pinning the action by commit SHA with the version in a trailing comment, as the existing workflows do.
- [ ] 3.3 Add the release job in the **same workflow**, conditioned on the release-pull-request step's own output. Do not add a second workflow keyed on `v*`: a tag pushed with `GITHUB_TOKEN` does not trigger tag-keyed workflows, and the result is a tag, an empty release, and no failed job anywhere to say so.
- [ ] 3.4 Give the workflow the narrowest permissions that work — contents and pull-requests write — and do not reach for a personal access token. If something appears to need one, that is a signal about the structure, not about the credential.
- [ ] 3.5 Generate the initial `CHANGELOG.md`. The existing history will produce poor entries; edit them in the first release pull request rather than pretending the generator got them right.

## 4. Verify the interaction nothing has tested

- [ ] 4.1 **Confirm on the first run that no part of release-plz's path compiles the workspace.** `web/dist` is gitignored, so the job has none, and a compile would fail in a way that reads as a tool bug rather than as the embedding decision biting. Expectation is `cargo metadata` only; confidence moderate, which is why this is a task and not a footnote.
- [ ] 4.2 If it does compile, do not add a Bun build step to the release-plz job as a reflex. Establish which path compiles and why first — the answer changes whether the fix is a build step, a configuration flag, or a different tool.
- [ ] 4.3 Confirm the release pull request opens with no checks reported, and that this is because pull requests opened with `GITHUB_TOKEN` do not trigger `pull_request` workflows. Write it down where a reader of the pull request would wonder. A checkless pull request in a repository that has checks looks broken.

## 5. Cut the first release

- [ ] 5.1 Merge the first release pull request and confirm the tag, the release, and the changelog all appear.
- [ ] 5.2 Confirm the released version matches `Cargo.toml` and `Cargo.lock`, and that the lockfile was updated by the bump. A workspace whose lockfile still names the old version is a broken release that builds fine.
- [ ] 5.3 Confirm `openspec-doc --version` reports the released version. `doctor` compares versions between binaries and reads this string; a release that does not move it makes that comparison meaningless.
- [ ] 5.4 Confirm nothing was published to a registry.
- [ ] 5.5 Say in the release notes, or in the repository, that this release carries no binaries yet and `add-release-binaries-and-installers` is what makes one worth downloading. A first release that looks like a distribution and is not will otherwise generate exactly one confused issue.

## 6. The text this falsifies

- [ ] 6.1 Update `README.md`'s install section, which offers `cargo install --path crates/cli` as the only route, with no version to ask for.
- [ ] 6.2 Update `docs/docs/quickstart.md` for the same reason.
- [ ] 6.3 Update the `docs/docs/roadmap.md` *"No release; setup is lossy"* row. Half of it closes here; the other half is the installer, and the row should say which half is which rather than being deleted.
- [ ] 6.4 Document how a version is produced, where a contributor will look: that commit typing decides the bump, that nothing verifies the typing, and that the release pull request is where a bad changelog line is still fixable.
