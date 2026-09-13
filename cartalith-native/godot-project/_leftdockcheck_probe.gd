extends Node
## MENUS lane, Part A, 2026-09-13. **Inverted from a defect-reproduction probe
## into a regression probe** now that the fix has landed -- every `_ok` below
## asserts the FIXED behaviour and must FAIL again if the fix is reverted.
##
## The confirmed root cause (unchanged from the 2026-09-07 finding this probe
## first encoded): `menus.gd::_window()`'s five region check items were built
## once, checked `true`, and only ever flipped LOCALLY by `id_pressed`
## (`_sync_region_checks()`) -- nothing read the real region state back into
## them. On phone the real state for Left dock is `_left_sheet_open`
## (`dcc_shell.gd:513`, inherited by `DccApp`), which boots `false` -- already
## disagreeing with the popup's hardcoded `true` before a single interaction,
## and any OTHER opener (`phone_menu.gd::_open_left_sheet()`, the sheet's own
## close button, a restored layout) drifted the two further apart with no
## resync in either direction.
##
## **The fix** (`app.gd::is_region_shown()`, `menus.gd::_window()`'s new
## `about_to_popup` handler, `menus.gd::_capture_layout()`): a query
## counterpart to `toggle_region()`, same phone/desktop branches, read every
## time the Window popup is ABOUT to be shown -- which is the only moment its
## checkmarks are visible to anyone. **That "about to be shown" moment matters
## for this probe**: the checkmarks are not live-bound, so a scenario that
## changes the real state without ever re-opening the menu correctly shows the
## PRE-existing checked state until `about_to_popup` fires again, exactly like
## `_workspace_popup`/`_windows_popup`/`_layouts_popup` already did before this
## fix. Every checkpoint below fires `popup.about_to_popup.emit()` -- the same
## call `_seedlayout_probe.gd` uses -- immediately before reading
## `is_item_checked()`, because that is what opening the menu does.
##
## Windowed (pixel-probe rules aside, this reads Control/script state, not
## pixels, but the brief specifies windowed 1080x2340 and this follows it
## literally), `--force-touch` -- desktop-emulated phone, not a real handset;
## said explicitly in the printed output, per the phone-shaped-verification
## rule.
##
##   Godot_v4.7.1 --path . --resolution 1200x800 _leftdockcheck_probe.tscn -- --force-touch

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var forced := "--force-touch" in OS.get_cmdline_user_args()
	print("[BOOT] force-touch=", forced)

	var vp := SubViewport.new()
	vp.size = Vector2i(1080, 2340)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(45)

	_ok("classified as touch", DccTheme.is_touch(), true)
	_ok("classified as PHONE", DccTheme.is_phone(), true)
	_ok("and NOT tablet", DccTheme.is_tablet(), false)
	if not DccTheme.is_phone():
		print("[FATAL] not phone-classified -- nothing below is meaningful"); get_tree().quit(1); return

	if not app.has_method("is_region_shown"):
		print("[FATAL] DccApp.is_region_shown() is missing -- the fix is not present"); get_tree().quit(1); return

	var popup: PopupMenu = null
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == "Window":
			popup = (ch as MenuButton).get_popup()
	if popup == null:
		print("[FATAL] no Window popup found on app.menu_bar_row"); get_tree().quit(1); return
	var idx := popup.get_item_index(DccMenus.ID_WIN_LEFT)
	if idx < 0:
		print("[FATAL] ID_WIN_LEFT not present in the Window popup"); get_tree().quit(1); return
	print("  Window popup found, Left dock at item index ", idx, " text='", popup.get_item_text(idx), "'")

	## Every checkpoint re-opens the menu (fires `about_to_popup`) before
	## reading the checkmark -- that is the only moment the fix promises to
	## have re-read the real state, and the only moment a real user ever sees
	## the checkmark at all.
	print("")
	print("-- boot state, popup opened for the first time --")
	popup.about_to_popup.emit()
	await _frames(2)
	var shadow0: bool = popup.is_item_checked(idx)
	var real0: bool = bool(app.get("_left_sheet_open"))
	print("  popup checked (what MORE ▸ Window ▸ Left dock's switch draws) = ", shadow0)
	print("  real _left_sheet_open (is the sheet actually open)            = ", real0)
	_ok("FIXED: switch matches the real (closed) state at first open", shadow0, real0)
	_ok("...and that real state is actually closed", real0, false)

	print("")
	print("-- press the row itself (production path: popup.id_pressed.emit) --")
	popup.id_pressed.emit(DccMenus.ID_WIN_LEFT)
	await _frames(2)
	popup.about_to_popup.emit()
	await _frames(2)
	var shadow1: bool = popup.is_item_checked(idx)
	var real1: bool = bool(app.get("_left_sheet_open"))
	print("  after 1 press: shadow=", shadow1, "  real=", real1)
	_ok("pressing the row flips the real sheet", real1, not real0)
	_ok("...and the switch still matches it", shadow1, real1)

	popup.id_pressed.emit(DccMenus.ID_WIN_LEFT)
	await _frames(2)
	popup.about_to_popup.emit()
	await _frames(2)
	var shadow2: bool = popup.is_item_checked(idx)
	var real2: bool = bool(app.get("_left_sheet_open"))
	print("  after 2 presses: shadow=", shadow2, "  real=", real2, "  (back to boot real)")
	_ok("back to boot real state", real2, real0)
	_ok("...and the switch matches it", shadow2, real2)

	print("")
	print("-- REGRESSION CHECK: open the sheet a DIFFERENT way, bypassing this row's own press --")
	if not app.has_method("_set_sheet_open"):
		print("[FATAL] DccApp/DccShell has no _set_sheet_open -- cannot stage this leg"); get_tree().quit(1); return
	app.call("_set_sheet_open", "left", true)
	await _frames(2)
	## Deliberately NOT firing `about_to_popup` yet: the fix re-reads on open,
	## not live, so the checkmark is allowed to still show whatever it showed
	## the last time the menu was opened until the menu is opened again. This
	## is what `_workspace_popup`/`_windows_popup`/`_layouts_popup` already do
	## and is not the bug this probe targets.
	var shadow3_stale: bool = popup.is_item_checked(idx)
	print("  sheet opened via _set_sheet_open, popup NOT re-opened: shadow=", shadow3_stale,
		"  real=", bool(app.get("_left_sheet_open")), "  (stale is expected here)")
	popup.about_to_popup.emit()
	await _frames(2)
	var shadow3: bool = popup.is_item_checked(idx)
	var real3: bool = bool(app.get("_left_sheet_open"))
	print("  ...menu re-opened: shadow=", shadow3, "  real=", real3)
	_ok("FIXED: re-opening the menu resyncs the switch to a sheet opened some OTHER way",
		shadow3, real3)
	_ok("...and that real state is actually open", real3, true)

	app.call("_set_sheet_open", "left", false)
	await _frames(2)
	popup.about_to_popup.emit()
	await _frames(2)
	var shadow4: bool = popup.is_item_checked(idx)
	var real4: bool = bool(app.get("_left_sheet_open"))
	print("  sheet closed via _set_sheet_open directly, menu re-opened: shadow=", shadow4,
		"  real=", real4)
	_ok("FIXED: re-opening the menu resyncs the switch to a sheet CLOSED some OTHER way",
		(shadow4 == false and real4 == false), true)

	print("")
	print("-- sanity: all five WIN_REGION_IDS agree with is_region_shown() after one more open --")
	popup.about_to_popup.emit()
	await _frames(2)
	var all_ok := true
	for rid in DccMenus.WIN_REGION_IDS:
		var i := popup.get_item_index(rid)
		if i < 0:
			print("  [FATAL] region id ", rid, " missing from the Window popup"); all_ok = false
			continue
		var checked: bool = popup.is_item_checked(i)
		var real: bool = bool(app.call("is_region_shown", rid))
		if checked != real:
			all_ok = false
		print("  ", popup.get_item_text(i), ": checked=", checked, " is_region_shown=", real,
			"  ", "ok" if checked == real else "MISMATCH")
	_ok("all five region rows agree with is_region_shown() after an open", all_ok, true)

	print("")
	print("_leftdockcheck_probe: ", "PASS (fix holds)" if _fail == 0 else str(_fail) + " unexpected result(s) -- fix is missing or broken")
	get_tree().quit(1 if _fail > 0 else 0)
