# openspec-doc

`openspec-doc` is an installable OpenSpec documentation and review CLI for local projects.

The goal is to make OpenSpec repositories easier to review by scanning `openspec/` folders, summarizing current specs and changes, and reporting structural warnings that help keep OpenSpec content healthy.

## Current commands

The initial CLI supports inspection and browser review commands:

```bash
openspec-doc summary
openspec-doc summary --format json
openspec-doc check
openspec-doc serve
```

All commands discover `openspec/config.yaml` from the target project root. By default the target root is the current working directory; pass `--root <path>` to inspect another project.

```bash
openspec-doc summary --root ../some-project
openspec-doc check --root ../some-project
openspec-doc serve --root ../some-project
```

## Local development

Install dependencies:

```bash
npm install
```

Run the CLI locally from source by building first and then executing the built entrypoint:

```bash
npm run build
node dist/cli/index.js summary --root .
node dist/cli/index.js summary --root . --format json
node dist/cli/index.js check --root .
node dist/cli/index.js serve --root .
```

You can also link the package locally after building:

```bash
npm run build
npm link
openspec-doc summary --root .
```

## Validation scripts

The repository provides repeatable scripts for development and CI-style validation:

```bash
npm run format-check
npm run lint
npm run type-check
npm run build
npm run test
npm run validate
```

`npm run validate` runs format checking, linting, type checking, build, and tests.

## What the initial implementation scans

`openspec-doc` currently scans a target project for:

- `openspec/config.yaml`
- current specs under `openspec/specs/**/spec.md`
- active changes under `openspec/changes/<change-name>/`, excluding `archive/`
- archived changes under `openspec/changes/archive/**/`

The parser extracts:

- `## Purpose` sections
- `### Requirement:` headings
- `#### Scenario:` headings
- proposal, design, tasks, and spec-delta artifact presence for changes
- task checklist counts from `tasks.md`

The analyzer reports initial warnings for:

- current specs missing `## Purpose`
- requirements without scenarios
- malformed scenario heading levels
- active changes missing expected `proposal.md`, `design.md`, or `tasks.md`

## Browser viewer

`openspec-doc serve` starts a local browser viewer for a target OpenSpec project:

```bash
openspec-doc serve
openspec-doc serve --root ../some-project
openspec-doc serve --root ../some-project --host 127.0.0.1 --port 4321
openspec-doc serve --no-open
openspec-doc serve --no-review-agent
```

By default, `serve` uses the current working directory as the target root, binds to `127.0.0.1`, selects an available port when `--port` is omitted or set to `0`, and opens the browser to the resolved local URL. Pass `--no-open` to print the URL without opening a browser. On successful startup it prints the local URL, for example:

```txt
Serving OpenSpec viewer at http://127.0.0.1:4321
Opened browser at http://127.0.0.1:4321
Press Ctrl+C to stop.
```

The viewer renders overview, current specs, active changes, active change review pages, archived changes, and warnings pages. It consumes the same scanner, parser, and analyzer output as `summary` and `check`. The default `serve` command may write only review sidecar feedback records and lightweight Pi review session events outside `openspec/`; it does not create, modify, or delete OpenSpec artifacts.

Active change review pages are available at `/changes/<change-name>/review`. They collect the proposal, design, tasks, spec deltas, warnings, and artifact completion details into a review-oriented page, and link to the companion page for browser-native feedback.

### Review companion mode

Review companion mode is enabled by default for a one-window browser review experience:

```bash
openspec-doc serve
openspec-doc serve --root ../some-project
```

To force a browser session that rejects feedback writes, disable companion mode explicitly:

```bash
openspec-doc serve --no-review-companion
openspec-doc serve --root ../some-project --no-review-companion
```

When companion mode is enabled, active change review pages link to `/changes/<change-name>/companion`. The companion page shows proposal artifacts, review guidance, existing feedback records, a comment form, and an Ask Pi area for live review analysis. It is labeled as review companion mode and states that comments and Pi analysis are feedback records/guidance only; browser actions do not directly edit `proposal.md`, `design.md`, `tasks.md`, or spec delta files.

Browser Pi review analysis is enabled by default for companion-enabled serve sessions. It can be disabled explicitly while preserving sidecar comments:

```bash
openspec-doc serve --no-review-agent
```

