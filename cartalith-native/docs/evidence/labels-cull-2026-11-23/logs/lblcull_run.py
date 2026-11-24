import sys, os, pathlib, shutil
tools = sys.argv[1]; sys.path.insert(0, tools)
import fennara_drive as fd
godot, form, dll, out, root, tag = sys.argv[2:8]
class L:
    def mkdir(s):
        pathlib.Path("C:/cw/.win_lock").mkdir()
        pathlib.Path("C:/cw/.win_lock/owner").write_text("lblcull_1123")
    def rmdir(s):
        if pathlib.Path("C:/cw/.win_lock/owner").read_text() == "lblcull_1123":
            pathlib.Path("C:/cw/.win_lock/owner").unlink(); pathlib.Path("C:/cw/.win_lock").rmdir()
fd.LOCK = L()
fd.FORMS["tablet1280"] = ["--resolution", "1280x800", "--", "--force-touch", "--density=1.0"]
os.environ["LBLC_TAG"] = tag
os.environ["LBLC_SHOTS"] = out
st, lines = fd.drive_shell(godot, form, "res://tools/labels_cull_drive.gd", dll, root, out + "/drv", 600)
print(st["status"], st.get("error", ""))
print(*[l for l in lines if "CENSUS" in l or "pinch" in l or "LABELS" in l or "FAIL" in l or "form canvas" in l or "ERROR" in l or "labels cull" in l or "POSITIVE" in l], sep="\n")
