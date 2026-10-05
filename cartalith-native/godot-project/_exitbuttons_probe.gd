extends Node
## Windowed verification for OUTSTANDING_WORK.md 2.11's "Exit buttons are
## inconsistent across windows" row, and Ruling AZ (LARGE_ITEM_RULINGS.md,
## 2026-09-28): Close for a window that only shows things or applies changes
## live; OK / Cancel only where changes are held until a confirm.
##
## Run windowed (not --headless): headless button `.text`/`.visible` reads are
## real (MISTAKES.md's headless caveat is about `ImageTexture`/pixels, not
## Control state), but this probe still opens real `popup_centered()` windows
## and a dummy rasteriser is the safer, established way to do that in this
## shell's own probes.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _exitbuttons_probe.tscn
##
## Inventory (walked from the code, not a list -- grep for `AcceptDialog.new`,
## `ConfirmationDialog.new`, `extends AcceptDialog`, `extends Window`,
## `popup_centered` across `shell/*.gd`, excluding the CM-7-owned files this
## lane does not touch: `map_overlay.gd`, `shell/context_*`, `shell/workspaces/
## *.gd`, `shell/engine_bridge.gd`, `shell/app.gd`, `shell/right_dock.gd`):
##
## DISMISS-ONLY (apply live / show only) -- must read "Close":
##   faction_roster_window, place_editor_window, settlement_types_window
##     (the three that read stock "OK" on desktop before this batch --
##     `dcc_widgets.gd::phone_window()` only set `ok_button_text = "Close"`
##     on its phone branch)
##   city_viewer_window, gen_info_dialog, world_data_window, shortcuts_dialog
##     (already explicit `ok_button_text = "Close"`)
##   generation_rules_window, travel_library_window (culture_profiles_window was
##     retired in FH-2 -- its content is the roster window's Identity tab)
##     (own hidden-OK + custom "Close" footer button)
##   asset_library_window, data_manager_window
##     (own hidden-OK + custom "Close <cross>" window-bar chip)
##   vault_window (main window: explicit "Close")
##   app.gd (DccShell)'s own desktop Find-on-map dialog: explicit "Close"
##   app.gd's `open_storage_locations()` / `open_about()`: no override at all
##     (same latent bug class as the three fixed windows -- fixed for free by
##     moving the assignment in `phone_window()`, verified here even though
##     `app.gd` is CM-7's file and was not edited)
##
## HELD-UNTIL-CONFIRM (kept as OK/Cancel, OK named for the action) --
## untouched by this batch, checked here to prove they still are:
##   new_world_dialog ("Create", commits only in `confirmed` -> `_on_create`)
##   open_project_dialog / browse_dialog (own "Cancel" + a named primary,
##     commits only on the primary press)
##   `menus.gd`'s three hand-built `ConfirmationDialog`s: pack metadata
##     ("Save"), save-layout ("Save"/"Replace"), forget-layout ("Forget") --
##     all commit only inside their own `confirmed.connect` closure
##
## DESTRUCTIVE CONFIRMATIONS -- kept as-is, not touched, not re-checked here
## (the row says so explicitly): `faction_roster_window.gd::_confirm_remove`,
## `place_editor_window.gd::confirm_delete`, `right_dock.gd::_confirm_revert`
## (CM-7's file), `civilization_workspace.gd`/`world_workspace.gd` (CM-7's
## workspaces), `app.gd::_close_requested`'s quit prompt.
##
## SKIPPED, not a desktop exit-button surface:
##   `phone_project_picker.gd` -- phone-only, own full-screen actions replace
##     an exit button entirely, no desktop dialog to check
##   `vault_window.gd`'s `_compare_dialog`/`_preview_dialog` -- private,
##     reachable only mid-flow (a real diff / a real pending write); read at
##     the symbol instead (see this file's own header comment and this
##     probe's doc comment above) rather than reconstructed in a probe
##   `journey_planner_view.gd` -- `extends Control`, an in-shell tool
##     takeover, not a `Window`/`AcceptDialog` (its own header records the
##     `extends AcceptDialog` shape was deleted)
##   `layers_popover.gd` -- `extends PopupPanel`, no OK/Close button at all

var _fails: Array = []
var _app: Node


func _check(name: String, kind: String, cond: bool, detail: String) -> void:
	if cond:
		print("EXIT     OK  [%s] %s -- %s" % [kind, name, detail])
	else:
		_fails.append("[%s] %s -- %s" % [kind, name, detail])
		print("EXIT     !!  [%s] %s -- %s" % [kind, name, detail])


func _find_button(n: Node, matcher: Callable) -> Button:
	if n is Button and matcher.call(String((n as Button).text)):
		return n
	for c in n.get_children():
		var r := _find_button(c, matcher)
		if r != null:
			return r
	return null


