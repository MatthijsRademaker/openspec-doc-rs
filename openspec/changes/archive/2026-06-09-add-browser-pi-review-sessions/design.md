## Context

`openspec-doc serve` already renders active change review/companion pages and accepts anchored browser feedback into sidecar JSONL files under `.pi/openspec-review/comments/`. Pi can later consume that feedback through a prompt template, but the reviewer must switch away from the browser and manually invoke Pi.

The desired next slice is a live, browser-native Pi analysis loop for active changes. The browser should remain the review cockpit while the local server mediates safe access to Pi. This change is intentionally non-mutating: Pi review sessions may read context, summarize feedback, answer follow-up questions, and propose updates, but they must not edit OpenSpec artifacts or mark comments addressed.

## Goals / Non-Goals

**Goals:**

- Let reviewers start a Pi analysis session from an active change companion page by default, with an explicit opt-out for users who do not want browser-triggered Pi sessions.
- Bundle active change artifacts, analyzer warnings, and selected/open feedback records into the agent prompt automatically.
- Stream Pi session events into the browser so the user sees progress and response text without leaving the page.
- Persist lightweight session events so browser refreshes can show recent completed analysis.
- Provide clear disabled, unavailable, running, failed, and complete states, with retry affordances for failed session starts and failed runs where retry is safe.
- Keep existing sidecar feedback behavior and storage compatibility.
- Enforce active-change-only, local-server-only, non-mutating behavior for this first slice.

**Non-Goals:**

- Applying Pi-proposed artifact edits from the browser.
- Editing, resolving, or marking feedback statuses automatically based on Pi analysis.
- Supporting non-Pi agent providers.
- Hosting or remote multi-user collaboration.
- Replacing the existing Pi prompt-template workflow.

## Decisions

### Decision: default to review-agent analysis with explicit opt-out

Browser Pi review analysis should be enabled by default for companion-enabled `serve` sessions, with an explicit opt-out option such as `--review-agent=off` or `--no-review-agent`. If Pi SDK setup, credentials, or model availability are missing, the UI should show an unavailable state rather than failing the entire viewer.

Rationale: the intended product feel is live in-browser collaboration, not a hidden advanced mode. Default-on analysis makes “Ask Pi” part of the normal companion flow while the first slice remains non-mutating and local-only. An opt-out keeps users in control when they do not want browser-triggered Pi sessions.

Alternative considered: require an explicit opt-in option such as `--review-agent=analyze` or `--pi-review-agent`. Rejected because it would preserve the current handoff feel and make the desired #2 experience easy to miss.

### Decision: route all browser agent actions through `openspec-doc` server APIs

The browser will call local server endpoints for session creation, event streaming, follow-up messages, and aborts. The browser will not call Pi directly.

Rationale: the server can validate active changes, safe artifact paths, comment ids, enabled mode, and target root. It can also hide SDK details from the browser and provide stable product APIs.

Potential API shape:

```txt
POST /api/review-agent/<change-name>/sessions
GET  /api/review-agent/sessions/<session-id>/events
GET  /api/review-agent/<change-name>/sessions
POST /api/review-agent/sessions/<session-id>/message
POST /api/review-agent/sessions/<session-id>/abort
```

### Decision: create durable review bundles as the agent input contract

Session creation should produce an internal bundle containing the active change name, included artifact snapshots, current analyzer warnings, included feedback threads, anchor states, and the reviewer instruction. Bundles can be stored under `.pi/openspec-review/bundles/` or embedded as the first event in session JSONL storage.

Rationale: a bundle answers “what exactly did we send to Pi?” and decouples review feedback storage from Pi session execution. It also supports refresh recovery and future prompt-template reuse.

### Decision: stream normalized session events to the browser

The Pi SDK exposes session event subscriptions with text deltas, tool lifecycle events, agent lifecycle events, and queue updates. The server should translate those into a smaller browser event vocabulary, such as:

```txt
session_started
bundle_created
agent_start
text_delta
tool_start
tool_end
agent_end
error
session_finished
```

Rationale: normalizing events keeps the browser UI independent of Pi SDK internals and lets tests assert product behavior without coupling to every Pi event variant.

### Decision: start with non-mutating tool access

The initial Pi session should be configured to support analysis only. Prefer no write-capable tools for live browser sessions. If shell access is necessary, prompts and tool configuration should still prohibit artifact mutation; ideally first implementation can rely on provided snapshots and read-only tools.

Rationale: this change is about the #2 feel of live collaboration, not applying changes. Separating analysis from apply avoids conflating UX streaming with filesystem mutation safety.

### Decision: proposed updates are response content, not applied patches

Pi may describe proposed artifact updates and unresolved questions in its response. The browser can display them as analysis, but there is no apply endpoint in this change.

Rationale: preserving a hard boundary makes the first implementation easier to validate and avoids accidental `openspec/` mutation from browser-triggered requests. A later change can add explicit diff generation and approval.

## Risks / Trade-offs

- [Risk] Pi SDK APIs or package availability may vary in installed environments. → Mitigation: dynamically load or gate the integration, fail with an actionable disabled/unavailable state, provide retry when configuration may have changed, and keep the existing companion workflow working.
- [Risk] Browser-triggered agent sessions could surprise users. → Mitigation: visibly label that Pi analysis is enabled, keep sessions user-initiated from the browser, and provide an explicit opt-out serve option.
- [Risk] Streaming/session persistence adds server state complexity. → Mitigation: keep in-memory active sessions plus append-only lightweight JSONL events for recovery of completed sessions.
- [Risk] Agent responses may not produce structured proposed updates reliably. → Mitigation: use a strict prompt contract for summary, proposed updates, unresolved questions, and next actions; treat structure as advisory in this slice.
- [Risk] Artifact snapshots may go stale during an agent session. → Mitigation: include snapshot hashes/timestamps in the bundle and label output as based on the captured bundle.
- [Risk] Enabling any tools could permit unintended mutation. → Mitigation: start with provided bundle context and read-only/no-tool sessions where feasible; do not expose an apply endpoint.

## Migration Plan

1. Add default review-agent analysis configuration plus explicit opt-out and unavailable UI states without changing existing comment behavior.
2. Add bundle/session data types and server API validation.
3. Add Pi SDK bridge behind the review-agent mode with normalized streaming events.
4. Add companion UI controls for “Ask Pi”, retry, and session transcript display.
5. Add tests for default enabled mode, opt-out mode, active-change validation, session creation, retry behavior, event persistence, and non-mutating behavior.

Rollback is straightforward: run without the review-agent serve option. Existing feedback comments and companion pages continue to work.

## Open Questions

- Which exact CLI opt-out spelling should be used: `--review-agent=off`, `--no-review-agent`, or another spelling?
- Should the first implementation persist full transcripts or only enough session events to recover summaries after refresh?
- Can the first implementation avoid all tools by passing complete artifact snapshots, or should it allow read-only tools for follow-up context?
