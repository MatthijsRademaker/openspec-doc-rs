## Context

The existing review companion capability defines a browser mode for active OpenSpec changes. It captures browser feedback as sidecar records outside `openspec/` and routes actual OpenSpec artifact edits through Pi. This change deepens that model into document-style review: a user can select text in rendered proposal/design/task/spec artifacts, attach comments, continue a thread, and ask Pi to synthesize feedback into concrete proposal updates.

The implementation makes companion mode the default browser review experience while preserving an explicit read-only opt-out. The `serve` command also opens the default browser to the server's resolved local URL unless `--no-open` is provided, so automatic port selection still opens the correct address. Browser comment actions are available in companion mode, can be disabled with `--no-review-companion`, and never directly modify target OpenSpec artifacts.

## Goals / Non-Goals

**Goals:**
- Let users create comments anchored to selected text in active-change artifacts.
- Store enough anchor context to show comments later, relocate them after minor edits, or mark them as orphaned when relocation fails.
- Support threaded back-and-forth and review status without deleting the original comment history.
- Provide a Pi-invokable synthesis workflow that compiles open feedback into a review digest and proposed OpenSpec artifact changes.
- Keep browser writes limited to documented sidecar review storage.

**Non-Goals:**
- Real-time multi-user collaborative editing.
- Direct browser editing of `openspec/` proposal, design, task, or spec files.
- A full embedded Pi chat client in the browser for the first implementation slice.
- Perfect re-anchoring across arbitrary rewrites.
- Replacing upstream OpenSpec validation or proposal workflows.

## Decisions

### Store anchored comments as sidecar review records

Comment records remain outside the target project's `openspec/` directory, under the documented review sidecar storage path. The first durable format should be append-friendly JSONL or another simple local file format that can be read by both the server and Pi workflow.

Each record should include:
- stable comment/thread id
- change name
- artifact path and artifact kind
- selected text
- nearest heading path or rendered section identifier when available
- short before/after text context
- optional offsets or line/column information when available
- comment body
- author/source label when available
- created/updated timestamps
- status such as `open`, `addressed`, or `resolved`

Rationale: sidecar storage preserves the existing safety boundary and works without requiring a database.

Alternative considered: write inline Markdown comments into proposal files. This was rejected because it mutates OpenSpec artifacts from the browser and pollutes proposal content.

### Use best-effort anchor resolution with explicit states

The viewer should resolve comment anchors against the latest rendered artifact content using a layered strategy:
1. artifact path/change name match
2. nearest heading or rendered section match
3. exact selected-text match
4. selected text with before/after context
5. fallback to artifact-level orphaned comment

Resolved comments should expose an anchor state such as `exact`, `fuzzy`, `orphaned`, or `missing`.

Rationale: Markdown artifacts are expected to change during review. A best-effort model keeps comments useful while making uncertainty visible.

Alternative considered: rely only on character offsets. This was rejected because small Markdown edits would frequently break offsets.

### Treat comments and synthesis as separate artifacts

Raw comments are factual review history. Synthesized review digests are derived outputs that group comments by theme/artifact and propose changes.

A digest should include:
- source comment ids
- grouped issues or themes
- summary of user intent
- proposed artifact updates
- unresolved questions
- clear indication that artifact edits require explicit user approval

Rationale: keeping raw feedback separate from generated synthesis makes the workflow auditable and lets Pi regenerate better digests as context changes.

Alternative considered: update comments in-place with agent conclusions only. This was rejected because it blurs user input and generated interpretation.

### Keep agent back-and-forth Pi-routed first

The first implementation should support back-and-forth through Pi reading sidecar comments, asking clarifying questions, producing digests, and applying approved OpenSpec edits. Browser UI may show comment threads and statuses but does not need to host a live Pi chat.

Rationale: this matches the current agent workflow and avoids designing an agent transport/session protocol too early.

Alternative considered: embed live agent chat in the browser companion panel. This may be valuable later, but it introduces authentication, streaming, session, and command-execution concerns beyond the initial document-commenting value.

## Risks / Trade-offs

- Anchor relocation can be wrong after large rewrites → show anchor state visibly and preserve selected/surrounding text in the comment panel.
- Sidecar files can grow or conflict under concurrent writes → use append-friendly writes with unique ids and design for later compaction if needed.
- Synthesis may misinterpret terse comments → include source comment ids and have Pi ask clarifying questions before applying edits.
- Users may expect Google Docs-level collaboration → explicitly frame companion mode as local review feedback, not real-time collaborative editing.
- Browser feedback writes introduce mutation in the local viewer → make the mutation boundary visible, provide `--no-review-companion` for read-only sessions, and keep writes outside `openspec/`.
