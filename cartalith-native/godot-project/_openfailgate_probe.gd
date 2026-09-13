extends Node
## Lane GATE part B (2026-09-13). A failed project open used to dismiss the
## welcome/picker surface anyway: `phone_project_picker.gd::_pick_world()` and
## four call sites in `open_project_dialog.gd` all called `hide()` BEFORE
## `_host.open_recent_project(path)`, so a corrupt or unreadable archive still
## refused (`app.gd::_load_project()`'s own `refusal` branch) but the surface
## the refusal's `set_status`/toast should have landed on was already gone --
## observed on device 2026-09-07 as a dashed title and no world, with no way
## back to the picker.
##
## **Fix.** `_load_project()`/`open_recent_project()` (`app.gd`) now return
## whether the open succeeded; every call site hides only when it did
## (`open_project_dialog.gd::_pick_path()`, `phone_project_picker.gd
## ::_pick_world()`).
##
## **What this probe drives, and how.** A real corrupt (non-zip) `.ctl` and a
## real valid save, through each of the four entry points named in the row,
## using the REAL registered functions/callbacks rather than a re-implementation:
##   - desktop welcome recent : a real picker TILE, found in the live tree and
##     clicked via its own `gui_input` signal (the actual wiring).
##   - desktop browse         : `open_project_dialog._browse_from_disk()` (real
##     function) spawns a real `DccBrowseDialog`; this probe sets its
##     `_selected` and calls its own `_confirm()`, which is the real path a
##     directory-tree pick would take through the SAME dialog/callback.
##   - phone recent           : `phone_project_picker._pick_world(path)` --
##     already a named, directly-callable function; identical to what the real
##     card's `gui_input` handler calls.
##   - phone browse           : same `DccBrowseDialog` injection technique,
##     under `app` (`_on_open_zip()`'s own `choose_file(_host, ...)` host).
##
## Deliberately does NOT call `DccSettings.remember_project()` for the corrupt
## leg (the failure branch never reaches it) and does not let a SUCCESS leg's
## unavoidable call to it corrupt the real `user://cartalith_settings.cfg` --
## see `_backup_settings()`/`_restore_settings()`. Both fixture saves live
## under the real `DccSettings.storage_root("projects")` only for the span of
## this probe and are deleted at the end (see `_cleanup_fixtures()`); nothing
## here touches the recents LIST.
##
## Mutation: restore the hide-before-open order on any one call site and that
## site's own FAILURE assertion below fails, while every SUCCESS assertion
## still passes (a successful open's sequence is unchanged by this fix, which
## is Gate B1).
##
## **Part B continued, same date -- the phone toast is drawn under the
## picker.** Refuted in review: `phone_project_picker.gd` is an `AcceptDialog`
## (a `Window`); an embedded `Window` always composites over the ordinary
## canvas `_show_phone_toast()` (`dcc_shell.gd`) draws its pill into, so the
## toast a refused open already fires was rendered and immediately covered by
## this very screen, and the desktop-only status hint is not on screen at all
## on a handset. Re-parenting the toast needs `dcc_shell.gd`, a file this lane
## does not own, so the fix is `phone_project_picker.gd`'s own inline warn-note
## `_error_wrap`/`_error_label` (`_build()`, `_show_error()`), a child of the
## picker's own window rather than of `_phone_root`. This probe's own former
## check read `app._status_labels["hint"].text` directly -- which is exactly
## the mistake the standing rule now names ("a phone check read the text of a
## label that was hidden"): that label is real and its text is right, but on a
## handset it lives inside the collapsed `PhoneMenuModel` "More" host and
## `is_visible_in_tree()` is false. Replaced with `_plainly_visible()` /
## `_plainly_visible_within()` below, which assert the three separate facts
## "the user sees this" actually requires -- visible in tree, non-empty rect,
## not covered by (desktop) or not outside (phone) the relevant window --
## applied to the status hint on desktop and to the new inline banner on
## phone, on BOTH the recent and the browse leg of each composition.
##
## Mutation (this part): (a) reorder `_pick_world()` back to unconditional
## `hide()` and the phone banner assertions fail exactly as the Gate B1
## mutation above predicts, alongside `pk.visible`; (b) leave the `if/else`
## intact but drop the `_show_error()` call from the `else` branch alone and
## ONLY `_assert_phone_error_banner()`'s two assertions fail -- `pk.visible`
## and `current_project_path` still pass, proving the banner check is not
## just restating the Gate B1 check under a new name.
##
## Run desktop:
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _openfailgate_probe.tscn -- --corrupt <path-to-a-non-zip-file>
## Run phone:
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _openfailgate_probe.tscn -- --force-touch --corrupt <path-to-a-non-zip-file>

