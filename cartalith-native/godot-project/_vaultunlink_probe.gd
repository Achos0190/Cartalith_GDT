extends Node
## `OUTSTANDING_WORK.md` §2.11 "Vault Unlink and Re-scan do not persist"
## (`ALIGNMENT_AUDIT.md` Part 1 item 17).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x900 \
##       _vaultunlink_probe.tscn
##
## Windowed, never `--headless`: the shell boot walks into null textures under
## the dummy rasterizer.
##
## Data ▸ Export ▸ Maps carries a vault block with `Re-scan vault` and `Unlink`
## chips. Before 2026-09-24 neither persisted: Unlink called `vault_disconnect`
## and stopped, so `VaultStore.PATH` kept the old binding and the next launch
## re-bound the folder; Re-scan rebuilt the index in memory and never wrote
## `VaultStore.INDEX_PATH`. This presses the real chips in the real window and
## then does what a relaunch does -- a FRESH `EngineBridge` and
## `VaultStore.load_into()` -- and asserts the result held.
##
## The user's own sidecar files are backed up first and restored byte for
## byte at the end, whatever happens in between.

var _vp: SubViewport
var _fail := 0
var _backup := {}

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

static func _read(p: String) -> String:
	if not FileAccess.file_exists(p):
		return ""
	var f := FileAccess.open(p, FileAccess.READ)
	if f == null:
		return ""
	var t := f.get_as_text()
	f.close()
	return t

static func _sidecar_binding() -> String:
	var parsed = JSON.parse_string(_read(VaultStore.PATH))
	if typeof(parsed) != TYPE_DICTIONARY:
		return "<unreadable>"
	return String((parsed as Dictionary).get("binding", ""))

## What a relaunch does: a bridge with nothing in memory, fed only the disk.
func _relaunch() -> EngineBridge:
	var b: EngineBridge = EngineBridge.new()
	add_child(b)
	await _frames(2)
	VaultStore.load_into(b)
	return b

static func _find_button(n: Node, text: String) -> Button:
	if n is Button and (n as Button).text == text:
		return n
	for c in n.get_children(true):
		var r := _find_button(c, text)
		if r != null:
			return r
	return null

func _save_backups() -> void:
	for p in [VaultStore.PATH, VaultStore.INDEX_PATH, VaultStore.PRE_PROJECT_PATH]:
		_backup[p] = _read(p) if FileAccess.file_exists(p) else null

func _restore_backups() -> void:
	for p in _backup:
		if _backup[p] == null:
			if FileAccess.file_exists(p):
				DirAccess.remove_absolute(ProjectSettings.globalize_path(p))
		else:
			var f := FileAccess.open(p, FileAccess.WRITE)
			f.store_string(String(_backup[p]))
			f.close()

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(2); return
	_save_backups()

	## A throwaway vault: two notes, one linking the other, so the index has
	## something in it to save.
	var dir_res := "user://_vaultunlink_tmp"
	var dir := ProjectSettings.globalize_path(dir_res)
	DirAccess.make_dir_recursive_absolute(dir)
	for pair in [["alpha.md", "# Alpha\nSee [[Beta]].\n"], ["Beta.md", "# Beta\nBack to [[alpha]].\n"]]:
		var f := FileAccess.open(dir_res + "/" + String(pair[0]), FileAccess.WRITE)
		f.store_string(String(pair[1]))
		f.close()

	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: DccApp = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(40)
	print("[BOOT] shell up")
	var bridge: EngineBridge = app.bridge
	if not bridge.world_gen.has_method("vault_disconnect"):
		print("  ABORT: the loaded extension has no vault surface; rebuild first")
		_restore_backups(); get_tree().quit(2); return

	print("\n=== 0: Link, through the vault panel's own save path ===")
	var r := bridge.vault_connect(dir, "unlink-probe")
	_ok("vault_connect succeeded", bool(r.get("ok", false)), true)
	app.vault_window.store_changed.emit()
	await _frames(2)
	_ok("the sidecar now carries the binding", _sidecar_binding(), dir)
	var b_ctrl := await _relaunch()
	_ok("CONTROL: a relaunch re-binds a linked vault (so the check below can fail)",
		bool(b_ctrl.vault_info().get("bound", false)), true)

	print("\n=== 1: Re-scan vault writes the index ===")
	if FileAccess.file_exists(VaultStore.INDEX_PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.INDEX_PATH))
	## The boot's open-project dialog is exclusive; out of the way first, or
	## the data manager's popup logs an exclusivity error.
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	app.open_data_manager_route("export_maps")
	await _frames(8)
	var dm: Node = app.data_manager_window
	var rescan := _find_button(dm, "Re-scan vault")
	_ok("the Re-scan vault chip is drawn", rescan != null, true)
	if rescan != null:
		_ok("...and enabled with a vault bound", rescan.disabled, false)
		rescan.pressed.emit()
		await _frames(6)
		_ok("the index file was written", FileAccess.file_exists(VaultStore.INDEX_PATH), true)
		_ok("...and it is the engine's own index, verbatim",
			_read(VaultStore.INDEX_PATH) == bridge.vault_backlink_index_json(), true)
		var b_idx := await _relaunch()
		var st: Dictionary = b_idx.vault_backlink_stats()
		print("  info relaunched index: ", st)
		_ok("a relaunch restores a built index", bool(st.get("built", false)), true)
		_ok("...over both notes", int(st.get("notes", 0)), 2)

	print("\n=== 2: Unlink holds across a relaunch ===")
	var unlink := _find_button(dm, "Unlink")
	_ok("the Unlink chip is drawn", unlink != null, true)
	if unlink != null:
		_ok("...and enabled with a vault bound", unlink.disabled, false)
		unlink.pressed.emit()
		await _frames(6)
		_ok("the live bridge is unbound", bool(bridge.vault_info().get("bound", true)), false)
		_ok("the sidecar's binding was cleared", _sidecar_binding(), "")
		var b_after := await _relaunch()
		_ok("a relaunch does NOT re-bind the unlinked vault",
			bool(b_after.vault_info().get("bound", true)), false)

	if dm is Window:
		(dm as Window).hide()
	_restore_backups()
	for fname in ["alpha.md", "Beta.md"]:
		DirAccess.remove_absolute(dir + "/" + fname)
	DirAccess.remove_absolute(dir)
	print("\n_vaultunlink_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
