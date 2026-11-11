#!/bin/bash
# RZ windowed runs for the rz2 lane. Takes C:/cw/.win_lock per run with an owner file "rz2" and releases only its own.
GODOT="/c/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
EV=/c/cw/rz2/cartalith-native/docs/evidence/rz-2026-11-11
mkdir -p $EV
OWNER="--grid 2048x1311 --river-density 1.55 --width-km 40075 --geology-model --metropolis"
SMALL="--grid 1024x656 --width-km 2000"
GLOBAL1024="--grid 1024x656 --width-km 40075"

run_one() { # name worktree args...
  local name=$1 wt=$2; shift 2
  while ! mkdir /c/cw/.win_lock 2>/dev/null; do sleep 20; done
  echo rz2 > /c/cw/.win_lock/owner_rz2
  echo "[$(date +%T)] $name start"
  ( cd $wt/cartalith-native/godot-project && timeout 1800 "$GODOT" --path . _riverzoom_probe.tscn -- --out "$EV" "$@" > $EV/$name.log 2>&1; echo "exit $?" >> $EV/$name.log )
  [ -f /c/cw/.win_lock/owner_rz2 ] && rm -f /c/cw/.win_lock/owner_rz2 && rmdir /c/cw/.win_lock
  echo "[$(date +%T)] $name done: $(tail -1 $EV/$name.log)"
}
NEW=/c/cw/rz2
BASE=/c/cw/rz2base
case "$1" in
 new)
  run_one rz_owner_246371 $NEW --seeds 246371 $OWNER --rz --rz-frames --rz-tag after_owner
  run_one rz_owner_others $NEW --seeds 483920,912345 $OWNER --rz
  run_one rz_small_2000km $NEW --seeds 246371 $SMALL --rz --rz-frames --rz-tag after_small
  run_one rz_1024_world $NEW --seeds 246371,483920 $GLOBAL1024 --rz
  run_one rz_cost_new $NEW --seeds 246371 $OWNER --rz-cost
  ;;
 base)
  run_one rz_base_owner $BASE --seeds 246371 $OWNER --rz-frames --rz-tag before_owner
  run_one rz_base_small $BASE --seeds 246371 $SMALL --rz-frames --rz-tag before_small
  run_one rz_cost_base $BASE --seeds 246371 $OWNER --rz-cost
  ;;
esac
