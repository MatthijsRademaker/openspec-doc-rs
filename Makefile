# Local development entry points.
#
# web/dist is a gitignored build artifact. crates/server embeds whatever dist is
# present at compile time (rust-embed), and cargo compile fails when it is
# missing — so every cargo target here depends on it existing.

.PHONY: build release test check e2e frontend

# The artifact the workspace embeds. Built when absent; `make frontend` forces it.
#
# Only its existence is the condition — make is not tracking web/src, so editing
# the frontend and running `make test` tests against the *previous* dist. That is
# tolerable here for one reason: the Rust gates embed the frontend and never
# assert on its contents, while the frontend's own gates (`bun run check`,
# `bun run test:e2e`) build fresh. If a Rust test ever asserts on embedded
# content, this is the decision to revisit.
#
# The conditional matters in CI, where the frontend arrives as a downloaded
# artifact: a gate target that rebuilt it would cost a Bun install per platform
# leg and produce a different dist from the one the artifact carried, which is
# the single-build guarantee gone.
web/dist:
	$(MAKE) frontend

frontend:
	cd web && bun install --frozen-lockfile && bun run build

build: web/dist
	cargo build --locked

# A release embeds a freshly built frontend rather than whatever dist is lying
# around, so this forces the rebuild.
release: frontend
	cargo build --locked --release

test: web/dist
	cargo test --workspace

check: web/dist
	cargo clippy --workspace --all-targets
	cargo fmt --all --check
	cd web && bun run check
	cd docs && bun install --frozen-lockfile && bun run build

# Builds frontend and binary itself, then drives the embedded app in Chromium.
e2e:
	cd web && bun install --frozen-lockfile && bun run test:e2e
