extends Node
## Non-headless verification of the §2.1 project lifecycle
## (`GUI_GAP_REGISTER.md` FI-01): Save, Save as, Revert, Close, and the
## "unsaved changes" prompt's Save button -- driven through the real
## `DccApp`, in a real window, against the real GDExtension.
##
##   godot --path . _lifecycle_check.tscn

var app: Node
var _fails: Array[String] = []

func _check(ok: bool, what: String) -> void:
	print(("  ok   " if ok else "  FAIL ") + what)
	if not ok:
		_fails.append(what)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	app.open_project_dialog.hide()
	await _frames(2)

	var path := OS.get_user_data_dir().path_join("_lifecycle.zip")
	if FileAccess.file_exists(path):
		DirAccess.remove_absolute(path)

	# -- a world ---------------------------------------------------------------
	_check(app.bridge.save_api, "the bridge sees a save writer")
	app.bridge.generate({"seed": 24601, "width_km": 640.0, "grid_w": 48, "grid_h": 30})
	while app.bridge.generating:
		await get_tree().process_frame
	await _frames(3)
	_check(app.bridge.has_world, "a world generated")
	_check(app.bridge.world_dirty, "a fresh generation counts as unsaved")

	# -- Save ------------------------------------------------------------------
	app._write_project(path)
	await _frames(2)
	_check(FileAccess.file_exists(path), "Save wrote the file")
	_check(app.current_project_path == path, "current_project_path moved to it")
	_check(not app.bridge.world_dirty, "saving cleared the unsaved flag")
	_check(DccSettings.recent_projects().has(path), "the save is in Recent worlds")

	# -- Save as… through the shell's own browser ------------------------------
	## The dialog, not `_write_project` -- `DccBrowseDialog`'s SAVE mode is new
	## and is the only untested piece of this path.
	app.save_project_as()
	await _frames(3)
	var browse: DccBrowseDialog = null
	for c in app.get_children():
		if c is DccBrowseDialog and (c as DccBrowseDialog).visible:
			browse = c
	_check(browse != null, "Save as… opened the shell's own browser")
	if browse != null:
		_check(browse._name_edit != null, "...with a name field")
		_check(browse._name_edit.text == path.get_file(), "...prefilled with the current file name (%s)" % browse._name_edit.text)
		browse._name_edit.text = "_lifecycle_as"
		browse._refresh_primary()
		_check(browse._save_path() == OS.get_user_data_dir().path_join("_lifecycle_as.zip"),
			"...appending .zip to a bare name (%s)" % browse._save_path())
		browse._confirm()
		await _frames(4)
	var as_path := OS.get_user_data_dir().path_join("_lifecycle_as.zip")
	_check(FileAccess.file_exists(as_path), "Save as… wrote the new file")
	_check(app.current_project_path == as_path, "...and the project followed it")
	DirAccess.remove_absolute(as_path)
	app.current_project_path = path

	# -- Revert ----------------------------------------------------------------
	## Regenerate at a different seed, then revert: the world must come back
	## as the saved one, not the newer one.
	app.bridge.generate({"seed": 777, "width_km": 640.0, "grid_w": 48, "grid_h": 30})
	while app.bridge.generating:
		await get_tree().process_frame
	await _frames(2)
	_check(app.bridge.world_gen.get_seed() == 777, "a second world replaced the first")
	app.revert_to_saved()
	await _frames(2)
	var revert_dlg := _find_dialog("", "Revert to last save?")
	_check(revert_dlg != null, "Revert asked before discarding")
	if revert_dlg != null:
		revert_dlg.get_ok_button().emit_signal("pressed")
		await _frames(4)
	_check(app.bridge.world_gen.get_seed() == 24601, "Revert restored the saved world (seed %d)" % app.bridge.world_gen.get_seed())
	_check(app.bridge.world_gen.get_width() == 48, "...at its own size")
	_check(int(app.bridge.world_gen.get_params().get("tect.plates", -1)) > 0, "...with its parameters")

	# -- Close, via the prompt's Save button ------------------------------------
	## Dirty it again so the prompt's Save half has something to do.
	app.bridge.world_dirty = true
	DirAccess.remove_absolute(path)
	app.close_project()
	await _frames(2)
	var close_dlg := _find_dialog("", "Close project")
	_check(close_dlg != null, "Close asked before discarding")
	if close_dlg != null:
		var save_btn := _find_button(close_dlg, "Save and close")
		_check(save_btn != null, "the prompt offers Save and close")
		if save_btn != null:
			save_btn.emit_signal("pressed")
			await _frames(6)
	_check(FileAccess.file_exists(path), "Save and close wrote the file it was about to discard")
	_check(not app.bridge.has_world, "the world is closed")
	_check(app.current_project_path == "", "and the project path is cleared")

	# -- Reopen ----------------------------------------------------------------
	app.open_recent_project(path)
	await _frames(4)
	_check(app.bridge.has_world, "reopened")
	_check(app.bridge.world_gen.get_seed() == 24601, "the same world came back")
	_check(not app.bridge.world_dirty, "a freshly opened world is not dirty")
	var tex = app.bridge.world_gen.build_color_texture()
	_check(tex != null and tex.get_image().get_size() == Vector2i(48, 30), "it renders")

	# -- Autosave --------------------------------------------------------------
	app.bridge.world_dirty = true
	app._autosave_tick()
	await _frames(2)
	var auto := path.get_basename() + ".autosave.zip"
	_check(FileAccess.file_exists(auto), "autosave wrote %s" % auto.get_file())
	_check(app.bridge.world_dirty, "autosave did not pretend the project was saved")
	DirAccess.remove_absolute(auto)
	DirAccess.remove_absolute(path)

	print("PASS" if _fails.is_empty() else "FAILED: %s" % str(_fails))
	get_tree().quit(0 if _fails.is_empty() else 1)

func _find_dialog(text: String, dialog_title: String = "") -> AcceptDialog:
	for c in app.get_children():
		if c is AcceptDialog:
			var d := c as AcceptDialog
			if not d.visible:
				continue
			if dialog_title != "" and d.title == dialog_title:
				return d
			if text != "" and d.dialog_text.begins_with(text):
				return d
	return null

func _find_button(root: Node, label: String) -> Button:
	if root is Button and (root as Button).text == label:
		return root as Button
	for c in root.get_children(true):
		var found := _find_button(c, label)
		if found != null:
			return found
	return null

func _all_buttons(root: Node) -> Array:
	var out: Array = []
	if root is Button:
		out.append(root)
	for c in root.get_children(true):
		out.append_array(_all_buttons(c))
	return out
