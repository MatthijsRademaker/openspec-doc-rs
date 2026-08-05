## Context

`add-dashboard-server` provides routes and SSE plumbing with placeholder content. `add-anchored-comments` provides the comment engine. This change is the first point where real files are rendered and real user interactions are wired — the point at which a human can actually use the review loop, even though nothing yet talks back to a live agent session.

## Goals / Non-Goals

**Goals:**
- A human can open the dashboard, read a scratch note or change artifact, select text, leave a comment, and submit a verdict — entirely through server-rendered HTML.
- Comment/verdict writes go through the persistence primitives `add-anchored-comments` already defined — no parallel storage format invented here.
- Live updates from `add-dashboard-server`'s SSE endpoint are visibly exercised: a comment added in one browser tab appears in another without reload.

**Non-Goals:**
- No Vue, no client-side framework, no build step — deferred to the full-parity migration per the MVP decision.
- No hook/directive wiring — that integration is `add-directive-verdict-loop`.

## Decisions

- **Server-rendered HTML plus a small amount of vanilla JS** for text-selection capture and SSE subscription, rather than a client-side framework — matches "CLI first, plain HTML, Vue later" and avoids a frontend build pipeline before the mechanism is proven.
- **Rendering is shared Rust functions, not a template engine.** Reversed during implementation: the reason for wanting a templating helper was that the session and change pages must share comment-list rendering, and a shared function delivers exactly that. `askama` would have added a dependency and a parallel `templates/` tree next to the `format!`-and-`escape()` string building the placeholder pages already used, for no sharing a function does not already give. The shared partial is `page::review::fragment`, which both pages call and which is also served on its own to satisfy the live-update requirement.
- **Artifacts are rendered as their own markdown source in a `<pre>`, not converted to HTML.** This is what makes the anchoring risk below go away rather than be mitigated: what the browser selects is already byte-for-byte a substring of the file, so there is no mapping from rendered DOM ranges back onto source offsets to get wrong, and no markdown parser to add. The cost is that a reviewer reads `## Why` rather than a styled heading — acceptable for an MVP whose point is proving the review loop, and reversible later behind the same anchoring contract.
- **The client sends only the selected text; the server creates the anchor.** Offsets and surrounding context are never accepted from the browser. `comments::add` re-finds the selection in the file as it stands on disk, so a selection made against a stale page is refused loudly instead of anchored to a guess.
- **Phase-verdict is its own record type**, not shoehorned into the comment sidecar's per-comment `status` field — a session-level decision needs a place to live that isn't attached to any single comment. It is an append-only JSONL sidecar at `.openspec-doc/verdicts/`, keyed exactly as a comment sidecar is, so a later verdict does not clobber the one it followed and `add-directive-verdict-loop` can find the untranslated ones.
- **A verdict is refused when it does not apply to its scope, and a keep-exploring verdict is refused with empty notes.** Both are invalid states rather than defaults to paper over: a change has no explore phase to keep exploring, and a keep-exploring verdict whose entire content is what remains open says nothing if that content is blank.

## Risks / Trade-offs

- [Risk] Hand-writing selection-to-anchor JS (capturing selected text and surrounding context in the browser) is fiddly and easy to get subtly wrong across multiple DOM nodes → Resolved by the rendering decision above: the browser only ever supplies `selection.toString()`, and the server does all the anchoring. There is no context capture in the client to get wrong.
- [Risk] Anchor offsets are computed against raw markdown source, but browsers select against rendered HTML DOM ranges — these do not trivially map onto each other → Resolved rather than mitigated: the rendered artifact *is* the raw markdown source, so no mapping exists to be wrong. A selection spanning two artifacts is not submittable (the composer records one `artifact_path`), and a selection that no longer matches the file is refused with `SelectionNotFound` and a 400 telling the reviewer to reselect.
- [Risk] The one thing still not provable without a browser is the client script itself — selection capture, the composer reveal, and the SSE-driven fragment swap. The route layer underneath each is covered by tests (`the_review_fragment_reflects_a_comment_added_after_the_page_was_served`, `a_new_comment_is_pushed_to_the_scopes_event_stream`), so a failure there is isolated to the ~30 lines of JS → Mitigation: tasks 4.2 and the browser half of 5.1/5.2 are human-run against a real browser, and the forms degrade to plain POST-and-redirect so a broken script cannot make a comment unrecordable.