var _fail := 0
var _settings_path := "user://cartalith_settings.cfg"
var _settings_existed := false
var _settings_backup := PackedByteArray()
var _projects_root := ""
var _corrupt_path := ""
var _good_path := ""

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(cond: bool, msg: String) -> void:
	if cond:
		print("  OK    ", msg)
	else:
		_fail += 1
		print("  FAIL  ", msg)

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

## Lane GATE part B, 2026-09-13 -- replaces this probe's old
## `app._status_labels["hint"].text` read, which passed on the phone leg with
## the label HIDDEN (`hint` lives inside the collapsed `PhoneMenuModel` "More"
## host on a handset -- `app.gd`'s own comment on `_load_project()`). "The
## user sees this" is three separate facts, asserted separately so a probe
## that reads text off a hidden node cannot pass again: the node is
## `is_visible_in_tree()`, its rect is non-empty, and no currently-visible
## window in `blockers` (each compared in the SAME viewport-local coordinate
## space `get_global_rect()`/`.position`+`.size` already share for a Control
## and an embedded `Window` alike) covers it.
func _plainly_visible(node: Control, blockers: Array) -> Dictionary:
	if node == null or not is_instance_valid(node):
		return {"ok": false, "why": "node is null"}
	if not node.is_visible_in_tree():
		return {"ok": false, "why": "not is_visible_in_tree()"}
	var r := node.get_global_rect()
	if r.size.x <= 0.0 or r.size.y <= 0.0:
		return {"ok": false, "why": "empty rect %s" % r}
	for w in blockers:
		if w != null and is_instance_valid(w) and w.visible:
			var wr := Rect2(Vector2(w.position), Vector2(w.size))
			if wr.intersects(r):
				return {"ok": false, "why": "covered by %s at %s (node at %s)" % [w.name, wr, r]}
	return {"ok": true, "why": ""}

## The phone-specific shape of the same question: `node` lives INSIDE
## `window` (an `AcceptDialog`/`Window`), so `get_global_rect()` is already in
## that window's own local space -- the check is whether it falls inside
## `window`'s own drawn content, i.e. it was not scrolled or clipped out of
## the one window a phone screen actually shows.
func _plainly_visible_within(node: Control, window: Window) -> Dictionary:
	if node == null or not is_instance_valid(node):
		return {"ok": false, "why": "node is null"}
	if window == null or not is_instance_valid(window) or not window.visible:
		return {"ok": false, "why": "host window not visible"}
	if not node.is_visible_in_tree():
		return {"ok": false, "why": "not is_visible_in_tree()"}
	var r := node.get_global_rect()
	if r.size.x <= 0.0 or r.size.y <= 0.0:
		return {"ok": false, "why": "empty rect %s" % r}
	var win_rect := Rect2(Vector2.ZERO, Vector2(window.size))
	if not win_rect.encloses(r):
		return {"ok": false, "why": "rect %s not inside window rect %s" % [r, win_rect]}
	return {"ok": true, "why": ""}

