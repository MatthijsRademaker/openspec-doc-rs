## 1. Package and Tooling Setup

- [x] 1.1 Create `package.json` with package metadata, `bin` entry, package files, Node engine, ESM type, and scripts for build, type-check, test, lint, format-check, and aggregate validation
- [x] 1.2 Add TypeScript configuration for source compilation to `dist/` and no-emit type checking
- [x] 1.3 Add Vitest configuration and test setup conventions
- [x] 1.4 Add Biome configuration for formatting and linting
- [x] 1.5 Add `.gitignore` entries for dependencies, build output, coverage, and local environment files

## 2. Core Types and Scanner

- [x] 2.1 Create shared core types for scanned projects, parsed specs, parsed changes, summaries, warnings, and fatal errors
- [x] 2.2 Implement target root resolution and `openspec/config.yaml` discovery
- [x] 2.3 Implement recursive discovery for current specs under `openspec/specs/**/spec.md`
- [x] 2.4 Implement active change discovery under `openspec/changes/<change-name>/` excluding `archive/`
- [x] 2.5 Implement archived change discovery under `openspec/changes/archive/**/`

## 3. Parser and Analyzer

- [x] 3.1 Implement Markdown section parsing for `## Purpose`
- [x] 3.2 Implement requirement and scenario heading parsing with requirement-to-scenario association
- [x] 3.3 Implement active change artifact parsing for proposal, design, tasks, and spec delta presence
- [x] 3.4 Implement task checklist counting for change task files
- [x] 3.5 Implement analyzer summary counts for specs, changes, requirements, scenarios, tasks, and warnings
- [x] 3.6 Implement structural warnings for missing purpose, requirements without scenarios, malformed scenario heading levels, and active changes missing expected artifacts

## 4. CLI Commands

- [x] 4.1 Create CLI entrypoint and command registration with help text and version output
- [x] 4.2 Implement shared `--root <path>` handling for initial commands
- [x] 4.3 Implement `summary` command with human-readable output
- [x] 4.4 Implement `summary --format json` output as valid JSON
- [x] 4.5 Implement `check` command with warning output and exit code `0` for pass, `1` for warnings or fatal errors
- [x] 4.6 Ensure `summary` and `check` do not mutate target project files

## 5. Fixtures and Tests

- [x] 5.1 Add fixture project with minimal valid OpenSpec config and spec content
- [x] 5.2 Add fixture project with active and archived changes
- [x] 5.3 Add fixture project with malformed or incomplete specs for warning coverage
- [x] 5.4 Add scanner unit tests for discovery and missing config behavior
- [x] 5.5 Add parser and analyzer unit tests for metadata extraction, counts, and warnings
- [x] 5.6 Add CLI tests or smoke tests for `summary`, `summary --format json`, and `check` exit behavior

## 6. Documentation and Validation

- [x] 6.1 Update `README.md` with dependency installation, development scripts, build steps, and local CLI usage
- [x] 6.2 Run formatter/linter checks and fix reported issues
- [x] 6.3 Run type checking and fix reported issues
- [x] 6.4 Run automated tests and fix reported failures
- [x] 6.5 Run the aggregate validation script
- [x] 6.6 Run `openspec validate add-initial-project-setup` and fix any OpenSpec artifact issues
