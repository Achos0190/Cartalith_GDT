#!/usr/bin/env bash
# Build the Windows setup (Ruling HH) from a finished Windows export.
# Usage: tools/build_installer.sh <export-folder> <output-folder>
# Why a script: the version must come from godot-project/version.txt so the setup's name and
# AppVersion can never disagree with the exe inside it; and the export folder is an argument
# because exports go to varying places (builds/windows, scratch worktrees).
set -euo pipefail
exp=$(cd "${1:?export folder}" && pwd -W 2>/dev/null || pwd)
out="${2:?output folder}"; mkdir -p "$out"; out=$(cd "$out" && pwd -W 2>/dev/null || pwd)
here=$(cd "$(dirname "$0")/.." && pwd)
ver=$(tr -d '\r\n' < "$here/godot-project/version.txt")
# Fail loudly on a missing file: ISCC would too, but this names the cause.
for f in Cartalith.exe Cartalith.pck cartalith_godot.dll; do
  [ -f "$exp/$f" ] || { echo "missing $exp/$f - export first" >&2; exit 1; }
done
iscc="${ISCC:-$LOCALAPPDATA/Programs/Inno Setup 6/ISCC.exe}"
[ -x "$iscc" ] || iscc=$(command -v ISCC.exe) || { echo "ISCC.exe not found (winget install JRSoftware.InnoSetup)" >&2; exit 1; }
# MSYS_NO_PATHCONV stops Git Bash rewriting the /D and /O switches as paths.
MSYS_NO_PATHCONV=1 "$iscc" "/DExportDir=$exp" "/DAppVer=$ver" "/O$out" "$(cd "$here/installer" && pwd -W 2>/dev/null || pwd)/cartalith.iss"
echo "built $out/Cartalith-Setup-$ver.exe"