## `phone_project_picker.gd::_error_wrap`/`_error_label` -- the inline warn
## note that function's own `_show_error()` populates in place of a phone
## toast a full-screen `AcceptDialog` would only ever draw underneath (see
## that file's `_build()` comment). Read via `get()`, matching this probe's
## own established pattern for another dialog's private field
## (`_click_picker_tile()`'s `dlg.get("_picker_tiles")` above).
func _assert_phone_error_banner(pk: Node) -> void:
	var wrap: Control = pk.get("_error_wrap")
	var label: Label = pk.get("_error_label")
	var seen := _plainly_visible_within(wrap, pk)
	_ok(seen.ok, "FAILURE: inline error banner plainly visible inside the picker window (%s)" % seen.why)
	var text := String(label.text) if label != null else ""
	_ok(text.findn("engine refused") >= 0,
		"FAILURE: the existing refusal reaches the inline banner (got \"%s\")" % text)

# ---------------------------------------------------------------------------
# user://cartalith_settings.cfg -- back up, never leave written
# ---------------------------------------------------------------------------

func _backup_settings() -> void:
	_settings_existed = FileAccess.file_exists(_settings_path)
	if _settings_existed:
		_settings_backup = FileAccess.get_file_as_bytes(_settings_path)
		print("[settings] backed up %d byte(s) of %s before any success-path test" % [_settings_backup.size(), _settings_path])
	else:
		print("[settings] %s does not exist yet -- will be removed afterward if this run creates it" % _settings_path)

func _restore_settings() -> void:
	if _settings_existed:
		var f := FileAccess.open(_settings_path, FileAccess.WRITE)
		f.store_buffer(_settings_backup)
		f.close()
		var now := FileAccess.get_file_as_bytes(_settings_path)
		var same: bool = now == _settings_backup
		print("[settings] restored -- byte-identical to the pre-probe file: ", same)
		if not same:
			_fail += 1
			print("  FAIL  settings restore was not byte-identical")
	elif FileAccess.file_exists(_settings_path):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(_settings_path))
		print("[settings] removed %s (this run created it; it did not exist before)" % _settings_path)

# ---------------------------------------------------------------------------
# Fixtures -- a real corrupt file and a real valid save, both under the real
# projects root only for the span of this probe.
# ---------------------------------------------------------------------------

func _make_fixtures(app: Node) -> bool:
	_projects_root = DccSettings.storage_root("projects")
	DirAccess.make_dir_recursive_absolute(_projects_root)

	var src_path := _arg("--corrupt", "")
	if src_path == "" or not FileAccess.file_exists(src_path):
		print("[FATAL] --corrupt <path> not given or does not exist: '%s'" % src_path)
		return false
	var corrupt_bytes := FileAccess.get_file_as_bytes(src_path)
	_corrupt_path = _projects_root.path_join("_gateprobe_corrupt.ctl")
	var cf := FileAccess.open(_corrupt_path, FileAccess.WRITE)
	cf.store_buffer(corrupt_bytes)
	cf.close()
	print("[fixture] corrupt file: %s (%d bytes, copied from %s)" % [_corrupt_path, corrupt_bytes.size(), src_path])

	var wg: WorldGen = WorldGen.new()
	wg.set_params({"tect.plates": 6})
	wg.generate_sized(1310, 640.0, 64, 48)
	_good_path = _projects_root.path_join("_gateprobe_good.ctl")
	var w: Dictionary = wg.project_save(_good_path)
	print("[fixture] good save: %s ok=%s" % [_good_path, w.get("ok")])

	## The welcome picker's own cap (`PICKER_MAX_TILES == 3`,
	## `open_project_dialog.gd`) means a real dev machine's own pre-existing
	## recents can crowd both fixtures out of the "all"-scope union
	## `_welcome_paths()` builds -- measured on this machine: neither fixture's
	## tile was found until this was added. Pushed to the FRONT of the real
	## recents list so the welcome-tile test is deterministic regardless of
	## how many other worlds this machine has open before. Covered by the same
	## `_backup_settings()`/`_restore_settings()` this file already carries for
	## the success legs' own unavoidable `remember_project()` call.
	DccSettings.remember_project(_good_path)
	DccSettings.remember_project(_corrupt_path)
	return bool(w.get("ok", false)) and FileAccess.file_exists(_good_path)

