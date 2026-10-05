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
##
## CM-5 residuals (`OUTSTANDING_WORK.md`'s "CM-5 residuals" row, left by
## `ba01574`): the three legs below.
##
##   SEL  a stacked multi-hit spot (a settlement + a label + an icon on the
##        same cell) draws §8.1.4's "Select ▸ N" chip; opening it forces the
##        `half` detent and shows one row per hit; picking one re-resolves the
##        sheet to that object and the select list closes
##   PW   WORLD, a live sculpt draft: the phone's shown rows are compared
##        against `context_broker.gd::collect()`'s own live merge for the SAME
##        request, opened in a real `ContextCard` instance (`_card_row_labels`)
##        rather than read off a constant -- every row missing from the phone
##        is checked to carry `children` or `param`, its own stated touch
##        reason
##   PC   CARTOGRAPHY, the same stacked settlement/label/icon cell: the same
##        comparison, for the label and icon rows this residual named
##   STL   a settlement tap (a short press, not a hold) opens the phone's
##        right sheet (`right_dock.gd::_show_on_phone()`), the same way a
##        landmark or icon tap already did (`19d3ba8`)
##   FR    CM-2 follow-ups, the CIVIL per-cell faction rows (`sample_cell`'s
##         `controlling_faction`): "Open <faction>…" / "Claim for
##         <faction>" appear on a long-press of a CLAIMED land cell and not on
##         ocean or unclaimed land (the latter on a regenerated island world);
##         the Open chip lands the roster on that
##         faction (see `_leg_faction`)
##   PP    CM-2 follow-ups item (2) (`OUTSTANDING_WORK.md`): the phone peek
##        sheet pauses timeline playback while open and resumes it on close,
##        read off the live `CivilizationWorkspace` `Timer.paused` state --
##        the phone leg the residual row named as missing, matching
##        `_ctxring_probe.gd`'s own leg T for the desktop ring/card

const SEED := 552017

## FR's second world: SEED's claims every land cell (scratch scan of its whole
## grid), so the "unclaimed land has no faction rows" side regenerates with a
## seed that leaves land unreachable by `road_dijkstra` unowned
## (`cartalith_civ::assign_territory`). Measured at 256x192, sea level 0.42:
## seed 2 leaves 206 of 12288 sampled cells unclaimed land. A labelled
## judgement; `_leg_faction` asserts the fixture exists so a generator change
## fails loudly.
const ISLAND_SEED := 2

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


## Diagnostic only, gated on an env var so a normal CI run does not write
## files: `CTXPHONE_SHOT_DIR` set -> saves the live viewport as a PNG there.
func _shot(tag: String) -> void:
	var dir := OS.get_environment("CTXPHONE_SHOT_DIR")
	if dir == "":
		return
	var img: Image = _vp.get_texture().get_image()
	img.save_png(dir.path_join("ctxphone_%s.png" % tag))


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


