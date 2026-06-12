## Context

`openspec-doc` already has a read-only browser viewer and this repository already provides Pi prompt templates for OpenSpec flows. The missing link is a convenient review entrypoint: after a proposal is created, the user should not need to manually remember how to start the viewer, find the random port, and navigate to the relevant active change page.

This first slice should remain deliberately small. The viewer continues to be read-only and comments remain in Pi/chat for now. The design should nevertheless preserve a clear path toward the later "review companion" implementation where browser comments are stored as sidecar review feedback and routed back into Pi for artifact iteration.

## Goals / Non-Goals

**Goals:**
- Provide a Pi-facing workflow that starts or reuses the local `openspec-doc` viewer for the current project.
- Link directly to an active change review URL when a change name is known.
- Make active change pages more review-oriented without adding browser write controls.
- Keep the default `openspec-doc serve` command read-only.
- Document future extension points for browser comment capture and Pi feedback routing.

**Non-Goals:**
- Browser-based editing of OpenSpec artifacts.
- Browser comment submission in this first implementation slice.
- Replacing `/opsx-propose` or the upstream OpenSpec CLI.
- Hosted or remote review workflows.

## Decisions

### Decision: Add a Pi workflow resource rather than coupling proposal creation to `serve`

Create a Pi-facing workflow entrypoint, such as a prompt template `/opsx-review`, that can be invoked after or alongside `/opsx-propose`. The workflow starts `openspec-doc serve`, reports the viewer URL, and directs the user to review a specific active change.

Alternatives considered:
- Automatically modify `/opsx-propose` to always start the server. This is convenient, but surprising for users who only want artifacts and no browser process.
- Add a dedicated CLI command only. This helps non-Pi users but does not provide the desired Pi workflow integration by itself.
- Build a full Pi extension immediately. This is the right long-term direction for lifecycle management and browser-to-Pi feedback, but it is larger than the first review slice.

### Decision: Use the existing viewer server in read-only mode for the first slice

The initial review workflow should call the existing `serve` behavior and depend on its read-only guarantees. It may compute and display a direct route such as `/changes/<change-name>` or `/changes/<change-name>/review`, but the browser page itself must not write files.

Alternatives considered:
- Introduce `openspec-doc review` immediately. This is likely appropriate when comments are writable, but premature while review is only navigational.
- Add POST endpoints to the existing server now. That would violate the current read-only contract before the comment feedback model is specified.

### Decision: Add review-oriented active change rendering as navigation, not mutation

The browser viewer can add a review-oriented page or section for active changes that emphasizes proposal, design, tasks, spec deltas, warnings, and instructions for sending feedback through Pi. This page should not expose text fields, save buttons, or endpoints that mutate the target project.

Alternatives considered:
- Use the existing active change detail page unchanged. This is usable, but does not communicate the intended review workflow.
- Add browser comment controls now but disable persistence. That risks confusing users and overpromising the future companion flow.

### Decision: Preserve a sidecar feedback model for the later review companion

The future review companion should treat browser comments as review feedback records, not as direct edits to OpenSpec artifacts. A likely storage model is a sidecar path outside the artifacts, for example `.pi/openspec-review/comments/*.jsonl` or a documented `openspec/changes/<change>/review.md` file. A Pi extension can watch those records and call `pi.sendUserMessage(...)` so the agent can interpret comments and update proposal/design/spec/tasks intentionally.

Alternatives considered:
- Let the browser directly edit OpenSpec Markdown. This blurs authorship and makes it harder to preserve the OpenSpec iteration model.
- Store comments only in Pi session state. That is simple for Pi, but makes browser comments less durable and harder to audit.

## Risks / Trade-offs

- [Risk] Starting a long-lived server from a prompt template can leave orphaned processes. → Mitigate by documenting manual shutdown in the first slice and planning a Pi extension for managed lifecycle later.
- [Risk] Review UX may appear interactive even though comments are not browser-native yet. → Mitigate by clearly stating that feedback should be sent in Pi for this slice.
- [Risk] Adding Pi resources could make the product feel Pi-only. → Mitigate by keeping the underlying CLI/viewer behavior generic and treating Pi integration as an optional resource.
- [Risk] Direct links may fail if the reviewed change name is missing or archived. → Mitigate by validating active changes in the workflow or falling back to `/changes`.
