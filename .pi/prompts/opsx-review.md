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

2. **Start or reuse the viewer**

   Start the local viewer for the current project when one is not already running:

   ```bash
   openspec-doc serve --root . --host 127.0.0.1 --port 4321
   ```

   If port `4321` is already serving this project, reuse it. If the port is occupied by something else, choose another available local port and report it.

   Keep the process running while the user reviews. Document that `Ctrl+C` stops a manually started viewer.

3. **Provide the review URL**

   When the active change name is known, provide the direct companion review URL:

   ```txt
   http://127.0.0.1:<port>/changes/<change-name>/companion
   ```

   If the change name cannot be confirmed, provide the active changes index instead:

   ```txt
   http://127.0.0.1:<port>/changes
   ```

4. **Explain review behavior**

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
Viewer: http://127.0.0.1:4321
Review: http://127.0.0.1:4321/changes/add-auth/companion

This review session keeps OpenSpec artifacts read-only from the browser. Browser comments are sidecar feedback records only; Pi applies proposal/design/spec/task updates only after explicit approval.
```

**Guardrails**
- Do not edit OpenSpec artifacts as part of starting review.
- Do not apply OpenSpec artifact edits from browser feedback without explicit user approval.
- Prefer reusing an existing local viewer when practical.
- If the selected change is missing or archived, pause and ask whether to pick another active change or open the changes index.
