## Why

`openspec-doc` currently contains only repository guidance and an initialized OpenSpec folder, so contributors cannot install, run, test, or validate the intended CLI tool. Establishing the initial project setup creates a concrete, maintainable foundation for implementing OpenSpec scanning and review workflows.

## What Changes

- Add npm package metadata for an installable Node.js CLI package named `openspec-doc`.
- Add a TypeScript build setup with explicit source, test, and distribution boundaries.
- Add a CLI entrypoint with initial `summary` and `check` commands that operate read-only against a target project containing `openspec/config.yaml`.
- Add core scanner, parser, analyzer, and shared type modules for OpenSpec metadata.
- Add initial fixtures and automated tests for discovery, parsing, analysis, and command behavior.
- Add formatter/linter configuration and repository scripts for build, type-check, test, and CI-style validation.
- Document local development and initial CLI usage.

## Capabilities

### New Capabilities

- `project-setup`: Defines the package, tooling, source layout, development scripts, and installable CLI foundation for `openspec-doc`.
- `openspec-summary`: Defines read-only summary and health-check behavior for projects with an `openspec/` directory.

### Modified Capabilities

None.

## Impact

- Adds package and tooling files such as `package.json`, `tsconfig.json`, Vitest configuration, and formatter/linter configuration.
- Adds initial source under `src/` and tests under `tests/`.
- Adds fixtures for representative OpenSpec projects.
- Introduces npm dependencies for the CLI, TypeScript execution/building, testing, Markdown parsing where needed, and formatting/linting.
- Does not mutate target projects when running `summary` or `check`.