func _cleanup_fixtures() -> void:
	for p in [_corrupt_path, _good_path]:
		if p != "" and FileAccess.file_exists(p):
			DirAccess.remove_absolute(p)
			print("[cleanup] removed ", p, " -- still present: ", FileAccess.file_exists(p))

# ---------------------------------------------------------------------------
# Desktop welcome recent -- a real tile, clicked via its real gui_input signal
# ---------------------------------------------------------------------------

func _click_picker_tile(dlg: Node, path: String) -> bool:
	var basename := path.get_file().get_basename()
	var tiles: HFlowContainer = dlg.get("_picker_tiles")
	for wrap in tiles.get_children(true):
		var title: Label = wrap.find_child("Title", true, false)
		if title != null and String(title.text) == basename:
			var ev := InputEventMouseButton.new()
			ev.button_index = MOUSE_BUTTON_LEFT
			ev.pressed = true
			wrap.gui_input.emit(ev)
			return true
	print("[FATAL] no picker tile found for '%s'" % basename)
	return false

func _test_desktop_welcome_recent(app: Node) -> void:
	print("\n=== desktop welcome recent ===")
	var dlg: OpenProjectDialog = app.open_project_dialog

	print("--- success control (Gate B1) ---")
	dlg.call("open_welcome")
	await _frames(5)
	_ok(dlg.visible, "precondition: welcome dialog visible")
	var clicked_good := _click_picker_tile(dlg, _good_path)
	_ok(clicked_good, "precondition: the good save's own picker tile was found and clicked")
	await _frames(5)
	if clicked_good:
		_ok(not dlg.visible, "SUCCESS: welcome dialog hidden exactly as before this fix")
		_ok(app.current_project_path == _good_path, "SUCCESS: current_project_path set to the opened save")

	print("--- corrupt zip (the fix) ---")
	dlg.call("open_welcome")
	await _frames(5)
	_ok(dlg.visible, "precondition: welcome dialog re-opened and visible")
	var clicked_bad := _click_picker_tile(dlg, _corrupt_path)
	_ok(clicked_bad, "precondition: the corrupt file's own picker tile was found and clicked")
	await _frames(5)
	if clicked_bad:
		_ok(dlg.visible, "FAILURE: welcome dialog STILL visible after a corrupt-zip tile click")
		_ok(app.current_project_path == _good_path,
			"FAILURE: current_project_path unchanged by the refused open (still the earlier good save)")
		var hint: Label = app._status_labels["hint"]
		var seen := _plainly_visible(hint, [dlg])
		_ok(seen.ok, "FAILURE: status hint plainly visible, not covered by the dialog (%s)" % seen.why)
		_ok(String(hint.text).findn("engine refused") >= 0,
			"FAILURE: the existing refusal reaches the status hint (got \"%s\")" % String(hint.text))
	dlg.hide()

# ---------------------------------------------------------------------------
# Browse -- inject a selection into the real spawned DccBrowseDialog and
# confirm it, which calls the real registered callback.
# ---------------------------------------------------------------------------

func _find_browse_dialog(parent: Node) -> Node:
	for c in parent.get_children(true):
		if c is DccBrowseDialog:
			return c
	return null

func _confirm_browse(dlg: Node, path: String) -> bool:
	if dlg == null:
		print("[FATAL] no DccBrowseDialog spawned")
		return false
	dlg.set("_selected", path)
	dlg.call("_confirm")
	return true

