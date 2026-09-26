extends Node
## CM-5 (`MAP_CONTEXT_SCOPE.md` §8.1, §11; Ruling AX F1): the phone's own
## noun surface -- long-press -> sample pin + peek chips -> half-detent card,
## with a draggable pin that re-resolves the card. Drives real input through
## `map_overlay.gd::_gui_input()` (`InputEvent.DEVICE_ID_EMULATION`, the same
## marker `_ctxtablet_probe.gd` and `_ctxbroker_probe.gd` already use), and
## reads `shell/phone_menu.gd`'s own peek-card introspection rather than pixels.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _ctxphone_probe.tscn -- --vp 1080x2340 --force-touch
##
## **Windowed, not `--headless`**: leg F reads real global rects against the
## real viewport. `--vp WxH` sizes the host SubViewport and `--force-touch` is
## the shell's own flag (`DccShell._ready()`); both are required or the probe
## aborts (leg FORM).
##
## Legs:
##   T  timing: a 400 ms hold does nothing (no pin, no sheet); 480 ms+ drops
##      the pin and opens the peek sheet, with the chip row populated
##   S  the peek sheet's header and chips for a settlement
##   E  the peek sheet's header and chips for empty ground (no settlement)
##   H  swiping (dragging the grab handle up) expands peek -> half, and the
##      full sectioned row list appears in place of the chip row
##   D  dragging the pin to a new spot re-resolves the card's content, at the
##      SAME detent it was already at (not reset to peek)
##   R  the peek panel's and the half card's own rects stay inside the phone
##      viewport, measured in both px and dp
##   X  a scrim tap dismisses the sheet AND clears the map's sample pin

const SEED := 552017

var app: Node
var _vp: SubViewport
var _fails := 0
var _checks := 0


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, got: Variant, want: Variant) -> void:
	_checks += 1
	var pass_: bool = typeof(got) == typeof(want) and got == want
	if not pass_:
		_fails += 1
	print("%s  %s  got=%s want=%s" % ["PASS" if pass_ else "FAIL", what, str(got), str(want)])


func _menu():
	return app._phone_menu


## `InputEvent.DEVICE_ID_EMULATION`: `map_overlay.gd`'s own `mb.device < 0`
## gate for the withheld-touch-press branch.
func _touch_press(ov: Control, pos: Vector2) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = true
	ev.position = pos
	ev.device = InputEvent.DEVICE_ID_EMULATION
	ov._gui_input(ev)


func _touch_release(ov: Control, pos: Vector2) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = false
	ev.position = pos
	ev.device = InputEvent.DEVICE_ID_EMULATION
	ov._gui_input(ev)


func _touch_move(ov: Control, pos: Vector2) -> void:
	var mv := InputEventMouseMotion.new()
	mv.position = pos
	mv.button_mask = MOUSE_BUTTON_MASK_LEFT
	mv.device = InputEvent.DEVICE_ID_EMULATION
	ov._gui_input(mv)


## Press, wait `secs`, and settle -- with the finger still down, so a caller
## can release at whatever moment it wants to test.
func _hold_wait(ov: Control, pos: Vector2, secs: float) -> void:
	_touch_press(ov, pos)
	await get_tree().create_timer(secs).timeout
	await _frames(2)


## The grab handle's own drag, driven through `PhoneMenu.peek_grab_input()`
## (the probe seam) rather than through `_gui_input()` -- the handle is not a
## `Control` whose own `_gui_input` this file could call directly; its press
## is bound to a `gui_input` SIGNAL on a private child. `dy` is a local-space
## offset from the handle's own origin, matching what `_on_peek_grab_input()`
## itself reads off a real event.
func _grab_drag(menu, dy_target: float) -> void:
	var down := InputEventMouseButton.new()
	down.button_index = MOUSE_BUTTON_LEFT
	down.pressed = true
	down.position = Vector2.ZERO
	menu.peek_grab_input(down)
	await _frames(1)
	var mv := InputEventMouseMotion.new()
	mv.position = Vector2(0.0, dy_target)
	menu.peek_grab_input(mv)
	await _frames(2)
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = Vector2(0.0, dy_target)
	menu.peek_grab_input(up)
	await _frames(6)


