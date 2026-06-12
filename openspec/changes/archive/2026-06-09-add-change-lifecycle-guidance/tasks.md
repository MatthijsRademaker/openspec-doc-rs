## 1. Lifecycle model

- [x] 1.1 Add lifecycle phase types and guidance data structures for Explore, Propose, Review, Apply, and Archive.
- [x] 1.2 Implement project-level lifecycle guidance for no-active-change and active-change queue states.
- [x] 1.3 Implement active-change lifecycle inference from artifact presence, task counts, warnings, and archive state.
- [x] 1.4 Add unit tests for phase inference, reasons, and next-action output across representative fixtures.

## 2. Browser viewer integration

- [x] 2.1 Add dashboard lifecycle overview and active change lifecycle queue rendering.
- [x] 2.2 Add phase badges and primary next actions to active change index/list items.
- [x] 2.3 Add lifecycle guidance panels to active change detail and review pages.
- [x] 2.4 Ensure archived change pages avoid active next-action guidance and indicate lifecycle completion where useful.

## 3. Review companion integration

- [x] 3.1 Add lifecycle-aware guidance to the review companion header or guidance section.
- [x] 3.2 Show phase-specific review/apply/archive guidance without adding browser-side mutation endpoints.
- [x] 3.3 Preserve existing annotation, sidecar feedback, and Pi analysis behavior while lifecycle guidance is visible.

## 4. Command guidance and safety

- [x] 4.1 Render copyable command or Pi handoff snippets for phase-specific next actions.
- [x] 4.2 Add safety messaging that browser lifecycle guidance is advisory and non-mutating.
- [x] 4.3 Recommend validation before archive guidance and before marking implementation complete.

## 5. Validation

- [x] 5.1 Add or update render/model tests for dashboard, active change review, and companion lifecycle guidance.
- [x] 5.2 Run `openspec validate add-change-lifecycle-guidance`.
- [x] 5.3 Run relevant project validation scripts, including tests and type checking.
