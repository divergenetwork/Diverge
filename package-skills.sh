#!/usr/bin/env bash
# Pack the skills into one archive: what a release ships beside its
# platform archives, the same for every platform.
#
#   ./package-skills.sh <out-dir>
#
# The archive is <out-dir>/diverge-skills-<version>.zip, with
# <archive>.sha256 beside it. Inside it is every folder of
# `diverge-skills/`, at the top level and under its own name, with no
# folder round them: a skill is one folder, named for the skill,
# holding its SKILL.md, so the archive unpacks straight into a skills
# directory. `diverge update <SKILLS_DIR>` installs it so.
#
# The version is the SDK crate's, as `package-diverge.sh` reads it.
# Nothing is built.
set -euo pipefail

[[ $# -eq 1 && "$1" != --* ]] || { echo "usage: $0 <out-dir>" >&2; exit 2; }
out="$1"

workspace="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
version="$(awk -F'"' '/^version = "[^"]+"/ { print $2; exit }' "$workspace/diverge-sdk-rs/Cargo.toml")"
[[ -n "$version" ]] || { echo "error: no version in diverge-sdk-rs/Cargo.toml" >&2; exit 1; }

skills="$workspace/diverge-skills"
[[ -d "$skills" ]] || { echo "error: no skills at $skills" >&2; exit 1; }
# Every skill is a folder holding a SKILL.md; anything else there is
# not one, and is left out.
names=()
for folder in "$skills"/*/; do
    name="$(basename "$folder")"
    [[ -f "$folder/SKILL.md" ]] || { echo "error: $name holds no SKILL.md" >&2; exit 1; }
    names+=("$name")
done
[[ ${#names[@]} -gt 0 ]] || { echo "error: no skill in $skills" >&2; exit 1; }

mkdir -p "$out"
out="$(cd "$out" && pwd)"
archive="diverge-skills-$version.zip"
rm -f "$out/$archive"
if command -v zip >/dev/null 2>&1; then
    (cd "$skills" && zip -r -q -X "$out/$archive" "${names[@]}")
else
    # Git Bash has no `zip`; Windows' own tar is bsdtar, and writes a
    # zip by the name's extension.
    (cd "$skills" && "${SYSTEMROOT:-C:\\Windows}/System32/tar.exe" -a -c -f "$out/$archive" "${names[@]}")
fi

if command -v sha256sum >/dev/null 2>&1; then
    (cd "$out" && sha256sum "$archive" > "$archive.sha256")
else
    (cd "$out" && shasum -a 256 "$archive" > "$archive.sha256")
fi
echo "packed $out/$archive (${#names[@]} skills)"
