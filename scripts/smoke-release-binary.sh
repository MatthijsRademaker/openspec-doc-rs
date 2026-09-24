#!/bin/sh
# Proves a release binary serves the dashboard it embedded: the shell page and
# the first script it references. A binary built against an empty web/dist
# compiles, runs, and answers `--version`, then fails on the first request for
# the shell — so `--version` alone would pass exactly the binary this exists to
# catch.
#
# Usage: scripts/smoke-release-binary.sh <path-to-openspec-doc>
set -eu

bin=$1
port=4399
work=$(mktemp -d)
trap 'kill "$pid" 2>/dev/null || true; rm -rf "$work"' EXIT

mkdir -p "$work/project/openspec"
echo "schema: spec-driven" > "$work/project/openspec/config.yaml"

XDG_STATE_HOME="$work/state" "$bin" serve --root "$work/project" --port "$port" --no-open \
	> "$work/serve.log" 2>&1 &
pid=$!

i=0
until curl -fsS "http://127.0.0.1:$port/" -o "$work/index.html" 2>/dev/null; do
	i=$((i + 1))
	if [ "$i" -ge 40 ] || ! kill -0 "$pid" 2>/dev/null; then
		echo "error: the dashboard did not answer on port $port. Server output:" >&2
		cat "$work/serve.log" >&2
		exit 1
	fi
	sleep 0.5
done

asset=$(grep -o '/assets/[^"]*\.js' "$work/index.html" | head -n 1)
if [ -z "$asset" ]; then
	echo "error: the served shell references no script; the embedded dist is not a dashboard build" >&2
	exit 1
fi
curl -fsS "http://127.0.0.1:$port$asset" -o /dev/null
echo "ok: $bin served / and $asset"
