## 1. Binary crate setup

- [x] 1.1 Create the `cli` binary crate in the workspace, depending on `core`
- [x] 1.2 Add `clap` (derive feature) as a dependency

## 2. Command surface

- [x] 2.1 Define the top-level `Cli` struct with a global `--root` option and a `Commands` subcommand enum (`Summary`, `Serve`, `Hook`, `Comment`)
- [x] 2.2 Write one-line descriptions for top-level `--help` and each subcommand's `--help`
- [x] 2.3 Wire root resolution (via `core`) shared across all subcommands, with a clear error path on failure

## 3. Summary subcommand

- [x] 3.1 Implement `summary` printing resolved root, active changes, archived changes, current specs
- [x] 3.2 Add an integration test running `summary --root <fixture>` against a fixture OpenSpec project

## 4. Placeholder subcommands

- [x] 4.1 Implement `serve`, `hook`, `comment` as non-zero-exit "not yet implemented" stubs
- [x] 4.2 Add tests asserting each placeholder's exit code and message

## 5. Manual verification

- [x] 5.1 Run `openspec-doc --help` and each subcommand's `--help` and confirm output is accurate
- [x] 5.2 Run `openspec-doc summary` against this repo and confirm it lists this MVP's own changes
