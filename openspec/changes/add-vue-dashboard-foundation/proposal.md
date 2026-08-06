## Why

The dashboard's interface has to be replaced — comments render hundreds of lines from the text they annotate, artifacts never live-update, there is no dark mode — and the owner has chosen Vue with shadcn-vue to replace it. Before any of that is worth writing, four questions have to have answers that can be checked rather than assumed:

- Does a Vite build, a committed `dist/` and `rust-embed` actually produce a binary that serves the interface?
- Does `cargo install` still work on a machine with no Node?
- Does a stale `dist/` get caught, or does it silently ship a frontend that does not match the source?
- Does shadcn-vue's theming give a dark mode, or does that turn into hand-written CSS anyway?

Answering those inside the review-surface migration means finding out at the end of a thirty-five-task change. This change answers them first, and it answers them with a real screen rather than a scaffold, because a scaffold with nothing behind it cannot be told from a broken one.

**The screen is the index.** It is the right first target for three reasons. It carries real data — title, identifier, last-modified time, open-comment count, standing verdict — so it exercises a JSON endpoint rather than a placeholder. It has no comment surface, so the old server-rendered scope pages can stay exactly as they are for the duration without the hazard that made the superseded change insist on a single cutover. And it is the only navigation into every page the migration then rewrites: `crates/server/src/page/index.rs` sits inside the directory that change deletes, which is precisely how working behaviour gets removed by a change that never mentions it.

## What Changes

- **`web/` is added**: Vite, Vue, Tailwind and shadcn-vue, with the light and dark themes coming from shadcn's theming rather than hand-written media queries.
- **The built assets are committed and embedded** with `rust-embed`, so `cargo install` needs no Node toolchain.
- **A CI check rebuilds the frontend and fails if the committed assets differ.** Committing a build artifact is only safe with this; without it the decision is not the one being made here.
- **`/` serves the built application**, and a JSON endpoint serves the index's data — every discovered session and change with the fields the index already shows.
- **`crates/server/src/page/index.rs` is deleted** and its rendering replaced. The title derivation in `crates/core/src/scratch/title.rs` is reused unchanged; only the rendering moves.
- **The scope pages are untouched.** `/sessions/<id>` and `/changes/<name>` keep serving server-rendered HTML until `migrate-dashboard-review-to-vue` replaces them.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dashboard-server`: gains the requirement that the interface is a built application served from assets embedded in the binary, that those assets are verifiably current, and that the index's data is available as JSON.
- `dashboard-html-views`: gains a theming requirement. Its index-content requirement is **not** modified — every field it names survives verbatim, which is the point of doing the index first.

## Impact

- A new `web/` directory: the Vue application, `package.json`, Vite configuration.
- A committed `web/dist/` and `rust-embed` as a new dependency.
- `crates/server/src/page/index.rs` — deleted.
- `crates/server/src/routes.rs` — `/` and the index data endpoint; the scope routes untouched.
- CI configuration — the `dist/` freshness check.
- `openspec/specs/dashboard-html-views/spec.md` — the Purpose sentence "no framework, no build step" becomes false for the index and must say so.
- `docs/docs/` — anything asserting the no-build-step property.

## Dependencies and ordering

- **Nothing blocks this.** It shares no code with `add-artifact-block-model` and the two can land in either order or in parallel.
- **`migrate-dashboard-review-to-vue` depends on this** for its toolchain, its embedding, its theme and its navigation.

## What is given up, deliberately

**`cargo build` gains a committed build artifact.** The alternative — `npm ci && npm run build` from `build.rs` — keeps the repository tidy and makes a Rust build depend on a working Node toolchain. Requiring Node to `cargo install` a local review tool is the worse tax. The cost is diff noise on frontend commits and the risk of a stale `dist/`, which is why the CI check is in this change and not deferred.

**Two frontend toolchains in one repository.** The only existing one is React, in the rspress docs site. `shadcn-vue` is a community port that tracks behind the React original. The two are separate artifacts with no shared code; the choice is the owner's and is made.

**The index is briefly the only Vue screen**, so for one change the dashboard is half one thing and half another. That is the cost of not finding out about `rust-embed` at task thirty of the next change.
