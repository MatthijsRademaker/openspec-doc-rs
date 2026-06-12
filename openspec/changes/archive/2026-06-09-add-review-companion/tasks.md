## 1. Companion Mode Server and Storage

- [x] 1.1 Add an explicit review companion enablement option, such as `openspec-doc serve --review-companion`, while keeping default `serve` read-only.
- [x] 1.2 Add sidecar feedback storage for active changes outside `openspec/`, for example `.pi/openspec-review/comments/<change-name>.jsonl`.
- [x] 1.3 Add server endpoints for listing and appending feedback only when companion mode is enabled.
- [x] 1.4 Validate change names and reject missing, archived, or path-traversal feedback write requests.
- [x] 1.5 Add tests proving default serve rejects feedback writes and companion mode writes only to sidecar storage.

## 2. One-Window Companion UI

- [x] 2.1 Add an active change companion route or mode that shows artifacts and a persistent companion panel on one browser page.
- [x] 2.2 Show review guidance, existing feedback, and feedback capture controls in the companion panel without requiring a separate Pi/chat window for reading review context.
- [x] 2.3 Add navigation from active change review pages to companion mode only when companion mode is enabled.
- [x] 2.4 Ensure the companion page clearly states that comments are feedback records and do not directly edit proposal/design/spec/task artifacts.
- [x] 2.5 Add rendering tests for the split-pane layout, guidance visibility, feedback list, and companion navigation behavior.

## 3. Pi Feedback Workflow

- [x] 3.1 Add a Pi-invokable workflow resource, such as `.pi/prompts/opsx-review-feedback.md`, for reading sidecar feedback for an active change.
- [x] 3.2 Make the workflow summarize open comments and ask the user how to iterate proposal/design/spec/tasks.
- [x] 3.3 Ensure the workflow states that feedback application is intentional and not automatic browser editing.
- [x] 3.4 Document the companion flow in README or adjacent docs, including the one-window browser review experience and sidecar storage path.

## 4. Validation

- [x] 4.1 Run `openspec validate add-review-companion` and fix any proposal/spec/task issues.
- [x] 4.2 Run relevant project validation scripts for changed code and tests when implementation begins.
