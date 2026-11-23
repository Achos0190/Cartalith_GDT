"""Named literal mutants for the label-editing-on-touch fix (owner report 2026-11-23, `tools/labels_touch_drive.gd`).
Each replaces one literal that occurs exactly once in the SHIPPED source, copies it into the drive's scratch project, runs the drive on
the named form, and requires the named check to be FAIL. The shipped file is restored in `finally`.
Usage:  python labels_touch_mutants.py <godot exe> <dll> <scratch root> [name ...]
SURVIVED means that check guards nothing; BUILD_ERROR means the drive did not run or the literal is gone (not a kill).
The scratch root is the one `fennara_drive.py --shell` builds (`--root`); it must already exist (one unmutated run creates it)."""
import sys, pathlib, shutil, subprocess

HERE = pathlib.Path(__file__).resolve().parent
PROJ = HERE.parent / "godot-project"
G, DLL, ROOT = sys.argv[1], sys.argv[2], pathlib.Path(sys.argv[3])
DRIVE = "res://tools/labels_touch_drive.gd"
VH, CW = "shell/viewport_host.gd", "shell/workspaces/cartography_workspace.gd"
DW = "shell/dcc_widgets.gd"
M = [  # name, file, old, new, form, expected FAIL text
 ("grab_off", VH, "elif _tool_kind == TOOL_PLACING and touch_grab_cb.is_valid()", "elif false and touch_grab_cb.is_valid()",
  "phone", "H4 a drag on the selected label moves it"),
 ("grab_body", CW, "if String(h[\"kind\"]) == \"label\" and int(h[\"id\"]) == sel:", "if String(h[\"kind\"]) == \"label\" and int(h[\"id\"]) == sel + 1000:",
  "phone", "H4 a drag on the selected label moves it"),
 ("grab_auto", CW, "app.viewport.zoom()) == _auto_sel:", "app.viewport.zoom()) == _auto_sel + \"x\":",
  "phone", "G4 a drag on the selected generated label moves it"),
 ("pad_22dp", CW, "return 22.0 * (DccTheme.phone_scale()", "return 2.0 * (DccTheme.phone_scale()",
  "phone", "H5b dragging the handle resizes the label"),
 ("reveal_half", CW, "\tapp.set_phone_detent(\"half\")\n\tget_tree().create_timer(0.4)", "\tapp.set_phone_detent(\"peek\")\n\tget_tree().create_timer(0.4)",
  "phone", "H2 the label's form"),
 ("reveal_relocate", CW, "\t\tif at.y < top_y - 60.0:\n\t\t\treturn", "\t\tif true:\n\t\t\treturn",
  "phone", "G4 a drag on the selected generated label moves it"),
 ("open_labels", CW, "\topen_category(\"Labels\")\n\tif not DccTheme.is_phone() or app.phone_detent() != \"peek\":",
  "\tpass\n\tif not DccTheme.is_phone() or app.phone_detent() != \"peek\":", "phone", "H2 the label's form"),
 ("slider_fill", DW, "slider.size_flags_vertical = Control.SIZE_FILL\n\tline2.add_child(slider)",
  "slider.size_flags_vertical = Control.SIZE_SHRINK_CENTER\n\tline2.add_child(slider)", "tablet", "G5b ... and at least 44 dp tall to grab"),
 ("focus_rebuild", CW, "`add_child` on a null parent).\n\ttext_edit.focus_exited.connect(_rebuild_label_panel.call_deferred)",
  "`add_child` on a null parent).\n\ttext_edit.focus_exited.connect(_rebuild_label_panel)", "tablet", "Cannot call method 'add_child' on a null value"),
 ("form_scroll", CW, "\tif form_scroll != null:\n\t\t(form_scroll as ScrollContainer).ensure_control_visible.call_deferred(text_row)",
  "\tif false:\n\t\t(form_scroll as ScrollContainer).ensure_control_visible.call_deferred(text_row)",
  "tablet", "H2 the label's form"),
]


def sync(rel):
    dst = ROOT / "godot-project" / rel
    shutil.copy(PROJ / rel, dst)


def main():
    names = sys.argv[4:]
    for name, f, old, new, form, want in M:
        if names and name not in names:
            continue
        p = PROJ / f
        raw = p.read_bytes()
        txt = raw.decode("utf-8")
        crlf = "\r\n" in txt
        t = txt.replace("\r\n", "\n")
        if t.count(old) != 1:
            print(name, "BUILD_ERROR: literal occurs", t.count(old))
            continue
        try:
            p.write_bytes(t.replace(old, new).replace("\n", "\r\n" if crlf else "\n").encode("utf-8"))
            sync(f)
            out_dir = "C:/cw/lbl_mut_" + name
            r = subprocess.run([sys.executable, "-I", str(HERE / "fennara_drive.py"), G, "--shell", form, DRIVE, "--dll", DLL,
                                "--out", out_dir, "--root", str(ROOT), "--timeout", "300"],
                               capture_output=True, text=True, encoding="utf-8", errors="replace")
            out = r.stdout
            if "completed" not in out:
                print(name, "BUILD_ERROR", out[:300])
                continue
            hit = [l for l in out.splitlines() if l.startswith(("FAIL", "SCRIPT ERROR")) and want in l]   # an engine error kills too
            print(name, "KILLED" if hit else "SURVIVED", "|", hit[:1])
        finally:
            p.write_bytes(raw)
            sync(f)


main()
