# Roadmap

## Shipped

Nine capabilities, each with a spec under `openspec/specs/` and an archived change behind it.

| Capability | What it gives you |
| --- | --- |
| `project-scanner` | Locates the project root and enumerates changes and specs by directory structure alone |
| `cli-surface` | The `openspec-doc` binary and its subcommand tree |
| `dashboard-server` | The axum server, filesystem watcher, and SSE update channel |
| `dashboard-html-views` | Vue session and change workbenches: rendered blocks, anchored threads, reviewer actions, delivery state, persistent verdict controls |
| `anchored-comments` | Comments attached to spans of markdown, with honest drift reporting |
| `scratch-note-workflow` | The exploration note's lifecycle, and claim-based promotion onto a change |
| `agent-hook-bridge` | Turn-boundary hook for Claude Code and pi.dev; block and reinject |
| `directive-verdict-loop` | Verdict to directive translation, consumed exactly once |
| `explore-note-capture` | Detecting the explore command and telling the agent where to write |

The loop these compose is closed and verified on live sessions across both agents. See
[Vision & MVP scope](/vision.md) for what that does and does not establish.

## Open

Triaged against [the completion criterion](/vision.md#how-the-mvp-is-judged-complete): the reviewer
comments in the browser, says "move to proposal" in the terminal, and the agent produces a proposal that
accounts for those comments.

That criterion is what decides this table, and it reorders it. What matters is whether the comments reach
the agent **before** it writes the proposal; whether the reviewer used a terminal does not.

**The criterion was met on 2026-08-06.** Both changes that blocked it have shipped, so nothing below is
MVP-blocking; the table is now a priority order for what comes after. See
[What is proven](/vision.md#the-criterion-has-been-met).

| Change | Tasks | Why |
| --- | --- | --- |
| `add-setup-diagnostics` | 31 | Setup is lossy and fails silently: hook config is gitignored, so a fresh clone has none and nothing says so — an unwired project is pixel-identical to one where nothing has happened yet. Found the hard way, by the hooks in *this* repo being absent for an unknown length of time without anyone noticing. Adds a `doctor` that runs the hooks rather than reading the settings file, because a check that passes on a broken install is worse than no check. |
| `add-artifact-block-model` | 15 | Decomposes an artifact into blocks carrying both their rendered form and the exact source they were sliced from, so markdown can be rendered without giving up anchoring. Also fixes a live hazard: `anchor::create` takes the *first* occurrence of the selected text, so two identical list items in one file already mis-anchor silently, and block commenting would make that routine. Rust only, settleable by `cargo test`. |
| `add-vue-dashboard-foundation` | 24 | The toolchain, the committed `dist/`, `rust-embed`, dark mode, and a CI check that a stale build cannot ship — proven on one real screen, the index, rather than on a scaffold that cannot be told from a broken one. The index also has no comment surface, so the old scope pages stay untouched alongside it. |
| `add-dashboard-development-harness` | 50 | The Bun toolchain, bounded frontend gate, embedded Playwright lane, working Vite API proxy, Router boundary, review-workbench design source, mirrored skills, and pinned shadcn-vue registry access that the interaction-heavy cutover needs. |
| `add-live-artifact-updates` | 21 | The artifact never live-updates, so the reviewer watches an exploration that does not move — the vision's own headline claim, stated and false. The watcher already fires and the client throws the information away. Small, and the only one of the four that fixes something the product already promises. |
| `extend-comment-model` | 22 | Unanchored and editable comments. A prerequisite of the rewrite rather than a decoration on it: the rewrite routes verdict notes through the comment sidecar, which first has to accept a comment with no anchor. |
| `add-session-titles` | 21 | The index is a list of UUIDs. The title is already on disk — the agent's own `#` heading in the scratch note — and the index simply does not read it. Cheapest real improvement on this list. |
| `add-comment-thread-actions` | 20 | Replying or closing a comment means leaving the browser for the CLI. The `addressed` status it adds is how an agent records that it responded to a comment rather than marking its own work resolved, and `directive-verdict-loop` already refers to it as a dependency. |
| `add-dashboard-lifecycle` | 28 | `serve` must already be running or the loop silently does nothing: hooks fire, the agent writes, and there is no reviewer at the other end. A trap for the operator rather than a break in the loop. Overlaps `add-setup-diagnostics`; decide which owns the "is this thing on?" question before starting either. |
| `add-change-approval-gate` | 28 | Nothing requires anyone to say a change is ready before implementation starts. A genuinely new concept rather than a missing piece of the review loop, and it would change what the tool *is*. Leave it alone until the rest has been used in anger. |

## Known defects and debt

| Item | Severity | Notes |
| --- | --- | --- |
| A typo in `hook prompt`'s command string is unrecoverable | Low | **Fixed 2026-08-06.** Clap exits 2 on a usage error and Claude Code treats that as a block, so one typo refused every prompt in the session. `main.rs` now exits zero for any argument or root-resolution failure reaching for `hook prompt`. Residual: a typo in the word `prompt` itself is not detectable and still exits 2. |
| No release; setup is lossy | Moderate | 0.1.0, `cargo install --path` from a clone, no publish workflow. Hook config lives in gitignored local settings, so a fresh clone has none and nothing says so — the hooks in *this* repo were absent for an unknown period without anyone noticing. Deliberately outside the MVP boundary; `add-setup-diagnostics` covers the second half. **Narrowed 2026-09-04** by `add-continuous-verification`: what was missing is now only the *publishing* half. `.github/workflows/rust.yml` runs `make check` and `make test` on Linux and macOS for every push and pull request, so this row no longer also means that no automation has ever compiled the workspace — which it did for forty-two commits. `add-release-binaries-and-installers` and `add-release-automation` own what is left. |
| Agents may formalize an exploration without claiming it | Low | Downgraded from moderate. Did not occur on the criterion run, which went through promotion — the agent ran `scratch claim` unprompted. One observation is not proof, but it is no longer unobserved, and it degrades to a visibly un-promoted note rather than a corrupted one. |
| ~~Three `watch.rs` filesystem-watcher tests fail~~ | — | **Fixed 2026-09-04** by `fix-repository-verification-gates`. The fixture handed the watcher a `TMPDIR` path reached through a symlink, so FSEvents reported `/private/var/…` against a scope registered as `/var/…` and `classify`'s literal prefix match discarded every event. The tests canonicalize their temp root now. This row stood for weeks first as a timing bug and then as an unexplained recovery, and neither was true — the tests were failing on macOS the whole time. Nothing would have caught that, because nothing ran the suite anywhere; `add-continuous-verification` adds the macOS leg that makes the same mistake loud. |

## Suggested order

Steps 1–4 of the previous order are done: the move-to-proposal directive points at the comments,
`add-prompt-time-directive-delivery` shipped, the criterion was run live and passed, and the MVP is
declared done. What follows is post-MVP.

1. **Use it.** The 230 open tasks below are guesses about what will annoy you. A fortnight of real use will
   reorder this list and probably delete some of it. This is the step most likely to be skipped and the one
   with the highest return — every change below was written before the loop had ever been used in anger.
2. **`add-session-titles`** — 21 tasks, no new state, and it fixes the first thing you see. The cheapest
   way to find out whether the dashboard is worth investing in.
3. **`add-setup-diagnostics`** — the only item that blocks anyone *else* using this, and the failure it
   catches has already happened here once. Do it before the first time you try to install this elsewhere,
   not after.
4. **`add-live-artifact-updates`** — 21 tasks, now implemented in the Vue workbench and SSE watcher. Artifact
   events replace rendered blocks when clean, preserve scroll position, and defer behind unsent composer text;
   review-state events remain immediate. Finish its live browser verification before archiving.
5. `add-dashboard-lifecycle` overlaps the "is this thing on?" scope with `add-setup-diagnostics`; reconcile
   before starting either.
6. `add-change-approval-gate` last, if ever. It changes what the tool is.
