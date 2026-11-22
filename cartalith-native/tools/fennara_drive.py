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
Never point it at the owner's APPDATA: use a project copy with its own config/name (SESSION_HANDOFF.md).
"""
import json, os, pathlib, shutil, subprocess, sys, tempfile, time

ADDON = pathlib.Path(__file__).resolve().parents[1] / "godot-project" / "addons" / "fennara"


def drive(godot, proj, script, scene="res://main.tscn", game_args=(), timeout=30):
    proj = pathlib.Path(proj).resolve()
    work = pathlib.Path(tempfile.mkdtemp(prefix="fdrive_"))
    (work / "cmd").mkdir()
    spec, status, log = work / "spec.json", work / "status.json", work / "game.log"
    spec.write_text(json.dumps({"mode": "runtime_session", "session_id": "s1", "command_dir": str(work / "cmd"),
                                "artifact_dir": str(work / "art"), "scene_path": scene}))
    p = subprocess.Popen([godot, "--path", str(proj), *game_args], stdout=open(log, "w"),
                         stderr=subprocess.STDOUT, env=dict(os.environ, FENNARA_RT_SPEC=str(spec)))
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
        p.terminate(); p.wait(timeout=15)
    st = json.loads(status.read_text()) if status.exists() else {"status": "no_status"}
    lines = [l for l in log.read_text(errors="replace").splitlines() if l.startswith("FENNARA_SCRIPT_LOG") or "SCRIPT ERROR" in l]
    shutil.rmtree(work, ignore_errors=True)
    return st, lines


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
    a = sys.argv[1:]
    if len(a) == 2 and a[1] == "--selftest":
        selftest(a[0])
    else:
        rest = a[3:]; scene = "res://main.tscn"
        if rest[:1] == ["--scene"]: scene, rest = rest[1], rest[2:]
        print(*drive(a[0], a[1], a[2], scene, rest[1:] if rest[:1] == ["--"] else rest), sep="\n")