## The rule, applied to one already-opened dismiss-only window. Reads the
## real control state (`.text`, `.visible`), never the scene graph shape.
func _assert_close(name: String, dlg: AcceptDialog) -> void:
	var ok := dlg.get_ok_button()
	if ok.visible:
		_check(name, "dismiss", ok.text == "Close",
			"stock OK button visible, text='%s'" % ok.text)
		return
	var chip := _find_button(dlg, func(t: String): return t.begins_with("Close"))
	if chip == null:
		_check(name, "dismiss", false, "stock OK hidden and no visible 'Close...' button found")
		return
	_check(name, "dismiss", chip.visible, "custom close control text='%s' visible=%s" % [chip.text, chip.visible])


func _assert_held_ok_text(name: String, dlg: AcceptDialog, expected: String) -> void:
	var ok := dlg.get_ok_button()
	_check(name, "held", ok.visible and ok.text == expected,
		"OK button visible=%s text='%s' (expected '%s')" % [ok.visible, ok.text, expected])


func _assert_held_custom_footer(name: String, dlg: AcceptDialog, primary_text: String) -> void:
	var cancel := _find_button(dlg, func(t: String): return t == "Cancel")
	var primary := _find_button(dlg, func(t: String): return t == primary_text)
	var ok_hidden := not dlg.get_ok_button().visible
	_check(name, "held",
		ok_hidden and cancel != null and cancel.visible and primary != null and primary.visible,
		"stock OK hidden=%s, Cancel found=%s, primary '%s' found=%s" %
			[ok_hidden, cancel != null, primary_text, primary != null])


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		push_error("_exitbuttons_probe: run WINDOWED (no --headless).")
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

	# -- positive control: a hand-built AcceptDialog that never went through
	# `DccWidgets.phone_window()`, so it keeps stock `AcceptDialog`'s "OK".
	# Proves the dismiss checker actually fails something rather than passing
	# vacuously -- the same reason every pixel probe in this shell needs one.
	var control := AcceptDialog.new()
	control.title = "Positive control (expected to fail)"
	add_child(control)
	control.popup_centered()
	await _frames(2)
	var control_ok := control.get_ok_button()
	var control_would_pass := control_ok.visible and control_ok.text == "Close"
	print("EXIT     control: stock AcceptDialog reads '%s' -- %s" %
		[control_ok.text, "WRONG, checker is vacuous" if control_would_pass else "correctly not Close"])
	if control_would_pass:
		_fails.append("POSITIVE CONTROL DID NOT TRIP -- checker is vacuous")
	control.hide()
	control.queue_free()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	var bridge = _app.bridge

	# -- open_project_dialog / browse_dialog, held-change, BEFORE it is hidden
	# for the rest of the run --------------------------------------------------
	if _app.open_project_dialog and _app.open_project_dialog.visible:
		_assert_held_custom_footer("open_project_dialog (welcome)",
			_app.open_project_dialog, "Open selected")
	_app.open_project_dialog.hide()
	await _frames(2)

	bridge.generate({"seed": 55219, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.5).timeout

	var settlements: Array = bridge.settlements()
	_check("world: a settlement exists to open place/city/roster windows",
		"setup", settlements.size() > 0, "%d settlements" % settlements.size())
	if settlements.is_empty():
		_finish()
		return

	# -- the three fixed windows -----------------------------------------------
	_app.faction_roster_window.open()
	await _frames(3)
	_assert_close("faction_roster_window", _app.faction_roster_window)
	_app.faction_roster_window.hide()
	await _frames(2)

	_app.place_editor_window.open_for(0)
	await _frames(3)
	_assert_close("place_editor_window", _app.place_editor_window)
	_app.place_editor_window.hide()
	await _frames(2)

	_app.settlement_types_window.open()
	await _frames(3)
	_assert_close("settlement_types_window", _app.settlement_types_window)
	_app.settlement_types_window.hide()
	await _frames(2)

	# -- already-compliant dismiss-only windows, regression coverage ----------
	_app.city_viewer_window.open(0)
	await _frames(3)
	_assert_close("city_viewer_window", _app.city_viewer_window)
	_app.city_viewer_window.hide()
	await _frames(2)

	_app.gen_info_dialog.open()
	await _frames(3)
	_assert_close("gen_info_dialog", _app.gen_info_dialog)
	_app.gen_info_dialog.hide()
	await _frames(2)

	_app.world_data_window.open()
	await _frames(3)
	_assert_close("world_data_window", _app.world_data_window)
	_app.world_data_window.hide()
	await _frames(2)

	_app.shortcuts_dialog.open()
	await _frames(3)
	_assert_close("shortcuts_dialog", _app.shortcuts_dialog)
	_app.shortcuts_dialog.hide()
	await _frames(2)

	_app.generation_rules_window.open()
	await _frames(3)
	_assert_close("generation_rules_window", _app.generation_rules_window)
	_app.generation_rules_window.hide()
	await _frames(2)

	_app.travel_library_window.open()
	await _frames(3)
	_assert_close("travel_library_window", _app.travel_library_window)
	_app.travel_library_window.hide()
	await _frames(2)

	_app.asset_library_window.open()
	await _frames(3)
	_assert_close("asset_library_window", _app.asset_library_window)
	_app.asset_library_window.hide()
	await _frames(2)

	_app.data_manager_window.open()
	await _frames(3)
	_assert_close("data_manager_window", _app.data_manager_window)
	_app.data_manager_window.hide()
	await _frames(2)

	_app.vault_window.open_overview()
	await _frames(3)
	_assert_close("vault_window (main)", _app.vault_window)
	_app.vault_window.hide()
	await _frames(2)

	# -- app.gd's own two dialogs (same latent bug class, fixed for free) -----
	_app.open_storage_locations()
	await _frames(3)
	var storage_dlg: AcceptDialog = _find_recent_dialog(_app, "Storage locations")
	if storage_dlg != null:
		_assert_close("app.gd::open_storage_locations", storage_dlg)
		storage_dlg.hide()
	else:
		_check("app.gd::open_storage_locations", "dismiss", false, "dialog not found by title")
	await _frames(2)

	_app.open_about()
	await _frames(3)
	var about_dlg: AcceptDialog = _find_recent_dialog(_app, "About Cartalith")
	if about_dlg != null:
		_assert_close("app.gd::open_about", about_dlg)
		about_dlg.hide()
	else:
		_check("app.gd::open_about", "dismiss", false, "dialog not found by title")
	await _frames(2)

	# -- the dcc_shell (DccApp IS-A DccShell) desktop search dialog ------------
	_app._ensure_desktop_search_dialog()
	_app._open_desktop_find_on_map()
	await _frames(3)
	_assert_close("dcc_shell desktop Find on map", _app._desktop_search_dialog)
	_app._desktop_search_dialog.hide()
	await _frames(2)

	# -- held-change dialogs, untouched by this batch --------------------------
	_app.open_new_world()
	await _frames(3)
	_assert_held_ok_text("new_world_dialog", _app.new_world_dialog, "Create")
	_app.new_world_dialog.hide()
	await _frames(2)

	_app.menus._open_pack_metadata()
	await _frames(3)
	var pack_dlg: AcceptDialog = _find_recent_confirmation(_app, "Pack metadata")
	if pack_dlg != null:
		_assert_held_ok_text("menus._open_pack_metadata", pack_dlg, "Save")
		pack_dlg.hide()
	else:
		_check("menus._open_pack_metadata", "held", false, "dialog not found by title")
	await _frames(2)

	_app.menus._prompt_save_layout()
	await _frames(3)
	var layout_dlg: AcceptDialog = _find_recent_confirmation(_app, "Save layout as")
	if layout_dlg != null:
		_assert_held_ok_text("menus._prompt_save_layout", layout_dlg, "Save")
		layout_dlg.hide()
	else:
		_check("menus._prompt_save_layout", "held", false, "dialog not found by title")
	await _frames(2)

	_app.menus._prompt_forget_layout()
	await _frames(3)
	var forget_dlg: AcceptDialog = _find_recent_confirmation(_app, "Forget layout")
	if forget_dlg != null:
		_assert_held_ok_text("menus._prompt_forget_layout", forget_dlg, "Forget")
		forget_dlg.hide()
	else:
		## `_prompt_forget_layout()` no-ops if there are no saved layouts yet
		## (`names.is_empty()` guard) -- not a probe failure, just nothing to
		## check this run.
		print("EXIT     --  menus._prompt_forget_layout -- no saved layouts this run, skipped")

	_app.open_project_dialog.open()
	await _frames(3)
	_assert_held_custom_footer("open_project_dialog (browser)", _app.open_project_dialog, "Open selected")
	_app.open_project_dialog.hide()

	_finish()


func _find_recent_dialog(host: Node, title: String) -> AcceptDialog:
	for c in host.get_children():
		if c is AcceptDialog and String((c as AcceptDialog).title) == title:
			return c
	return null


func _find_recent_confirmation(host: Node, title: String) -> AcceptDialog:
	for c in host.get_children():
		if c is ConfirmationDialog and String((c as ConfirmationDialog).title) == title:
			return c
	return null


func _finish() -> void:
	if _fails.is_empty():
		print("EXIT     ALL CHECKS PASSED")
	else:
		print("EXIT     %d FAILED:" % _fails.size())
		for f in _fails:
			print("EXIT       - %s" % f)
	get_tree().quit(0 if _fails.is_empty() else 1)
