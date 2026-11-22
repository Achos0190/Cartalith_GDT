#!/bin/bash
# usage: runlive.sh <ctx> <scenario> <n>   (the window lock is held for the Godot run only)
G="C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
CTX=$1; SC=$2; N=$3
OUT=C:/cw/fx_scratch/c$CTX; mkdir -p $OUT/shots
TAG=${SC}_r$N
LOG=$OUT/proxy_$TAG.jsonl; rm -f $LOG
for i in $(seq 1 400); do mkdir C:/cw/.win_lock 2>/dev/null && break; sleep 5; done
[ -d C:/cw/.win_lock ] || { echo NOLOCK; exit 3; }
python -I C:/cw/fx/cartalith-native/docs/evidence/fx-context-2026-11-22/tools/asst_proxy.py 1235 1234 $LOG & PP=$!
sleep 1
rm -rf C:/cw/fx_appdata_live; mkdir -p C:/cw/fx_appdata_live C:/cw/fx_vault
cd C:/cw/fx_proj/godot-project
timeout 900 env APPDATA=C:/cw/fx_appdata_live "$G" --path . --resolution 1700x1000 res://_llmlive_run_probe.tscn -- --scenario $SC --model qwen3.8-27b --port 1235 --vault C:/cw/fx_vault --out $OUT --shots $OUT/shots --tag _r$N --ctx $CTX > $OUT/godot_$TAG.log 2>&1
echo "rc=$? $SC r$N ctx $CTX"
kill $PP 2>/dev/null
rmdir C:/cw/.win_lock
