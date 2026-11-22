"""Drive a Godot game through Fennara's in-game helper WITHOUT the editor or its daemon (M2 proof of concept).

TOOLING_PLAN.md s.M2. The helper autoload (addons/fennara/runtime/game_capture_helper.gd) frees itself
unless FENNARA_RT_SPEC names a JSON file; with one it polls a command directory for
{"action":"run_runtime_script", ...} files and runs a `res://` script `extends RefCounted; func run(ctx)`
whose `ctx` is runtime_script_context.gd (click_at, click_button, tap_key, snapshot, capture ...).
This launcher writes that spec, starts the game itself (so it controls the scene, the user args and a
scratch project copy), submits one script and prints the status and the script's own log lines.

  python -I fennara_drive.py GODOT_EXE PROJECT_DIR res://probe.gd [--scene res://main.tscn] [-- game args]
  python -I fennara_drive.py GODOT_EXE --selftest      # builds a scratch project, clicks a Button, asserts

WINDOWED ONLY: headless has no framebuffer and the helper's first frame wait never ends. A windowed run
steals the mouse: take C:/cw/.win_lock (mkdir) first, release it only if you made it (MISTAKES.md).
REAL SHELL (TP-2): `--shell FORM` runs the real app (`res://shell/app.tscn`) from a scratch COPY of godot-project (own
config/name, APPDATA/LOCALAPPDATA = an empty scratch dir, the repo's `target/debug/cartalith_godot.dll` copied beside it)
at the form's window size and user args, under C:/cw/.win_lock. Helpers for the script: `res://tools/drive_ctx.gd`.

  python -I fennara_drive.py GODOT --shell desktop|tablet|phone res://tools/my_script.gd --dll PATH --out DIR [--keep]

Never point it at the owner's APPDATA: use a project copy with its own config/name (SESSION_HANDOFF.md).
"""
import json, os, pathlib, shutil, subprocess, sys, tempfile, time

ADDON = pathlib.Path(__file__).resolve().parents[1] / "godot-project" / "addons" / "fennara"


# Window size and user args per form (Ruling GP item 2). The shell decides its form once at boot from these.
FORMS = {"desktop": ["--resolution", "1600x900"],
         "tablet": ["--resolution", "1700x1000", "--", "--force-touch", "--density=1.0"],
         "phone": ["--resolution", "450x975", "--", "--force-touch", "--density", "1.094"]}
LOCK = pathlib.Path("C:/cw/.win_lock")


def _kill(p):
    """Godot's *_console.exe is a launcher for the real exe: kill the whole tree, only OUR pid."""
    subprocess.run(["taskkill", "/T", "/F", "/PID", str(p.pid)], capture_output=True)


def drive(godot, proj, script, scene="res://main.tscn", game_args=(), timeout=30, env=None, out=None):
    proj = pathlib.Path(proj).resolve()
    work = pathlib.Path(tempfile.mkdtemp(prefix="fdrive_"))
    (work / "cmd").mkdir()
    spec, status, log = work / "spec.json", work / "status.json", work / "game.log"
    spec.write_text(json.dumps({"mode": "runtime_session", "session_id": "s1", "command_dir": str(work / "cmd"),
                                "artifact_dir": str(out or work / "art"), "scene_path": scene}))
    p = subprocess.Popen([godot, "--path", str(proj), *game_args], stdout=open(log, "w"),
                         stderr=subprocess.STDOUT, env=dict(os.environ, FENNARA_RT_SPEC=str(spec), **(env or {})))
    try:
        end = time.time() + timeout
        while time.time() < end and "FENNARA_RUNTIME_SESSION_READY" not in log.read_text(errors="replace"):
            time.sleep(0.2)
        (work / "cmd" / "c1.json").write_text(json.dumps({"action": "run_runtime_script", "script_run_id": "r1",
                                                         "session_id": "s1", "script_path": script, "status_path": str(status)}))
        while time.time() < end:
            if status.exists() and json.loads(status.read_text())["status"] in ("completed", "failed"):
                break
            time.sleep(0.2)
    finally:
        _kill(p)
    st = json.loads(status.read_text()) if status.exists() else {"status": "no_status"}
    def msg(l):  # a ctx.log line becomes its text; a ctx.error line becomes "FAIL ctx.error: text" so the exit code sees it
        if l.startswith("FENNARA_SCRIPT_LOG"): return json.loads(l.split(": ", 1)[1])["message"]
        if l.startswith("FENNARA_SCRIPT_ERROR"): return "FAIL ctx.error: " + json.loads(l.split(": ", 1)[1])["message"]
        return l
    lines = [msg(l) for l in log.read_text(errors="replace").splitlines()
             if l.startswith(("FENNARA_SCRIPT_LOG", "FENNARA_SCRIPT_ERROR")) or "SCRIPT ERROR" in l]
    if out: pathlib.Path(out).mkdir(parents=True, exist_ok=True); shutil.copy(log, pathlib.Path(out) / "game.log")
    shutil.rmtree(work, ignore_errors=True)
    return st, lines


