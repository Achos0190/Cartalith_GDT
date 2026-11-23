"""Ruling HM: run `tools/town_bank_drive.gd` against the real shell on one form, from this lane's scratch project copy.

  python -I run_drive.py FORM OUT_DIR TIMEOUT_S [KEY=VALUE ...]     (FORM: desktop | tablet | phone; the KEY=VALUE are HM_* inputs)

It holds `C:/cw/.win_lock` only while the window is open, WITH an owner file written at once (MISTAKES #26: a lock with
no owner file is nobody's and may be removed), and reports LOCK LOST if the owner file is gone when the run ends, in
which case the frames must be taken again. APPDATA and LOCALAPPDATA are a scratch directory."""
import importlib.util, os, pathlib, sys, time

GODOT = "C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
PROJ = "C:/cw/hm_scratch/godot-project"
spec = importlib.util.spec_from_file_location("fd", "C:/cw/hm/cartalith-native/tools/fennara_drive.py")
fd = importlib.util.module_from_spec(spec); spec.loader.exec_module(fd)
form, out, timeout = sys.argv[1], sys.argv[2], int(sys.argv[3])
env = {"APPDATA": "C:/cw/hm_scratch/appdata_" + form, "LOCALAPPDATA": "C:/cw/hm_scratch/localappdata_" + form, "HM_OUT": out}
env.update(kv.split("=", 1) for kv in sys.argv[4:])
for k in ("APPDATA", "LOCALAPPDATA"): pathlib.Path(env[k]).mkdir(parents=True, exist_ok=True)
LOCK = pathlib.Path("C:/cw/.win_lock"); OWNER = LOCK / "owner_hm_ruling_lane.txt"
while True:
    try: LOCK.mkdir(); break
    except FileExistsError: time.sleep(5)
lost = False
try:
    OWNER.write_text("Ruling HM lane (worktree C:/cw/hm), town_bank_drive %s, pid %d, %s\n" % (form, os.getpid(), time.ctime()))
    st, lines = fd.drive(GODOT, PROJ, "res://tools/town_bank_drive.gd", "res://shell/app.tscn", fd.FORMS[form], timeout, env, None)
    lost = not OWNER.exists()
finally:
    if OWNER.exists():
        OWNER.unlink(); LOCK.rmdir()      # ours: we made it and our owner file is still in it
print(st["status"], st.get("error", "")); print(*lines, sep="\n")
if lost: print("LOCK LOST: the owner file was removed during the run; take these frames again"); sys.exit(3)
sys.exit(0 if st["status"] == "completed" and not any("FAIL" in l or "SCRIPT ERROR" in l for l in lines) else 1)
