# Design

## One dashboard, five binaries

```text
   ┌────────────────────────────────┐
   │ frontend                       │  bun install --frozen-lockfile
   │   → web/dist  (one artifact)   │  bun run build
   └────────────────┬───────────────┘
                    │ downloaded by every leg
   ┌────────┬───────┼────────┬─────────┬──────────┐
   ▼        ▼       ▼        ▼         ▼
 macos    macos   linux    linux    windows
 arm64    x86_64  x86_64   arm64    x86_64
                  musl     musl      msvc
   │        │       │        │         │
   └────────┴───────┴────┬───┴─────────┘
                         ▼
              archives + checksums
                         ▼
              upload to the draft release
                         ▼
              undraft → becomes "latest"
```

The single frontend build is the part of this that is a design decision rather than a build script. Everything else is mechanical.

Five per-leg Bun builds would produce five `web/dist` directories from one commit. Whether they would differ is genuinely unknown — a type-checker, a bundler, and a dependency tree resolved per runner are three places for it to happen, and nobody has measured it. That is the point: the fan-out means it does not have to be measured. The project gets to say "every platform ships the same dashboard" as a fact about the artifact graph instead of a hope about build determinism, and it costs one artifact upload.

`add-continuous-verification` already proved this shape on the gate run, deliberately, so that the first time it carries something users receive it is not also the first time it has run.

## Linux is musl, and only musl

A `curl | sh` installer's entire job is working on a machine you know nothing about. A glibc-linked binary built on the CI runner's image carries that image's glibc version floor into every user's environment, and the failure it produces — runs on the maintainer's distribution, dies on an older one — arrives as a bug report the maintainer cannot reproduce.

Static musl removes the class. It is unusually cheap here: every workspace dependency is pure Rust — `axum`, `tokio`, `notify`, `rust-embed`, `pulldown-cmark`, `dirs`, `uuid`, `chrono` — so there is no C library to find a musl build of.

Shipping both gnu and musl was rejected. It does not remove the decision, it relocates it into the installer, which would then have to detect the running libc — the hardest thing in the script, on the platform with the most variation, exercised only on machines the maintainer does not have. One artifact per architecture is one thing to test.

The cost, stated: statically linked Rust binaries on musl historically had a slower allocator than glibc. This process spends its life waiting on a filesystem watcher and an HTTP socket, so it is not a plausible concern here — but it is the kind of thing that gets discovered later and treated as a mystery, so it belongs written down.

## Draft until the assets exist

The installer will resolve `/releases/latest/download/…`, which is what lets it fetch without knowing a version number. That URL resolves to the most recent non-draft, non-prerelease release.

So a release created before its matrix finishes is current, and empty, for as long as the matrix runs — ten or fifteen minutes during which every `curl | sh` in the world resolves to a release with no binaries in it. Not a hypothetical: it is the normal case, every release, unless something prevents it.

Creating the release as a draft and undrafting it after the last asset uploads makes "this release is published" and "this release has binaries" the same event. The undraft step becomes the release's commit point, which is also a useful property when a build leg fails: a failed matrix leaves a draft nobody can install rather than a published release missing one platform.

`add-release-automation` deliberately did not draft. A draft protecting nothing is complexity with no argument behind it, and the argument arrives exactly here.

## The installer is a release asset, not a branch file

`raw.githubusercontent.com/…/main/install.sh` is the familiar shape and it is wrong. It pins every future installation to the tip of an unreviewed branch: a bad push breaks installs for everyone, immediately, and keeps breaking them until someone reverts it. The blast radius of a typo in a shell script becomes every new user in the interval.

`/releases/latest/download/install.sh` pins it to the last release instead. The user-facing one-liner is identical in length and shape; the installer only moves when a release moves, through the same review the release went through.

This makes the installer part of the release artifact set rather than repository content that happens to be executable, which is also the honest description of what it is.

## What the installer must refuse to lie about

`crates/core/src/scratch/promote.rs:110` shells out to `openspec validate`. The `openspec` CLI is a hard runtime dependency for promotion — the step where an exploration becomes a change, which is the middle of the loop this tool exists to support, not an edge feature.

An installer that fetches a binary, puts it on `PATH`, prints a checkmark, and exits has installed something whose central workflow fails the first time it is used. This repository's fifth rule is *"invalid states must fail loudly. Do not hide errors with silent defaults, swallowed exceptions, fake success values, or fallback behavior."* A green checkmark over a half-satisfied dependency graph is that rule broken at the front door, on the first interaction a user has with the project.

So: detect `openspec` on `PATH`, and when it is absent say so as part of the result rather than after it, naming what will fail. Do not install it — it is a Node tool with its own package manager and its own opinions, and an installer that starts bootstrapping other ecosystems has stopped being one-shot and is now a thing that modifies your machine in ways you did not ask about.

There is a second thing not to lie about, and it is smaller: `install.sh` should verify the checksum it downloaded. Not because the threat model is compelling on a `curl | sh` install — the script itself came from the same place — but because a truncated download is a real and common failure, and "the binary is corrupt" is a much better message than whatever a half-downloaded ELF file produces.

## macOS signing, declined with its consequence

Binaries fetched by `curl` carry no `com.apple.quarantine` attribute and run normally. The same file downloaded through a browser gets one, and Gatekeeper blocks it with a dialog offering no obvious way forward.

So the install script works, and the releases-page download does not, on the platform where a user is most likely to try the second one. Notarization requires a paid Apple developer account and an annual renewal.

Declining is the right call for a local-only tool with one maintainer. Declining *silently* is not: the asymmetry is invisible until someone hits it, and the person who hits it concludes the software is broken rather than unsigned. The caveat goes next to the download instructions, where the person about to do the wrong thing is looking.

## Which platforms, and saying so to the ones left out

Five targets. macOS on both architectures because Rosetta is not a distribution strategy; Linux on both because arm64 servers and single-board machines are ordinary now and public repositories get arm64 runners free; Windows on x86_64.

Everything else — 32-bit targets, Windows on ARM, the BSDs — is out, and the omission should be a sentence rather than an absence. A user on an unsupported platform currently reads install instructions that do not mention them and cannot tell whether they were considered and excluded or simply forgotten. "Build from source; these five are what CI produces" costs one line and answers it.
