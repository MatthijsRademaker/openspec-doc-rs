---
description: Start a Pi-assisted OpenSpec review in the browser viewer
---

Start a Pi-assisted OpenSpec review in the `openspec-doc` browser viewer.

**Input**: Optionally specify an active change name (e.g., `/opsx-review add-auth`). If omitted, infer from recent conversation when clear. If vague or ambiguous, inspect active changes and ask the user to select one.
**Provided arguments**: $@

**Steps**

1. **Select the active change**

   If a name is provided, use it. Otherwise:
   - Infer from conversation context if a single active change was just proposed or discussed.
   - Run `openspec list --json` to inspect active changes.
   - Auto-select when only one active change exists.
   - If multiple active changes exist, use the AskUserQuestion tool to let the user select which change to review.

   Announce: `Using change: <name>` and mention that `/opsx-review <other>` can review a different active change.

2. **Provide the review URL**

   Do not start, probe, or supervise a viewer. Each project has its own port, the turn-end hook brings a
   dashboard up on it, and picking a port on a conflict is a deterministic decision the binary makes. Ask
   it where the review is:

   ```bash
   openspec-doc serve url
   ```

   That prints the project's URL and whether a dashboard is serving it, and is correct either way. Append
   the change's path to it:

   ```txt
   <url>/changes/<change-name>
   ```

3. **Explain review behavior**

   Tell the user:
   - Starting review does not create, modify, or delete files under `openspec/`.
   - Review companion mode is on by default and stores browser comments only as sidecar feedback records under `.pi/openspec-review/comments/*.jsonl`.
   - Browser comments do not directly edit OpenSpec artifacts; requested proposal/design/spec/task changes are routed back through Pi and require explicit user approval.
   - If they want a browser session that rejects feedback writes, start the viewer with `--no-review-companion`.

**Output**

Summarize:
- Change selected
- Viewer URL
- Direct companion review URL
- OpenSpec artifact safety behavior
- Where browser feedback is stored and how Pi consumes it

Example:

```txt
Using change: add-auth
Viewer: http://127.0.0.1:4323 (serving)
Review: http://127.0.0.1:4323/changes/add-auth

This review session keeps OpenSpec artifacts read-only from the browser. Browser comments are sidecar feedback records only; Pi applies proposal/design/spec/task updates only after explicit approval.
```

**Guardrails**
- Do not edit OpenSpec artifacts as part of starting review.
- Do not apply OpenSpec artifact edits from browser feedback without explicit user approval.
- Prefer reusing an existing local viewer when practical.
- If the selected change is missing or archived, pause and ask whether to pick another active change or open the changes index.
