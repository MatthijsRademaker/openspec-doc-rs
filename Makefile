# Local development entry points.
#
# web/dist is a gitignored build artifact. crates/server embeds whatever dist is
# present at compile time (rust-embed), and cargo compile fails when it is
# missing — so on a fresh clone, build the frontend before any cargo command.

.PHONY: build release test check e2e

build:
	cd web && bun install --frozen-lockfile && bun run build
	cargo build --locked

release:
	cd web && bun install --frozen-lockfile && bun run build
	cargo build --locked --release

test: build
	cargo test --workspace

check:
	cargo clippy --workspace --all-targets
	cargo fmt --all --check
	cd web && bun run check
	cd docs && bun install --frozen-lockfile && bun run build

# Builds frontend and binary itself, then drives the embedded app in Chromium.
e2e:
	cd web && bun install --frozen-lockfile && bun run test:e2e
