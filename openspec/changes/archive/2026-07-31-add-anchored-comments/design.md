## Context

Ported from `openspec-doc-rs-example/src/core/review-companion.ts` and `review-anchors.ts`. The TS version already treats `artifactPath` generically (a map of artifact path to markdown content) rather than hardcoding proposal/design/tasks — this change preserves that generality, since explore-phase scratch notes (`add-scratch-note-workflow`) must anchor comments identically to formal artifacts.

## Goals / Non-Goals

**Goals:**
- Match the TS anchor-resolution algorithm's exact/fuzzy/orphaned semantics, so a comment created against a scratch note continues to resolve correctly by content even after promotion renames the underlying file.
- Keep one comment sidecar per scope key (not per artifact file), matching the TS example's one-sidecar-per-change model, disambiguating by the `artifactPath` field on each record.
- Relocate the comment sidecar in lockstep whenever its artifact's scope key changes (session id to change name), so comment history survives promotion.

**Non-Goals:**
- No dashboard UI (`add-dashboard-html-views`).
- No live-push wiring — the dashboard server (`add-dashboard-server`) is responsible for noticing the sidecar file changed; this change only owns reading and writing it.

## Decisions

- **One JSONL sidecar per scope key**, not per artifact file. A single change may have multiple artifacts (`proposal.md`, `design.md`, `tasks.md`, spec deltas); one comment stream per change keeps review discussion in one place, disambiguated by each record's `artifactPath` field — matching the TS example exactly. Alternative considered: one sidecar per artifact file — rejected, fragments a single change's review discussion for no benefit.
- **Append-only file, events differentiated by a `type` field** (`comment`, `reply`, `status`) — crash-safe (no read-modify-write of prior records) and gives a natural audit trail, exactly the TS model.
- **A fourth `relocate` event carries the artifact rename**, rather than rewriting the `artifactPath` recorded in each prior anchor. An anchor names its artifact by path, so promotion would otherwise leave every scratch-note comment resolving as `missing` — and rewriting prior records to fix that would give up the append-only property. Replaying a `relocate` event during read-back remaps the affected anchors instead. Alternative considered: following the moved-pointer marker promotion leaves at the vacated path — rejected, it couples the comment reader to the scratch-note pointer format for no gain.
- **Anchor resolution order is exact → fuzzy (heading-scoped, then before/after text) → orphaned/missing**, ported directly rather than redesigned — it is already a reasonable, previously-exercised algorithm.

## Risks / Trade-offs

- [Risk] Fuzzy resolution can silently attach a comment to the wrong location if similar text appears multiple times in the same artifact → Mitigation: preserve the TS behavior of reporting anchor state (`exact`/`fuzzy`/`orphaned`) to the caller, so the dashboard can visually flag fuzzy/orphaned anchors rather than presenting them as confidently placed.
- [Risk] Relocating the sidecar in lockstep with promotion could race with an in-flight comment write → Mitigation: rely on filesystem rename atomicity — a write arriving mid-rename lands deterministically at whichever path existed at the moment of the write; no additional reconciliation is attempted beyond that.
