## Status at archive

Two decisions argued for here were later reversed: committing `web/dist/` with a byte-comparison CI check, and shipping a light theme with a reviewer-selectable toggle. See the note at the head of `proposal.md`. The arguments are left standing as written, with the reversals marked where they were made — a design that quietly matched a decision it never argued for would be worse than one that shows its own reversal.

## Context

The dashboard was built to an explicit MVP constraint: server-rendered HTML, no framework, no build step. That bought a self-contained binary and a page provably incapable of mis-anchoring. It also produced a 395-line, 35KB page per change that renders six documents as unstyled source. The constraint has been paid for and the interface is what was bought.

Spending it is a decision with two independent halves: *how the interface is built and shipped*, and *what the interface shows*. Those were written up as one change of sixty-three tasks. They are not the same question and they do not have to be answered together.

This change is the first half. Its output is a working index in a new toolchain and four settled facts about distribution.

## Goals / Non-Goals

**Goals:**
- The binary still installs with `cargo install` and no other toolchain.
- A stale committed `dist/` fails a build rather than shipping. *(Reversed: `dist/` is gitignored, and an absent one fails the compile instead.)*
- Dark mode exists, from the component library's theming rather than hand-written CSS. *(Reversed: one hand-written dark identity, no light theme, no toggle.)*
- The index keeps every field it shows today.

**Non-Goals:**
- Not touching the scope pages, their comment surface, or their verdict controls.
- Not the block rendering. `add-artifact-block-model` produces blocks; the change after this one renders them.
- Not restyling the docs site, which is a separate artifact on a separate toolchain.

## Decisions

### The index is the first screen, and coexistence is safe *for it*

The superseded design argued the two frontends cannot coexist: "keeping both means maintaining two comment surfaces against one sidecar". That argument is correct and it is specifically about comment surfaces. **The index has none.** It reads counts and verdicts and links onward; it writes nothing and it offers no gesture that touches the sidecar.

So the objection does not reach this change, and the split it seemed to forbid is available. During this change the dashboard is Vue at `/` and server-rendered HTML at `/sessions/<id>` and `/changes/<name>`, with exactly one comment surface in existence throughout. The next change removes the second half in one cutover, which is the cutover the objection was actually about.

The alternative first screen — a stub page proving the build works — was rejected. A scaffold that renders "hello" answers the distribution questions and nothing else, and it cannot answer the theming question honestly, because a theme is only tested by real content: a table, a badge, a muted secondary line, a link.

### Distribution: commit `dist/`, embed with `rust-embed`

> **Reversed by `b2e120a`.** The comparison below is sound and the conclusion held for the half that mattered — embed with `rust-embed`, so `cargo install` needs no Node. The committing half was dropped: `web/dist/` is gitignored, the crate embeds whatever the local build produced, an absent build directory fails the compile, and CI proves a clean checkout can build and serve rather than comparing bytes. The third option this table does not list is the one that won — *generate it locally, embed it, and never store it* — which costs one build step before Cargo on a fresh clone and buys back the diff noise.

| | `cargo install` needs Node | build output in git |
|---|---|---|
| commit `dist/` + `rust-embed` | no | yes |
| `npm ci && npm run build` in `build.rs` | yes | no |

Chosen: commit `dist/`. This is a local tool someone installs once and then forgets about; making that install depend on a working Node toolchain is a worse tax than diff noise on frontend commits.

The cost is real. A stale `dist/` ships an interface that does not match the source, and it does it silently — the exact failure shape this project refuses everywhere else. So the CI check that rebuilds and compares is not a follow-up task; it is what makes the decision above defensible, and it lands in the change that creates the hazard.

The check must compare a build from a clean checkout, not an incremental one, or it verifies a cache.

### Index data as JSON, index rendering in Vue, index *derivation* untouched

`crates/core/src/scratch/title.rs` derives a session's title from its note's heading, and falls back to the change a promoted session became. That is core logic, it shipped in `add-session-titles`, and it is reusable exactly as it is. Only `crates/server/src/page/index.rs` — the rendering — is replaced.

This is worth saying plainly because the superseded change scheduled `page/index.rs` for deletion as part of deleting `page/`, and restated the index's content requirement in its own spec delta as though it were new. It is not new. It is a shipped requirement in `openspec/specs/dashboard-html-views/spec.md` with six scenarios, and this change satisfies it through different rendering. The delta therefore does not touch it — a modification that restates a requirement unchanged is noise that makes the real changes harder to see.

### The Purpose sentence goes now, not later

`dashboard-html-views` currently reads "Pages are server-rendered HTML with the minimum vanilla JavaScript that selection and live updates require — no framework, no build step." The moment the index is Vue that sentence is false, and a spec that is false about a shipped page is worse than a spec that is silent.

So this change edits the Purpose to describe the split state honestly: the interface is a built application, the scope pages are not yet part of it. The next change edits it again, to remove the second clause. Two edits to one sentence across two changes is the correct cost of splitting; leaving it stale for one change is not.

## Risks / Trade-offs

- **The interface is half one thing and half another for one change.** Mitigated by the index having no comment surface, and bounded by the next change being the immediate successor.
- **Two Node toolchains in one repository.** rspress/React for the docs, Vite/Vue for the dashboard. They share no code. The cost is a second `node_modules` and a second set of dependency updates.
- **`shadcn-vue` tracks behind the React original.** A component that exists in shadcn and not in shadcn-vue is a component to write by hand. Recorded because the choice is the owner's and was made with this known.
- **The `dist/` check is only as good as its determinism.** If Vite's output is not byte-reproducible across environments, the check fails on clean builds and gets disabled, at which point the distribution decision has quietly lost its safeguard. Pinning the toolchain version is part of the task, not an optimisation. *(This risk is what came true. The check was not disabled quietly — the artifact stopped being committed, which removed the need for byte-reproducibility along with the check. The pinned toolchain stayed, now Bun 1.3.2.)*

## Migration Plan

`/` moves from server-rendered HTML to the built application in one step; there is no period where both serve it. The scope pages are not touched, so nothing a reviewer does today stops working.

## Open Questions

- Is the docs site's React toolchain worth reusing for a shared theme, or do the two stay entirely separate? Recommendation: entirely separate.
