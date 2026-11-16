#!/bin/bash
# usage: fg_run.sh <label> <dll> <outdir> <probe args...>
G="C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
label=$1; dll=$2; out=$3; shift 3
P=C:/cw/fgleft/cartalith-native/godot-project
until mkdir C:/cw/.win_lock 2>/dev/null; do sleep 20; done
trap "rm -rf C:/cw/.win_lock" EXIT
echo "fgleft Ruling FG leftover capture $label $(date)" > C:/cw/.win_lock/owner
mkdir -p C:/cw/fgleft/cartalith-native/target/debug "$out"
cp "$dll" C:/cw/fgleft/cartalith-native/target/debug/cartalith_godot.dll
rm -rf C:/cw/fgleft_appdata; mkdir -p C:/cw/fgleft_appdata
cd $P
APPDATA="C:/cw/fgleft_appdata" timeout 5600 "$G" --path . _fg_probe.tscn -- --out "$out" "$@" > C:/cw/fgleft_$label.log 2>&1 &
gp=$!
wait $gp
echo "exit $?" >> C:/cw/fgleft_$label.log
rm -rf C:/cw/.win_lock
