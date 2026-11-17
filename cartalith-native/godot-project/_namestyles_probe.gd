extends Node
## Name styles probe (Ruling FZ): `Records ▸ Name styles…` on the live shell, runtime, windowed.
##
##   no world     the window opens from its menu row, lists the seven built-ins, says nothing is imported
##   import       the sample with bad rows: its good rows import, each bad row is listed in the check panel
##                by line number (literal lines), the style is selected and in the library
##   good import  the committed sample imports clean; a preview is drawn and a reseed changes it
##   assign       world default and per faction land in the engine (`ns_world_state`, `get_factions`)
##   rename all   a hand-given name stays, the generated ones change, one undo puts every name back
##   template     Export template then import it back: the same canonical CSV
##
## Run WINDOWED from `godot-project/` with a scratch APPDATA (it writes `user://name_styles/`):
##   Godot_v4.7.2-stable_win64_console.exe --path . _namestyles_probe.tscn -- --vp 1600x900 --out DIR [--force-touch --density=1.0]
## Result lines `NS ok|FAIL <check>`; `NS RESULT n checks, f failed (<form>)`. Exit 0 pass, 1 fail, 2 no extension, 3 watchdog.

var app: Node
var _vp: SubViewport
var _checks := 0
var _fails := 0
var _out := ""
var _form := "desktop"

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _check(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("NS %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	img.save_png(_out.path_join("ns_%s_%s.png" % [_form, name]))
	print("NS SHOT ns_%s_%s.png" % [_form, name])

func _texts(n: Node) -> String:
	var out: PackedStringArray = []
	for c in n.find_children("*", "Control", true, false):
		if c is Label:
			out.append((c as Label).text)
		elif c is Button:
			out.append((c as Button).text)
	return " | ".join(out)

func _records() -> PopupMenu:
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == "Records":
			return (ch as MenuButton).get_popup()
	return null

func _evidence(name: String) -> String:
	return ProjectSettings.globalize_path("res://").path_join("../docs/evidence/name-styles-2026-11-17").path_join(name)

func _names() -> Array:
	var out: Array = []
	for s in app.bridge.settlements():
		out.append(String(s["name"]))
	return out

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("NS WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("NS FATAL extension did not load")
		get_tree().quit(2)
		return
	_out = _arg("--out", "")
	if _out != "":
		DirAccess.make_dir_recursive_absolute(_out)
	var parts: PackedStringArray = _arg("--vp", "1600x900").split("x")
	var vw := int(parts[0])
	var vh := int(parts[1])
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(6)
	_form = ("phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")) + "_%dx%d" % [vw, vh]
	print("NS form=%s" % _form)
	var bridge = app.bridge
	# Start from an empty library so the probe is the only author of it.
	for f in DirAccess.get_files_at("user://name_styles"):
		DirAccess.remove_absolute(ProjectSettings.globalize_path("user://name_styles").path_join(f))

	var rec: PopupMenu = _records()
	_check("the Records menu has a Name styles row", rec != null and rec.get_item_index(DccMenus.ID_NAME_STYLES) >= 0)
	rec.id_pressed.emit(DccMenus.ID_NAME_STYLES)
	await _frames(6)
	var w = app.name_styles_window
	_check("the row opens the window", w != null and w.visible)
	var builtin_rows: Array = w.find_children("NsRow_*", "Button", true, false)
	_check("seven built-in cultures are listed", builtin_rows.size() == 7, str(builtin_rows.size()))
	_check("the drop zone and Import are drawn", w.find_child("NsDropZone", true, false) != null and w.find_child("NsImport", true, false) != null)
	await _shot("window_no_world")

	# -- a world
	bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.45})
	while bool(bridge.generating):
		await get_tree().create_timer(0.25).timeout
	await _frames(4)
	w.open()
	await _frames(4)

	# -- import with bad rows
	var bad := FileAccess.get_file_as_string(_evidence("fenmark_bad_rows.csv"))
	_check("the bad-rows sample is readable", bad.length() > 100)
	var r: Dictionary = w.import_text(bad, "fenmark_bad_rows.csv")
	await _frames(4)
	_check("a file with bad rows still imports its good rows", bool(r.get("ok", false)))
	var lines: Array = []
	for i in r.get("issues", []):
		if bool((i as Dictionary).get("error", false)):
			lines.append(int((i as Dictionary)["line"]))
	_check("the bad rows are reported at lines 6, 7, 8, 9, 14, 15 and 16", lines == [6, 7, 8, 9, 14, 15, 16], str(lines))
	var check: Node = w.find_child("NsCheck", true, false)
	var ct := _texts(check)
	_check("the check panel lists each line number", ct.contains("line 6:") and ct.contains("line 9:") and ct.contains("line 16:"), ct.left(200))
	_check("the library holds the style", w._entries.size() == 1 and String(w._entries[0]["name"]) == "Fenmark (bad rows)")
	await _shot("window_check_panel")

	# -- the good sample
	var good := FileAccess.get_file_as_string(_evidence("fenmark_sample.csv"))
	r = w.import_text(good, "fenmark_sample.csv")
	await _frames(4)
	var key := String(r.get("key", ""))
	_check("the committed sample imports with no issue", bool(r.get("ok", false)) and int(r.get("errors", -1)) == 0, str(r.get("issues", [])))
	_check("two styles are in the library", w._entries.size() == 2)
	var a: PackedStringArray = bridge.ns_preview(key, "Settlements", 1, 12)
	var b: PackedStringArray = bridge.ns_preview(key, "Settlements", 2, 12)
	_check("a preview is drawn (12 names) and a new seed changes it", a.size() == 12 and a != b, str(a))
	_check("the same seed gives the same names", a == bridge.ns_preview(key, "Settlements", 1, 12))
	var rivers: PackedStringArray = bridge.ns_preview(key, "Rivers", 1, 8)
	var river_ok := true
	for n in rivers:
		river_ok = river_ok and (String(n).begins_with("River ") or String(n).ends_with(" Beck") or String(n).begins_with("The "))
	_check("the Rivers section uses the style's own templates", river_ok, str(rivers))
	var cliffs: PackedStringArray = bridge.ns_preview(key, "Cliffs", 1, 8)
	var cliff_ok := true
	for n in cliffs:
		cliff_ok = cliff_ok and (String(n).ends_with("Cliffs") or String(n).ends_with("Scarp") or String(n).ends_with("Bluff"))
	_check("a section the style lacks falls back to the built-in templates", cliff_ok, str(cliffs))
	w._section = "Rivers"
	w._refresh()
	await _shot("window_preview")

	# -- assign
	_check("the world default is set", bridge.ns_set_world_default(key))
	_check("the engine reports it", String(bridge.ns_world_state().get("world_default", "")) == key)
	var assigned: bool = bridge.ns_assign_faction(2, key)
	var fac: Dictionary = {}
	for f in bridge.get_factions():
		if int((f as Dictionary).get("id", -1)) == 2:
			fac = f
	_check("faction 2 holds the style as its culture", assigned and String(fac.get("culture", "")) == key, str(fac.get("culture", "")))
	_check("an unknown key is refused", not bridge.ns_assign_faction(2, "style:nonesuch"))

	# -- rename all
	var sets: Array = bridge.settlements()
	var before := _names()
	_check("the world has places to rename", sets.size() >= 4, str(sets.size()))
	_check("a hand-given name is accepted", bridge.civ_edit_settlement(0, {"name": "Handy Rest"}))
	var st: Dictionary = bridge.ns_world_state()
	_check("the world tracks names and counts one hand-given", bool(st.get("tracked", false)) and int(st.get("hand_given", 0)) == 1, str(st))
	w._selected = key
	w._refresh()
	await _shot("window_assign_rename")
	var rr: Dictionary = w.do_rename_all()
	await _frames(4)
	var after := _names()
	_check("Rename all succeeded", bool(rr.get("ok", false)), str(rr))
	_check("the hand-given name is untouched", after[0] == "Handy Rest", after[0])
	var changed := 0
	for i in range(1, before.size()):
		if before[i] != after[i]:
			changed += 1
	_check("every other place was renamed", changed == before.size() - 1, "%d of %d" % [changed, before.size() - 1])
	_check("the new names come from the style (start with a fragment of it)", String(after[1]).begins_with("Ald") or String(after[1]).begins_with("Brae") or String(after[1]).begins_with("Cor") or String(after[1]).begins_with("Hol"), after[1])
	_check("the undo is offered", bool(bridge.ns_world_state().get("can_undo", false)))
	_check("one undo puts every name back", bridge.ns_rename_undo() and _names()[0] == "Handy Rest" and _names().slice(1) == before.slice(1))

	# -- export template, then import it back
	var tpl: String = bridge.ns_builtin_csv("highland")
	w._write_text("user://ns_probe_template.csv", tpl, "Exported")
	var back := FileAccess.get_file_as_string("user://ns_probe_template.csv")
	var pr: Dictionary = bridge.ns_parse(back, false)
	_check("the exported template imports clean and unchanged", bool(pr.get("ok", false)) and int(pr.get("errors", -1)) == 0 and String(pr["csv"]) == tpl)
	await _shot("window_end")
	print("NS RESULT %d checks, %d failed (%s)" % [_checks, _fails, _form])
	get_tree().quit(1 if _fails > 0 else 0)
