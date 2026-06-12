# AGENTS.md

Purpose: operating instructions for coding agents working in this repository. This repository is intended to become an installable, reusable OpenSpec documentation and review tool for coding projects.

## Project purpose

`openspec-doc` provides a browser-oriented documentation/review experience for projects that use OpenSpec. The tool should make it easy to inspect current specs, active changes, archived changes, requirements, scenarios, validation warnings, and project-level OpenSpec health from any repository that contains an `openspec/` directory.

The long-term product shape is an installable CLI package, for example:

```bash
npx openspec-doc serve
npx openspec-doc build
npx openspec-doc check
npx openspec-doc export --format json
```

The first versions should prioritize local usefulness over platform breadth:

- zero-config discovery of `openspec/config.yaml`
- readable browser UI for specs and changes
- generated overview pages
- OpenSpec-aware warnings and summaries
- static build output for CI or publishing

## Repository status

This repository is currently a scaffold. Agents should expect missing application code, package metadata, and implementation details until an OpenSpec change defines them.

OpenSpec has been initialized in this repository. Use OpenSpec proposals for non-trivial product, architecture, or behavior changes.

## Scope

In scope:

- CLI for reviewing OpenSpec documentation across projects
- scanning and parsing `openspec/` directories
- rendering current specs, active changes, and archived changes
- local browser preview server
- static documentation build output
- validation/status summaries
- JSON export of parsed OpenSpec metadata
- future AI/RAG integration points, if proposed explicitly

Out of scope for early versions unless explicitly proposed:

- editing specs in the browser
- replacing the upstream OpenSpec CLI
- hosted SaaS functionality
- project-specific documentation unrelated to OpenSpec
- deep support for every custom OpenSpec schema before core flows work

## Expected architecture

Prefer a layered architecture so the CLI, parser, analyzer, and UI can evolve independently.

```txt
                  target project
                with openspec/ folder
                         │
                         ▼
┌────────────────────────────────────────────────────┐
│                    CLI layer                       │
│  commands: serve, build, check, summary, export    │
└───────────────────────┬────────────────────────────┘
                        ▼
┌────────────────────────────────────────────────────┐
│                  scanner layer                     │
│  discovers config, specs, active changes, archive  │
└───────────────────────┬────────────────────────────┘
                        ▼
┌────────────────────────────────────────────────────┐
│                   parser layer                     │
│  extracts Purpose, Requirements, Scenarios, tasks  │
└───────────────────────┬────────────────────────────┘
                        ▼
┌────────────────────────────────────────────────────┐
│                  analyzer layer                    │
│  counts, warnings, validation hints, relationships │
└───────────────┬───────────────────────┬────────────┘
                ▼                       ▼
┌──────────────────────────┐  ┌──────────────────────┐
│       renderer/UI         │  │     machine export   │
│ browser docs/static site  │  │ JSON for tools/AI    │
└──────────────────────────┘  └──────────────────────┘
```

### Recommended source layout

When implementation begins, prefer a structure like:

```txt
src/
  cli/
    index.ts
    commands/
      serve.ts
      build.ts
      check.ts
      summary.ts
      export.ts
  core/
    scanner.ts
    parser.ts
    analyzer.ts
    types.ts
  render/
    markdown.ts
    site.ts
  server/
    dev-server.ts
  ui/
    # optional browser app if/when custom UI is needed

tests/
  fixtures/
    minimal-openspec/
    archived-changes/
    invalid-specs/
```

Do not create this layout until an implementation change requires it, but keep the architecture in mind when proposing or implementing work.

## Product principles

- **Zero-config first:** running from a project root should work if `openspec/config.yaml` exists.
- **Do not mutate target projects by default:** `serve`, `summary`, `check`, and `export` should be read-only unless a command explicitly documents generated output.
- **OpenSpec-aware, not just Markdown:** preserve raw Markdown rendering, but add value through indexes, requirement counts, scenario navigation, archive timelines, and validation warnings.
- **Portable across repos:** avoid assumptions from any single project.
- **Small core, extensible edges:** keep scanning/parsing/analyzing independent from the chosen UI renderer.
- **Graceful degradation:** malformed specs should produce clear warnings and still render where possible.

## OpenSpec expectations

Use the OpenSpec workflow for substantial changes:

- proposals should describe user-facing CLI behavior and target project impact
- designs should call out architecture, package format, and command behavior
- specs should define observable CLI/UI behavior
- tasks should include validation steps

Before implementing an OpenSpec change, read the relevant proposal/design/tasks/specs under `openspec/changes/<change-name>/`.

Useful commands:

```bash
openspec list
openspec list --specs
openspec validate <change-or-spec-name>
openspec view
```

## Setup expectations

The implementation stack has not been selected yet. Until a proposal decides otherwise, assume the likely baseline is:

- Node.js LTS
- TypeScript
- npm package distribution
- Vitest for tests
- Biome or a similarly explicit formatter/linter

Once package metadata exists, prefer repository scripts over ad-hoc commands. Typical future commands may include:

```bash
npm install
npm run build
npm run test
npm run type-check
npm run biome:ci
```

Do not invent persistent tooling or dependency choices without capturing the decision in OpenSpec first.

## Coding guidelines

- Keep changes small and focused.
- Prefer explicit types and clear module boundaries.
- Separate filesystem scanning, Markdown/OpenSpec parsing, analysis, and rendering.
- Avoid hidden global state; pass target root/config explicitly.
- Keep CLI output scriptable where practical.
- Treat file paths carefully and support projects outside this repository.
- Provide actionable errors with paths and suggested fixes.
- Add fixtures for parser/analyzer behavior instead of relying only on this repo's own `openspec/` folder.

## Validation guidelines

Validation should match the files changed:

- documentation-only change: review rendered Markdown if a docs renderer exists
- parser/analyzer change: run unit tests against fixtures
- CLI change: run command-level tests or smoke checks
- UI/rendering change: build and run relevant UI tests when available
- OpenSpec artifact change: run `openspec validate <change-or-spec-name>`

If validation cannot be run because tooling is not yet scaffolded, state that clearly in the final response.

## Safety notes

- Do not commit secrets or personal environment files.
- Do not modify unrelated projects while developing this package.
- Do not delete archived OpenSpec changes unless explicitly asked.
- Be careful with generated output directories; document them and add ignores when implementation introduces them.
- Avoid broad rewrites of OpenSpec artifacts unless the task is specifically about normalization or migration.

## Naming notes

The working repository name is `openspec-doc`. If package naming changes later, capture the decision in OpenSpec and update docs/config consistently.
