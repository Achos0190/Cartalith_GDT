"""Own runner (the --shell launcher's import step hangs 15 min on the console wrapper): scratch is already imported."""
import sys, os, pathlib, shutil, time
sys.path.insert(0, "C:/cw/coast/cartalith-native/tools")
import fennara_drive as fd
form, grid, tag = sys.argv[1], sys.argv[2], sys.argv[3]
script = sys.argv[4] if len(sys.argv) > 4 else "res://tools/coast_drive.gd"
G = "C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
root = pathlib.Path("C:/cw/coast_scratch")
# refresh helper scripts and changed .gd files from the worktree into the scratch copy
src = pathlib.Path("C:/cw/coast/cartalith-native/godot-project")
for f in ["map_overlay.gd", "shell/viewport_host.gd", "shell/engine_bridge.gd", "shell/export_dialog.gd", "shell/workspaces/render_workspace.gd", "shell/workspaces/cartography_workspace.gd", "shell/dcc_widgets.gd", "shell/export_overlay.gd", "tools/coast_drive.gd"]:
    shutil.copy(src / f, root / "godot-project" / f)
import filecmp
_src, _dst = pathlib.Path("C:/cw/coast_target/debug/cartalith_godot.dll"), root / "target" / "debug" / "cartalith_godot.dll"
for _try in range(20):
    try:
        if not (_dst.exists() and filecmp.cmp(_src, _dst, shallow=False)):
            shutil.copy(_src, _dst)
        break
    except PermissionError:
        time.sleep(3)
env = {"APPDATA": str(root / "appdata"), "LOCALAPPDATA": str(root / "localappdata"),
       "CARTALITH_CONFIG_PATH": str(root / "appdata" / "cartalith_settings.cfg"),
       "COAST_OUT": f"C:/cw/coast_frames/{tag}", "COAST_GRID": grid, "COAST_MUT": os.environ.get("COAST_MUT", "")}
pathlib.Path(env["COAST_OUT"]).mkdir(parents=True, exist_ok=True)
lock = pathlib.Path("C:/cw/.win_lock")
while True:
    try:
        lock.mkdir(); break
    except FileExistsError:
        time.sleep(5)
(lock / "owner_coast_ruling_hv.txt").write_text("coast lane (Ruling HV), python pid %d" % os.getpid())
try:
    st, lines = fd.drive(G, root / "godot-project", script, "res://shell/app.tscn", fd.FORMS[form], int(os.environ.get("COAST_TIMEOUT", "1500")), env, f"C:/cw/coast_art_{tag}")
finally:
    try: (lock / "owner_coast_ruling_hv.txt").unlink()
    except OSError: pass
    lock.rmdir()
print(st["status"], st.get("error", "")); print(*lines, sep="\n")
