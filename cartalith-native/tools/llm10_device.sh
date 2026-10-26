#!/usr/bin/env bash
# LLM-10 device helpers (`LLM_ASSISTANT_SCOPE.md` LLM-10; `ANDROID_BUILD_SCOPE.md` s.3.11).
# One subcommand per measurement, so a device pass is a list of short commands and every
# number in the record comes from a command printed beside it. Read-only on the handset except
# `launch`, `stop`, `tap`, `text` and `key` (which drive the app the way a finger would).
#
#   tools/llm10_device.sh sockets           # this app's TCP/UDP rows in /proc/net, by uid
#   tools/llm10_device.sh watch-sockets N   # sample `sockets` once a second for N seconds
#   tools/llm10_device.sh storage-grep STR  # copy app storage out with run-as and grep it for STR
#   tools/llm10_device.sh shot NAME [DIR]   # screencap to DIR/NAME.png (default: current dir)
#   tools/llm10_device.sh launch | stop     # am start / force-stop the app
#   tools/llm10_device.sh log [SECONDS]     # logcat for the app's pid (godot + errors), then exit
#   tools/llm10_device.sh tap X Y | text STR | key CODE
#
# Environment: ADB (default the SDK's adb.exe), SERIAL (default 9608b26b, the OnePlus 6T),
# PKG (default org.cartalith.walkingskeleton).
#
# WHY /proc/net and not `ss -p`: on Android 15 `ss -p` needs root to name the owning process,
# while /proc/net/{tcp,tcp6,udp,udp6} is readable from `adb shell` and carries the owning uid in
# column 8. A row is counted for the app when that uid equals the package's uid
# (`pm list packages -U`). LISTEN rows (state 0A) are counted separately: the zero-outbound bar
# is about connections, but a listener is reported too so nothing is hidden.
# MUST NEVER: print a key (storage-grep reports file names and counts, never matching bytes).
set -euo pipefail
ADB=${ADB:-/c/Users/Vincent/AppData/Local/Android/Sdk/platform-tools/adb.exe}
SERIAL=${SERIAL:-9608b26b}
PKG=${PKG:-org.cartalith.walkingskeleton}
a() { "$ADB" -s "$SERIAL" "$@"; }

uid() { a shell pm list packages -U "$PKG" | tr -d '\r' | sed -n "s/^package:$PKG uid://p" | head -1; }

sockets() {
  local u; u=$(uid)
  [ -n "$u" ] || { echo "package $PKG not installed" >&2; exit 2; }
  # Columns: sl local rem st ... uid(8). st 01 = ESTABLISHED, 0A = LISTEN (TCP).
  local rows; rows=$(mktemp)
  a shell "for f in tcp tcp6 udp udp6; do awk -v f=\$f -v u=$u 'NR>1 && \$8==u {print f, \$2, \$3, \$4}' /proc/net/\$f; done" \
    | tr -d '\r' > "$rows" || true
  local total listen
  total=$(grep -c . "$rows" || true)
  listen=$(awk '$4=="0A"' "$rows" | grep -c . || true)
  echo "uid=$u rows=$total listen=$listen nonlisten=$((total - listen))"
  cat "$rows"; rm -f "$rows"
}

case "${1:-}" in
  sockets) sockets ;;
  watch-sockets)
    n=${2:-10}
    for _ in $(seq 1 "$n"); do sockets | head -1; sleep 1; done ;;
  storage-grep)
    s=${2:?string to look for}
    out=$(mktemp -d)
    a exec-out run-as "$PKG" tar -cf - . > "$out/app.tar"
    tar -xf "$out/app.tar" -C "$out" 2>/dev/null || true
    files=$(find "$out" -type f ! -name app.tar | wc -l)
    hits=$(grep -rlF --exclude=app.tar -- "$s" "$out" || true)
    echo "files=$files tar_bytes=$(stat -c %s "$out/app.tar") hits=$(printf '%s' "$hits" | grep -c . || true)"
    printf '%s\n' "$hits" | sed "s#^$out/##" | grep . || true
    find "$out" -path '*llm_keystore*' -type f -printf 'keystore file %P %s bytes\n' || true
    rm -rf "$out" ;;
  shot)
    dir=${3:-.}; mkdir -p "$dir"
    a exec-out screencap -p > "$dir/$2.png"; echo "$dir/$2.png" ;;
  launch) a shell monkey -p "$PKG" -c android.intent.category.LAUNCHER 1 >/dev/null; echo launched ;;
  stop) a shell am force-stop "$PKG"; echo stopped ;;
  log)
    secs=${2:-5}
    pid=$(a shell pidof "$PKG" | tr -d '\r' || true)
    [ -n "$pid" ] || { echo "not running"; exit 1; }
    timeout "$secs" "$ADB" -s "$SERIAL" logcat --pid "$pid" -v brief || true ;;
  tap) a shell input tap "$2" "$3" ;;
  text) a shell input text "$2" ;;
  key) a shell input keyevent "$2" ;;
  *) sed -n '2,20p' "$0"; exit 2 ;;
esac
