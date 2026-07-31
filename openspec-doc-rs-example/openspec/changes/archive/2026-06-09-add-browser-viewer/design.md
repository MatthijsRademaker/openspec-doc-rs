## Context

`openspec-doc` already has requirements for read-only OpenSpec project discovery, parsing, analysis, summary output, and structural warnings. The browser viewer should build on those core layers instead of re-scanning or parsing files in a UI-specific way. The initial viewer should favor a dependable server-rendered/static HTML approach over a complex client application so the same rendering path can later support both `serve` and `build`.

The viewer must not mutate target projects. It should render useful pages even when OpenSpec files are incomplete or malformed, surfacing analyzer warnings close to the relevant content.

## Goals / Non-Goals

**Goals:**
- Provide `openspec-doc serve` for local browser review of a target OpenSpec project.
- Reuse scanner, parser, and analyzer outputs as the viewer data source.
- Render overview, specs, active changes, archived changes, and warning details as navigable HTML.
- Preserve raw Markdown visibility while adding indexes, counts, and warning summaries.
- Keep renderer/server boundaries separate so static build output can reuse rendering later.

**Non-Goals:**
- Browser-based editing of OpenSpec files.
- Replacing the upstream OpenSpec CLI or its validation semantics.
- Hosted or multi-user service behavior.
- A rich single-page app unless later requirements justify it.
- Exhaustive support for arbitrary custom OpenSpec schemas in the first viewer.

## Decisions

1. **Use core analysis output as the viewer model.**
   - The renderer receives typed project summary, spec, change, archive, and warning data from the core layers.
   - Rationale: keeps OpenSpec knowledge centralized and avoids divergent CLI/UI behavior.
   - Alternative considered: parse files directly inside rendering code. Rejected because it duplicates parser/analyzer logic and makes warnings inconsistent.

2. **Implement an HTML renderer before a client-heavy UI.**
   - Initial pages are generated HTML with minimal client-side JavaScript, if any.
   - Rationale: supports local preview, static export, easier tests, and graceful rendering of malformed content.
   - Alternative considered: React/Vite SPA. Deferred until navigation complexity or interactivity requires it.

3. **Serve generated responses through a small local HTTP server.**
   - The `serve` command discovers the target root, builds the viewer model per request or startup, renders HTML, and opens/listens on a local port.
   - Rationale: avoids persistent output in target projects and aligns with read-only defaults.
   - Alternative considered: always write a temporary static directory. Deferred because direct rendering is simpler for first preview behavior.

4. **Treat Markdown rendering as content display, not the source of structure.**
   - Parsed headings/counts/warnings come from core parsing; Markdown-to-HTML is used to display proposal, design, task, and spec content.
   - Rationale: preserves user content while keeping navigation based on structured metadata.

5. **Use stable internal routes based on OpenSpec-relative paths and identifiers.**
   - Pages include routes for `/`, `/specs`, `/specs/<name>`, `/changes`, `/changes/<name>`, `/archive`, and `/warnings`.
   - Rationale: simple URLs make review and testing predictable.

## Risks / Trade-offs

- [Risk] Rendering malformed Markdown may produce imperfect HTML. → Mitigation: escape unsafe content, keep raw-source links/sections available, and show analyzer warnings.
- [Risk] A server-rendered viewer may feel less interactive than an SPA. → Mitigation: start with fast navigation and add client enhancements only when requirements demand them.
- [Risk] Large OpenSpec projects may become slow if rescanned on every request. → Mitigation: scan at startup for the first version; later add explicit refresh/watch behavior if needed.
- [Risk] Opening a browser automatically may be surprising in automation. → Mitigation: provide clear host/port output and make any browser-opening behavior opt-in or suppressible.
