"""Exact-literal GDScript mutants; each must make the coast drive print a matching FAIL. Restores in finally."""
import subprocess, pathlib, os, hashlib, sys
root = pathlib.Path("C:/cw/coast/cartalith-native/godot-project")
NL = chr(13) + chr(10)
M = [
 ("G1 coast ink ignores the preset", "map_overlay.gd", '	if not (_style_treatment.get("coast_ink") is Color):\n		return COAST_LINE_INK', '	if true:\n		return COAST_LINE_INK', "", "coast line: its own ink"),
 ("G2 border ink ignores the preset", "map_overlay.gd", '	if not (_style_treatment.get("border_ink") is Color or _style_treatment.has("border_alpha")):\n		return base', '	if true:\n		return base', "", "line: its own ink"),
 ("G3 the export drops the coast", "map_overlay.gd", '	_show_coast_lines = bool(content.get("coast", false))\n', '	_show_coast_lines = false\n', "export", "export with coast differs"),
 ("G4 the layer arm does not switch the stroke", "shell/viewport_host.gd", '			if overlay.has_method("set_show_coast_lines"):\n				overlay.set_show_coast_lines(shown)', '			if overlay.has_method("set_show_coast_lines"):\n				pass', "", "clicking the row turns the layer on"),
 ("G5 hidden runs are never drawn", "map_overlay.gd", '			if (int(h["mask"]) & drawn) == 0 or ppc >= COAST_HIDE_BELOW_PX:', '			if false:', "", "is hidden while the border is drawn"),
 ("G6 the stroke is scaled by the wrong cell count", "map_overlay.gd", '	var xf := Transform2D(Vector2(rect.size.x / _gw, 0.0), Vector2(0.0, rect.size.y / _gh), rect.position)', '	var xf := Transform2D(Vector2(rect.size.x / (_gw + 1), 0.0), Vector2(0.0, rect.size.y / _gh), rect.position)', "", "stroke is drawn where the trace says"),
 ("G7 always the coarse copy", "map_overlay.gd", 'const COAST_COARSE_BELOW_PX := 1.5', 'const COAST_COARSE_BELOW_PX := 1000.0', "", "stroke is drawn where the trace says"),
 ("G9 always the 16th-vertex copy", "map_overlay.gd", 'const COAST_COARSEST_BELOW_PX := 0.5', 'const COAST_COARSEST_BELOW_PX := 1000.0', "", "stroke is drawn where the trace says"),
 ("G8 hidden runs never drawn at high zoom", "map_overlay.gd", 'const COAST_HIDE_BELOW_PX := 4.0', 'const COAST_HIDE_BELOW_PX := 100000.0', "", "from 4 px a cell up the run is drawn"),
]
sel = sys.argv[1:]
res = []
for name, f, old, new, mode, expect in M:
    if sel and name.split()[0] not in sel:
        continue
    p = root / f
    orig = p.read_bytes(); h0 = hashlib.sha256(orig).hexdigest()
    s = orig.decode("utf-8")
    old = old.replace(chr(10), NL); new = new.replace(chr(10), NL)
    assert s.count(old) == 1, (name, s.count(old))
    try:
        p.write_bytes(s.replace(old, new).encode("utf-8"))
        env = dict(os.environ, COAST_TIMEOUT="900")
        if mode != "export":
            env["COAST_SKIP_EXPORT"] = "1"
        else:
            env["COAST_EXPORT_BASE"] = "C:/cw/coast_frames/d1/desktop_export_off.png"
        log = pathlib.Path("C:/cw/coast_mut_%s.log" % name.split()[0])
        r = subprocess.run(["python", "-I", "C:/cw/coast_drive_run.py", "desktop", "1024", "m" + name.split()[0]], env=env, capture_output=True, text=True)
        out = r.stdout + r.stderr
        log.write_text(out, encoding="utf-8")
        fails = [l for l in out.splitlines() if l.startswith("FAIL") or "SCRIPT ERROR" in l]
        hit = any(expect in l for l in fails)
        done = any(l.startswith("PROOF") for l in out.splitlines())
        res.append((name, "KILLED" if hit else ("NO_RESULT" if not done else ("FAILED-ELSEWHERE" if fails else "SURVIVED")), fails[:3]))
    finally:
        p.write_bytes(orig)
        assert hashlib.sha256(p.read_bytes()).hexdigest() == h0
    print(res[-1], flush=True)
