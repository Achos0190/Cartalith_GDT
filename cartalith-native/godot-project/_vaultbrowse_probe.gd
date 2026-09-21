extends Node
## Windowed verification for the Markdown Vault's raw-text preview+edit panel
## (owner request, 2026-09-21) -- `vault_window.gd::_build_note_editor`, its
## two entry points (`_build_attach`'s extension and the standalone
## `open_browse()`), and the new `vault_read_file_for_edit`/`vault_write_file`
## bridge pair's hash guard.
##
## Run: godot4 --path . _vaultbrowse_probe.tscn   (WINDOWED -- no
## `--headless`; `get_viewport().get_texture()` is null under the dummy
## rasteriser, the same reason `_cultureprofiles_probe.gd` refuses to run
## under one. Screenshots are visual evidence and are not committed.)
##
##   1. Boot, generate a small world (a settlement is needed to reach the
##      scoped Attach a note section).
##   2. A real scratch vault, one hand-authored note.
##   3. Attach a note (`_app.open_vault`): pick the note (not attached to
##      anything), press "Preview & edit this note...", confirm the raw text
##      shown is exactly the file on disk. Screenshot (a).
##   4. Edit it and press Save. Confirm the file on disk changed to exactly
##      what was typed -- read from disk, not from the window's own state.
##      Screenshot (b).
##   5. The standalone entry point (`_app.open_vault_browse()`): no entity
##      scope, no Attach section, the same file picker and editor. Screenshot
##      (c).
##   6. The conflict guard: open the editor, let an external write land on
##      the same file, confirm both the bridge call directly and the real
##      Save button refuse and leave the external edit's bytes intact.

var _app: Node
var _fails: Array = []
const SEED := 883120
const NOTE := "Nareth.md"
const ORIGINAL := "# Nareth\n\nA river town at the third ford, written by hand.\n"


func _ok(label: String, cond: bool, detail: String = "") -> void:
	if cond:
		print("BROWSE   OK  %s" % label)
	else:
		_fails.append(label)
		print("BROWSE   !!  %s   %s" % [label, detail])


func _read(path: String) -> String:
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		return ""
	var s := f.get_as_text()
	f.close()
	return s


func _write(path: String, text: String) -> void:
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _find(n: Node, cls) -> Node:
	if is_instance_of(n, cls):
		return n
	for c in n.get_children():
		var r := _find(c, cls)
		if r != null:
			return r
	return null


## First `Button` (any subclass — `text_button`/`action` both return `Button`)
## under `n` whose visible text matches exactly.
func _find_button(n: Node, text: String) -> Button:
	if n is Button and String((n as Button).text) == text:
		return n
	for c in n.get_children():
		var r := _find_button(c, text)
		if r != null:
			return r
	return null


