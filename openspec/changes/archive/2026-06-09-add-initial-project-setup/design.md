## Context

The repository is an OpenSpec-initialized scaffold with product direction documented in `README.md` and `AGENTS.md`, but no package metadata, implementation source, tests, or runnable commands. The first setup change needs to turn the scaffold into an installable Node.js/TypeScript CLI package while preserving a layered architecture that can later support browser serving and static builds.

The initial implementation should prioritize read-only local usefulness: discovering an OpenSpec project, extracting enough metadata for summaries and health warnings, and exposing command behavior that can be validated in tests. More advanced rendering, serving, and custom schema support can build on this foundation later.

## Goals / Non-Goals

**Goals:**

- Establish a Node.js LTS, TypeScript, npm-distributed CLI foundation.
- Keep CLI, scanner, parser, analyzer, and output formatting separated by module boundaries.
- Provide initial `summary` and `check` commands that are useful against any project with `openspec/config.yaml`.
- Support explicit `--root <path>` target selection while defaulting to the current working directory.
- Add fixtures and automated tests for core scanning/parsing/analyzing and CLI command behavior.
- Add repeatable scripts for build, type-check, test, format/lint, and CI-style validation.

**Non-Goals:**

- Browser `serve` command, static `build` command, and rich UI rendering.
- Editing OpenSpec files or invoking upstream OpenSpec mutations.
- Full interpretation of every possible custom OpenSpec schema.
- Hosted service functionality or project documentation outside `openspec/`.

## Decisions

### Package and runtime

Use an npm package with ESM output, a `bin` entry named `openspec-doc`, and TypeScript sources compiled to `dist/`.

- Rationale: npm plus a bin entry matches the intended `npx openspec-doc ...` workflow and keeps distribution simple.
- Alternative considered: ship TypeScript directly with a runtime loader. This lowers build complexity but makes installation and startup less predictable for users.

### CLI framework

Use a small CLI command framework such as `commander` for command routing, help text, options, and exit behavior.

- Rationale: the project needs conventional command parsing but not a large application framework.
- Alternative considered: parse `process.argv` manually. This avoids a dependency but tends to duplicate help, validation, and option handling as commands grow.

### Source layout

Create the layered source layout recommended by repository guidance:

```txt
src/
  cli/
    index.ts
    commands/
      summary.ts
      check.ts
  core/
    scanner.ts
    parser.ts
    analyzer.ts
    types.ts
```

Add render/server/ui folders only when a future change needs them.

- Rationale: this keeps the first implementation focused while preserving boundaries for future `serve`, `build`, and `export` commands.
- Alternative considered: a single CLI file. This is faster initially but makes parser/analyzer tests and later reuse harder.

### Scanner behavior

The scanner accepts an explicit target root path, resolves it to an absolute path, and expects `openspec/config.yaml` beneath that root. It discovers:

- current spec files under `openspec/specs/**/spec.md`
- active change files under `openspec/changes/<change>/`, excluding `archive/`
- archived change files under `openspec/changes/archive/**/`

Missing `openspec/config.yaml` is a structured error for commands to render with an actionable message.

### Parser and analyzer behavior

The parser should use deterministic Markdown heading scanning rather than a heavyweight Markdown AST for the initial version. It should extract:

- spec name and path
- `## Purpose` text when present
- `### Requirement:` headings
- `#### Scenario:` headings
- change proposal/design/tasks file presence
- task checklist counts where straightforward

The analyzer should compute counts and warnings without failing the whole run on malformed files. Initial warnings include missing config, missing purpose, requirements without scenarios, malformed scenario heading levels, and active changes missing expected proposal/design/tasks files.

- Rationale: OpenSpec files are Markdown with predictable headings, and graceful degradation is more important than perfect Markdown rendering in this setup change.
- Alternative considered: use a full Markdown parser immediately. That may become useful for rendering later, but it is unnecessary for summary/check metadata and adds complexity before behavior stabilizes.

### Command behavior

`summary` prints a human-readable overview by default and optionally supports JSON output with `--format json`. It exits non-zero only for fatal project discovery errors.

`check` prints warnings and an overall status. It exits `0` when no warnings are found and `1` when warnings or fatal errors are found, making it useful in CI.

- Rationale: `summary` is an inspection command; `check` is a gate command.
- Alternative considered: make `summary` return non-zero for warnings. That would make casual inspection less ergonomic.

### Tooling

Use Vitest for tests, TypeScript for type checking/building, and Biome for formatting/linting.

- Rationale: this is a compact, explicit toolchain that aligns with repository guidance.
- Alternative considered: ESLint/Prettier/Jest. Those are mature but require more configuration for the same initial needs.

## Risks / Trade-offs

- Initial parser misses edge-case Markdown constructs → Keep parser output transparent, add fixtures for discovered cases, and avoid destructive behavior.
- CLI output shape changes as features grow → Test meaningful content and JSON schema-like fields rather than fragile formatting where practical.
- Dependency choices may need revision later → Keep dependencies small and document rationale in this design.
- `check` warnings may be too strict for existing OpenSpec projects → Start with actionable structural warnings and allow future changes to add severity/configuration.

## Migration Plan

This is an additive setup. Existing repository files remain valid. After implementation, contributors run `npm install`, then `npm run validate` or equivalent scripts to build, type-check, lint, and test.

No rollback strategy is required beyond reverting the added setup files because no target-project mutation is introduced.

## Open Questions

- Should the package eventually reserve the command name `specdoc` in addition to `openspec-doc`?
- Should JSON output be stabilized as a public schema in a dedicated future `export` capability?
