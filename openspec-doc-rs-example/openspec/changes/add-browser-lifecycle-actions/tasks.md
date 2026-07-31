## 1. Action model and server API

- [x] 1.1 Add lifecycle action types, result shapes, and allowlisted action definitions.
- [x] 1.2 Implement local `/api/lifecycle-actions` routing with active-change validation and unknown-action rejection.
- [x] 1.3 Implement command execution for validation and archive using explicit argv arrays in the served project root.
- [x] 1.4 Refresh the viewer model after successful mutating actions and expose failure details safely.

## 2. Task state mutation

- [x] 2.1 Add task checklist parsing metadata with stable line indexes and current checked state.
- [x] 2.2 Implement `mark-task` mutation that toggles only the requested checklist line in active change `tasks.md`.
- [x] 2.3 Reject stale, non-checklist, archived-change, or path-escaping task mutation requests.
- [x] 2.4 Add unit tests for task parsing and task mutation edge cases.

## 3. Browser lifecycle controls

- [x] 3.1 Render local lifecycle buttons alongside lifecycle guidance on dashboard, change detail, review, and companion pages.
- [x] 3.2 Add browser JavaScript for confirmation, POST requests, inline action results, and refresh behavior after successful mutations.
- [x] 3.3 Render task-level complete/incomplete buttons on active change pages where tasks are available.
- [x] 3.4 Preserve sidecar feedback, annotation, and Pi review analysis behavior while lifecycle controls are visible.

## 4. Handoff actions and safety messaging

- [x] 4.1 Implement Explore, Propose, and Apply handoff actions that return copyable local Pi/CLI guidance without mutating files.
- [x] 4.2 Update lifecycle safety copy to distinguish direct local actions from handoff-only actions.
- [x] 4.3 Ensure arbitrary command strings from browser requests are ignored or rejected.

## 5. Validation

- [x] 5.1 Add or update server and viewer tests for validation, archive, task mutation, handoffs, and rejected actions.
- [x] 5.2 Run `openspec validate add-browser-lifecycle-actions`.
- [x] 5.3 Run relevant project validation scripts, including tests and type checking.
