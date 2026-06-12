## 1. Review Viewer UX

- [x] 1.1 Add a review-oriented active change route or section that renders proposal, design, tasks, spec deltas, warnings, and artifact completion details for an active change.
- [x] 1.2 Add navigation from active change detail pages to the review-oriented view.
- [x] 1.3 Add copy that explains feedback should be provided in Pi for now and that browser-native comments are reserved for a later review companion mode.
- [x] 1.4 Add rendering tests for the review route, active-change navigation link, and read-only/no-comment-control behavior.

## 2. Pi Workflow Integration

- [x] 2.1 Add a Pi-invokable review workflow resource, such as `.pi/prompts/opsx-review.md`, that accepts an active change name or helps identify one.
- [x] 2.2 Make the workflow start or reuse `openspec-doc serve` for the current project and provide a direct active-change review URL when a change name is known.
- [x] 2.3 Ensure the workflow states that starting review is non-mutating and comments should be sent in Pi for this slice.
- [x] 2.4 Document the later review companion direction in the workflow or adjacent docs, including sidecar comments and Pi-routed iteration as future work.

## 3. Validation

- [x] 3.1 Run `openspec validate add-pi-assisted-review-flow` and fix any proposal/spec/task issues.
- [x] 3.2 Run the relevant project validation scripts for changed code and tests.
