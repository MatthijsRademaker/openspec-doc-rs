## 1. The build matrix

- [x] 1.1 Add a `frontend` job to `.github/workflows/release.yml` producing one `web/dist` artifact per release run. Same shape as the gate lane's, and for the stronger reason: here the artifact is what users receive.
- [x] 1.2 Add a build job matrixed over the five targets, depending on `frontend` and downloading its artifact into `web/dist` before any cargo command runs.
- [x] 1.3 Fail a leg whose downloaded `web/dist` is missing or empty, before cargo runs. Note the known gap the harness capability already records: an *absent* dist is a compile error, but an empty one compiles and the binary then panics on the first request for the shell. A release binary that panics on first use is the worst place for that gap to surface, so check here rather than relying on the compile error.
- [x] 1.4 `aarch64-apple-darwin` and `x86_64-apple-darwin` both from the macOS arm runner; the second needs only `rustup target add`, no linker configuration.
- [x] 1.5 `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`, statically linked. Public repositories get arm64 runners, so the arm leg needs no cross-compilation.
- [ ] 1.6 `x86_64-pc-windows-msvc`. **Blocked on `replace-hook-shell-form-with-exec-form`**, which is unapproved; the release carries the four POSIX targets until it lands.
- [x] 1.7 Confirm the musl binaries are actually static and actually run on an older distribution than the builder. `ldd` reporting "not a dynamic executable" is the check; a musl target that silently linked dynamically is the failure this target choice exists to prevent and it does not announce itself.
- [x] 1.8 Decide the release profile deliberately rather than inheriting `--release`. The binary carries an embedded frontend and links statically; look once at size and build time and record what was chosen and why. Chosen: the default profile. Measured on x86_64 musl: 6.0 MB, ~35 s clean; `strip = true` saves 1 MB and loses symbol names in panic backtraces, fat LTO saves 0.7 MB more for ~60% more build time. Recorded at the build step in `release.yml`.
- [x] 1.9 Archive per platform in that platform's conventional format, and emit a checksum per archive.

## 2. Draft, then publish

- [x] 2.1 Have the release step from `add-release-automation` create the release as a **draft**.
- [x] 2.2 Upload every archive and checksum to that draft.
- [x] 2.3 Undraft as the final job, after every leg has succeeded. This is the release's commit point: a failed leg must leave a draft nobody can install rather than a published release missing a platform.
- [x] 2.4 Keep all of this in the same workflow run as the release step. A tag created by automation does not trigger a tag-keyed workflow, and the failure is silent — a tag, an empty release, no failed job.
- [ ] 2.5 Confirm `/releases/latest/download/…` does not resolve to the draft while the matrix is running. This is the requirement's whole point and it is one `curl` to check.

## 3. `install.sh`

- [x] 3.1 Detect operating system and architecture, and map to one of the four POSIX targets. Refuse anything else by naming what is published — do not guess a nearest match.
- [x] 3.2 Resolve the current release rather than a version, so the script needs no version string and no update when a release is cut.
- [x] 3.3 Download the archive and its checksum, verify, and fail loudly on a mismatch. Truncated downloads are the common case this catches.
- [x] 3.4 Install to a user-writable location with no `sudo`. Report it when it is not on `PATH`, and say how to add it — the shell-specific part is where these scripts usually get vague, and a wrong instruction is worse than a generic one.
- [x] 3.5 Replace an existing installation in place, so re-running is the upgrade path. Do not build a self-update command.
- [x] 3.6 Check for `openspec` on `PATH` and, when absent, name it and what breaks — promotion, which is the middle of the loop — as part of the result rather than after it. Do not install it.
- [x] 3.7 `set -eu`, quote everything, and work under `sh` rather than assuming `bash`. This runs on machines you cannot inspect.
- [x] 3.8 Do not add a `--version` flag, a `--prefix` flag, or an uninstaller unless something asks for them. The deliverable is one line that works.

## 4. `install.ps1`

- [ ] 4.1 The same shape for Windows: resolve the current release, download, verify, install to a user-writable location. **Blocked on `replace-hook-shell-form-with-exec-form`**, which is unapproved; the release carries the four POSIX targets until it lands.
- [ ] 4.2 Add the install location to the user's `PATH` — the user's, not the machine's — and say that a new shell is needed. Windows users have no `.profile` convention to fall back on, so a script that installs and says nothing about `PATH` has installed something they cannot run. **Blocked on `replace-hook-shell-form-with-exec-form`**, which is unapproved; the release carries the four POSIX targets until it lands.
- [ ] 4.3 The same `openspec` check and the same refusal to install it. **Blocked on `replace-hook-shell-form-with-exec-form`**, which is unapproved; the release carries the four POSIX targets until it lands.
- [ ] 4.4 Test on a real Windows machine or a Windows runner. A PowerShell script that has only been read is a PowerShell script that does not work. **Blocked on `replace-hook-shell-form-with-exec-form`**, which is unapproved; the release carries the four POSIX targets until it lands.

## 5. Publish the installers with the release

- [ ] 5.1 Upload `install.sh` and `install.ps1` as assets of every release. `install.sh` is uploaded by the `publish` job; `install.ps1` is blocked with section 4.
- [x] 5.2 Document fetching them from the current release, never from `raw.githubusercontent.com` on a branch. A branch URL pins every future install to an unreviewed commit.
- [ ] 5.3 Confirm the documented one-liner works end to end, from a machine with nothing installed, on at least one platform per operating system.

## 6. Say what is not signed and what is not supported

- [x] 6.1 State next to the download instructions that executables are unsigned, and that on macOS a browser download will be refused by Gatekeeper while the install script works. The asymmetry is invisible until someone hits it, and they conclude the software is broken.
- [x] 6.2 Name the five supported platforms and say that anything else builds from source. An unlisted platform is otherwise indistinguishable from a forgotten one.
- [x] 6.3 Record notarization as a declined non-goal with its reason — a paid developer account — rather than as an unmentioned gap.

## 7. The text this falsifies

- [x] 7.1 Rewrite the install sections of `README.md` and `docs/docs/quickstart.md`. `cargo install --path crates/cli` becomes the contributor's path, not the user's.
- [x] 7.2 Update the `README.md` paragraph that promises this: *"Distribution will move to prebuilt binaries (release workflow + install script), so user machines need neither Bun nor cargo."* It is a promise until this lands and stale prose the moment it does.
- [x] 7.3 Close the remaining half of the `docs/docs/roadmap.md` *"No release; setup is lossy"* row.
- [x] 7.4 Check whether `docs/docs/development/conventions.md#the-frontend`, which the README links for the distribution claim, needs the same edit.

## 8. Live verification

The whole change is a claim about machines the maintainer does not have. Unit tests cannot reach any of it.

- [ ] 8.1 Cut a real release and install it with the published one-liner on macOS.
- [ ] 8.2 Install it on a Linux distribution deliberately older than the build runner. This is the single check that justifies the musl decision.
- [ ] 8.3 Install it on Windows and run `openspec-doc doctor`, which is the command that would have exposed the Windows defects the previous change fixed. **Blocked on `replace-hook-shell-form-with-exec-form`**, which is unapproved; the release carries the four POSIX targets until it lands.
- [ ] 8.4 Verify every platform's binary serves the dashboard, which is what proves the embedded artifact survived the fan-out. A binary that runs but serves nothing is exactly what a broken dist handoff produces.
- [ ] 8.5 Install on a machine without `openspec` and confirm the result names it rather than reporting an unqualified success.
- [ ] 8.6 Download an asset through a browser on macOS and confirm Gatekeeper refuses it, so the caveat in 6.1 is written from observation rather than from repute.
