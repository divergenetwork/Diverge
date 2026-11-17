#!/usr/bin/env bash
# Build every Diverge program for this host, in release, and pack them
# into one archive: what a release ships for one platform.
#
#   ./package-diverge.sh <out-dir> [--proxy <path>]
#
# The archive is <out-dir>/diverge-<version>-<os>-<arch>.tar.gz (.zip
# on Windows), with <archive>.sha256 beside it. Inside it is one
# folder of that name holding the programs, flat, as a config
# directory's `bin/` holds them:
#
#   diverge  diverge-daemon  diverge-provider  diverge-postgres
#   diverge-viewer  diverge-container-proxy
#
# each `.exe` on Windows but the container proxy, which runs inside
# Linux containers whatever the host and is always the musl-Linux
# build for the host's architecture. `--proxy <path>` hands over one
# already built (a release builds it once per architecture, on Linux);
# without it the proxy is built here through cargo-zigbuild, as
# `diverge-container-proxy/install.sh` does.
#
# The version is the SDK crate's, which `version.sh` keeps every other
# manifest equal to. The host's OS and architecture are read, never
# chosen: nothing is cross-compiled but the proxy.
#
# Needs: cargo; pnpm, with `pnpm install` done (the viewer's page is
# built and put inside its binary by `pnpm tauri build --no-bundle`);
# on Linux, Tauri's system libraries.
set -euo pipefail

usage() {
    echo "usage: $0 <out-dir> [--proxy <path>]" >&2
    exit 2
}

out=""
proxy=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --proxy)
            [[ $# -ge 2 && -z "$proxy" ]] || usage
            proxy="$2"
            shift 2
            ;;
        --*) usage ;;
        *)
            [[ -z "$out" ]] || usage
            out="$1"
            shift
            ;;
    esac
done
[[ -n "$out" ]] || usage

workspace="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
version="$(awk -F'"' '/^version = "[^"]+"/ { print $2; exit }' "$workspace/diverge-sdk-rs/Cargo.toml")"
[[ -n "$version" ]] || { echo "error: no version in diverge-sdk-rs/Cargo.toml" >&2; exit 1; }

case "$(uname -s)" in
    Linux) os="linux"; exe="" ;;
    Darwin) os="macos"; exe="" ;;
    MINGW*|MSYS*|CYGWIN*) os="windows"; exe=".exe" ;;
    *) echo "error: unsupported system $(uname -s)" >&2; exit 1 ;;
esac
case "$(uname -m)" in
    x86_64|amd64) arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *) echo "error: unsupported architecture $(uname -m)" >&2; exit 1 ;;
esac

# One cargo invocation for the four that are plain crates: they share
# one `target/` and its lock, so more would only wait on each other.
(cd "$workspace" && cargo build --release --locked \
    -p diverge-postgres -p diverge-daemon -p diverge-provider -p diverge-cli)

# The viewer, its page inside it.
(cd "$workspace/diverge-viewer" && pnpm tauri build --no-bundle)

if [[ -z "$proxy" ]]; then
    command -v cargo-zigbuild >/dev/null 2>&1 || {
        echo "error: no --proxy was given and cargo-zigbuild is not installed" >&2
        exit 1
    }
    (cd "$workspace" && cargo zigbuild --release --locked -p diverge-container-proxy --target "$arch-unknown-linux-musl")
    proxy="$workspace/target/$arch-unknown-linux-musl/release/diverge-container-proxy"
fi
[[ -f "$proxy" ]] || { echo "error: the container proxy is not at $proxy" >&2; exit 1; }

name="diverge-$version-$os-$arch"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
stage="$out/$name"
rm -rf "$stage"
mkdir -p "$stage"
for program in diverge diverge-daemon diverge-provider diverge-postgres diverge-viewer; do
    built="$workspace/target/release/$program$exe"
    [[ -f "$built" ]] || { echo "error: $built was not built" >&2; exit 1; }
    cp "$built" "$stage/$program$exe"
done
cp "$proxy" "$stage/diverge-container-proxy"
chmod +x "$stage"/*

if [[ "$os" == "windows" ]]; then
    archive="$name.zip"
    rm -f "$out/$archive"
    # tar is bsdtar on Windows runners and Windows itself, and writes a
    # zip by the name's extension; Git Bash has no `zip`.
    (cd "$out" && "${SYSTEMROOT:-C:\\Windows}/System32/tar.exe" -a -c -f "$archive" "$name")
else
    archive="$name.tar.gz"
    (cd "$out" && tar -czf "$archive" "$name")
fi
rm -rf "$stage"

if command -v sha256sum >/dev/null 2>&1; then
    (cd "$out" && sha256sum "$archive" > "$archive.sha256")
else
    (cd "$out" && shasum -a 256 "$archive" > "$archive.sha256")
fi
echo "packed $out/$archive"
