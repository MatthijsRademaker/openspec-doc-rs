# Changelog

All notable changes to `openspec-doc` are documented here.

## [Unreleased]

## [0.1.1](https://github.com/MatthijsRademaker/openspec-doc-rs/compare/v0.1.0...v0.1.1) - 2026-09-24

### Features

- publish prebuilt binaries for macOS (arm64, x86_64) and Linux (x86_64, arm64, statically linked), and a one-line installer: `curl -fsSL https://github.com/matthijsrademaker/openspec-doc-rs/releases/latest/download/install.sh | sh`

### Bug Fixes

- preserve markdown source selection anchors ([#7](https://github.com/MatthijsRademaker/openspec-doc-rs/pull/7))

## [0.1.0](https://github.com/MatthijsRademaker/openspec-doc-rs/releases/tag/v0.1.0) - 2026-09-22

### Added

- Local review dashboard for OpenSpec projects, with anchored comments and phase verdicts.
- Claude Code and pi.dev hook bridge for delivering review feedback at turn boundaries.
- Versioned release automation driven by reviewed release pull requests.
- MIT licensing and source-tag installation documentation.

### Fixed

- Hook prompt handling now treats malformed hook arguments as non-blocking feedback failures rather than rejecting the user's prompt.

### Notes

- This first release carries source and release metadata only. Prebuilt binaries and installers belong to `add-release-binaries-and-installers`.
