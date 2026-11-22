#!/bin/bash
# usage: runprobe.sh <tag> <resolution> <extra args after --> ; takes the window lock, runs the card probe, releases it
G="C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
TAG=$1; RES=$2; shift 2
for i in $(seq 1 100); do mkdir C:/cw/.win_lock 2>/dev/null && break; sleep 5; done
[ -d C:/cw/.win_lock ] || { echo NOLOCK; exit 3; }
cd C:/cw/f12_proj/godot-project
timeout 300 env APPDATA=C:/cw/f12_appdata "$G" --path . --resolution $RES res://_f12_card_probe.tscn -- --shots C:/cw/f12_scratch/shots --tag $TAG "$@" 2>&1 | grep -E "^(PASS|FAIL|RESULT|ABORT|WATCHDOG)|SCRIPT ERROR|Parse Error"
rc=${PIPESTATUS[0]}
rmdir C:/cw/.win_lock
exit $rc