## Lane GATE part B, 2026-09-13. **The one named entry point the inherited
## probe never actually drove.** `_pick_path()`'s own doc comment
## (`open_project_dialog.gd`) claims this was "reproduced with a corrupt
## `.ctl` through the welcome recent tile, this dialog's own Open button and
## the disk browser (`_openfailgate_probe.gd`)" -- but `_test_desktop_welcome
## _recent()` above only ever calls `open_welcome()`, whose own tiles
## (`_build_tile(path, meta, picker=true)`) bypass `_select()`/`_confirm()`
## entirely and open on one click (that function's own comment: "there is no
## selection on this screen and no `Open selected` button for one to feed").
## The real "Open selected" button (`_open_btn`, `_build_foot()`) only ever
## becomes enabled through the GALLERY composition -- `open()`, not
## `open_welcome()` -- where a single click `_select()`s a tile and either a
## double-click or this button `_confirm()`s it. A claim is not the code that
## backs it; this is that code.
func _test_desktop_open_button(app: Node) -> void:
	print("\n=== desktop Open button (\"Open selected\") ===")
	var dlg: OpenProjectDialog = app.open_project_dialog

	print("--- success control (Gate B1) ---")
	dlg.call("open")
	await _frames(5)
	_ok(dlg.visible, "precondition: gallery dialog visible")
	dlg.call("_select", _good_path)
	await _frames(2)
	var open_btn: Button = dlg.get("_open_btn")
	_ok(open_btn != null and not open_btn.disabled,
		"precondition: \"Open selected\" enabled for the good save's own tile")
	dlg.call("_confirm")
	await _frames(5)
	_ok(not dlg.visible, "SUCCESS: dialog hidden exactly as before this fix")
	_ok(app.current_project_path == _good_path, "SUCCESS: current_project_path set to the opened save")

	print("--- corrupt zip (the fix) ---")
	dlg.call("open")
	await _frames(5)
	_ok(dlg.visible, "precondition: gallery dialog re-opened and visible")
	dlg.call("_select", _corrupt_path)
	await _frames(2)
	_ok(not bool(open_btn.disabled),
		"precondition: \"Open selected\" enabled for the corrupt file's own tile")
	dlg.call("_confirm")
	await _frames(5)
	_ok(dlg.visible, "FAILURE: dialog STILL visible after a corrupt-zip Open-button confirm")
	_ok(app.current_project_path == _good_path, "FAILURE: current_project_path unchanged")
	var hint: Label = app._status_labels["hint"]
	var seen := _plainly_visible(hint, [dlg])
	_ok(seen.ok, "FAILURE: status hint plainly visible, not covered by the dialog (%s)" % seen.why)
	_ok(String(hint.text).findn("engine refused") >= 0,
		"FAILURE: the existing refusal reaches the status hint (got \"%s\")" % String(hint.text))
	dlg.hide()

func _test_desktop_browse(app: Node) -> void:
	print("\n=== desktop browse ===")
	var dlg: OpenProjectDialog = app.open_project_dialog

	print("--- success control (Gate B1) ---")
	dlg.call("open_welcome")
	await _frames(5)
	dlg.call("_browse_from_disk")
	await _frames(3)
	var bd1 := _find_browse_dialog(dlg)
	if _confirm_browse(bd1, _good_path):
		await _frames(5)
		_ok(not dlg.visible, "SUCCESS: dialog hidden exactly as before this fix")
		_ok(app.current_project_path == _good_path, "SUCCESS: current_project_path set to the opened save")

	print("--- corrupt zip (the fix) ---")
	dlg.call("open_welcome")
	await _frames(5)
	_ok(dlg.visible, "precondition: dialog re-opened and visible")
	dlg.call("_browse_from_disk")
	await _frames(3)
	var bd2 := _find_browse_dialog(dlg)
	if _confirm_browse(bd2, _corrupt_path):
		await _frames(5)
		_ok(dlg.visible, "FAILURE: dialog STILL visible after a corrupt-zip browse pick")
		_ok(app.current_project_path == _good_path, "FAILURE: current_project_path unchanged")
		var hint: Label = app._status_labels["hint"]
		var seen := _plainly_visible(hint, [dlg])
		_ok(seen.ok, "FAILURE: status hint plainly visible, not covered by the dialog (%s)" % seen.why)
		_ok(String(hint.text).findn("engine refused") >= 0,
			"FAILURE: the existing refusal reaches the status hint (got \"%s\")" % String(hint.text))
	dlg.hide()

