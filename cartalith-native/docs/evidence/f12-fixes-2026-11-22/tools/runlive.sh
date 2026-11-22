#!/bin/bash
# usage: runlive.sh <scenario> <n> [--revise 1]
G="C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
SC=$1; N=$2; shift 2
OUT=C:/cw/f12_scratch/live; mkdir -p $OUT/shots
LOG=$OUT/proxy_${SC}_r$N.jsonl; rm -f $LOG
for i in $(seq 1 200); do mkdir C:/cw/.win_lock 2>/dev/null && break; sleep 5; done
[ -d C:/cw/.win_lock ] || { echo NOLOCK; exit 3; }
python -I C:/cw/f12_scratch/asst_proxy.py 1235 1234 $LOG & PP=$!
sleep 1
rm -rf C:/cw/f12_appdata_live; mkdir -p C:/cw/f12_appdata_live C:/cw/f12_vault
cd C:/cw/f12_proj/godot-project
timeout 900 env APPDATA=C:/cw/f12_appdata_live "$G" --path . --resolution 1700x1000 res://_llmlive_run_probe.tscn -- --scenario $SC --model qwen3.8-27b --port 1235 --vault C:/cw/f12_vault --out $OUT --shots $OUT/shots --tag _r$N "$@" > $OUT/godot_${SC}_r$N.log 2>&1
echo "rc=$? $SC r$N"
kill $PP 2>/dev/null
rmdir C:/cw/.win_lock
