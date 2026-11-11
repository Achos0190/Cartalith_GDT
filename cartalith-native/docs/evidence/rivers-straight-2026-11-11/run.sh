#!/bin/bash
# Rivers lane probe runs. Windowed runs take C:/cw/.win_lock with owner file owner_rivers; release only our own.
GODOT="/c/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
EV=/c/cw/rivers_ev
WT=/c/cw/rivers
OWNER="--grid 2048x1311 --river-density 1.55 --width-km 40075 --geology-model --metropolis"
release() { [ -f /c/cw/.win_lock/owner_rivers ] && rm -f /c/cw/.win_lock/owner_rivers && rmdir /c/cw/.win_lock 2>/dev/null; }
headless() { local name=$1; shift; mkdir -p $EV/$name
  ( cd $WT/cartalith-native/godot-project && timeout 1500 "$GODOT" --headless --path . _riverzoom_probe.tscn -- --out "$EV/$name" "$@" > $EV/$name.log 2>&1; echo "exit $?" >> $EV/$name.log ); }
windowed() { local name=$1; shift; mkdir -p $EV/$name
  while ! mkdir /c/cw/.win_lock 2>/dev/null; do python -c "import time; time.sleep(20)"; done
  echo rivers > /c/cw/.win_lock/owner_rivers; trap release EXIT
  ( cd $WT/cartalith-native/godot-project && timeout 1800 "$GODOT" --path . _riverzoom_probe.tscn -- --out "$EV/$name" "$@" > $EV/$name.log 2>&1; echo "exit $?" >> $EV/$name.log )
  release; trap - EXIT; }
"$@"
