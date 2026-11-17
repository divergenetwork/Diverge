#!/usr/bin/env bash
# version.sh — set the version of every Diverge package to a single value.
# Touches Rust crates and JS packages, the specification's latest module
# (the folder named after the version), and every file that writes the
# version out as a literal. Skips pnpm-lock.yaml, which regenerates on
# the next pnpm install.
#
# The old version is read from diverge-sdk-rs/Cargo.toml before anything
# is changed; the literals are that string, replaced by the new one.
#
# Usage:
#   bash version.sh <new-version>
# Example:
#   bash version.sh 2.1.0

set -euo pipefail

if [ "$#" -ne 1 ] || [ -z "${1:-}" ]; then
  echo "Usage: $0 <new-version>" >&2
  echo "Example: $0 2.1.0" >&2
  exit 1
fi

NEW_VERSION="$1"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# The version being replaced: the SDK crate's, which is the specification's
# revision. Read before any file is touched.
OLD_VERSION="$(awk -F'"' '/^version = "[^"]+"/ { print $2; exit }' "$REPO_ROOT/diverge-sdk-rs/Cargo.toml")"
if [ -z "$OLD_VERSION" ]; then
  echo "diverge-sdk-rs/Cargo.toml has no version" >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Primitives
# ---------------------------------------------------------------------------
# Everything is awk-based so the script is portable between GNU and BSD
# toolchains (macOS sed's `-i` flag and GNU sed's `0,/pat/` range syntax
# don't agree).

# Rewrite the first line matching $pat with the literal replacement $repl.
# If no match is found, the file is left unchanged (no error).
first_line_replace() {
  local file="$1"
  local pat="$2"
  local repl="$3"
  local tmp
  tmp=$(mktemp)
  awk -v pat="$pat" -v repl="$repl" '
    !done && $0 ~ pat { print repl; done=1; next }
    { print }
  ' "$file" > "$tmp"
  mv "$tmp" "$file"
}

# On each line matching $line_pat, replace every occurrence of $token_pat
# with $token_repl. Other lines pass through unchanged.
inline_substitute() {
  local file="$1"
  local line_pat="$2"
  local token_pat="$3"
  local token_repl="$4"
  local tmp
  tmp=$(mktemp)
  awk -v lp="$line_pat" -v tp="$token_pat" -v tr="$token_repl" '
    $0 ~ lp { gsub(tp, tr) }
    { print }
  ' "$file" > "$tmp"
  mv "$tmp" "$file"
}

# Insert $insert_line right after the first line matching $pat. Idempotent
# guard left to the caller — this always inserts if the anchor matches.
insert_after_first() {
  local file="$1"
  local pat="$2"
  local insert_line="$3"
  local tmp
  tmp=$(mktemp)
  awk -v pat="$pat" -v ins="$insert_line" '
    { print }
    !done && $0 ~ pat { print ins; done=1 }
  ' "$file" > "$tmp"
  mv "$tmp" "$file"
}

# Replace every occurrence of the literal string $old with $new. No
# pattern is involved, so the dots of a version match only dots. A file
# without the string is left untouched, byte for byte.
replace_literal() {
  local file="$1"
  local old="$2"
  local new="$3"
  if ! grep -qF -- "$old" "$file"; then
    return
  fi
  local tmp
  tmp=$(mktemp)
  awk -v old="$old" -v new="$new" '
    {
      out = ""
      while ((i = index($0, old)) > 0) {
        out = out substr($0, 1, i - 1) new
        $0 = substr($0, i + length(old))
      }
      print out $0
    }
  ' "$file" > "$tmp"
  mv "$tmp" "$file"
}

# ---------------------------------------------------------------------------
# Per-file-type updaters
# ---------------------------------------------------------------------------

# Cargo.toml [package] version.
# Convention in our repo: the first `^version = "..."` line is the package
# version. Third-party dependency version specs never appear at column 0
# without a leading `= {` so this targets the right line.
set_toml_package_version() {
  local file="$1"
  first_line_replace "$file" \
    '^version = "[^"]+"' \
    "version = \"$NEW_VERSION\""
}

# Root "version": "..." in a package.json or a tauri.conf.json. Relies on the standard layout
# where "version" is the first occurrence in the file (right after "name").
set_package_json_version() {
  local file="$1"
  first_line_replace "$file" \
    '^[[:space:]]*"version":[[:space:]]*"[^"]+"' \
    "  \"version\": \"$NEW_VERSION\","
}

# ---------------------------------------------------------------------------
# File lists
# ---------------------------------------------------------------------------

