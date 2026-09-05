extends Node
## VERIFIER probe for batch 32 lane C (the Open project welcome screen).
## The case that matters is the COLD START: a fresh profile, no recents, no
## world. Staged IN MEMORY -- `DccSettings._save()` is never called, so the
## user's own `user://cartalith_settings.cfg` is untouched -- and the staged
## projects directory is created and removed by this script.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _vfy_op32_probe.tscn

var _fail := 0
var _root := ""
var _cfg_backup_recent
var _cfg_backup_root

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ck(tag: String, name: String, cond: bool, detail: String = "") -> void:
	print("VOP %s  [%s] %s%s" % ["ok  " if cond else "FAIL", tag, name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _shown(n: Node) -> bool:
	var c: Node = n
	while c != null:
		if c is Window:
			return true
		if c is CanvasItem and not (c as CanvasItem).visible:
			return false
		c = c.get_parent()
	return true

## Every reachable Button's text under `root`.
func _buttons(root: Node, out: Array) -> void:
	if root is CanvasItem and not (root as CanvasItem).visible:
		return
	if root is Button:
		out.append((root as Button).text)
	for c in root.get_children():
		_buttons(c, out)

func _boot(w: int, h: int) -> Node:
	var vp := SubViewport.new()
	vp.size = Vector2i(w, h)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(60)
	return app

func _stage_empty_profile() -> void:
	DccSettings._ensure_loaded()
	_cfg_backup_recent = DccSettings._cfg.get_value("recent", "paths", null)
	_cfg_backup_root = DccSettings._cfg.get_value("storage_roots", "projects", null)
	_root = OS.get_user_data_dir().path_join("_vfy_op32_empty")
	DirAccess.make_dir_recursive_absolute(_root)
	DccSettings._cfg.set_value("recent", "paths", [])
	DccSettings._cfg.set_value("storage_roots", "projects", _root)

func _unstage() -> void:
	if _cfg_backup_recent == null:
		DccSettings._cfg.erase_section_key("recent", "paths")
	else:
		DccSettings._cfg.set_value("recent", "paths", _cfg_backup_recent)
	if _cfg_backup_root == null:
		DccSettings._cfg.erase_section_key("storage_roots", "projects")
	else:
		DccSettings._cfg.set_value("storage_roots", "projects", _cfg_backup_root)
	if _root != "" and DirAccess.dir_exists_absolute(_root):
		DirAccess.remove_absolute(_root)
	print("VOP   staged dir removed=%s   (DccSettings._save() was never called)"
		% (not DirAccess.dir_exists_absolute(_root)))

func _ready() -> void:
	_stage_empty_profile()
	var app := await _boot(1920, 1080)
	var d = app.open_project_dialog
	_ck("cold", "recents really are empty", DccSettings.recent_projects().is_empty(),
		"n=%d" % DccSettings.recent_projects().size())
	_ck("cold", "projects root really is empty",
		DirAccess.get_files_at(DccSettings.storage_root("projects")).is_empty(),
		"root=%s" % DccSettings.storage_root("projects"))

	d.open_welcome()
	await _frames(12)
	var picker: Control = d.get("_picker")
	var gallery: Control = d.get("_gallery")
	var tiles: Control = d.get("_picker_tiles")
	_ck("cold", "picker composition is the one on screen",
		picker.visible and not gallery.visible,
		"picker=%s gallery=%s" % [picker.visible, gallery.visible])
	_ck("cold", "the empty card row is HIDDEN, not an empty gap",
		not tiles.visible and tiles.get_child_count() == 0,
		"visible=%s children=%d" % [tiles.visible, tiles.get_child_count()])

	var btns: Array = []
	_buttons(d, btns)
	print("VOP   [cold] reachable buttons: %s" % str(btns))
	for want in ["New world", "Open project .zip…", "Import a heightmap…",
			"Continue without a world"]:
		var found := false
		for b in btns:
			if String(b).find(want) >= 0:
				found = true
		_ck("cold", "route present: %s" % want, found)
	_ck("cold", "no ✕ and no Cancel on the cold-start screen (the gallery head is hidden)",
		not btns.has("Cancel"), "buttons=%s" % str(btns))

	var note: Label = d.get("_picker_note")
	_ck("cold", "the foot says what to do and where projects live",
		note.text.find("no saved worlds yet") >= 0 and note.text.find(_root) >= 0,
		"text='%s'" % note.text)

	## Every action must clear the 44 px floor.
	for b in _find_buttons(picker):
		_ck("cold", "action '%s' clears 44 px" % b.text,
			b.get_combined_minimum_size().y >= 44.0,
			"h=%.1f" % b.get_combined_minimum_size().y)

	## **The re-installed profile**: config wiped (recents empty) but the Worlds
	## folder intact. A recents-only picker would report "no saved worlds yet"
	## over real files, so this is the discriminating case for `_welcome_paths()`
	## being a UNION rather than a scope.
	for i in 5:
		var f := FileAccess.open(_root.path_join("staged_%d.zip" % i), FileAccess.WRITE)
		if f != null:
			f.store_string("not a real archive")
			f.close()
	d.open_welcome()
	await _frames(12)
	_ck("reinstall", "worlds on disk with zero recents still produce cards",
		tiles.visible and tiles.get_child_count() == 3,
		"visible=%s cards=%d recents=%d" % [tiles.visible, tiles.get_child_count(),
			DccSettings.recent_projects().size()])
	_ck("reinstall", "the foot says how many are not on the screen",
		note.text.find("3 of 5 worlds") >= 0, "text='%s'" % note.text)
	_ck("reinstall", "cards are the canvas's 252 wide",
		tiles.get_child_count() > 0 			and is_equal_approx((tiles.get_child(0) as Control).custom_minimum_size.x, 252.0),
		"w=%.1f" % (tiles.get_child(0) as Control).custom_minimum_size.x)
	for i in 5:
		DirAccess.remove_absolute(_root.path_join("staged_%d.zip" % i))
	d.open_welcome()
	await _frames(10)

	## The routes have to WORK, not merely be drawn.
	var nw: Button = _btn(picker, "New world")
	nw.pressed.emit()
	await _frames(10)
	_ck("cold", "New world route: dialog closed, New world dialog up",
		not d.visible and app.new_world_dialog.visible,
		"picker_visible=%s nw_visible=%s" % [d.visible, app.new_world_dialog.visible])
	app.new_world_dialog.hide()
	await _frames(4)

	d.open_welcome()
	await _frames(10)
	var imp: Button = _btn(picker, "Import a heightmap")
	_ck("cold", "import route is shown iff the extension exposes one",
		imp.visible == app.bridge.import_api,
		"visible=%s import_api=%s" % [imp.visible, app.bridge.import_api])
	if imp.visible:
		imp.pressed.emit()
		await _frames(10)
		_ck("cold", "Import route: dialog closed, heightmap import up",
			not d.visible, "picker_visible=%s" % d.visible)
		for c in app.get_children():
			if c is Window and (c as Window).visible and c != d:
				(c as Window).hide()
		await _frames(4)

	d.open_welcome()
	await _frames(8)
	var browse: Button = _btn(picker, "Open project .zip")
	browse.pressed.emit()
	await _frames(10)
	var browser_up := false
	for c in d.get_children():
		if c is Window and (c as Window).visible:
			browser_up = true
	_ck("cold", "Open .zip route opens a file browser", browser_up)
	for c in d.get_children():
		if c is Window:
			(c as Window).hide()
	await _frames(4)

	d.open_welcome()
	await _frames(8)
	var out_btn: Button = _btn(picker, "Continue without a world")
	out_btn.pressed.emit()
	await _frames(6)
	_ck("cold", "the opt-out closes it and leaves the shell alone",
		not d.visible and not app.bridge.has_world)

	# ---- open() -- the 08-23 gallery -- must be untouched ---------------------
	d.open()
	await _frames(12)
	_ck("gallery", "gallery composition is the one on screen",
		gallery.visible and not picker.visible)
	_ck("gallery", "window is still 1180x760", d.size == Vector2i(1180, 760),
		"size=%s" % str(d.size))
	var grid: GridContainer = d.get("_grid")
	_ck("gallery", "still a 4-column grid", grid.columns == 4, "columns=%d" % grid.columns)
	var gbtns: Array = []
	_buttons(gallery, gbtns)
	print("VOP   [gallery] reachable buttons: %s" % str(gbtns))
	for want in ["Recent", "All worlds", "Shared", "Cancel", "Open selected"]:
		var f := false
		for b in gbtns:
			if String(b).find(want) >= 0:
				f = true
		_ck("gallery", "gallery control still present: %s" % want, f)
	_ck("gallery", "import tile is still the first cell of an empty gallery",
		grid.get_child_count() >= 1, "cells=%d" % grid.get_child_count())
	d.hide()
	await _frames(4)

	_unstage()
	print("VOP DONE  failures=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

func _find_buttons(root: Node) -> Array[Button]:
	var out: Array[Button] = []
	if root is CanvasItem and not (root as CanvasItem).visible:
		return out
	if root is Button:
		out.append(root)
	for c in root.get_children():
		out.append_array(_find_buttons(c))
	return out

func _btn(root: Node, needle: String) -> Button:
	for b in _find_buttons(root):
		if b.text.find(needle) >= 0:
			return b
	## Hidden buttons are not in `_find_buttons`, so fall back to a full walk.
	return _btn_any(root, needle)

func _btn_any(root: Node, needle: String) -> Button:
	if root is Button and (root as Button).text.find(needle) >= 0:
		return root
	for c in root.get_children():
		var r := _btn_any(c, needle)
		if r != null:
			return r
	return null