func _shot(name: String) -> void:
	await get_tree().process_frame
	var img := get_viewport().get_texture().get_image()
	var path := "res://%s.png" % name
	img.save_png(path)
	print("BROWSE   shot: %s (%dx%d)" % [path, img.get_width(), img.get_height()])


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		push_error("_vaultbrowse_probe: run WINDOWED (no --headless) -- the dummy "
			+ "rasteriser returns a null viewport image, so screenshots would pass vacuously.")
		print("PROBE REFUSED: headless")
		get_tree().quit(2)
		return

	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_app.open_project_dialog.hide()
	await get_tree().process_frame

	var bridge = _app.bridge
	bridge.generate({
		"seed": SEED, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.5).timeout

	var settlements: Array = bridge.settlements()
	_ok("world: a settlement exists to scope the Attach section", settlements.size() > 0)
	if settlements.is_empty():
		_finish()
		return
	var s: Dictionary = settlements[0]
	var tid := int(s.get("tid", 0))
	var sname := String(s.get("name", ""))

	# -- a real scratch vault, one hand-authored note --------------------------
	var root := OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vaultbrowse-probe"
	DirAccess.make_dir_recursive_absolute(root)
	var note_path := root + "/" + NOTE
	_write(note_path, ORIGINAL)
	var conn: Dictionary = bridge.vault_connect(root, "BrowseProbeVault")
	_ok("connect: a real folder binds", bool(conn.get("ok", false)), String(conn.get("error", "")))

	# -- §3 Attach a note: preview & edit an UNATTACHED, picked file -----------
	_app.open_vault("settlement", tid, sname)
	await get_tree().process_frame
	await get_tree().process_frame
	var vw := _find(_app, VaultWindow)
	_ok("panel: the vault window opened", vw != null)
	if vw == null:
		_finish()
		return
	vw._pick_file = NOTE
	vw._rebuild()
	await get_tree().process_frame

	var open_btn := _find_button(vw, "Preview & edit this note…")
	_ok("attach: the raw-editor button is offered for a picked, unattached file", open_btn != null)
	if open_btn == null:
		_finish()
		return
	open_btn.pressed.emit()
	await get_tree().process_frame
	await get_tree().process_frame

	_ok("attach: the raw editor opened on the picked file", vw._browse_path == NOTE, vw._browse_path)
	_ok("attach: the shown text is exactly the file on disk", vw._browse_edit != null
		and vw._browse_edit.text == ORIGINAL, str(vw._browse_edit.text if vw._browse_edit else "<null>"))
	_ok("attach: the note is still unattached -- no link was created", bridge.vault_links_for("settlement", tid).is_empty())
	await _shot("_vaultbrowse_probe_a_attach_edit")

	# -- §4 edit and Save: the real file on disk changes -----------------------
	var edited := "# Nareth\n\nA river town at the third ford, edited in the browse panel.\n"
	vw._browse_edit.text = edited
	vw._browse_edit.text_changed.emit()
	_ok("editor: the in-window buffer tracks the edit", vw._browse_text == edited, vw._browse_text)

	var save_btn := _find_button(vw, "Save")
	_ok("editor: a Save button is offered", save_btn != null)
	if save_btn == null:
		_finish()
		return
	save_btn.pressed.emit()
	await get_tree().process_frame
	await get_tree().process_frame

	var on_disk := _read(note_path)
	_ok("save: the real file changed to exactly what was typed", on_disk == edited, on_disk)
	var saved_hash: String = vw._browse_hash
	_ok("save: the window's hash guard advanced past the write", saved_hash != "")
	await _shot("_vaultbrowse_probe_b_attach_saved")

	# -- §5 the standalone entry point, no entity scope ------------------------
	_app.open_vault_browse()
	await get_tree().process_frame
	await get_tree().process_frame
	var vw2 := _find(_app, VaultWindow)
	_ok("standalone: the same window instance reopens", vw2 == vw)
	var texts: Array = []
	_collect_texts(vw2, texts)
	var joined := "\n".join(PackedStringArray(texts))
	_ok("standalone: the title/body reads as the browse view", joined.findn("Browse a note") >= 0, joined)
	_ok("standalone: no Attach button -- this entry point scopes no entity", joined.find("Attach to ") < 0, joined)

	vw2._pick_file = NOTE
	vw2._rebuild()
	await get_tree().process_frame
	var open_btn2 := _find_button(vw2, "Preview & edit this note…")
	_ok("standalone: the same raw-editor button is offered here too", open_btn2 != null)
	if open_btn2 != null:
		open_btn2.pressed.emit()
		await get_tree().process_frame
		await get_tree().process_frame
		_ok("standalone: it shows the file as saved a moment ago", vw2._browse_edit != null
			and vw2._browse_edit.text == edited, str(vw2._browse_edit.text if vw2._browse_edit else "<null>"))
	await _shot("_vaultbrowse_probe_c_standalone")

	# -- §6 the conflict guard --------------------------------------------------
	## The window still holds `_browse_hash` from the read a moment ago. An
	## edit lands on the file from OUTSIDE Cartalith -- another editor, another
	## process -- before this session's own edit is saved.
	var theirs := "# Nareth\n\nSomeone else edited this file in their own editor.\n"
	_write(note_path, theirs)

	# Direct bridge call, the same shape `_vault_probe.gd`'s own stale-guard
	# check uses: assert the refusal precisely, independent of the UI.
	var refused: Dictionary = bridge.vault_write_file(NOTE, "mine, and it must not land", saved_hash)
	_ok("conflict: vault_write_file refuses a stale hash", not bool(refused.get("ok", true)), String(refused.get("error", "")))
	_ok("conflict: the direct-call refusal left the external edit untouched", _read(note_path) == theirs)

	# The real Save button, same stale hash, same refusal -- and the file must
	# still read back as the external editor's own bytes, not the button's.
	vw2._browse_edit.text = "mine too, from the real Save button"
	vw2._browse_edit.text_changed.emit()
	var save_btn2 := _find_button(vw2, "Save")
	_ok("conflict: the Save button is present to press", save_btn2 != null)
	if save_btn2 != null:
		save_btn2.pressed.emit()
		await get_tree().process_frame
		await get_tree().process_frame
		_ok("conflict: pressing Save does not overwrite the external edit", _read(note_path) == theirs, _read(note_path))
		var hint := ""
		if _app._status_labels.has("hint"):
			hint = String((_app._status_labels["hint"] as Label).text)
		_ok("conflict: the status hint reports the refusal", hint.findn("refused") >= 0, hint)

	_finish()


func _collect_texts(n: Node, out: Array) -> void:
	if n is Label:
		out.append(String((n as Label).text))
	elif n is Button:
		out.append(String((n as Button).text))
	for c in n.get_children():
		_collect_texts(c, out)


func _finish() -> void:
	if _fails.is_empty():
		print("BROWSE   ALL CHECKS PASSED")
	else:
		print("BROWSE   %d FAILED: %s" % [_fails.size(), ", ".join(PackedStringArray(_fails))])
	get_tree().quit(0 if _fails.is_empty() else 1)
