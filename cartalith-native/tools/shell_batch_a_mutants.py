"""Named literal mutants for shell batch A (2026-11-23). Each replaces one literal that occurs exactly once, runs the drive
(or probe) that guards it, and requires the named check to be FAIL. Restores in `finally`. Usage:
  python shell_batch_a_mutants.py <repo godot-project dir> <drive wrapper (C:/cw/shellA_drive.py)> [name ...]
SURVIVED means the guard is a decoration; BUILD_ERROR means the drive did not run (not a kill)."""
import sys, pathlib, subprocess
P, W = pathlib.Path(sys.argv[1]), sys.argv[2]
D = "res://tools/shell_batch_a_drive.gd"; R = "res://tools/shell_batch_a_portrait_drive.gd"
M = [  # name, file, old, new, form, drive, extra, expected FAIL text
 ("seam", "shell/dcc_settings.gd", 'return v if v != "" else "user://cartalith_settings.cfg"', 'return "user://cartalith_settings.cfg"', "desktop", D, [], "S CONFIG_PATH"),
 ("welcome_width", "shell/open_project_dialog.gd", "out.size_flags_horizontal = Control.SIZE_EXPAND_FILL", "out.size_flags_horizontal = Control.SIZE_SHRINK_CENTER", "desktop", D, [], "W Continue fills"),
 ("phone_generation", "shell/new_world_dialog.gd", 'DccWidgets.section(card if _phone else rest, "Generation")', 'DccWidgets.section(rest, "Generation")', "phone", D, [], "N phone: Imperial"),
 ("create_gate", "shell/new_world_dialog.gd", "if bridge.has_world and bridge.world_dirty and app != null", "if false and app != null", "desktop", D, [], "C Create over unsaved"),
 ("phone_saved_at", "shell/app.gd", '	_phone_saved_at = ""   ## the phone overflow', '	## MUTANT   ## the phone overflow', "phone", D, [], "P a new world"),
 ("landmark_link", "shell/workspaces/civilization_workspace.gd", "cb.toggled.connect(func(on: bool): vp.set_landmark_kind_visible(key, on))", "cb.toggled.connect(func(_on: bool): pass)", "desktop", D, [], "L clicking the box"),
 ("memory_menu", "shell/menus.gd", "var used := _bridge.memory_bytes() if _bridge != null else OS.get_static_memory_usage()", "var used := OS.get_static_memory_usage()", "desktop", D, [], "M the Working set menu row"),
 ("memory_overlay", "shell/resource_overlay.gd", "(_bridge.memory_bytes() / 1048576.0)", "(OS.get_static_memory_usage() / 1048576.0)", "desktop", D, [], "M the resource overlay"),
 ("dock_label", "shell/right_dock.gd", "name_label.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS  ## an ellipsis, not a hard cut (OUTSTANDING \"Dock labels\", 2026-11-23)\n\tname_label.tooltip_text = name_label.text", "name_label.clip_text = true\n\tname_label.tooltip_text = name_label.text", "desktop", D, [], "D it ellipsises"),
 ("light_spin", "shell/dcc_widgets.gd", 'le.add_theme_stylebox_override(state, DccTheme.field_box(state, spin_x, spin_y))\n\t\tle.add_theme_color_override("font_color"', 'le.add_theme_stylebox_override(state, DccTheme.flat(Color(0.098, 0.11, 0.118)))\n\t\tle.add_theme_color_override("font_color"', "tabP", D, ["light"], "T light palette"),
 ("toggle_spacer", "shell/dcc_widgets.gd", "\trow.add_child(cb)\n\treturn cb\n\n## A labelled dropdown row", "\trow.add_child(DccTheme.spacer())\n\trow.add_child(cb)\n\treturn cb\n\n## A labelled dropdown row", "tabL", R, ["light"], "T portrait"),
]
def main():
    names = sys.argv[3:]
    for name, f, old, new, form, drive, extra, want in M:
        if names and name not in names: continue
        p = P / f; raw = p.read_bytes(); txt = raw.decode("utf-8")
        crlf = "\r\n" in txt; t = txt.replace("\r\n", "\n")
        if t.count(old) != 1: print(name, "BUILD_ERROR: literal occurs", t.count(old)); continue
        try:
            p.write_bytes(t.replace(old, new).replace("\n", "\r\n" if crlf else "\n").encode("utf-8"))
            r = subprocess.run([sys.executable, W, form, drive, "C:/cw/shellA_out/mut_" + name] + extra, capture_output=True, text=True, encoding="utf-8", errors="replace")
            out = r.stdout
            if "completed" not in out: print(name, "BUILD_ERROR", out[:300]); continue
            hit = [l for l in out.splitlines() if l.startswith("FAIL") and want in l]
            print(name, "KILLED" if hit else "SURVIVED", "|", hit[:1])
        finally:
            p.write_bytes(raw)
main()
