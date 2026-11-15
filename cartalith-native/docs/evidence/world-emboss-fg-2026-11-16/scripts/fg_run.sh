#!/bin/bash
# usage: fg_run.sh <label> <dll> <outdir> <probe args...>
G="C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
label=$1; dll=$2; out=$3; shift 3
P=C:/cw/fgfix/cartalith-native/godot-project
until mkdir C:/cw/.win_lock 2>/dev/null; do sleep 20; done
echo "fgfix Ruling FG capture $label $(date)" > C:/cw/.win_lock/owner
mkdir -p C:/cw/fgfix/cartalith-native/target/debug "$out"
cp "$dll" C:/cw/fgfix/cartalith-native/target/debug/cartalith_godot.dll
rm -rf C:/cw/fg_appdata; mkdir -p C:/cw/fg_appdata
cd $P
APPDATA="C:/cw/fg_appdata" timeout 5600 "$G" --path . _fg_probe.tscn -- --out "$out" "$@" > C:/cw/fg_$label.log 2>&1 &
gp=$!
# peak working set of the Godot process, sampled every 2 s
( peak=0; while kill -0 $gp 2>/dev/null; do ws=$(powershell -NoProfile -Command "(Get-Process -Name 'Godot_v4.7.2-stable_win64' -ErrorAction SilentlyContinue | Measure-Object WorkingSet64 -Maximum).Maximum" 2>/dev/null | tr -d ''); [ -n "$ws" ] && [ "$ws" -gt "$peak" ] 2>/dev/null && peak=$ws && echo $peak > C:/cw/fg_${label}_peakmem.txt; sleep 3; done ) &
wait $gp
echo "exit $?" >> C:/cw/fg_$label.log
rm -rf C:/cw/.win_lock