# ---------------------------------------------------------------------------
# Phone
# ---------------------------------------------------------------------------

func _test_phone_recent(app: Node) -> void:
	print("\n=== phone recent ===")
	var pk = app.phone_project_picker

	print("--- success control (Gate B1) ---")
	pk.call("open")
	await _frames(5)
	_ok(pk.visible, "precondition: phone picker visible")
	pk.call("_pick_world", _good_path)
	await _frames(5)
	_ok(not pk.visible, "SUCCESS: phone picker hidden exactly as before this fix")
	_ok(app.current_project_path == _good_path, "SUCCESS: current_project_path set to the opened save")

	print("--- corrupt zip (the fix) ---")
	pk.call("open")
	await _frames(5)
	_ok(pk.visible, "precondition: phone picker re-opened and visible")
	pk.call("_pick_world", _corrupt_path)
	await _frames(5)
	_ok(pk.visible, "FAILURE: phone picker STILL visible after a corrupt-zip recent tap")
	_ok(app.current_project_path == _good_path, "FAILURE: current_project_path unchanged")
	_assert_phone_error_banner(pk)
	pk.hide()

func _test_phone_browse(app: Node) -> void:
	print("\n=== phone browse ===")
	var pk = app.phone_project_picker

	print("--- success control (Gate B1) ---")
	pk.call("open")
	await _frames(5)
	pk.call("_on_open_zip")
	await _frames(3)
	## `_on_open_zip()` passes `_host` (the app root) as `choose_file()`'s host,
	## not the picker -- `phone_project_picker.gd`'s own call, matched here.
	var bd1 := _find_browse_dialog(app)
	if _confirm_browse(bd1, _good_path):
		await _frames(5)
		_ok(not pk.visible, "SUCCESS: phone picker hidden exactly as before this fix")
		_ok(app.current_project_path == _good_path, "SUCCESS: current_project_path set to the opened save")

	print("--- corrupt zip (the fix) ---")
	pk.call("open")
	await _frames(5)
	_ok(pk.visible, "precondition: phone picker re-opened and visible")
	pk.call("_on_open_zip")
	await _frames(3)
	var bd2 := _find_browse_dialog(app)
	if _confirm_browse(bd2, _corrupt_path):
		await _frames(5)
		_ok(pk.visible, "FAILURE: phone picker STILL visible after a corrupt-zip browse pick")
		_ok(app.current_project_path == _good_path, "FAILURE: current_project_path unchanged")
		_assert_phone_error_banner(pk)
	pk.hide()

# ---------------------------------------------------------------------------

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return

	_backup_settings()

	var phone_requested := "--force-touch" in OS.get_cmdline_user_args()
	var vp := SubViewport.new()
	vp.size = Vector2i(1080, 2340) if phone_requested else Vector2i(1600, 900)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(50)
	var phone := bool(app.is_phone())
	if phone_requested and not phone:
		print("[FATAL] --force-touch given but did not boot into phone composition")
		_restore_settings(); get_tree().quit(1); return
	print("[BOOT] composition=%s" % ("phone" if phone else "desktop"))

	if not _make_fixtures(app):
		_restore_settings()
		get_tree().quit(1)
		return

	if phone:
		await _test_phone_recent(app)
		await _test_phone_browse(app)
	else:
		await _test_desktop_welcome_recent(app)
		await _test_desktop_open_button(app)
		await _test_desktop_browse(app)

	_cleanup_fixtures()
	_restore_settings()
	print("\n_openfailgate_probe (%s): %d failure(s)" % [("phone" if phone else "desktop"), _fail])
	get_tree().quit(1 if _fail > 0 else 0)
