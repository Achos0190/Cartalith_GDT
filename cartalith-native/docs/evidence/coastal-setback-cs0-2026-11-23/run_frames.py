"""CS-0 frames: run `tools/town_bank_drive.gd` against the real shell (desktop form) from this lane's scratch project copy.

  python -I run_frames.py OUT_DIR TIMEOUT_S [KEY=VALUE ...]
      KEY=VALUE are the drive's inputs (HM_SEED, HM_GW, HM_GH, HM_CELLS, HM_MAP_ZOOMS, HM_PARAMS; HM_OUT is OUT_DIR).

Adapted from `docs/evidence/town-on-bank-hn-2026-11-23/run_drive.py`. It holds `C:/cw/.win_lock` only while the window
is open, with an owner file written at once (MISTAKES #26: a lock with no owner file is never ours; it is waited for and
never deleted), and reports LOCK LOST if the owner file is gone when the run ends. APPDATA and LOCALAPPDATA are a
scratch directory, so the owner's settings, worlds and Vault are never read."""
import importlib.util, os, pathlib, sys, time

GODOT = "C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
ROOT = "C:/cw/cs0_scratch"
PROJ = ROOT + "/godot-project"
spec = importlib.util.spec_from_file_location("fd", "C:/cw/cs0/cartalith-native/tools/fennara_drive.py")
fd = importlib.util.module_from_spec(spec); spec.loader.exec_module(fd)
out, timeout = sys.argv[1], int(sys.argv[2])
env = {"APPDATA": ROOT + "/appdata", "LOCALAPPDATA": ROOT + "/localappdata", "HM_OUT": out,
       "CARTALITH_CONFIG_PATH": ROOT + "/appdata/cartalith_settings.cfg"}
env.update(kv.split("=", 1) for kv in sys.argv[3:])
for k in ("APPDATA", "LOCALAPPDATA"): pathlib.Path(env[k]).mkdir(parents=True, exist_ok=True)
LOCK = pathlib.Path("C:/cw/.win_lock"); OWNER = LOCK / "owner_cs0_lane.txt"
while True:
    try: LOCK.mkdir(); break
    except FileExistsError: time.sleep(5)
lost = False
try:
    OWNER.write_text("CS-0 lane (worktree C:/cw/cs0), town_bank_drive desktop, pid %d, %s\n" % (os.getpid(), time.ctime()))
    st, lines = fd.drive(GODOT, PROJ, "res://tools/town_bank_drive.gd", "res://shell/app.tscn", fd.FORMS["desktop"], timeout, env, None)
    lost = not OWNER.exists()
finally:
    if OWNER.exists():
        OWNER.unlink(); LOCK.rmdir()      # ours: we made it and our owner file is still in it
print(st["status"], st.get("error", "")); print(*lines, sep="\n")
if lost: print("LOCK LOST: the owner file was removed during the run; take these frames again"); sys.exit(3)