CARGO_TOMLS=(
  diverge-agents/cc/Cargo.toml
  diverge-agents/codex/Cargo.toml
  diverge-agents/hermes/Cargo.toml
  diverge-agents/lone/Cargo.toml
  diverge-agents/python/Cargo.toml
  diverge-container-proxy/Cargo.toml
  diverge-agents/openrouter/Cargo.toml
  diverge-sdk-rs/Cargo.toml
  diverge-provider/Cargo.toml
  diverge-postgres/Cargo.toml
  diverge-daemon/Cargo.toml
  diverge-cli/Cargo.toml
  diverge-tools/identity-diverge-network/Cargo.toml
  diverge-examples/agents/human/Cargo.toml
  diverge-examples/tools/alarm/Cargo.toml
  diverge-examples/tools/chat/Cargo.toml
  diverge-examples/tools/courier/Cargo.toml
  diverge-examples/tools/memory/Cargo.toml
  diverge-examples/tools/taskboard/Cargo.toml
  diverge-viewer/config/Cargo.toml
  diverge-viewer/src-tauri/Cargo.toml
)

PACKAGE_JSONS=(
  diverge-provider-web/package.json
  diverge-sdk-js/package.json
  diverge-viewer/package.json
  diverge-viewer/src-tauri/tauri.conf.json
)

# The specification's modules. The latest is the folder named after the
# SDK crate's version, and the site's build fails when no folder has that
# name, so the folder is renamed with the version.
SPEC_DIR=diverge-provider-web/src/content/spec

# Files that write the version out as a literal: the specification's
# contract and the notes beside it, doc comments that show an `info`
# answer, reports, skills, and the viewer's stub. Every file of the
# latest specification module is added to these below.
LITERALS=(
  diverge-provider-web/CLAUDE.md
  diverge-provider-web/CONTRACT.md
  diverge-provider-web/Containerfile
  diverge-provider-web/skills/spec-prose/SKILL.md
  diverge-provider-web/src/content.config.ts
  diverge-provider-web/src/include.mjs
  diverge-sdk-js/reports/build-order.md
  diverge-sdk-rs/reports/5-active-inactive-local-connection.md
  diverge-sdk-rs/src/daemon/endpoints/info/server/response/frame.rs
  diverge-sdk-rs/src/daemon/endpoints/info/server/response/info.rs
  diverge-sdk-rs/src/provider/endpoints/info/server/response/frame.rs
  diverge-skills/diverge-logs/SKILL.md
)

# ---------------------------------------------------------------------------
# Apply
# ---------------------------------------------------------------------------

update() {
  local kind="$1"
  local rel="$2"
  local file="$REPO_ROOT/$rel"
  if [ ! -f "$file" ]; then
    echo "  skip   $rel (not found)"
    return
  fi
  echo "  $kind  $rel"
  case "$kind" in
    cargo)
      set_toml_package_version "$file"
      ;;
    pkg)
      set_package_json_version "$file"
      ;;
    lit)
      replace_literal "$file" "$OLD_VERSION" "$NEW_VERSION"
      ;;
  esac
}

echo "Setting version to $NEW_VERSION (was $OLD_VERSION)"

for rel in "${CARGO_TOMLS[@]}";   do update cargo "$rel"; done
for rel in "${PACKAGE_JSONS[@]}"; do update pkg   "$rel"; done

if [ "$OLD_VERSION" != "$NEW_VERSION" ]; then
  # The specification's latest module takes the new version's name.
  if [ -d "$REPO_ROOT/$SPEC_DIR/$OLD_VERSION" ]; then
    if [ -e "$REPO_ROOT/$SPEC_DIR/$NEW_VERSION" ]; then
      echo "  skip   $SPEC_DIR/$OLD_VERSION ($SPEC_DIR/$NEW_VERSION is already there)"
    else
      echo "  move   $SPEC_DIR/$OLD_VERSION -> $SPEC_DIR/$NEW_VERSION"
      mv "$REPO_ROOT/$SPEC_DIR/$OLD_VERSION" "$REPO_ROOT/$SPEC_DIR/$NEW_VERSION"
    fi
  fi

  for rel in "${LITERALS[@]}"; do update lit "$rel"; done

  # Every page of the latest module that states the revision.
  if [ -d "$REPO_ROOT/$SPEC_DIR/$NEW_VERSION" ]; then
    while IFS= read -r file; do
      update lit "${file#"$REPO_ROOT/"}"
    done < <(grep -rlF -- "$OLD_VERSION" "$REPO_ROOT/$SPEC_DIR/$NEW_VERSION" || true)
  fi
fi

# Sync Cargo.lock to the new workspace versions, so that no later cargo
# invocation rewrites the lockfile mid-build.
if command -v cargo >/dev/null 2>&1; then
  echo
  echo "Refreshing Cargo.lock workspace versions..."
  ( cd "$REPO_ROOT" && cargo update -w >/dev/null 2>&1 ) && echo "  Cargo.lock synced." \
    || echo "  Cargo.lock refresh failed — run 'cargo update -w' manually."
else
  echo
  echo "WARNING: cargo not found on PATH. Run 'cargo update -w' manually before"
  echo "         pushing."
fi

echo
echo "Done. pnpm-lock.yaml will refresh on next pnpm install."
