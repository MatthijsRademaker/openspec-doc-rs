## 1. Anchor algorithm port

- [x] 1.1 Port `createAnchorContext` (selected text lookup, heading path, before/after context, offsets) to Rust, failing loudly when selected text isn't found
- [x] 1.2 Port `resolveAnchorContext` (exact, then fuzzy heading/text/before-after, then orphaned/missing) to Rust
- [x] 1.3 Add unit tests mirroring the TS test cases for exact, fuzzy, and orphaned resolution

## 2. Comment/reply/status model and sidecar

- [x] 2.1 Define comment/reply/status-update record types and the JSONL append-only sidecar format
- [x] 2.2 Implement append operations for comment/reply/status, keyed by scope key (`.openspec-doc/comments/<key>.jsonl`)
- [x] 2.3 Implement read-back: reconstruct each comment's current status and thread history from the JSONL stream
- [x] 2.4 Implement sidecar relocation in lockstep with scope-key rename (session id to change name), preserving all events

## 3. Comment CLI subcommand

- [x] 3.1 Implement `comment add`, `comment list`, `comment reply`, `comment resolve`, filling the placeholder from `add-cli-surface`
- [x] 3.2 Add integration tests exercising the CLI against a fixture artifact and sidecar

## 4. Manual verification

- [x] 4.1 Add a comment against a fixture scratch note via the CLI, simulate promotion, and confirm the comment sidecar relocates and remains readable at the new key
- [x] 4.2 Confirm a fuzzy-resolved comment is reported as `fuzzy`, not silently reported as `exact`
