#!/bin/bash
# usage: run_export.sh <scratch_project_dir> <width> <label> [extra probe args]
# Takes C:/cw/.win_lock (owner file = expo-1128), runs the export probe windowed, releases only if owner is ours.
PROJ=$1; W=$2; L=$3; shift 3
GODOT="C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
LOCK=C:/cw/.win_lock; TAG=expo-1128-$L-$$
until mkdir $LOCK 2>/dev/null && echo $TAG > $LOCK/owner; do sleep 5; done
trap 'grep -q "$TAG" $LOCK/owner 2>/dev/null && rm -rf $LOCK' EXIT
APPDATA=$PROJ/../appdata LOCALAPPDATA=$PROJ/../localappdata "$GODOT" --path $PROJ $GODOT_EXTRA --resolution 1600x1000 res://_exportfull_probe.tscn -- --width $W "$@" 2>&1 | grep -E "RESULT|wall|run:|phase|step|generated|ABORT|WATCH|FILE|SCRIPT ERROR|Parse"
