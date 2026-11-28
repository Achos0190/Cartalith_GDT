import sys, os, pathlib, time
sys.path.insert(0, "C:/cw/expo/cartalith-native/tools")
import fennara_drive as fd
G = "C:/Users/Vincent/AppData/Local/Microsoft/WinGet/Packages/GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe/Godot_v4.7.2-stable_win64_console.exe"
root = pathlib.Path("C:/cw/expo_new")
tag = sys.argv[1]
env = {"APPDATA": str(root/"appdata"), "LOCALAPPDATA": str(root/"localappdata"), "CARTALITH_CONFIG_PATH": str(root/"appdata"/"cartalith_settings.cfg"),
       "EXP_W": sys.argv[2], "EXP_GRID": sys.argv[3], "EXP_COAST": sys.argv[4]}
lock = pathlib.Path("C:/cw/.win_lock")
while True:
    try:
        lock.mkdir(); break
    except FileExistsError:
        time.sleep(5)
(lock/"owner").write_text("expo-1128-drive-" + tag)
try:
    st, lines = fd.drive(G, root/"godot-project", "res://tools/export_speed_drive.gd", "res://shell/app.tscn", fd.FORMS["desktop"], int(os.environ.get("T", "900")), env, f"C:/cw/expo_ev/art_{tag}")
finally:
    if (lock/"owner").read_text().strip() == "expo-1128-drive-" + tag:
        (lock/"owner").unlink(); lock.rmdir()
print(st["status"], st.get("error", "")); print(*lines, sep="\n")
