"""The overlay's mutant: `_road_points_drawn` no longer draws a named end at its marker below the town layer.

  python -I drive_mutant.py SCRATCH_ROOT OUT_DIR

Mutates SCRATCH_ROOT/godot-project/map_overlay.gd (a scratch copy, never the worktree's) by exact literal, runs
`tools/road_attach_drive.gd` on seed 483920 at 384 x 288 through `run_drive.py` (the window lock, a scratch APPDATA),
and restores the file in a `finally`. Check 2 must FAIL."""
import hashlib, pathlib, subprocess, sys

root, out = pathlib.Path(sys.argv[1]), sys.argv[2]
f = root / "godot-project/map_overlay.gd"
OLD = "\t\t\tout[k[1]] = _settle_grid(s)"
NEW = "\t\t\tout[k[1]] = out[k[1]]"
src = f.read_bytes()
h0 = hashlib.sha256(src).hexdigest()
text = src.decode("utf-8")
assert text.count(OLD) == 1, "the literal occurs %d times" % text.count(OLD)
try:
    f.write_bytes(text.replace(OLD, NEW).encode("utf-8"))
    p = subprocess.run([sys.executable, "-I", "C:/cw/regr_s/run_drive.py", str(root), "desktop", "1500", "res://tools/road_attach_drive.gd",
                        "RA_SEED=483920", "RA_GW=384", "RA_GH=288", "RA_TOWNS=19,0,10", "RA_OUT=" + out], capture_output=True, text=True, timeout=2400)
    lines = [l for l in p.stdout.splitlines() if l.startswith(("ok", "FAIL", "RESULT"))]
    print(*lines, sep="\n")
    print("D1 the low-zoom end at the marker ->", "KILLED" if any(l.startswith("FAIL  2.") for l in lines) else "SURVIVED")
finally:
    f.write_bytes(src)
    assert hashlib.sha256(f.read_bytes()).hexdigest() == h0
