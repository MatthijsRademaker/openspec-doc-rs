## Context

`openspec-doc` already scans OpenSpec projects, renders dashboard/change/review/companion pages, stores sidecar review feedback, and provides Pi review analysis without directly mutating OpenSpec artifacts. The current UI is artifact-oriented: it shows what exists, warnings, tasks, and comments. The requested expansion is workflow-oriented: help users understand and move through the OpenSpec lifecycle from early exploration through proposal, review, implementation, and archiving.

The initial implementation should fit the existing read-only browser model. It can recommend commands and Pi workflows, but it should not add browser endpoints that run `openspec archive`, apply code changes, or edit proposal artifacts automatically.

## Goals / Non-Goals

**Goals:**

- Present OpenSpec work as Explore → Propose → Review → Apply → Archive across dashboard, active change detail/review pages, and review companion pages.
- Derive lifecycle state from data already scanned where possible: active changes, artifact presence, task counts, warnings, and archived changes.
- Show concise next actions with copyable command snippets and safety notes for each phase.
- Connect browser review feedback and Pi analysis to lifecycle decisions without bypassing explicit user approval.
- Keep the guidance useful even when projects are partially scaffolded or malformed.

**Non-Goals:**

- Do not implement browser-side artifact editing.
- Do not execute `openspec new`, proposal generation, task application, validation, or archive commands from the browser.
- Do not replace upstream OpenSpec CLI workflows or Pi agent skills.
- Do not require sidecar feedback to exist before a change can be considered ready for implementation.
- Do not introduce a new persistent lifecycle database; lifecycle guidance should be derived from scanned state for this change.

## Decisions

### Derive lifecycle guidance from existing project state

Add a small lifecycle guidance model that can be computed from `ViewerModel` and `ViewerChange` rather than persisted separately. The model should expose:

- a stable phase id: `explore`, `propose`, `review`, `apply`, or `archive`
- a user-facing label and summary
- reasons for the phase decision
- recommended next actions
- command snippets for the user to run outside the browser
- safety notes, especially around read-only behavior and approval gates

Rationale: derived state keeps the feature zero-config, works across any OpenSpec project, and avoids creating another source of truth. Alternatives considered: storing explicit lifecycle state in sidecar files or mutating `.openspec.yaml`; both add synchronization risk and are unnecessary for initial guidance.

### Use heuristic, explainable phase detection

For active changes, phase detection should remain transparent and conservative:

- `propose` when proposal/design/tasks/spec deltas are incomplete or analyzer warnings indicate missing change artifacts.
- `review` when artifacts are present but tasks have not clearly started, or review/warning attention is still useful before implementation.
- `apply` when tasks exist and at least one task is incomplete, indicating implementation work remains.
- `archive` when tasks exist, all tasks are complete, required artifacts are present, and no active-change warnings are present.

For the project dashboard, show an `explore` entry point when there are no active changes and a lifecycle queue when active changes exist.

Rationale: users need guidance, not hidden automation. The UI should always show the reasons so users can override judgment manually. Alternatives considered: invoking `openspec status` per change for authoritative schema status; useful later, but the current scanner is designed to work without shelling out to OpenSpec for every render.

### Render command guidance, not command execution

Lifecycle surfaces should provide copyable command snippets such as:

- `openspec list`
- `openspec validate <change-name>`
- Pi/agent prompts like “explore this idea”, “propose a change”, or “apply <change-name>”
- `openspec archive <change-name>` when archive-ready
- existing `openspec-doc serve` URLs and review companion links

Rationale: this preserves the existing safety model. Browser pages remain local documentation/review surfaces and do not perform irreversible transitions. Alternatives considered: adding POST endpoints for archive/apply; those are higher-risk and should be proposed separately.

### Integrate guidance into existing pages first

Add lifecycle sections to:

- dashboard: lifecycle overview, no-active-change Explore prompt, and active change lifecycle queue
- active changes index/list items: phase badges and next-action summaries
- active change detail/review: lifecycle panel near the top
- review companion: phase-aware review guidance alongside annotation guidance

Rationale: this makes the lifecycle visible where users already work. A separate `/lifecycle` route can be added later if the embedded surfaces become too dense.

### Keep lifecycle guidance independent from sidecar feedback internals

The initial guidance may mention open sidecar feedback and review companion usage but should not require full comment status aggregation to compute readiness. If comment summaries are already available in render context later, they can enrich the guidance.

Rationale: review companion feedback is append-only event data and not currently part of the core scanner/analyzer model. Tying lifecycle phase detection to that storage would increase scope.

## Risks / Trade-offs

- Phase heuristics may not match every team's exact OpenSpec workflow → show reasons and command suggestions instead of enforcing state transitions.
- Adding lifecycle copy to many pages can clutter the UI → keep panels concise, use badges, and link users to relevant pages for details.
- Archive readiness could be overstated if implementation validation was run outside the app and failed → recommend validation commands before archive and treat archive guidance as advisory.
- Users may expect browser buttons to perform propose/apply/archive → label command snippets clearly and state that browser guidance is non-mutating.
