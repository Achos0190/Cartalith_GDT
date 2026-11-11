#!/bin/bash
# MS-1 B5 runner (S4.6): the parts of the bar that need more than one process.
#   ms1_b5.sh <godot-console-exe> <godot-project-dir> <out-dir>
# What it runs, each WINDOWED under the shared window lock (`C:/cw/.win_lock`, this script's own owner file, released on every
# exit path) with a WIPED user directory (APPDATA / LOCALAPPDATA point at an empty folder, `Worlds` pre-made so the engine's
# directory-listing error is not noise):
#   1. `--leg live --clean --noworld --trips 10`: ten round trips through the real resize path (orphan nodes and the receivers of
#      the seven named signals equal to the first visit's at every trip), writing `<out>/live.phone` and `<out>/live.tablet`.
#   2. `--leg norm` of a cold boot of each class at the same size, writing `<out>/norm.phone` and `<out>/norm.tablet`; the two
#      pairs are compared with `diff` (empty allow-list, spec B5: "a dump of the composition equals a cold boot's").
#   3. `--leg b5` five times, each a separate process (spec B5: "the median (min..max) of five serial runs, run alone"): each
#      prints one `B5TIME` line with that process's cold boot and the median of its twelve swaps; the medians of the five are
#      printed with their min..max. Labelled MEASURED ON THIS MACHINE; nothing here is a benchmark of another device.
# `SCRIPT ERROR` in any captured log fails the run. Exit 0 only when every leg passed and both dumps matched.
set -u
GODOT="$1"; PROJ="$2"; OUT="$3"
mkdir -p "$OUT"
LOCK=/c/cw/.win_lock
OWNER="$LOCK/owner_ms1b5"
release() { [ -f "$OWNER" ] && rm -f "$OWNER" && rmdir "$LOCK" 2>/dev/null; }
trap release EXIT
fail=0
run() {  # name, probe args...
  local name=$1; shift
  until mkdir "$LOCK" 2>/dev/null; do python -c "import time; time.sleep(10)"; done
  echo ms1b5 > "$OWNER"
  local ad="$OUT/appdata_$name"
  rm -rf "$ad"; mkdir -p "$ad/Godot/app_userdata/Cartalith Terrain Generator/Worlds"
  ( cd "$PROJ" && APPDATA="$(cygpath -w "$ad")" LOCALAPPDATA="$(cygpath -w "$ad")" timeout 900 "$GODOT" --path . --resolution 1600x1000 _ms1swap_probe.tscn -- "$@" > "$OUT/$name.log" 2>&1; echo "exit $?" >> "$OUT/$name.log" )
  release
  local res; res=$(grep -h "^RESULT" "$OUT/$name.log" | tail -1)
  echo "$name: $res"
  grep -q "SCRIPT ERROR" "$OUT/$name.log" && { echo "  $name: SCRIPT ERROR in the log"; fail=1; }
  echo "$res" | grep -q "RESULT: PASS" || fail=1
}
run live --leg live --clean --noworld --trips 10 --force-touch --vp 1800x1000 --density 1 --out "$(cygpath -w "$OUT")/live"
run norm_phone --leg norm --force-touch --vp 880x1564 --density 1 --out "$(cygpath -w "$OUT")/norm.phone"
run norm_tablet --leg norm --force-touch --vp 1800x1000 --density 1 --out "$(cygpath -w "$OUT")/norm.tablet"
for c in phone tablet; do
  if diff -q "$OUT/live.$c" "$OUT/norm.$c" > /dev/null; then echo "dump $c: equal to a cold boot ($(wc -l < "$OUT/norm.$c") lines, empty allow-list)"
  else echo "dump $c: DIFFERS from a cold boot"; diff "$OUT/live.$c" "$OUT/norm.$c" | head -20; fail=1; fi
done
for i in 1 2 3 4 5; do run b5_$i --leg b5 --force-touch --vp 1800x1000 --density 1; done
python - "$OUT" <<'PYEOF'
import re, sys, statistics
out = sys.argv[1]
rows = []
for i in range(1, 6):
    t = open(f"{out}/b5_{i}.log", encoding="utf-8", errors="replace").read()
    m = re.search(r"B5TIME cold_boot=([\d.]+) swap_sync_median=([\d.]+) swap_whole_median=([\d.]+) engine_start=(\d+)", t)
    if m:
        rows.append([float(x) for x in m.groups()])
def med(v):
    return "%.0f ms (%.0f..%.0f)" % (statistics.median(v), min(v), max(v))
print("B5 timing, five serial runs, MEASURED ON THIS MACHINE; each figure is the median (min..max) over the runs:")
if len(rows) == 5:
    cb = [r[0] for r in rows]; sy = [r[1] for r in rows]; wh = [r[2] for r in rows]; en = [r[3] for r in rows]
    tot = [r[0] + r[3] for r in rows]
    print("  cold boot, instantiate + enter tree     %s" % med(cb))
    print("  engine start-up before it (a restart)   %s" % med(en))
    print("  cold boot with the engine start-up      %s" % med(tot))
    print("  swap, synchronous part                  %s" % med(sy))
    print("  swap, whole (with its frames)           %s" % med(wh))
    print("  swap (synchronous) < instantiate alone: %s; < a restart (engine + instantiate): %s" % (
        "yes" if statistics.median(sy) < statistics.median(cb) else "NO",
        "yes" if statistics.median(sy) < statistics.median(tot) else "NO"))
else:
    print("  only %d of 5 runs produced a B5TIME line" % len(rows))
PYEOF
echo "overall: $([ $fail = 0 ] && echo PASS || echo FAIL)"
exit $fail