func _run() -> void:
	app.select_domain("civilization")
	await _frames(4)
	var ov: Control = app.viewport.overlay
	var rect: Rect2 = ov._displayed_rect()
	var inter: Rect2 = ov._interior_rect(rect)
	var vis: Rect2 = app.get_viewport().get_visible_rect()

	## Fixture: a settlement the overlay's own hit test resolves to itself,
	## and an empty cell -- the same fixture-finding method `_ctxbroker_probe
	## .gd` and `_ctxtablet_probe.gd` already use.
	var sets: Array = ov.get("_settlements")
	var sk := -1
	var spos := Vector2.ZERO
	for i in sets.size():
		var s: Dictionary = sets[i]
		var p: Vector2 = ov._cell_to_screen(Vector2(s["x"], s["y"]), rect)
		if not inter.grow(-60.0).has_point(p):
			continue
		if ov._hit_test_settlement(p, inter, rect) == i:
			sk = i
			spos = p
			break
	_ok("a settlement to hold on exists", sk >= 0, true)
	if sk < 0:
		return
	var sname := String(app.bridge.settlements()[sk].get("name", "(unnamed)"))
	print("FIXTURE settlement k=%d name=%s screen=%s" % [sk, sname, spos])

	var epos := Vector2.ZERO
	var found_empty := false
	for tries in 400:
		var cand: Vector2 = inter.position + Vector2(
			randf() * inter.size.x, randf() * inter.size.y)
		if ov._hit_test_settlement(cand, inter, rect) != -1:
			continue
		var near_settlement := false
		for i in sets.size():
			var sp: Vector2 = ov._cell_to_screen(Vector2(sets[i]["x"], sets[i]["y"]), rect)
			if cand.distance_to(sp) < 80.0:
				near_settlement = true
				break
		if near_settlement:
			continue
		epos = cand
		found_empty = true
		break
	_ok("an empty cell (no settlement pin) exists", found_empty, true)
	if not found_empty:
		return
	print("FIXTURE empty screen=%s" % epos)

	# -- T: timing -- 400 ms does nothing, 480 ms+ drops the pin ------------------
	await _hold_wait(ov, spos, 0.4)
	_ok("T 400ms: no sample pin yet", ov.has_sample_pin(), false)
	_ok("T 400ms: no peek sheet yet", _menu().peek_card_is_open(), false)
	_touch_release(ov, spos)
	await _frames(4)
	_ok("T 400ms release (a tap, not a hold): no pin left behind", ov.has_sample_pin(), false)

	await _hold_wait(ov, spos, 0.85)
	_ok("T 600ms: the sample pin dropped", ov.has_sample_pin(), true)
	_ok("T 600ms: the peek sheet opened", _menu().peek_card_is_open(), true)
	_ok("T 600ms: opened at the peek detent", _menu().peek_detent(), "peek")
	_touch_release(ov, spos)
	await _frames(4)
	_ok("T lift after the hold: the pin is still down (not a swallowed drag)", ov.has_sample_pin(), true)
	_ok("T lift after the hold: the sheet is still open", _menu().peek_card_is_open(), true)

	# -- S: the peek sheet's header and chips for a settlement --------------------
	_ok("S peek header names the settlement", _menu().peek_title_text(), sname)
	var s_chips: PackedStringArray = _menu().peek_chip_labels()
	print("S settlement chips=%s" % [s_chips])
	_ok("S peek shows at least one chip for a settlement", s_chips.size() > 0, true)
	_ok("S peek shows at most four chips (§8.1.2)", s_chips.size() <= 4, true)

	# -- H: swipe the grab handle up -> half detent, full row list ---------------
	## A generous upward drag; the release snaps to the nearest detent.
	await _grab_drag(_menu(), -900.0)
	_ok("H drag-up: the sheet reached the half detent", _menu().peek_detent(), "half")
	var full_rows: PackedStringArray = _menu().peek_full_row_labels()
	print("H settlement full rows=%s" % [full_rows])
	_ok("H half detent: the full row list is non-empty (§8.1.3)", full_rows.size() > 0, true)
	_ok("H half detent: at least as many rows as chips", full_rows.size() >= s_chips.size(), true)

	# Collapse back to peek before the next leg, so D starts clean.
	await _grab_drag(_menu(), 900.0)
	_ok("collapsed back to peek before leg D", _menu().peek_detent(), "peek")

	# -- D: dragging the pin re-resolves the card, at the SAME detent ------------
	_touch_press(ov, spos)
	await _frames(2)
	_touch_move(ov, epos)
	await _frames(2)
	_ok("D mid-drag: still no drag emitted to the armed tool (pin claimed the press)",
		ov.has_sample_pin(), true)
	_touch_release(ov, epos)
	await _frames(6)
	_ok("D pin dropped onto the empty cell", ov.has_sample_pin(), true)
	var pin: Dictionary = ov.sample_pin_grid()
	print("D pin grid after drag=%s" % [pin])
	_ok("D the sheet re-resolved: header is now 'Here' (empty ground)", _menu().peek_title_text(), "Here")
	_ok("D the sheet stayed at whichever detent it already had (peek)", _menu().peek_detent(), "peek")
	var e_chips: PackedStringArray = _menu().peek_chip_labels()
	print("D empty-ground chips after drag=%s" % [e_chips])
	_ok("D empty ground still shows at least one chip (global Go/Measure rows)",
		e_chips.size() > 0, true)

	# -- E: the peek sheet's own header/chips for empty ground, from a fresh hold
	_menu().go_back()  ## scrim tap -> `_on_scrim_input()` -> `go_back()`
	await _frames(6)
	_ok("scrim-tap cleanup before leg E: sheet closed", _menu().peek_card_is_open(), false)
	_ok("scrim-tap cleanup before leg E: pin cleared", ov.has_sample_pin(), false)

	await _hold_wait(ov, epos, 0.85)
	_touch_release(ov, epos)
	await _frames(4)
	_ok("E empty-ground hold: the pin dropped", ov.has_sample_pin(), true)
	_ok("E empty-ground header reads 'Here'", _menu().peek_title_text(), "Here")

	# -- R: both the peek panel's and (at half) the card's rects stay in-screen --
	var scale: float = app.phone_scale()
	var peek_rect: Rect2 = _menu().peek_panel_rect()
	print("R peek panel rect px=%s dp=%s viewport_px=%s" %
		[peek_rect, Rect2(peek_rect.position / scale, peek_rect.size / scale), vis.size])
	_ok("R peek panel stays inside the viewport", vis.grow(1.0).encloses(peek_rect), true)

	await _grab_drag(_menu(), -900.0)
	var half_rect: Rect2 = _menu().peek_panel_rect()
	print("R half card rect px=%s dp=%s viewport_px=%s" %
		[half_rect, Rect2(half_rect.position / scale, half_rect.size / scale), vis.size])
	_ok("R half card stays inside the viewport", vis.grow(1.0).encloses(half_rect), true)
	_ok("R half card is taller than peek", half_rect.size.y > peek_rect.size.y, true)

	# -- X: a scrim tap dismisses the sheet and clears the pin -------------------
	_menu().go_back()
	await _frames(6)
	_ok("X scrim tap: the sheet closed", _menu().peek_card_is_open(), false)
	_ok("X scrim tap: the sample pin was cleared", ov.has_sample_pin(), false)


func _ready() -> void:
	var vp_size := Vector2i(1080, 2340)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a == "--vp" and i + 1 < args.size():
			var wh := args[i + 1].split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
			continue
		if a == "--force-touch":
			i += 1
			continue
		print("### CTXPHONE ABORT: unknown argument '%s' ###" % a)
		get_tree().quit(2)
		return
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)

	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.0).timeout

	var bridge = app.bridge
	bridge.generate({
		"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)

	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("FORM booted as %s at %s" % [form, _vp.size])
	if form != "phone":
		print("### CTXPHONE ABORT: CM-5 needs the phone form -- booted as %s. Use --force-touch and a phone-sized --vp (e.g. 1080x2340) ###" % form)
		get_tree().quit(2)
		return
	if _menu() == null:
		print("### CTXPHONE ABORT: no PhoneMenu on this build ###")
		get_tree().quit(2)
		return
	await _run()
	print("### CTXPHONE %s  %d/%d checks passed ###" % [
		"GREEN" if _fails == 0 else "RED", _checks - _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
