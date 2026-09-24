#!/bin/sh
# Installs openspec-doc from the current GitHub release.
#
#   curl -fsSL https://github.com/matthijsrademaker/openspec-doc-rs/releases/latest/download/install.sh | sh
#
# Published as an asset of the release it installs, never served from a branch:
# a branch URL would pin every install to an unreviewed commit. Re-running it is
# the upgrade path.
set -eu

repo="matthijsrademaker/openspec-doc-rs"
install_dir="$HOME/.local/bin"
published="macOS (arm64, x86_64) and Linux (x86_64, arm64)"

fail() {
	echo "error: $*" >&2
	exit 1
}

command -v curl > /dev/null 2>&1 || fail "curl is required to download the release"
command -v tar > /dev/null 2>&1 || fail "tar is required to unpack the release"

os=$(uname -s)
arch=$(uname -m)
case "$os/$arch" in
	Darwin/arm64 | Darwin/aarch64) target="aarch64-apple-darwin" ;;
	Darwin/x86_64) target="x86_64-apple-darwin" ;;
	Linux/x86_64 | Linux/amd64) target="x86_64-unknown-linux-musl" ;;
	Linux/aarch64 | Linux/arm64) target="aarch64-unknown-linux-musl" ;;
	*) fail "no published binary for $os $arch. Published: $published. Anything else builds from source: https://github.com/$repo#install" ;;
esac

if command -v sha256sum > /dev/null 2>&1; then
	sha256() { sha256sum "$1" | cut -d ' ' -f 1; }
elif command -v shasum > /dev/null 2>&1; then
	sha256() { shasum -a 256 "$1" | cut -d ' ' -f 1; }
else
	fail "sha256sum or shasum is required to verify the download"
fi

archive="openspec-doc-$target.tar.gz"
base="https://github.com/$repo/releases/latest/download"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

echo "Downloading $archive from the current release"
curl -fsSL "$base/$archive" -o "$work/$archive" || fail "could not download $base/$archive"
curl -fsSL "$base/$archive.sha256" -o "$work/$archive.sha256" || fail "could not download $base/$archive.sha256"

expected=$(cut -d ' ' -f 1 < "$work/$archive.sha256")
actual=$(sha256 "$work/$archive")
[ "$expected" = "$actual" ] || fail "checksum mismatch for $archive: expected $expected, got $actual. The download is corrupt or truncated; nothing was installed"

tar -xzf "$work/$archive" -C "$work"
mkdir -p "$install_dir"
# Copied next to the target and renamed over it, so an existing install is
# replaced in one step rather than left half-written if the copy fails.
cp "$work/openspec-doc" "$install_dir/.openspec-doc.new"
chmod 755 "$install_dir/.openspec-doc.new"
mv "$install_dir/.openspec-doc.new" "$install_dir/openspec-doc"
version=$("$install_dir/openspec-doc" --version) || fail "the installed binary at $install_dir/openspec-doc does not run"

on_path=true
case ":$PATH:" in
	*":$install_dir:"*) ;;
	*) on_path=false ;;
esac

has_openspec=true
command -v openspec > /dev/null 2>&1 || has_openspec=false

echo
if $has_openspec; then
	echo "Installed $version to $install_dir/openspec-doc"
else
	echo "Installed $version to $install_dir/openspec-doc, but it is NOT ready to use:"
	echo "  the \`openspec\` CLI is not on PATH. Promotion runs \`openspec validate\`, so an"
	echo "  exploration cannot become a change until it is installed. This script does not"
	echo "  install it: https://github.com/Fission-AI/OpenSpec"
fi

if ! $on_path; then
	echo
	echo "$install_dir is not on your PATH. Add it:"
	case "$(basename "${SHELL:-}")" in
		zsh) echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc && exec zsh" ;;
		bash)
			if [ "$os" = Darwin ]; then rc=.bash_profile; else rc=.bashrc; fi
			echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/$rc && exec bash"
			;;
		fish) echo "  fish_add_path ~/.local/bin" ;;
		*) echo "  export PATH=\"\$HOME/.local/bin:\$PATH\"   # in your shell's startup file" ;;
	esac
fi

echo
echo "Next, in an OpenSpec project: openspec-doc init"
