## Context

`openspec-doc` now provides a read-only active-change review page and a Pi prompt that helps users open it. That flow is useful for visual review, but it still assumes the user keeps Pi/chat open separately to read review instructions, record feedback, and ask the agent to iterate artifacts. The next step is an explicit companion mode that brings review context and feedback capture into the same browser surface as the artifacts, while keeping Pi responsible for deciding how feedback becomes OpenSpec changes.

The main constraint is that default `serve` behavior must remain read-only. Any browser-side writes must be opt-in, clearly scoped to sidecar feedback records, and must not directly edit `openspec/` proposal/design/spec/task artifacts.

## Goals / Non-Goals

**Goals:**
- Provide a one-window review experience where artifacts, review guidance, feedback history, and comment capture are visible together.
- Add an explicit opt-in companion mode for active changes, separate from the default read-only viewer.
- Store browser comments as sidecar feedback records outside OpenSpec artifacts.
- Provide a Pi workflow that reads sidecar feedback and routes it into Pi for intentional iteration.
- Preserve clear non-mutating semantics for default `openspec-doc serve`.

**Non-Goals:**
- Browser-based editing of proposal, design, specs, or tasks.
- Automatic application of comments to OpenSpec artifacts without Pi/user confirmation.
- Hosted, multi-user, or synchronized review collaboration.
- Authentication or remote network exposure beyond local development use.
- Replacing Pi chat as the place where the agent reasons about and applies requested changes.

## Decisions

### Decision: Make companion mode explicit and opt-in

Add a dedicated route and startup option for writable review sidecars, for example `openspec-doc serve --review-companion` with `/changes/<change>/companion`. The existing `/changes/<change>/review` route remains read-only.

Alternatives considered:
- Add comment controls to every review page. This would surprise users because `serve` is documented as read-only.
- Add a separate `openspec-doc review` command immediately. This may be useful later, but a serve option keeps the implementation close to the current viewer while still being explicit.

### Decision: Use a split-pane companion layout to avoid two windows

The companion route should keep the proposal artifacts readable in the main pane and provide a persistent companion side panel for review instructions, selected artifact context, existing sidecar feedback, and a comment form. The panel should be available on the same browser page, not in a separate Pi/browser window.

Alternatives considered:
- Link users back to Pi for guidance. This keeps implementation simple but fails the one-window review requirement.
- Embed a full Pi chat UI. That would be significantly larger and depends on Pi extension APIs that are not needed for the first companion slice.

### Decision: Store feedback as append-only sidecar JSONL records

Use a sidecar path such as `.pi/openspec-review/comments/<change-name>.jsonl` in the inspected project. Each record should include an id, timestamp, change name, artifact path or section anchor when known, selected text when provided, severity/category, comment body, and status such as `open`, `sent-to-pi`, or `resolved`.

Alternatives considered:
- Store comments under `openspec/changes/<change>/review.md`. This keeps feedback near the change but mutates OpenSpec directories and risks accidental archival semantics.
- Store comments only in browser local storage. This avoids filesystem writes but is hard for Pi and CLI workflows to consume.
- Directly patch artifacts from the browser. This bypasses the OpenSpec/Pi iteration model and makes authorship unclear.

### Decision: Route feedback to Pi through an explicit workflow resource

Add or extend a Pi prompt, for example `/opsx-review-feedback <change>`, that reads sidecar records, summarizes open comments, and asks the user how to iterate proposal/design/spec/tasks. Marking records as sent/resolved can be handled by a later task or explicit status update endpoint; the first workflow should at minimum make sidecar feedback easy for Pi to consume.

Alternatives considered:
- Browser sends messages directly into Pi. This may be the long-term best UX through a Pi extension, but prompt-based consumption is simpler and works without extension lifecycle complexity.
- CLI automatically rewrites artifacts based on comments. This is too risky and removes the review conversation.

### Decision: Constrain writes to the sidecar directory

The server should reject companion write requests unless companion mode is enabled. When enabled, write endpoints should only append to the documented sidecar path for the selected active change. Requests should validate the change name and avoid path traversal.

Alternatives considered:
- Reuse generic file write helpers. This increases accidental write risk.
- Allow writing anywhere under `.pi/`. This is broader than needed and complicates safety review.

## Risks / Trade-offs

- [Risk] Users may assume comments automatically update artifacts. → Mitigate with clear copy: comments are feedback records until Pi applies changes intentionally.
- [Risk] Sidecar writes violate the mental model of `serve` as read-only. → Mitigate with an explicit `--review-companion` option and route labeling.
- [Risk] Long comments and many records make the page noisy. → Mitigate with filtering by status/category and compact summaries.
- [Risk] Local write endpoints could be misused if exposed on a public interface. → Mitigate by defaulting to localhost, documenting local-only use, and scoping writes tightly.
- [Risk] Pi prompt consumption may be less seamless than direct browser-to-Pi messaging. → Mitigate by treating prompt consumption as the first slice and reserving direct integration for a future Pi extension.
