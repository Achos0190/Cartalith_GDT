"""Named literal mutants for shell batch B (2026-11-23). Each replaces one literal that occurs exactly once, runs the drive that
guards it, and requires the named check to be FAIL. Restores in `finally`. Usage:
  python shell_batch_b_mutants.py <repo godot-project dir> <wrapper (C:/cw/shellB_run.py)> [name ...]
SURVIVED means the guard is a decoration; BUILD_ERROR means the drive did not run (not a kill).
The scratch copy is refreshed by the wrapper on every run (`--refresh`), because it copies the project once and a mutant edits the repo."""
import sys, pathlib, subprocess
P, W = pathlib.Path(sys.argv[1]), sys.argv[2]
U = "res://tools/shell_batch_b_drive.gd"; L = "res://tools/shell_batch_b_labels_drive.gd"; C = "res://tools/shell_batch_b_census_drive.gd"
M = [  # name, file, old, new, form, drive, expected FAIL text
 ("group_counts", "shell/app.gd", "or undo_groups.wants_undo() or _planner_stops(\"stops_can_undo\")", "or _planner_stops(\"stops_can_undo\")", "phone", U, "U a grouped step shows the chip"),
 ("year_signal", "shell/year_record_history.gd", "\tcursor = hist.size()\n\tchanged.emit()\n", "\tcursor = hist.size()\n", "phone", U, "U a recorded year shows the chip"),
 ("chip_link", "shell/dcc_shell.gd", "\t\t\t\t\th.changed.connect(_refresh_phone_undo_chip)\n", "\t\t\t\t\tpass\n", "phone", U, "U a grouped step shows the chip"),
 ("strip_shows", "shell/asset_library_window.gd", "			visible = get_child_count() > 0", "			visible = false", "phone", U, "A a lifted tile shows the drop strip"),
 ("label_view", "shell/workspaces/cartography_workspace.gd", "var ppc := _label_cull_ppc()", "var ppc: float = app.viewport.label_px_per_cell() if app.viewport != null else 0.0", "tablet", L, "L5 the same seed in a narrower window"),
 ("spin_gate", "shell/dcc_shell.gd", "if ctl is SpinBox and DccTheme.is_tablet():", "if false and ctl is SpinBox and DccTheme.is_tablet():", "tablet", C, "T1 a drag that starts on a tablet window's SpinBox scrolls"),
 ("progress_passive", "shell/dcc_widgets.gd", "or c is RichTextLabel or c is ProgressBar or", "or c is RichTextLabel or", "tablet", C, "P1 touch_passive() turns a ProgressBar PASS"),
 ("strip_target", "shell/asset_library_window.gd", "btn.coll_name = String(c[\"name\"])\n\t\tbtn.text", "btn.coll_name = \"Nope\"\n\t\tbtn.text", "phone", U, "A dropping on the row adds the tile"),
]
def main():
    names = sys.argv[3:]
    for name, f, old, new, form, drive, want in M:
        if names and name not in names: continue
        p = P / f; raw = p.read_bytes(); txt = raw.decode("utf-8")
        crlf = "\r\n" in txt; t = txt.replace("\r\n", "\n")
        if t.count(old) != 1: print(name, "BUILD_ERROR: literal occurs", t.count(old)); continue
        try:
            p.write_bytes(t.replace(old, new).replace("\n", "\r\n" if crlf else "\n").encode("utf-8"))
            r = subprocess.run([sys.executable, W, form, drive, "C:/cw/shellB_out/mut_" + name, "--timeout", "180"], capture_output=True, text=True, encoding="utf-8", errors="replace")
            out = r.stdout
            if "completed" not in out: print(name, "BUILD_ERROR", out[:300]); continue
            hit = [l for l in out.splitlines() if l.startswith("FAIL") and want in l]
            print(name, "KILLED" if hit else "SURVIVED", "|", hit[:1])
        finally:
            p.write_bytes(raw)
main()
