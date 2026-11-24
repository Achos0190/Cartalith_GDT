"""Literal mutants of the sea-lane end attachment, each run against `tools/sea_lane_ends_drive.gd` in the real shell.

  python -I mutants.py FORM OUT_DIR

Each mutant replaces one exact literal (it must occur exactly once) in the PROJECT COPY's `map_overlay.gd`
(`C:/cw/lanes_proj/godot-project`, or SL_PROJ), runs the drive, and restores the file in a `finally`. KILLED = the drive
exits 1 with a FAIL line; SURVIVED = it exits 0; BUILD_ERROR = a script error or no result. The file's hash is checked
against its value before the run at the end."""
import hashlib, os, pathlib, subprocess, sys

PROJ = pathlib.Path(os.environ.get("SL_PROJ", "C:/cw/lanes_proj/godot-project"))
F = PROJ / "map_overlay.gd"
form, out = sys.argv[1], sys.argv[2]
MUTANTS = [
    ("no attach", "points = _attach_route_ends(points, route[\"brks\"])", "pass"),
    ("last end only", "for e in [points.size() - 1, 0]:", "for e in [points.size() - 1]:"),
    ("never an exact corner", "if not (p.is_equal_approx(Vector2(c)) and _settle_by_cell.has(c)):", "if true:"),
    ("cell index off by one", "_settle_by_cell[Vector2i(int(st[\"x\"]), int(st[\"y\"]))] = i", "_settle_by_cell[Vector2i(int(st[\"x\"]) + 1, int(st[\"y\"]))] = i"),
    ("no cut (end vertex moved only)", "	out = _cut_to(out, m)", "	out[out.size() - 1] = m"),
    ("cut reach without the cell", "var reach := end.distance_to(m) + 1.0", "var reach := 0.0"),
]
orig = F.read_bytes()
h0 = hashlib.sha256(orig).hexdigest()
here = pathlib.Path(__file__).resolve().parent
res = []
try:
    for name, a, b in MUTANTS:
        t = orig.decode("utf-8")
        assert t.count(a) == 1, (name, t.count(a))
        F.write_bytes(t.replace(a, b).encode("utf-8"))
        r = subprocess.run([sys.executable, "-I", str(here / "run_drive.py"), form, "%s/%s" % (out, name.replace(" ", "_")), "900"],
                           capture_output=True, text=True, encoding="utf-8", errors="replace")
        o = r.stdout
        if "SCRIPT ERROR" in o or not o.startswith(("completed", "failed")):
            v = "BUILD_ERROR"
        else:
            v = "KILLED" if r.returncode == 1 and "FAIL" in o else ("SURVIVED" if r.returncode == 0 else "BUILD_ERROR")
        fails = [l for l in o.splitlines() if l.startswith("FAIL")]
        res.append("%-32s %s  %s" % (name, v, " | ".join(fails)[:300]))
        print(res[-1], flush=True)
finally:
    F.write_bytes(orig)
assert hashlib.sha256(F.read_bytes()).hexdigest() == h0, "map_overlay.gd not restored"
print("restored, hash", h0[:12])
