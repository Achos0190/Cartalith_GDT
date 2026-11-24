"""Sea-lane ends (owner report 2026-11-23): run `tools/sea_lane_ends_drive.gd` against the real shell on one form.

  python -I run_drive.py FORM OUT_DIR TIMEOUT_S [KEY=VALUE ...]
      FORM: desktop | tablet | phone; KEY=VALUE are the script's inputs (SL_*; SL_OUT is set to OUT_DIR).
      Environment: SL_PROJ, the project copy (default C:/cw/lanes_proj/godot-project, made by `fennara_drive.scratch_shell`
      with the owner's *.import files copied in); SL_ROOT, where the scratch APPDATA lives (default C:/cw/lanes_proj).

Holds `C:/cw/.win_lock` only while the window is open, WITH an owner file written at once (MISTAKES #26: a lock with no
owner file is never ours; it is waited for and never deleted), and says LOCK LOST if the owner file is gone at the end.
APPDATA, LOCALAPPDATA and CARTALITH_CONFIG_PATH are scratch, per form, so the owner's settings, worlds and Vault are never read."""
import importlib.util, os, pathlib, sys, time

GODOT = "C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
ROOT = os.environ.get("SL_ROOT", "C:/cw/lanes_proj")
PROJ = os.environ.get("SL_PROJ", ROOT + "/godot-project")
here = pathlib.Path(__file__).resolve()
spec = importlib.util.spec_from_file_location("fd", str(here.parents[3] / "tools" / "fennara_drive.py"))
fd = importlib.util.module_from_spec(spec); spec.loader.exec_module(fd)
form, out, timeout = sys.argv[1], sys.argv[2], int(sys.argv[3])
env = {"APPDATA": "%s/appdata_%s" % (ROOT, form), "LOCALAPPDATA": "%s/localappdata_%s" % (ROOT, form), "SL_OUT": out,
       "CARTALITH_CONFIG_PATH": "%s/config_%s/cartalith.cfg" % (ROOT, form)}
env.update(kv.split("=", 1) for kv in sys.argv[4:])
for k in ("APPDATA", "LOCALAPPDATA"): pathlib.Path(env[k]).mkdir(parents=True, exist_ok=True)
pathlib.Path(env["CARTALITH_CONFIG_PATH"]).parent.mkdir(parents=True, exist_ok=True)
LOCK = pathlib.Path("C:/cw/.win_lock"); OWNER = LOCK / "owner_sea_lane_ends_lane.txt"
while True:
    try: LOCK.mkdir(); break
    except FileExistsError: time.sleep(5)
lost = False
try:
    OWNER.write_text("sea-lane ends lane (worktree C:/cw/lanes), %s, pid %d, %s\n" % (form, os.getpid(), time.ctime()))
    st, lines = fd.drive(GODOT, PROJ, "res://tools/sea_lane_ends_drive.gd", "res://shell/app.tscn", fd.FORMS[form], timeout, env, out)
    lost = not OWNER.exists()
finally:
    if OWNER.exists():
        OWNER.unlink(); LOCK.rmdir()      # ours: we made it and our owner file is still in it
print(st["status"], st.get("error", "")); print(*lines, sep="\n")
if lost: print("LOCK LOST: the owner file was removed during the run; take these frames again"); sys.exit(3)
sys.exit(0 if st["status"] == "completed" and not any("FAIL" in l or "SCRIPT ERROR" in l for l in lines) else 1)