Startup output reports whether Pi review analysis is enabled, disabled, or unavailable. Enabled sessions are non-mutating: the local server bundles the active change artifacts, analyzer warnings, and selected or open sidecar feedback, streams Pi's analysis back to the browser, and may persist lightweight session events under `.pi/openspec-review/sessions/`. If the Pi SDK, credentials, or model availability prevent startup, the companion page shows an unavailable state and existing comment creation/thread review remains available.

Browser feedback writes are user-initiated and scoped to append-only sidecar JSONL files outside `openspec/`:

```txt
.pi/openspec-review/comments/<change-name>.jsonl
```

Default `openspec-doc serve` accepts anchored feedback only for active changes with safe change names, known active-change artifact paths, and selected text that exists in the referenced artifact. It rejects archived or missing changes and keeps OpenSpec artifacts unchanged. `openspec-doc serve --no-review-companion` rejects feedback write requests entirely.

Each sidecar line is an event object. Comment events include a stable id, change name, artifact path/kind, selected text, nearest heading path when available, before/after text context, optional offsets, body, author/source, timestamps, and status (`open`, `addressed`, or `resolved`). Reply and status events append thread history without rewriting earlier comments:

```json
{"type":"comment","comment":{"id":"...","changeName":"add-core","artifactPath":"openspec/changes/add-core/proposal.md","anchor":{"artifactPath":"openspec/changes/add-core/proposal.md","selectedText":"Add core behavior.","headingPath":["Why"],"beforeText":"...","afterText":"..."},"body":"Clarify impact.","source":"browser","status":"open","createdAt":"...","updatedAt":"..."}}
{"type":"reply","reply":{"id":"...","commentId":"...","changeName":"add-core","body":"Agreed.","source":"browser","createdAt":"..."}}
{"type":"status","status":{"id":"...","commentId":"...","changeName":"add-core","status":"addressed","source":"browser","createdAt":"..."}}
```

Pi can still consume sidecar feedback with the prompt resource `.pi/prompts/opsx-review-feedback.md`. That workflow reads open feedback records, summarizes them into a digest with source comment ids, separates proposed artifact updates from unresolved questions, and requires explicit approval before intentionally iterating proposal, design, spec, or task artifacts through Pi. The browser Ask Pi workflow is complementary: it provides immediate in-browser analysis and follow-up discussion, but it does not expose an apply endpoint and never directly applies OpenSpec artifact edits.

## Intended product shape

The long-term shape is an npm-installable CLI that can be run from any project containing an `openspec/` directory:

```bash
npx openspec-doc serve
npx openspec-doc build
npx openspec-doc check
npx openspec-doc summary
npx openspec-doc export --format json
```

The default workflow should remain zero-config:

```bash
cd path/to/project-with-openspec
npx openspec-doc serve
```

Future changes can add browser preview, static build output, and richer OpenSpec-aware navigation on top of the scanner/parser/analyzer foundation.

## Architecture

The implementation is layered:

```txt
Target project with openspec/
        │
        ▼
CLI commands
        │
        ▼
Scanner
  - find openspec/config.yaml
  - find specs
  - find active changes
  - find archived changes
        │
        ▼
Parser
  - Purpose sections
  - Requirements
  - Scenarios
  - proposal/design/tasks files
        │
        ▼
Analyzer
  - counts
  - warnings
  - validation hints
        │
        ├──────────────▶ Browser viewer renderer/server
        │
        └──────────────▶ Future machine export
```

Source layout:

```txt
src/
  cli/
  core/
  server/
  viewer/
tests/
  fixtures/
```

Core modules return typed data and avoid printing user-facing CLI output. CLI commands handle rendering and exit codes. The viewer layer builds a browser model from core outputs, and the server layer serves generated HTML without mutating target projects.

## Non-goals for early versions

Out of scope unless explicitly proposed later:

- editing OpenSpec files in the browser
- replacing the upstream OpenSpec CLI
- hosted SaaS functionality
- project documentation unrelated to OpenSpec
- deep custom-schema support before the core workflow is useful
- AI chat/RAG functionality before reliable scanning and export exist

## OpenSpec workflow

Use OpenSpec for meaningful changes:

```bash
openspec list
openspec list --specs
openspec view
openspec validate <change-or-spec-name>
```

Create proposals before implementing non-trivial product or architecture decisions.

## Agent guidance

Coding agents should read `AGENTS.md` before making changes in this repository. It contains durable implementation guidance, safety notes, expected architecture, and validation expectations for this project.
