"""Ruling HS frames: `tools/town_bank_drive.gd` in the real shell (desktop) from a project copy.

  python -I run_frames.py ROOT OUT_DIR TIMEOUT_S [KEY=VALUE ...]
      ROOT holds godot-project/ and target/debug/cartalith_godot.dll (the before or the after build); KEY=VALUE are the
      drive's inputs (HM_SEED, HM_GW, HM_GH, HM_CELLS, HM_MAP_ZOOMS, HM_PARAMS; HM_OUT is OUT_DIR).

From `docs/evidence/coastal-setback-cs0-2026-11-23/run_frames.py`. It takes `C:/cw/.win_lock` with mkdir and writes its
owner file at once, holds it only while the window is open, never deletes a lock it did not make (MISTAKES #26), and
runs under a scratch APPDATA and CARTALITH_CONFIG_PATH."""
import importlib.util, os, pathlib, sys, time

GODOT = "C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
root, out, timeout = sys.argv[1], sys.argv[2], int(sys.argv[3])
spec = importlib.util.spec_from_file_location("fd", "C:/cw/hz/cartalith-native/tools/fennara_drive.py")
fd = importlib.util.module_from_spec(spec); spec.loader.exec_module(fd)
env = {"APPDATA": root + "/appdata", "LOCALAPPDATA": root + "/localappdata", "HM_OUT": out,
       "CARTALITH_CONFIG_PATH": root + "/appdata/cartalith_settings.cfg"}
env.update(kv.split("=", 1) for kv in sys.argv[4:])
for k in ("APPDATA", "LOCALAPPDATA"): pathlib.Path(env[k]).mkdir(parents=True, exist_ok=True)
LOCK = pathlib.Path("C:/cw/.win_lock"); OWNER = LOCK / "owner_hz_lane.txt"
while True:
    try: LOCK.mkdir(); break
    except FileExistsError: time.sleep(5)
lost = False
try:
    OWNER.write_text("Ruling HZ lane (worktree C:/cw/hz), town_bank_drive desktop, pid %d, %s\n" % (os.getpid(), time.ctime()))
    st, lines = fd.drive(GODOT, root + "/godot-project", os.environ.get("CS_SCRIPT", "res://tools/town_bank_drive.gd"), "res://shell/app.tscn", fd.FORMS["desktop"], timeout, env, None)
    lost = not OWNER.exists()
finally:
    if OWNER.exists():
        OWNER.unlink(); LOCK.rmdir()
print(st["status"], st.get("error", "")); print(*lines, sep="\n")
if lost: print("LOCK LOST"); sys.exit(3)