## CM-5 residual (`PW`/`PC`): the labels a real `ContextCard` would draw for
## `req`/`actions` -- `_build_main()`'s own loop draws every action regardless
## of `children`/`param` (verified by reading it: `_row_for()` calls
## `_param_row()`/`_disabled_row()`/the plain action branch for every entry
## that matches the empty filter, and all three set the SAME `card_row` meta
## key `drawn_rows()` reads), so this opens one for real rather than asserting
## that fact against itself. Read as evidence for the desktop/tablet side of a
## parity check, never as the phone side -- the phone's own live
## `_menu().peek_full_row_labels()` is that.
func _card_row_labels(req: Dictionary, actions: Array) -> PackedStringArray:
	var card = load("res://shell/context_card.gd").new()
	card.setup(app)
	app.add_child(card)
	card.open(req, actions, Vector2(80, 80), Callable())
	var out := PackedStringArray()
	for r in card.drawn_rows():
		if String(r.get("kind", "")) in ["action", "disabled", "param"]:
			out.append(String(r.get("text", "")))
	card.hide()
	card.queue_free()
	return out


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

	# -- SEL: a stacked multi-hit spot shows Select ▸; picking re-resolves -------
	## Fixture: a label and an icon dropped on the settlement's own cell, the
	## same recipe `_ctxcard_probe.gd`'s own "S" leg uses for the desktop card.
	app.select_domain("cartography")
	await _frames(2)
	var bridge = app.bridge
	var s_k: Dictionary = bridge.settlements()[sk]
	var lbl_id: int = bridge.label_create(float(s_k["x"]) + 0.5, float(s_k["y"]) + 0.5, "CTXPHONE FIXTURE")
	var pack_path: String = ProjectSettings.globalize_path("res://") \
		+ "../crates/cartalith-assets/tests/fixtures/reference_pack.zip"
	var pack_ok: bool = bridge.world_gen.load_asset_pack(pack_path)
	var icon_id := -1
	if pack_ok and bridge.icon_arm("feature", 0, 1.0, 0.0, 0.0):
		icon_id = bridge.icon_place(float(s_k["x"]) + 0.5, float(s_k["y"]) + 0.5)
	bridge.icon_disarm()
	_ok("SEL fixture: a label and an icon land on the settlement", lbl_id >= 0 and icon_id >= 0, true)
	app.viewport.refresh_annotations()
	await _frames(4)
	var stack_hits: Array = ov.request_at(spos, "touch").get("hits", [])
	print("SEL stack hits=%s" % [stack_hits])
	_ok("SEL the stack really is 3+ hits (settlement, label, icon)", stack_hits.size() >= 3, true)

	await _hold_wait(ov, spos, 0.85)
	_touch_release(ov, spos)
	await _frames(6)
	_ok("SEL a fresh drop over the stack opens the sheet", _menu().peek_card_is_open(), true)
	var sel_chips: PackedStringArray = _menu().peek_chip_labels()
	print("SEL chips=%s" % [sel_chips])
	_ok("SEL the FIRST chip is Select ▸ N (§8.1.4)",
		sel_chips.size() > 0 and String(sel_chips[0]) == "Select ▸ %d" % stack_hits.size(), true)
	_ok("SEL not yet in the select list before it is opened", _menu().peek_in_select(), false)
	_shot("sel_peek_chip")

	var chip_row: HBoxContainer = _menu().get("_peek_chip_row")
	var sel_btn: Button = null
	for c in chip_row.get_children():
		if c is Button and String((c as Button).text).begins_with("Select ▸"):
			sel_btn = c
			break
	_ok("SEL found the Select chip's own node", sel_btn != null, true)
	if sel_btn != null:
		sel_btn.pressed.emit()
	await _frames(4)
	_ok("SEL opening the chip forces the half detent (no room for a list at peek)",
		_menu().peek_detent(), "half")
	_ok("SEL the sheet is now showing §8.1.4's hit list, not the action rows",
		_menu().peek_in_select(), true)
	_shot("sel_half_list")
	var sel_rows: PackedStringArray = _menu().peek_full_row_labels()
	print("SEL rows=%s" % [sel_rows])
	_ok("SEL one row per hit, plus the ‹ Back row", sel_rows.size(), stack_hits.size() + 1)

	var full_col: VBoxContainer = _menu().get("_peek_full_col")
	var label_btn: Button = null
	for wrap in full_col.get_children():
		if wrap is VBoxContainer:
			for c in (wrap as VBoxContainer).get_children():
				if c is Button and String((c as Button).text).begins_with("CTXPHONE FIXTURE"):
					label_btn = c
	_ok("SEL found the label's own row in the list", label_btn != null, true)
	if label_btn != null:
		label_btn.pressed.emit()
	await _frames(6)
	_ok("SEL picking the label re-resolves the sheet -- its own name is the header now",
		_menu().peek_title_text(), "CTXPHONE FIXTURE")
	_ok("SEL the select list closed on re-resolve (a fresh `peek_card()` call)",
		_menu().peek_in_select(), false)
	var after_rows: PackedStringArray = _menu().peek_full_row_labels()
	print("SEL rows after pick=%s" % [after_rows])
	var has_label_edit := false
	for r in after_rows:
		if String(r).begins_with("Edit text of “CTXPHONE FIXTURE”"):
			has_label_edit = true
	_ok("SEL the resolved sheet now carries the LABEL's own rows (Edit text of...)", has_label_edit, true)

	_menu().go_back()
	await _frames(6)
	ov.clear_sample_pin()
	await _frames(2)

	# -- PW: WORLD, a live sculpt draft -- phone rows vs. the card's own merge ---
	app.select_domain("world")
	await _frames(4)
	app.arm_tool("sculpt")
	await _frames(3)
	bridge.sculpt_begin_stroke()
	bridge.sculpt_add_point(256.0, 192.0)
	bridge.sculpt_add_point(260.0, 195.0)
	bridge.sculpt_end_stroke()
	await _frames(2)
	_ok("PW fixture: a sculpt draft is live", bridge.sculpt_stamp_count() > 0, true)
	var draft_pos: Vector2 = ov._cell_to_screen(Vector2(256.0, 192.0), rect)
	await _hold_wait(ov, draft_pos, 0.85)
	_touch_release(ov, draft_pos)
	await _frames(6)
	_ok("PW the hold opened the phone sheet over the draft", _menu().peek_card_is_open(), true)
	var pw_req: Dictionary = app.context_broker.last_request
	var pw_actions: Array = app.context_broker.last_actions
	_ok("PW the broker's own live merge has rows for this request", pw_actions.size() > 0, true)
	var pw_desktop := _card_row_labels(pw_req, pw_actions)
	var pw_phone: PackedStringArray = _menu().peek_full_row_labels()
	print("PW desktop=%s" % [pw_desktop])
	print("PW phone=%s" % [pw_phone])
	var pw_missing: Array = []
	for l in pw_desktop:
		if not pw_phone.has(l):
			pw_missing.append(l)
	print("PW missing from phone=%s" % [pw_missing])
	var pw_reasoned := true
	for l in pw_missing:
		var found := false
		for a in pw_actions:
			if String((a as Dictionary).get("label", "")) == l \
					and ((a as Dictionary).has("children") or (a as Dictionary).has("param")):
				found = true
		if not found:
			pw_reasoned = false
	_ok("PW every row missing from the phone form has a stated touch reason " +
		"(children/param -- an inline stepper or a submenu the sheet has no row type for)",
		pw_reasoned, true)
	var pw_draft_rows := 0
	for l in pw_phone:
		if String(l).begins_with("Commit") or String(l).begins_with("Undo") or String(l).begins_with("Discard"):
			pw_draft_rows += 1
	_ok("PW the phone shows the Draft section's Commit/Undo/Discard (plain taps, not gated)",
		pw_draft_rows >= 2, true)
	_menu().go_back()
	await _frames(4)
	ov.clear_sample_pin()
	bridge.sculpt_discard()
	app.arm_tool("inspect")
	await _frames(2)

	# -- PC: CARTOGRAPHY, the same stacked cell -- label/icon rows vs. the card --
	app.select_domain("cartography")
	await _frames(2)
	await _hold_wait(ov, spos, 0.85)
	_touch_release(ov, spos)
	await _frames(6)
	_ok("PC the hold opened the phone sheet over the stack", _menu().peek_card_is_open(), true)
	var pc_req: Dictionary = app.context_broker.last_request
	var pc_actions: Array = app.context_broker.last_actions
	var pc_desktop := _card_row_labels(pc_req, pc_actions)
	var pc_phone: PackedStringArray
	## The stack's own Select ▸ chip is at `peek`; swap to the hit list, then
	## back out is not needed -- `peek_full_row_labels()` at `peek` still reads
	## whichever content is currently drawn, so swipe to `half` first to read
	## the CARD's own full row list rather than the chip row's four labels.
	await _grab_drag(_menu(), -900.0)
	pc_phone = _menu().peek_full_row_labels()
	print("PC desktop=%s" % [pc_desktop])
	print("PC phone=%s" % [pc_phone])
	var pc_missing: Array = []
	for l in pc_desktop:
		if not pc_phone.has(l):
			pc_missing.append(l)
	print("PC missing from phone=%s" % [pc_missing])
	var pc_reasoned := true
	for l in pc_missing:
		var found := false
		for a in pc_actions:
			if String((a as Dictionary).get("label", "")) == l \
					and ((a as Dictionary).has("children") or (a as Dictionary).has("param")):
				found = true
		if not found:
			pc_reasoned = false
	_ok("PC every row missing from the phone form has a stated touch reason " +
		"(View field ▸ / Style preset ▸ carry `children`, a submenu the sheet has no row for)",
		pc_reasoned, true)
	var has_edit_label := false
	var has_delete_icon := false
	for l in pc_phone:
		if String(l).begins_with("Edit text of"):
			has_edit_label = true
		if String(l).begins_with("Delete icon"):
			has_delete_icon = true
	_ok("PC the phone form carries CARTO's label row (Edit text of...)", has_edit_label, true)
	_ok("PC the phone form carries CARTO's icon row (Delete icon...)", has_delete_icon, true)
	_menu().go_back()
	await _frames(4)
	ov.clear_sample_pin()

	# -- STL: a settlement TAP (not a hold) opens the phone's right sheet -------
	app.right_dock.visible = false
	await _frames(2)
	_ok("STL precondition: the right dock sheet is closed", app.right_dock.visible, false)
	_touch_press(ov, spos)
	await _frames(2)
	_touch_release(ov, spos)
	await _frames(6)
	_ok("STL a settlement tap opens the phone's right sheet " +
		"(`right_dock.gd::_show_on_phone()`, same as a landmark/icon tap)",
		app.right_dock.visible, true)
	_shot("stl_right_sheet")
	_ok("STL the right dock's own context is Settlement",
		String(app.right_dock_ctrl.get("_context")), "settlement")

	# -- PP: the phone peek sheet pauses timeline playback while open -----------
	## CM-2 follow-ups, item (2) (`OUTSTANDING_WORK.md`): "the peek-sheet pause
	## is wired but has no probe leg." `phone_menu.gd::peek_card()` calls
	## `_shell.call("pause_for_context_surface")` on a genuinely fresh open
	## (guarded on `not was_open`, so a re-resolve of an already-open sheet
	## does not pause a second time) and `_dismiss_peek_card()` calls the
	## matching `resume_from_context_surface()` -- the SAME fan-out
	## `_ctxring_probe.gd`'s own leg T already exercises for the desktop ring
	## and card (see `1ca95c5`). Read entirely off the live
	## `CivilizationWorkspace` state (`_tl_playing`, the Timer's own `paused`/
	## `time_left`) rather than a re-declared expectation, matching leg T's
	## own discipline, so this fails first without the peek side of the
	## wiring rather than merely re-asserting that a constant equals itself.
	##
	## Protects: a future edit to `peek_card()`/`_dismiss_peek_card()` that
	## drops or double-fires the pause/resume call (the `was_open` guard is
	## exactly the kind of condition a refactor can invert) from silently
	## letting the phone's own timeline keep ticking under a modal sheet, or
	## from leaving it paused forever after the sheet closes.
	app.select_domain("civilization")
	await _frames(4)
	var pp_civ: Node = null
	for ws in app._workspaces:
		if ws.has_method("pause_playback_for_context"):
			pp_civ = ws
			break
	_ok("PP fixture: the timeline workspace exists", pp_civ != null, true)
	if pp_civ != null:
		bridge.civ_add_year(0)
		bridge.civ_add_year(200)
		pp_civ.call("_tl_start_play")
		await _frames(2)
		_ok("PP fixture: playback actually started", pp_civ.get("_tl_playing"), true)
		var pp_timer: Timer = pp_civ.get("_tl_play_timer")
		_ok("PP fixture: the play timer exists and is not paused",
			pp_timer != null and not pp_timer.paused, true)

		await _hold_wait(ov, epos, 0.85)
		_touch_release(ov, epos)
		await _frames(6)
		_ok("PP the hold opened the peek sheet", _menu().peek_card_is_open(), true)
		_ok("PP peek sheet open pauses the timer", pp_timer.paused, true)
		_ok("PP ...still reads as playing (held, not stopped)", pp_civ.get("_tl_playing"), true)
		var pp_held_left := pp_timer.time_left
		await get_tree().create_timer(0.3).timeout   ## Longer than the pause would tick if it were live.
		_ok("PP paused: time_left does not advance while the sheet is open",
			pp_timer.time_left, pp_held_left)

		_menu().go_back()   ## scrim tap -> `_on_scrim_input()` -> `go_back()`, same dismissal leg X uses
		await _frames(6)
		_ok("PP scrim-dismiss closed the sheet", _menu().peek_card_is_open(), false)
		_ok("PP sheet close resumes the timer", pp_timer.paused, false)
		_ok("PP ...playback state untouched by the pause/resume itself",
			pp_civ.get("_tl_playing"), true)
		pp_civ.call("_tl_stop_play")
		await _frames(2)
	ov.clear_sample_pin()
	app.select_domain("world")
	app.arm_tool("inspect")
	await _frames(2)

	await _leg_faction(ov)


