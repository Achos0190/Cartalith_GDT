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
# Optional self-signing (Ruling HI; SignPath replaces this later, see TOOLCHAIN.md). Signs with the
# CurrentUser\My code-signing cert whose subject is CN=Cartalith (self-signed), found by subject; the
# key is never exported or printed. No such cert: warn and skip, the build still succeeds.
sign() {
  # PSModulePath is unset: a PowerShell 7 parent leaves it pointing at modules Windows PowerShell 5.1 cannot load.
  env -u PSModulePath CART_SIGN_FILES="$*" powershell.exe -NoProfile -Command '
    $c = Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert | Where-Object { $_.Subject -eq "CN=Cartalith (self-signed)" } | Select-Object -First 1
    if (-not $c) { exit 3 }
    foreach ($f in $env:CART_SIGN_FILES -split "\|") {
      $r = Set-AuthenticodeSignature -FilePath $f -Certificate $c -HashAlgorithm SHA256
      if ($r.Status -ne "Valid" -and $r.Status -ne "UnknownError") { Write-Error "$f : $($r.Status)"; exit 1 }
    }' </dev/null
}
signing=1
signs=("$exp/Cartalith.exe" "$exp/cartalith_godot.dll")
[ -f "$exp/Cartalith.console.exe" ] && signs+=("$exp/Cartalith.console.exe")
joined=$(IFS='|'; echo "${signs[*]}")
if sign "$joined"; then echo "signed: ${signs[*]}"; else rc=$?
  [ "$rc" = 3 ] && { echo "WARNING: no 'CN=Cartalith (self-signed)' code-signing cert in CurrentUser\My; building unsigned" >&2; signing=0; } || exit "$rc"
fi
# MSYS_NO_PATHCONV stops Git Bash rewriting the /D and /O switches as paths.
MSYS_NO_PATHCONV=1 "$iscc" "/DExportDir=$exp" "/DAppVer=$ver" "/O$out" "$(cd "$here/installer" && pwd -W 2>/dev/null || pwd)/cartalith.iss"
[ "$signing" = 1 ] && sign "$out/Cartalith-Setup-$ver.exe" && echo "signed setup"
echo "built $out/Cartalith-Setup-$ver.exe"
