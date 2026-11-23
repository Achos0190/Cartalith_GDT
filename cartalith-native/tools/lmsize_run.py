"""Runs `_lmsize_probe.gd` on the three forms under the window lock (MISTAKES #26) and a scratch APPDATA.

  python -I lmsize_run.py GODOT OUT_DIR TAG [--mutant | --before]

`--before` runs the probe against the tree's committed `map_overlay.gd` (HEAD) in place of the working copy, restored in a `finally`:
the pre-fix numbers and frames. Use it before committing the fix.

`--mutant` puts the OLD landmark law back (the zoom-law factor on the landmark head) by exact-literal replacement in
`map_overlay.gd`, restores it in a `finally`, and expects the probe to FAIL on a form (exit 1); a run that cannot start (exit 2/3)
is BUILD_ERROR, never KILLED. Exit 0 = every form held (or, with --mutant, the mutant was killed).
"""
import os, pathlib, subprocess, sys, shutil, time

ROOT = pathlib.Path(__file__).resolve().parents[1] / "godot-project"
SRC = ROOT / "map_overlay.gd"
LOCK = pathlib.Path("C:/cw/.win_lock")
FORMS = {"desktop": ["--vp", "1600x900"], "tablet": ["--vp", "1700x1000", "--force-touch", "--density=1.0"],
         "phone": ["--vp", "450x975", "--force-touch", "--density", "1.094"]}
MUT = ("return _pin_head_r(_landmark_size_units(lm), _displayed_rect())", "return _pin_head_r(_landmark_size_units(lm), _displayed_rect()) * _landmark_zoom_k()")


def run_forms(godot, out, tag):
    rcs = {}
    for form, args in FORMS.items():
        while True:
            try:
                LOCK.mkdir(); break
            except FileExistsError:
                time.sleep(5)
        (LOCK / "owner").write_text("lmsize probe %s" % form)
        try:
            env = dict(os.environ, APPDATA="C:/cw/lmsize_appdata", CARTALITH_CONFIG_PATH="C:/cw/lmsize_appdata/cfg.json")
            r = subprocess.run([godot, "--path", str(ROOT), "--rendering-driver", "opengl3", "res://_lmsize_probe.tscn", "--", *args,
                                "--out", out, "--tag", tag], capture_output=True, text=True, encoding="utf-8", errors="replace", env=env)
            print("\n".join(l for l in r.stdout.splitlines() if l.startswith(("SIZE", "RATIO", "FAIL", "PASS", "==", "form", "ABORT"))))
            rcs[form] = r.returncode
        finally:
            shutil.rmtree(LOCK, ignore_errors=True)
    return rcs


if __name__ == "__main__":
    godot, out, tag = sys.argv[1:4]
    pathlib.Path(out).mkdir(parents=True, exist_ok=True)
    if "--before" in sys.argv:
        cur = SRC.read_bytes()
        try:
            SRC.write_bytes(subprocess.run(["git", "-C", str(ROOT), "show", "HEAD:cartalith-native/godot-project/map_overlay.gd"], capture_output=True, check=True).stdout)
            rcs = run_forms(godot, out, tag)
        finally:
            SRC.write_bytes(cur)
        print(rcs); sys.exit(0)
    if "--mutant" not in sys.argv:
        rcs = run_forms(godot, out, tag); print(rcs); sys.exit(0 if all(v == 0 for v in rcs.values()) else 1)
    orig = SRC.read_text(encoding="utf-8", newline="")
    a, b = MUT
    assert orig.count(a) == 1, "mutant literal must occur once"
    try:
        SRC.write_text(orig.replace(a, b), encoding="utf-8", newline="")
        rcs = run_forms(godot, out, tag + "_mut")
    finally:
        SRC.write_text(orig, encoding="utf-8", newline="")
    print(rcs)
    if any(v in (2, 3) for v in rcs.values()):
        print("BUILD_ERROR"); sys.exit(2)
    print("KILLED" if any(v == 1 for v in rcs.values()) else "SURVIVED"); sys.exit(0 if any(v == 1 for v in rcs.values()) else 1)
