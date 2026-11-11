#!/bin/bash
# Delta lane windowed runs. Lock C:/cw/.win_lock with owner file owner_delta; release only our own.
GODOT="/c/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
EV=/c/cw/delta_ev
OWNER="--grid 2048x1311 --river-density 1.55 --width-km 40075 --geology-model --metropolis"
release() { [ -f /c/cw/.win_lock/owner_delta ] && rm -f /c/cw/.win_lock/owner_delta && rmdir /c/cw/.win_lock 2>/dev/null; }
run_one() { # name worktree args...
  local name=$1 wt=$2; shift 2
  while ! mkdir /c/cw/.win_lock 2>/dev/null; do python -c "import time; time.sleep(20)"; done
  echo delta > /c/cw/.win_lock/owner_delta
  trap release EXIT
  echo "[$(date +%T)] $name start"
  ( cd $wt/cartalith-native/godot-project && timeout 1500 "$GODOT" --path . _riverzoom_probe.tscn -- --out "$EV/$name" "$@" > $EV/$name.log 2>&1; echo "exit $?" >> $EV/$name.log )
  release; trap - EXIT
  echo "[$(date +%T)] $name done: $(tail -1 $EV/$name.log)"
}
mode=$1; wt=$2; tag=$3
case "$mode" in
 ownerframes)
  mkdir -p $EV/${tag}_ownerf
  run_one ${tag}_ownerf $wt --seeds 246371 $OWNER --rz-frames --rz-delta-pair --rz-tag ${tag}_owner
  ;;
 frames)
  mkdir -p $EV/${tag}_owner $EV/${tag}_s483920 $EV/${tag}_s912345
  run_one ${tag}_owner $wt --seeds 246371 $OWNER --stats-only --rz-frames --rz-delta-pair --rz-tag ${tag}_owner --rim4 --rim4-n 3 --rim4-zooms 1,4,16,64
  run_one ${tag}_s483920 $wt --seeds 483920 --grid 1024x656 --rz-frames --rz-delta-pair --rz-tag ${tag}_483920 --rim4 --rim4-n 3 --rim4-zooms 1,4,16,64
  run_one ${tag}_s912345 $wt --seeds 912345 --grid 1536x983 --rz-frames --rz-delta-pair --rz-tag ${tag}_912345 --rim4 --rim4-n 3 --rim4-zooms 1,4,16,64
  ;;
esac
