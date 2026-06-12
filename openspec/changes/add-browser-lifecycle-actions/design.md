## Context

The viewer currently has local HTTP APIs for review sidecar feedback and browser Pi review sessions. Lifecycle guidance is rendered as advisory copyable commands and explicitly avoids mutating OpenSpec artifacts or running lifecycle transitions. The user now wants strictly local browser buttons that can issue full lifecycle state changes.

This changes the safety posture from read-only guidance to local command/file mutation. The implementation must therefore keep command execution allowlisted, scoped to the served project root, and explicit in the UI.

## Goals / Non-Goals

**Goals:**

- Add browser buttons for lifecycle actions on dashboard, active change detail, review, and companion pages.
- Support full lifecycle actions in a local-first way: start exploration/proposal handoffs, continue/propose validation, task checkbox mutation, apply handoff, validation, and archive.
- Provide local API endpoints that execute only allowlisted actions against discovered active changes.
- Return structured action results with status, stdout/stderr or mutation details, and a refreshed viewer model when relevant.
- Preserve explicit confirmation before mutating files or running commands.

**Non-Goals:**

- Do not expose arbitrary shell command execution.
- Do not run a hosted/SaaS action API.
- Do not implement browser-side code generation or autonomous task implementation in this change.
- Do not bypass Pi/agent approval for generated code or artifact edits.
- Do not mutate archived changes except by the explicit archive action that moves an active change through OpenSpec.

## Decisions

### Add an allowlisted local lifecycle action API

Introduce endpoints under `/api/lifecycle-actions/...` rather than overloading feedback or review-agent APIs. The server validates that the requested change exists in the scanned active change list before accepting change-specific actions.

Initial action kinds:

- project actions: `explore`, `new-change-handoff`
- change actions: `validate`, `archive`, `apply-handoff`, `proposal-handoff`, `mark-task`

Command-running actions execute explicit command arrays, never interpolated shell strings. Non-command handoffs return copyable Pi/CLI text and mark the action as informational.

Rationale: users asked for full lifecycle buttons, but local safety still requires constrained action semantics. Alternatives considered: arbitrary command textbox or browser shell endpoint; rejected because it turns the viewer into a remote command execution surface.

### Keep apply/propose as handoff buttons, not autonomous implementation

The browser can start local validation/archive commands and mutate task checkboxes, but apply/propose remain handoff actions that provide a Pi prompt or CLI command. They do not autonomously edit proposal artifacts or code because implementation requires model/tool approvals and context that the static local server cannot safely provide.

Rationale: this satisfies the state-change button workflow where the browser can issue local state changes it owns while preserving explicit approval gates for generated edits. A future change can integrate a task runner or Pi SDK workflow if it defines approval semantics.

### Implement task checkbox mutation as structured file edit

Add a server utility to parse `tasks.md`, identify checklist entries by stable line index, and toggle `- [ ]`/`- [x]`. The API validates the line belongs to the active change's `tasks.md` artifact and writes only that file.

Rationale: task completion is the most common lifecycle state change and can be safely implemented as a targeted markdown edit. Alternatives considered: invoking `openspec` to mark tasks; no such existing command is assumed.

### Refresh model after mutating actions

After `mark-task` or successful `archive`, rebuild the viewer model immediately. If watch mode is enabled, this complements file watching; if watch mode is disabled, actions still update subsequent rendered pages.

Rationale: browser buttons should show accurate post-action state without requiring server restart.

### Require client confirmation and show local-only warning

Rendered buttons include data attributes for action kind, confirmation text, and target change/task. The browser script confirms mutating actions before POSTing, then displays action results inline.

Rationale: command execution and file mutation are intentional local operations, not passive documentation viewing.

## Risks / Trade-offs

- Local server bound beyond loopback could expose action endpoints on a LAN → default host remains `127.0.0.1`, UI labels actions local-only, and endpoints are allowlisted.
- Archive command may move files and change routes while a page is open → refresh model and report the new archive location/action result.
- Task line indexes can become stale if files change while a page is open → validate current line content before toggling and return a conflict if it no longer matches a checklist item.
- Apply/propose handoffs may not feel like full automation → this avoids hidden artifact/code generation without approval and can be expanded by a future explicit Pi workflow design.