## FR: the per-cell faction rows on the phone (`sample_cell`'s
## `controlling_faction`, `OUTSTANDING_WORK.md` "CM-2 follow-ups"). A long-press
## on a CLAIMED land cell puts "Open <faction>…" and "Claim for
## <faction>" into the half-detent row list (and, being OBJECT-section rows,
## ahead of the PLACE rows in the four-chip peek strip), a long-press on OCEAN or
## on UNCLAIMED land puts neither, and the Open chip lands the roster window on
## that faction -- parked on a different one first, so it cannot pass by the
## window merely still sitting there.
##
## Protects: the faction rows staying gated on the key on the PHONE form, where
## `phone_menu.gd` rebuilds the chips and rows from the broker's own merge and
## so would silently drop a row shape it did not know; and `open_faction_roster`
## reaching a phone-presented roster window (`faction_roster_window.gd`'s
## `phone_present` branch, which skips `popup_centered`).
func _leg_faction(ov: Control) -> void:
	app.select_domain("civilization")
	await _frames(4)
	var claimed := _find_cell(ov, "land", true)
	var ocean := _find_cell(ov, "ocean", false)
	_ok("FR fixture: a claimed land cell exists", not claimed.is_empty(), true)
	_ok("FR fixture: an ocean cell exists", not ocean.is_empty(), true)
	if claimed.is_empty():
		return
	var cell: Dictionary = claimed["cell"]
	var fid := int(cell["controlling_faction"])
	var fname := String(cell["controlling_faction_name"])
	var open_label := "Open %s…" % fname
	var claim_label := "Claim for %s" % fname
	var cpos: Vector2 = claimed["pos"]

	# Half-detent full list over the claimed cell.
	await _hold_wait(ov, cpos, 0.85)
	_touch_release(ov, cpos)
	await _frames(6)
	_ok("FR claimed: the long-press opened the sheet", _menu().peek_card_is_open(), true)
	var chips: PackedStringArray = _menu().peek_chip_labels()
	print("FR claimed chips=%s" % [chips])
	_shot("fr_peek_claimed")
	await _grab_drag(_menu(), -900.0)
	var rows: PackedStringArray = _menu().peek_full_row_labels()
	print("FR claimed rows=%s" % [rows])
	_shot("fr_half_claimed")
	_ok("FR claimed: the full list carries 'Open <faction>…'", rows.has(open_label), true)
	_ok("FR claimed: the full list carries 'Claim for <faction>'", rows.has(claim_label), true)
	_menu().go_back()
	await _frames(6)
	ov.clear_sample_pin()

	# The Open chip lands the roster on that faction.
	var other := 2 if fid != 2 else 1
	app.faction_roster_window.open(other)
	await _frames(3)
	app.faction_roster_window.hide()
	await _frames(2)
	_ok("FR precondition: the roster window is parked on another faction",
		int(app.faction_roster_window.get("_selected")), other)
	await _hold_wait(ov, cpos, 0.85)
	_touch_release(ov, cpos)
	await _frames(6)
	var open_btn: Button = null
	for c in (_menu().get("_peek_chip_row") as HBoxContainer).get_children():
		if c is Button and String((c as Button).text) == open_label:
			open_btn = c
			break
	_ok("FR claimed: the Open row is one of the peek chips (an OBJECT row, first four)", open_btn != null, true)
	if open_btn != null:
		open_btn.pressed.emit()
		await _frames(8)
		_ok("FR the Open chip closed the sheet", _menu().peek_card_is_open(), false)
		_ok("FR the roster window landed on THAT faction",
			int(app.faction_roster_window.get("_selected")), fid)
		## FH-1: the chip opens the hub on the Territory tab (the row is a
		## *where* row); on the phone that tab must be the live one, too.
		_ok("FR ... on the Territory tab (FH-1)", String(app.faction_roster_window.get("_tab")), "territory")
		_ok("FR ... showing its inspector, not the master list (phone: the pick IS the navigation)",
			(app.faction_roster_window.get("_phone_list_pane") as Control).visible, false)
		_shot("fr_roster")
		app.faction_roster_window.hide()
		await _frames(2)
	else:
		_menu().go_back()
		await _frames(4)
	ov.clear_sample_pin()
	await _frames(2)

	# The absent cases: the half list over ocean, then over unclaimed land.
	await _leg_faction_absent(ov, "ocean", ocean)

	# Unclaimed land needs a world that has some (SEED's claims every land
	# cell); FR runs last in this probe, so replacing the world is safe. The
	# control is a claimed cell of the new world still showing the rows.
	app.select_domain("civilization")
	await _regen_island_world()
	var u_ov: Control = app.viewport.overlay
	var u_unclaimed := _find_cell(u_ov, "land", false, 8.0, 4.0)
	var u_claimed := _find_cell(u_ov, "land", true)
	_ok("FR fixture: the island world has an unclaimed land cell", not u_unclaimed.is_empty(), true)
	_ok("FR fixture: ... and a claimed one beside it (the control)", not u_claimed.is_empty(), true)
	if not u_claimed.is_empty():
		var cp: Vector2 = u_claimed["pos"]
		await _hold_wait(u_ov, cp, 0.85)
		_touch_release(u_ov, cp)
		await _frames(6)
		var cc: PackedStringArray = _menu().peek_chip_labels()
		_ok("FR island control: a claimed cell of the new world shows the faction rows",
			_labels_have_faction_row(cc), true)
		_menu().go_back()
		await _frames(6)
		u_ov.clear_sample_pin()
		await _frames(2)
	await _leg_faction_absent(u_ov, "unclaimed land", u_unclaimed)
	app.select_domain("world")
	await _frames(2)


