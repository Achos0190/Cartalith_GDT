"""Named literal mutants for the live label cull (2026-11-23). Each replaces one literal that occurs exactly once in a SCRATCH copy of
the project, runs `labels_cull_drive.gd` HEADLESS (the census is geometry, not pixels, so no window lock; the 64 px headless viewport
is a degenerate fit that makes overlaps likely, which is what a mutant needs), and requires a FAIL line. Restores in `finally`.
Usage: python labels_cull_mutants.py <scratch godot-project dir> <wrapper (C:/cw/lblcull_head.py) args...>
SURVIVED means the guard is a decoration; BUILD_ERROR means the drive did not run (not a kill)."""
import sys, pathlib, subprocess
P = pathlib.Path(sys.argv[1]); CMD = sys.argv[2:]   # CMD: the lblcull_head.py command line up to (not incl.) nothing else
M = [  # name, file, old, new, expected FAIL text
 ("no_collision_test", "map_overlay.gd", "			if k.intersects(box):", "			if false and k.intersects(box):", "FAIL overlaps"),
 ("toggle_not_mirrored", "shell/workspaces/cartography_workspace.gd", "set_label_live_cull(_label_cull)", "set_label_live_cull(true)", "FAIL toggle off"),
 ("zoom_not_in_key", "map_overlay.gd", "var key := [_label_live_cull, _labels_rev, _camera_zoom, rect.size,", "var key := [_label_live_cull, _labels_rev, rect.size,", "FAIL the live cull cache"),
 ("rank_inverted", "map_overlay.gd", "			return ra < rb\n", "			return ra > rb\n", "FAIL the continental"),
]
for name, f, old, new, want in M:
    p = P / f; raw = p.read_bytes(); t = raw.decode("utf-8").replace("\r\n", "\n")
    if t.count(old) != 1: print(name, "BUILD_ERROR: literal occurs", t.count(old)); continue
    try:
        p.write_bytes(t.replace(old, new).encode("utf-8"))
        r = subprocess.run(CMD, capture_output=True, text=True, encoding="utf-8", errors="replace")
        out = r.stdout
        if "completed" not in out: print(name, "BUILD_ERROR", out[:300]); continue
        hit = [l for l in out.splitlines() if l.startswith("FAIL") and want in l]
        print(name, "KILLED" if hit else "SURVIVED", "|", hit[:1])
    finally:
        p.write_bytes(raw)
