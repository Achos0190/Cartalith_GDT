#!/bin/bash
# usage: zoom_run.sh <srcdir godot-project> <dll> <tag> <runs>   ; counts frames over 16.7 per run
SRC=$1; DLL=$2; TAG=$3; N=${4:-5}
GODOT="C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
S=C:/cw/zoom_scratch_$TAG
rm -rf $S; mkdir -p $S/target/debug $S/appdata $S/out
cp -r $SRC $S/godot-project
cp $DLL $S/target/debug/cartalith_godot.dll
# untracked import files from the owner's tree
(cd C:/Users/Vincent/Cartalith_GDT/cartalith-native/godot-project && find . -name '*.import' -o -name '*.uid' | grep -v "^./.godot" | cpio -pdmu $S/godot-project 2>/dev/null) 
cp -rn C:/Users/Vincent/Cartalith_GDT/cartalith-native/godot-project/.godot $S/godot-project/ 2>/dev/null
if [ -n "$IMPORT" ]; then rm -f $S/godot-project/.godot/global_script_class_cache.cfg; fi
sed -i "s/^config\/name=.*/config\/name=\"zoomscratch_$TAG\"/" $S/godot-project/project.godot
if [ -n "$IMPORT" ]; then APPDATA=$S/appdata "$GODOT" --headless --path $S/godot-project --import > $S/import.log 2>&1; fi
for i in $(seq 1 $N); do
  until mkdir C:/cw/.win_lock 2>/dev/null; do sleep 5; done
  echo zoom_$TAG > C:/cw/.win_lock/owner
  APPDATA=$S/appdata CARTALITH_CONFIG_PATH=$S/appdata/cfg "$GODOT" --path $S/godot-project _zoomcost_probe.tscn --resolution 1600x900 -- --out $S/out --seed 24601 --tag ${RTAG:-r}$i $EXTRA > $S/out/run$i.log 2>&1
  grep -q zoom_$TAG C:/cw/.win_lock/owner && rm -rf C:/cw/.win_lock
  echo "$TAG run$i: $(grep 'frames over' $S/out/run$i.log | tr -s ' ') | $(tasklist | grep -ci 'cargo\|rustc') build procs"
done
