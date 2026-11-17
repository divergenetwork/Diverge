#!/usr/bin/env bash
# Build diverge-viewer and install its binary into a config directory's `bin/`.
#
#   diverge-viewer/install.sh <config-dir> (--debug | --release)
#
# <config-dir> is the Diverge config directory — the root the programs
# are started against (`--config <dir>`, or `~/.diverge`) — and the
# binary lands at <config-dir>/bin/diverge-viewer{.exe}, beside the
# other Diverge binaries installed the same way, which is where
# `diverge viewer start` looks for it. Exactly one profile is given;
# both or neither is an error.
#
# The viewer is a Tauri app: its page is built and put inside the
# binary by the Tauri CLI (`pnpm tauri build --no-bundle`), so the
# binary runs with no dev server. It needs `pnpm install` run once
# from the workspace root.
#
# Multiplatform: runs under bash on Linux, macOS and Windows (Git Bash
# or MSYS); the host's OS is read, never chosen.
set -euo pipefail

usage() {
    echo "usage: $0 <config-dir> (--debug | --release)" >&2
    exit 2
}

config=""
profile=""
for arg in "$@"; do
    case "$arg" in
        --debug|--release)
            if [[ -n "$profile" ]]; then
                echo "error: --debug and --release are each given at most once, and not together" >&2
                usage
            fi
            profile="${arg#--}"
            ;;
        --*)
            echo "error: unknown option $arg" >&2
            usage
            ;;
        *)
            if [[ -n "$config" ]]; then
                echo "error: one config directory, given once" >&2
                usage
            fi
            config="$arg"
            ;;
    esac
done
[[ -n "$config" && -n "$profile" ]] || usage

workspace="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) host_exe=".exe" ;;
    *) host_exe="" ;;
esac

installed="diverge-viewer$host_exe"
flags=(--no-bundle)
[[ "$profile" == "debug" ]] && flags+=(--debug)
(cd "$workspace/diverge-viewer" && pnpm tauri build "${flags[@]}")
built="$workspace/target/$profile/$installed"

[[ -f "$built" ]] || { echo "error: $built was not built" >&2; exit 1; }
mkdir -p "$config/bin"
# Staged beside its place and moved in, so a reader never sees half a
# binary; a running copy keeps its old file open on Unix and refuses
# the move on Windows, where the viewer is stopped first.
cp "$built" "$config/bin/$installed.installing"
mv -f "$config/bin/$installed.installing" "$config/bin/$installed"
chmod +x "$config/bin/$installed"
echo "installed $config/bin/$installed ($profile)"
