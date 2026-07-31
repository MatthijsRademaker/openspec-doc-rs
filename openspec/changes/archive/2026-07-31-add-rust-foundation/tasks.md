## 1. Workspace setup

- [x] 1.1 Initialize a Cargo workspace at the repo root with a virtual manifest
- [x] 1.2 Create the `core` library crate for project scanning and root discovery
- [x] 1.3 Add `.gitignore` entries for `target/`

## 2. Root discovery

- [x] 2.1 Implement explicit-root resolution, validating `openspec/config.yaml` exists under the given path
- [x] 2.2 Implement walk-up discovery from a starting directory
- [x] 2.3 Implement the "no project found" error case
- [x] 2.4 Add unit tests covering explicit root, walk-up success, and not-found

## 3. Change and spec discovery

- [x] 3.1 Implement active-change directory enumeration excluding `archive`
- [x] 3.2 Implement archived-change directory enumeration under `changes/archive/`
- [x] 3.3 Implement current-spec directory enumeration under `specs/`
- [x] 3.4 Add fixture-based tests (temp dirs) covering populated, missing, and empty directory cases

## 4. Manual verification

- [x] 4.1 Add a throwaway debug binary or example that prints a scan result for a given root
- [x] 4.2 Run it against this repo and confirm it reports this repo's own active changes correctly
