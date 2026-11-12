#!/bin/bash
# River-realism scope lane: windowed frames of the shipped look. Lock C:/cw/.win_lock, owner file owner_realism; release only our own.
GODOT="/c/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
WT=/c/cw/rivers_scope
EV=/c/cw/realism_ev
OWNER="--grid 2048x1311 --river-density 1.55 --width-km 40075 --geology-model --metropolis"
release() { [ -f /c/cw/.win_lock/owner_realism ] && rm -f /c/cw/.win_lock/owner_realism && rmdir /c/cw/.win_lock 2>/dev/null; }
run_one() { local name=$1; shift
  while ! mkdir /c/cw/.win_lock 2>/dev/null; do sleep 20; done
  echo realism > /c/cw/.win_lock/owner_realism; trap release EXIT
  echo "[$(date +%T)] $name start"
  ( cd $WT/cartalith-native/godot-project && timeout 1500 "$GODOT" --path . _riverzoom_probe.tscn -- --out "$EV/$name" "$@" > $EV/$name.log 2>&1; echo "exit $?" >> $EV/$name.log )
  release; trap - EXIT
  echo "[$(date +%T)] $name done: $(tail -1 $EV/$name.log)"
}
run_one owner --seeds 246371 $OWNER --zooms 1,64 --rz-frames --rz-tag cur_owner
run_one s483920 --seeds 483920 --grid 1024x656 --zooms 1,64 --rz-frames --rz-tag cur_483920
