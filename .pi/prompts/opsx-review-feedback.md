Review sidecar feedback for an active OpenSpec change and intentionally route it back into Pi-driven artifact iteration.

**Input**: Optionally specify a change name (for example, `/opsx-review-feedback add-auth`). If omitted, infer from conversation context. If ambiguous, list active changes with `openspec list --json` and ask the user to choose one.

**Workflow**

1. Announce the selected change and how to override it.
2. Confirm the change is active with `openspec status --change "<name>" --json`.
   - If the change is missing or archived, stop and report that review companion feedback is only consumed for active changes.
3. Read sidecar feedback records from:

   ```txt
   .pi/openspec-review/comments/<change-name>.jsonl
   ```

4. If the file does not exist or contains no records, report: `No open review companion feedback is available for <change-name>.`
5. Parse each JSONL record. New records are event objects with `type: "comment" | "reply" | "status"`; older files may contain direct feedback records. Reconstruct each thread by comment id, apply the latest status event, and focus on comments whose status is absent or `open`. If records exist but none are open, report: `No open review companion feedback is available for <change-name>.` Also mention `addressed`, `resolved`, or legacy `sent-to-pi` records separately if present.
6. Summarize open comments by theme, severity, artifact path, selected text, anchor context, and thread replies where available. Include source comment ids in every grouped digest item.
7. For each digest group, include proposed updates to proposal, design, tasks, or spec deltas when the intent is inferable. Put ambiguous or question-like feedback in a separate unresolved questions section instead of inventing an artifact change.
8. Ask the user how they want to iterate `proposal.md`, `design.md`, `tasks.md`, or spec deltas. Do not apply changes automatically.
9. If the user asks to address feedback, use the normal OpenSpec artifact-editing workflow: read the relevant files, make intentional changes, and explain what changed.

**Important safety notes**

- Browser comments are feedback records only; they are not edits to OpenSpec artifacts.
- Applying feedback must be intentional and confirmed through Pi/user direction.
- Before editing any file under `openspec/`, present the proposed artifact updates and ask for explicit user approval.
- Do not mutate sidecar feedback status unless the user explicitly asks for status updates.
- Do not write under `openspec/` solely because a browser comment exists.

**Summary format**

```md
## Review Companion Feedback: <change-name>

Found <N> open feedback record(s).

### Themes
- <theme>: <summary>

### Records
- [<severity>] <artifactPath or general>: <body>
  - Context: <selectedText if present>

How would you like to iterate the OpenSpec artifacts?
```
