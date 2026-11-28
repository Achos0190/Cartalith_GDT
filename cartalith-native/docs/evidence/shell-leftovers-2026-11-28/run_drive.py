"""Run one drive script on one form from the shellc scratch (own root; lock held only while the window is open)."""
import sys, os, pathlib, shutil, time
sys.path.insert(0, "C:/cw/shellc/cartalith-native/tools")
import fennara_drive as fd
form, script, tag = sys.argv[1], sys.argv[2], sys.argv[3]
G = "C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
root = pathlib.Path("C:/cw/shellc_scratch"); S = pathlib.Path("C:/cw/shellc/cartalith-native/godot-project")
for f in ([] if os.environ.get("SC_NOCOPY") else list(S.glob("*.gd")) + list((S / "shell").rglob("*.gd")) + list((S / "tools").glob("*.gd"))):
    if f.name.startswith("_"): continue
    d = root / "godot-project" / f.relative_to(S); d.parent.mkdir(parents=True, exist_ok=True); shutil.copy(f, d)
src, dst = pathlib.Path("C:/cw/shellc_target/debug/cartalith_godot.dll"), root / "target" / "debug" / "cartalith_godot.dll"
shutil.copy(src, dst)
env = {"APPDATA": str(root / "appdata"), "LOCALAPPDATA": str(root / "localappdata"),
       "CARTALITH_CONFIG_PATH": str(root / "appdata" / "cartalith_settings.cfg")}
for k, v in os.environ.items():
    if k.startswith(("PINCH_", "COAST_", "SC_")): env[k] = v
lock = pathlib.Path("C:/cw/.win_lock")
while True:
    try: lock.mkdir(); break
    except FileExistsError: time.sleep(5)
(lock / "owner_shellc.txt").write_text("shellc lane, python pid %d" % os.getpid())
try:
    st, lines = fd.drive(G, root / "godot-project", script, "res://shell/app.tscn", fd.FORMS[form], int(os.environ.get("SC_TIMEOUT", "240")), env, f"C:/cw/shellc_art_{tag}")
finally:
    try: (lock / "owner_shellc.txt").unlink()
    except OSError: pass
    lock.rmdir()
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
print(st["status"], st.get("error", "")); print(*lines, sep="\n")