## FR helper: a long-press at `d["pos"]` (a `_find_cell` result, skipped when
## empty) must open a sheet whose chips and half-detent list carry neither
## faction row, while still listing other rows (the control that the sheet is
## real and the absence is the gate's doing, not an empty list).
func _leg_faction_absent(ov: Control, what: String, d: Dictionary) -> void:
	if d.is_empty():
		return
	var p: Vector2 = d["pos"]
	await _hold_wait(ov, p, 0.85)
	_touch_release(ov, p)
	await _frames(6)
	_ok("FR %s: the long-press opened the sheet (the control: a sheet exists)" % what,
		_menu().peek_card_is_open(), true)
	var pc: PackedStringArray = _menu().peek_chip_labels()
	await _grab_drag(_menu(), -900.0)
	var pr: PackedStringArray = _menu().peek_full_row_labels()
	print("FR %s chips=%s rows=%s" % [what, pc, pr])
	_shot("fr_half_%s" % what.replace(" ", "_"))
	_ok("FR %s: neither faction row, in chips or list" % what,
		_labels_have_faction_row(pc) or _labels_have_faction_row(pr), false)
	_ok("FR %s: the list is not empty (the control: other rows are there)" % what, pr.size() > 0, true)
	_menu().go_back()
	await _frames(6)
	ov.clear_sample_pin()
	await _frames(2)


