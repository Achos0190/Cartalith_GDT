"""Ruling HU: run a drive script against the real shell on one form, from this lane's scratch project copy (HU_ROOT, default C:/cw/hu_dev).

  python -I run_drive.py FORM OUT_DIR TIMEOUT_S SCRIPT [KEY=VALUE ...]
      FORM: desktop | tablet | phone;  SCRIPT: res://tools/bank_offer_drive.gd or res://tools/town_bank_drive.gd;
      the KEY=VALUE are the script's inputs (HN_* or HM_*; HN_OUT and HM_OUT are set to OUT_DIR).
      Environment: HU_PROJ, a project copy other than this lane's scratch one; HU_APPDATA_TAG, another scratch APPDATA.

It holds `C:/cw/.win_lock` only while the window is open, WITH an owner file written at once (MISTAKES #26: a lock with
no owner file is never ours; it is waited for and never deleted), and reports LOCK LOST if the owner file is gone when
the run ends, in which case the frames must be taken again. APPDATA and LOCALAPPDATA are a scratch directory per form,
so the owner's settings, worlds and Vault are never read."""
import importlib.util, os, pathlib, sys, time

GODOT = "C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
ROOT = os.environ.get("HU_ROOT", "C:/cw/hu_dev")
PROJ = os.environ.get("HU_PROJ", ROOT + "/godot-project")   # HU_PROJ: another project copy (the build before this ruling, for the BEFORE frames)
spec = importlib.util.spec_from_file_location("fd", "C:/cw/hu/cartalith-native/tools/fennara_drive.py")
fd = importlib.util.module_from_spec(spec); spec.loader.exec_module(fd)
form, out, timeout, script = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4]
tag = os.environ.get("HU_APPDATA_TAG", form)
env = {"APPDATA": "%s/appdata_%s" % (ROOT, tag), "LOCALAPPDATA": "%s/localappdata_%s" % (ROOT, tag), "HN_OUT": out, "HM_OUT": out, "CARTALITH_CONFIG_PATH": "%s/config_%s/cartalith.cfg" % (ROOT, tag)}
env.update(kv.split("=", 1) for kv in sys.argv[5:])
for k in ("APPDATA", "LOCALAPPDATA"): pathlib.Path(env[k]).mkdir(parents=True, exist_ok=True)
pathlib.Path(env["CARTALITH_CONFIG_PATH"]).parent.mkdir(parents=True, exist_ok=True)
LOCK = pathlib.Path("C:/cw/.win_lock"); OWNER = LOCK / "owner_hu_ruling_lane.txt"
while True:
    try: LOCK.mkdir(); break
    except FileExistsError: time.sleep(5)
lost = False
try:
    OWNER.write_text("Ruling HU lane (worktree C:/cw/hu), %s %s, pid %d, %s\n" % (script, form, os.getpid(), time.ctime()))
    st, lines = fd.drive(GODOT, PROJ, script, "res://shell/app.tscn", fd.FORMS[form], timeout, env, None)
    lost = not OWNER.exists()
finally:
    if OWNER.exists():
        OWNER.unlink(); LOCK.rmdir()      # ours: we made it and our owner file is still in it
print(st["status"], st.get("error", "")); print(*lines, sep="\n")
if lost: print("LOCK LOST: the owner file was removed during the run; take these frames again"); sys.exit(3)
sys.exit(0 if st["status"] == "completed" and not any("FAIL" in l or "SCRIPT ERROR" in l for l in lines) else 1)
