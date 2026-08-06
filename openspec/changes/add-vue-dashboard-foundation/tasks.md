## 1. The toolchain

- [x] 1.1 Create `web/` with Vite, Vue and TypeScript
- [x] 1.2 Add Tailwind and shadcn-vue, with its light and dark themes
- [x] 1.3 Pin the Node and toolchain versions. The `dist/` freshness check in section 3 is only meaningful if the build is reproducible across environments — Node `24.15.0` exactly, in `.nvmrc` and `engines`, with `engine-strict=true` in `web/.npmrc` so a different Node fails the install rather than producing different bytes
- [x] 1.4 Add a dark-mode toggle that persists across reloads, and verify both themes against real index content — a table, a badge, a muted secondary line, a link. **PASSED** against this repository (9 sessions, 8 changes, one change with 2 open comments, two promoted sessions, one verdict of each kind): both themes render the table, the `most recently active` / verdict / open-comment badges, the muted identifier line and the row links; toggling sets `dark` and survives a reload in both directions; no console errors
- [x] 1.5 Add `web/` to the repository's lint and format tooling, or record deliberately that it has its own — **recorded**: this repository has no JavaScript linter or formatter at all, for the docs site or for `web/`, so there was nothing to add it to. `vue-tsc -b` runs as the first half of `npm run build`
- [x] 1.6 Pin Tailwind's source list explicitly — `source(none)` plus `@source` in `src/style.css`. Found while verifying 1.3: Tailwind's automatic detection scans everything the repository does not gitignore, and 3.1 deliberately un-gitignores `web/dist/`, so it read class names out of the previous build's own output. The CSS grew from 27KB to 42KB and the bytes depended on whether `dist/` existed when the build started — the build was not reproducible, which is the one property 3.2 needs

## 2. Embedding and serving

- [x] 2.1 Add `rust-embed` and serve the built assets from the binary — with `debug-embed`, so a debug build and the tests embed the assets the way a release build does rather than reading `web/dist` off disk at run time
- [x] 2.2 Serve the application at `/`, replacing the server-rendered index
- [x] 2.3 Delete `crates/server/src/page/index.rs`. Leave the rest of `crates/server/src/page/` alone — the scope pages are the next change's cutover
- [x] 2.4 Confirm `crates/core/src/scratch/title.rs` is reused unchanged. Only the rendering is replaced — `scope::session_name` and `scope::change_summary` still call it; `git diff` touches no file under `crates/core/`

## 3. Distribution is checked, not assumed

- [x] 3.1 Commit `web/dist/`
- [x] 3.2 Add a CI check that builds the frontend from a clean checkout and fails if the output differs from the committed assets. Compare a clean build, not an incremental one, or the check verifies a cache — `.github/workflows/frontend-assets.yml`, with no dependency cache and no restored `node_modules`
- [x] 3.3 Confirm by hand that the check fails when `dist/` is stale. An assertion that has never gone red is not known to work. **PASSED**: the step's own shell logic, run against this working tree with a one-word change to `App.vue` and `dist/` rebuilt, reported the old assets deleted (`AD`), `index.html` modified (`AM`) and the newly hashed bundles untracked (`??`), and exited 1. The untracked half is what needed proving — it only shows up because `!web/dist/` un-ignores the directory, and without it a rename-only drift would have gone unreported
- [x] 3.4 Confirm `cargo install --path .` produces a working dashboard on a machine with no Node. **PASSED**: `cargo install --path crates/cli --locked` with `PATH=/usr/bin:/bin:~/.cargo/bin` (`command -v node` → not found) built and installed; the installed binary, run from a temporary project outside this repository on the same Node-free `PATH`, served `/` (the shell), `/api/index` (both scopes), its own `/assets/*.js` and `*.css`, both scope pages, and a 404 for an unknown URL. There is no `build.rs` anywhere in the workspace, so nothing could have invoked a toolchain

## 4. The index

- [x] 4.1 Add a JSON endpoint serving every discovered session and change with the fields the index shows: title where one exists, identifier, last-modified time, open-comment count, standing verdict — `GET /api/index`
- [x] 4.2 Render the index from it
- [x] 4.3 Keep every behaviour the shipped requirement names: a titled session leads with its heading and shows its id; a promoted session is named by the change it became and still links to its own page; an untitled session falls back to its id; the most recently active session is marked, and is not described as live; links address a scope by identifier and never by title
- [x] 4.4 Confirm no scope becomes unreachable. The index is the only navigation into every scope page, including the ones still served as server-rendered HTML. **PASSED**: all 17 scopes this repository discovers appear in `/api/index`, each row's `href` is its identifier, and every one of the 17 returns 200
- [x] 4.5 Tests at the route level: the endpoint lists every discovered scope with its fields; a titled session, a promoted one and an untitled one each render as specified; an empty project renders an empty index rather than an error — plus two that the shell and its bundle come out of the binary, which is what 3.1 traded a build artifact in git for

## 5. Specs and docs

- [x] 5.1 Update the `dashboard-html-views` Purpose. "No framework, no build step" is false for the index the moment this lands, and it must describe the split state honestly rather than be left stale for one change
- [x] 5.2 Update `docs/docs/` wherever it asserts the no-build-step property or describes how the index is rendered — `reference/routes.md` gains `/api/index`, says `/` serves the built frontend, and says the index is rendered in the browser from that endpoint while the derivation stays server-side
- [x] 5.3 Add a `web/` section to the contributor documentation: how to build, and that `dist/` is committed and checked — `docs/docs/development/conventions.md`, plus the layout block and the test counts in `README.md`, which this change falsified

## 6. Verify against a real project

- [x] 6.1 Run `serve` against this repository and confirm every session and change is listed and reachable. **PASSED**: 9 sessions and 8 changes, including two promoted sessions named by their change, six untitled sessions falling back to their ids, a titled session, a change carrying its promoted note's title and 2 open comments, and one session marked most recently active — and nothing labelled live
- [x] 6.2 Confirm the scope pages still work exactly as before, in both the server-rendered pages and the links into them. **PASSED**: clicking the index's link to `add-setup-diagnostics` loads `/changes/add-setup-diagnostics`, headed by its title, rendering all six artifacts with their `data-artifact-path` attributes, both open anchored comments with their anchor resolutions, and the verdict forms