## Whether any label is "Open <faction>…" or "Claim for <faction>".
func _labels_have_faction_row(labels: PackedStringArray) -> bool:
	for l in labels:
		var s := String(l)
		if (s.begins_with("Open ") and s.ends_with("…")) or s.begins_with("Claim for "):
			return true
	return false


## Replaces the loaded world with `ISLAND_SEED`'s (256x192, the same aspect as
## SEED's 512x384, so the overlay's displayed rect keeps its shape) and waits
## for the generate to land.
func _regen_island_world() -> void:
	var bridge = app.bridge
	bridge.generate({
		"seed": ISLAND_SEED, "width_km": 1200.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)


## FR: a screen position over a cell with water word `water` whose
## `sample_cell` does (`claimed`) or does not carry `controlling_faction`, as
## `{pos, cell}`, or `{}`. 9 px lattice over the map interior, skipping any
## spot the overlay treats as an object hit so the sheet is the bare-ground one.
## `inset` (default 60 px) and `step` (default 9 px) bound the lattice; the
## island sub-leg tightens both because its unclaimed land is a small island.
func _find_cell(ov: Control, water: String, claimed: bool, inset: float = 60.0, step: float = 9.0) -> Dictionary:
	var rect: Rect2 = ov._displayed_rect()
	var inter: Rect2 = ov._interior_rect(rect)
	var core := inter.grow(-inset)
	var y := core.position.y
	while y < core.end.y:
		var x := core.position.x
		while x < core.end.x:
			var c := Vector2(x, y)
			x += step
			if ov._hit_test_settlement(c, inter, rect) != -1 or not ov.hits_at(c).is_empty():
				continue
			var g: Dictionary = ov._grid_point(c, rect, inter)
			var cell: Dictionary = app.bridge.sample_cell(int(g["gx"]), int(g["gy"]))
			if String(cell.get("water", "")) == water and cell.has("controlling_faction") == claimed:
				return {"pos": c, "cell": cell}
		y += step
	return {}


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