def scratch_shell(dll, root):
    """A scratch copy of godot-project (probes `_*` left out) with its own config/name, the dll beside it, imported once.
    Reused while `root` exists; delete `root` to refresh after the project or dll changes."""
    root = pathlib.Path(root)
    proj = root / "godot-project"
    if not proj.exists():
        src = pathlib.Path(__file__).resolve().parents[1] / "godot-project"
        shutil.copytree(src, proj, ignore=shutil.ignore_patterns("_*", ".godot", "*.uid", "*.apk"))
        (root / "target" / "debug").mkdir(parents=True)
        shutil.copy(dll, root / "target" / "debug" / "cartalith_godot.dll")
        pg = proj / "project.godot"
        t = pg.read_text(encoding="utf-8")
        assert t.count('config/name="Cartalith Terrain Generator"') == 1
        pg.write_text(t.replace('config/name="Cartalith Terrain Generator"', 'config/name="cartalith_drive_scratch"'), encoding="utf-8")
    # the helper scripts are what a session edits: refresh them (a NEW class_name needs the root deleted so the import re-runs)
    shutil.copytree(pathlib.Path(__file__).resolve().parents[1] / "godot-project" / "tools", proj / "tools", dirs_exist_ok=True,
                    ignore=shutil.ignore_patterns("*.uid"))
    return proj


def drive_shell(godot, form, script, dll, root, out, timeout=90):
    """TP-2: run `script` (res://, `func run(ctx)`) against the real shell in the given form. Returns (status, lines)."""
    proj = scratch_shell(dll, root)
    env = {"APPDATA": str(pathlib.Path(root) / "appdata"), "LOCALAPPDATA": str(pathlib.Path(root) / "localappdata")}
    for v in env.values(): pathlib.Path(v).mkdir(exist_ok=True)
    if not (proj / ".godot").exists():
        subprocess.run([godot, "--headless", "--path", str(proj), "--import"], capture_output=True, timeout=900, env=dict(os.environ, **env))
    shutil.rmtree(out, ignore_errors=True)   # a stale capture from an earlier run must never be mistaken for this one
    made = False
    while not made:                      # the window lock is held only while the window is open
        try: LOCK.mkdir(); made = True
        except FileExistsError: time.sleep(5)
    try:
        return drive(godot, proj, script, "res://shell/app.tscn", FORMS[form], timeout, env, out)
    finally:
        LOCK.rmdir()                     # we made it (the loop only exits on our own mkdir)


def selftest(godot):
    d = pathlib.Path(tempfile.mkdtemp(prefix="fdrive_proj_"))
    shutil.copytree(ADDON, d / "addons" / "fennara", ignore=shutil.ignore_patterns("bin", "dist", "*.gdextension*"))
    (d / "project.godot").write_text('config_version=5\n\n[application]\nconfig/name="fdrive_selftest"\n'
        'run/main_scene="res://main.tscn"\nconfig/features=PackedStringArray("4.7")\n\n[autoload]\n\n'
        '_fennara_game_capture="*res://addons/fennara/runtime/game_capture_helper.gd"\n')
    (d / "main.tscn").write_text('[gd_scene format=3]\n\n[node name="Main" type="Control"]\nlayout_mode=3\nanchors_preset=15\n'
        'anchor_right=1.0\nanchor_bottom=1.0\n\n[node name="Btn" type="Button" parent="."]\nlayout_mode=0\n'
        'offset_left=20.0\noffset_top=20.0\noffset_right=200.0\noffset_bottom=60.0\ntext="Generate"\n')
    (d / "probe.gd").write_text('extends RefCounted\nfunc run(ctx: Variant) -> void:\n\tvar b: Button = ctx.node("/root/Main/Btn")\n'
        '\tvar n := [0]\n\tb.pressed.connect(func(): n[0] += 1)\n\tawait ctx.click_button(b)\n\tawait ctx.wait(0.2)\n'
        '\tctx.log("pressed_count=%d" % n[0])\n')
    subprocess.run([godot, "--headless", "--path", str(d), "--import"], capture_output=True, timeout=120)
    st, lines = drive(godot, d, "res://probe.gd", game_args=["--resolution", "640x360"])
    shutil.rmtree(d, ignore_errors=True)
    assert st["status"] == "completed" and any("pressed_count=1" in l for l in lines), (st, lines)  # a real mouse click reached the button
    print("selftest ok", lines)


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")  # script logs hold menu glyphs
    a = sys.argv[1:]
    if len(a) == 2 and a[1] == "--selftest":
        selftest(a[0])
    elif a[1:2] == ["--shell"]:
        import argparse
        ap = argparse.ArgumentParser(); ap.add_argument("godot"); ap.add_argument("--shell", choices=FORMS); ap.add_argument("script")
        ap.add_argument("--dll", required=True); ap.add_argument("--out", required=True); ap.add_argument("--root", default="C:/cw/drive_scratch")
        ap.add_argument("--timeout", type=int, default=90)
        n = ap.parse_args(a)
        st, lines = drive_shell(n.godot, n.shell, n.script, n.dll, n.root, n.out, n.timeout)
        print(st["status"], st.get("error", "")); print(*lines, sep="\n")
        sys.exit(0 if st["status"] == "completed" and not any("FAIL" in l or "SCRIPT ERROR" in l for l in lines) else 1)
    else:
        rest = a[3:]; scene = "res://main.tscn"
        if rest[:1] == ["--scene"]: scene, rest = rest[1], rest[2:]
        print(*drive(a[0], a[1], a[2], scene, rest[1:] if rest[:1] == ["--"] else rest), sep="\n")
