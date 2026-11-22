"""Ruling HG shell mutants: exact literal replace in the SCRATCH project copy (never the worktree), occurs once,
BUILD_ERROR (a parse error) apart from SURVIVED, restore in finally. Each runs `_bankframes_probe.gd --quick` under the
window lock (`run_frames.sh` takes and releases it). Usage: gd_mutants.py [G1 G5 ...] (none: all)."""
import subprocess, pathlib, hashlib, os, sys

PROJ = pathlib.Path("C:/cw/hg2_scratch/godot-project")
OVERLAY = PROJ / "map_overlay.gd"
BRIDGE = PROJ / "shell" / "engine_bridge.gd"
GODOT = "C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
ENV = dict(os.environ, APPDATA="C:/cw/hg2_scratch/appdata", LOCALAPPDATA="C:/cw/hg2_scratch/localappdata")
MUTANTS = [
    ("G0 control: no mutation", OVERLAY, None, None),
    ("G1 the bank offset not added to the marker", OVERLAY, "+ (b if b is Vector2 else Vector2.ZERO), rect)", "+ Vector2.ZERO, rect)"),
    ("G2 the density rounded up to a step", OVERLAY, "floorf(log(ppc) / log(2.0) * BANK_ZOOM_STEPS)", "ceilf(log(ppc) / log(2.0) * BANK_ZOOM_STEPS)"),
    ("G3 BANK_ZOOM_STEPS 8.0 -> 1.0", OVERLAY, "const BANK_ZOOM_STEPS := 8.0", "const BANK_ZOOM_STEPS := 1.0"),
    ("G4 a zoom change does not ask again", OVERLAY, "\t_bank_dirty = true   ## Ruling HG: the drawn river's width in cells follows the zoom", "\tpass   ## Ruling HG: the drawn river's width in cells follows the zoom"),
    ("G5 the switch is not announced (no live toggle)", BRIDGE, "\t\tif key == \"civ.bank_sites\":\n\t\t\tbank_params_changed.emit()\n", "\t\tif key == \"civ.bank_sites\":\n\t\t\tpass\n"),
    ("G6 a river handed to the map again does not mark the offsets stale", PROJ / "shell" / "viewport_host.gd", "\tif overlay != null:\n\t\toverlay.bank_stale()\n", "\tif overlay != null:\n\t\tpass\n"),
]
only = sys.argv[1:]
for name, path, old, new in MUTANTS:
    if only and name.split()[0] not in only:
        continue
    src = path.read_bytes()
    try:
        if old is not None:
            text = src.decode("utf-8").replace("\r\n", "\n")
            assert text.count(old) == 1, (name, text.count(old))
            path.write_bytes(text.replace(old, new).replace("\n", "\r\n").encode("utf-8"))
            res_path = "res://" + path.relative_to(PROJ).as_posix()
            chk = subprocess.run([GODOT, "--headless", "--path", str(PROJ), "--check-only", "--script", res_path], env=ENV, capture_output=True, text=True, timeout=240)
            if "Parse Error" in chk.stdout + chk.stderr or "Failed to load script" in chk.stdout + chk.stderr:
                print(name, "-> BUILD_ERROR", flush=True)
                continue
        tag = name.split()[0]
        r = subprocess.run(["sh", "C:/cw/hg2_scratch/run_frames.sh", "C:/cw/hg2_scratch/out/gd_" + tag, "--quick"], capture_output=True, text=True, timeout=3400)
        out = r.stdout + r.stderr
        fails = [l.strip()[:150] for l in out.splitlines() if l.strip().startswith("FAIL")]
        res = [l for l in out.splitlines() if l.startswith("RESULT:")]
        if not res:
            verdict = "UNCLEAR (no RESULT line) rc=%d" % r.returncode
        elif fails:
            verdict = ("KILLED" if old is not None else "CONTROL FAILED") + ": " + " | ".join(fails)
        else:
            verdict = "SURVIVED" if old is not None else "passes (" + res[0] + ")"
        print(name, "->", verdict, flush=True)
    finally:
        path.write_bytes(src)
        assert hashlib.sha256(path.read_bytes()).hexdigest() == hashlib.sha256(src).hexdigest()
print("done")
