extends Node
## CM-2 (`MAP_CONTEXT_SCOPE.md` §11): the context card (`shell/context_card.gd`)
## that replaced CX-01's `PopupMenu` on desktop and tablet.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _ctxcard_probe.tscn
##   ... _ctxcard_probe.tscn -- --vp 2560x1600 --force-touch     (tablet)
##
## `--vp WxH` sizes the host SubViewport (default 1600x900); any other
## argument aborts the run (exit 2), as does a boot that came up as a phone.
##
## **Windowed, not `--headless`**: leg P reads pixels, and the card is an
## embedded `Window` whose rect is part of what is asserted. Leg P FORCES each
## palette (after the shell boots -- `apply_theme()` before boot is inert) and
## asserts the card's ground differs between the two before trusting either.
##
## Every row is clicked and every key is pushed through the SubViewport
## (`push_input`), the path a real pointer takes into an embedded popup --
## never by calling the card's own handlers.
##
## Legs:
##   R  RMB: nothing on press; the card on release; nothing after a drag of
##      >= 8 px (even one that comes back to where it started)
##   A  CIVIL, settlement: CX-01's five rows, same text, order, enabled state
##      and sections as before CM-2 (the `ROW` lines are the comparison)
##   B  CIVIL, empty cell: Drop settlement here · Info here
##   C  CIVIL, PH-02 touch hold: A's rows, and the withheld press placed nothing
##   X  each of the five CIVIL rows, clicked, does what CX-01's did
##   S  Select ▸ over a settlement + label + icon lists all three; picking the
##      label re-resolves for the label alone
##   D  a disabled row draws its reason, inside the card
##   K  the keyboard: Down moves, Enter runs, typing filters, Backspace
##      unfilters, Esc closes
##   W  three of CM-2's new rows run what they name: Measure from here arms
##      Measure with one point; Copy coordinate ▸ opens in place and copies;
##      CARTO's View field ▸ lists eight views and draws the one picked
##   T  an inline Tool row: WORLD with Biome paint armed, Radius `+` steps the
##      brush and the value on the row follows
##   E  the card stays on screen at the four edges (its rect is measured)
##   P  contrast of the card's text against the card's own ground, both
##      palettes, with a ground-only crop as the control that must read ~1:1

const SEED := 483920

var app: Node
var _vp: SubViewport
var _fails := 0
var _checks := 0
var _k := -1
var _sname := ""


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, got: Variant, want: Variant) -> void:
	_checks += 1
	var pass_: bool = typeof(got) == typeof(want) and got == want
	if not pass_:
		_fails += 1
	print("%s  %s  got=%s want=%s" % ["PASS" if pass_ else "FAIL", what, str(got), str(want)])


func _card():
	var b = app.get("context_broker")
	return b.card if b != null else null


func _open() -> bool:
	var c = _card()
	return c != null and c.visible


func _close() -> void:
	var c = _card()
	if c != null and c.visible:
		c.hide()
	await _frames(2)


## Kinds: action/disabled/param rows, in drawn order, as text.
func _texts(kinds: Array = ["action", "disabled", "param"]) -> Array:
	var out: Array = []
	var c = _card()
	if c == null:
		return out
	for r in c.drawn_rows():
		if kinds.has(String(r["kind"])):
			out.append(String(r["text"]))
	return out


func _row(prefix: String, kinds: Array = ["action", "select", "back"]) -> Control:
	var c = _card()
	if c == null:
		return null
	for r in c.drawn_rows():
		if kinds.has(String(r["kind"])) and String(r["text"]).begins_with(prefix):
			return r["node"]
	return null


func _dump(tag: String) -> void:
	var c = _card()
	if c == null or not c.visible:
		print("ROW %s card=closed" % tag)
		return
	print("ROW %s card rect=%s" % [tag, Rect2(c.position, c.size)])
	for r in c.drawn_rows():
		var extra := ""
		if r.has("reason"):
			extra = "  {reason: %s}" % r["reason"]
		if r.has("value"):
			extra = "  {value: %s}" % r["value"]
		if r.has("subtitle"):
			extra = "  {subtitle: %s}" % r["subtitle"]
		print("ROW %s %s | %s%s" % [tag, r["kind"], r["text"], extra])


## A node inside the card, in the SubViewport's space.
func _in_vp(n: Control) -> Rect2:
	var c = _card()
	var r := n.get_global_rect()
	return Rect2(r.position + Vector2(c.position), r.size)


func _click_at(p: Vector2) -> void:
	var mv := InputEventMouseMotion.new()
	mv.position = p
	mv.global_position = p
	_vp.push_input(mv)
	await _frames(1)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = p
		ev.global_position = p
		_vp.push_input(ev)
		await _frames(1)
	await _frames(3)


func _click_row(n: Control) -> void:
	await _click_at(_in_vp(n).get_center())


func _key(code: int, uni: int = 0) -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.unicode = uni
		ev.pressed = down
		_vp.push_input(ev)
		await _frames(1)
	await _frames(2)


func _rmb_press(ov: Control, pos: Vector2) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_RIGHT
	ev.pressed = true
	ev.position = pos
	ov._gui_input(ev)


func _rmb_release(ov: Control, pos: Vector2) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_RIGHT
	ev.pressed = false
	ev.position = pos
	ov._gui_input(ev)


func _rmb(ov: Control, pos: Vector2) -> void:
	_rmb_press(ov, pos)
	_rmb_release(ov, pos)
	await _frames(4)


func _pos_of(ov: Control, k: int) -> Vector2:
	var rect: Rect2 = ov._displayed_rect()
	var s: Dictionary = ov.get("_settlements")[k]
	return ov._cell_to_screen(Vector2(s["x"], s["y"]), rect)


## An empty LAND cell near the settlement (Drop settlement here refuses water).
## `free`: also a cell no settlement's own grid radius claims
## (`civ_pick_place_at` -1) -- measured on the tablet run: a drop inside one
## answers with that settlement's index and adds nothing, card or no card.
func _empty_pos(ov: Control, free: bool = false) -> Vector2:
	var rect: Rect2 = ov._displayed_rect()
	var inter: Rect2 = ov._interior_rect(rect)
	var base := _pos_of(ov, _k)
	for tries in 600:
		var ang := tries * 0.61
		var rad := 40.0 + tries * 0.9
		var c := base + Vector2(cos(ang), sin(ang)) * rad
		if not inter.grow(-60.0).has_point(c) or ov._hit_test_settlement(c, inter, rect) != -1:
			continue
		var g: Dictionary = ov._grid_point(c, rect, inter)
		var cell: Dictionary = app.bridge.sample_cell(int(g["gx"]), int(g["gy"]))
		if String(cell.get("water", "")) == "land" and ov.hits_at(c).is_empty():
			if free and app.bridge.civ_pick_place_at(float(g["gx"]), float(g["gy"])) != -1:
				continue
			return c
	return Vector2.ZERO


## The five CIVIL rows as the card drew them: `text` and which band they sit
## under, and whether they are enabled (drawn as `action`).
func _civ_rows() -> Array:
	var out: Array = []
	var band := ""
	var c = _card()
	if c == null:
		return out
	for r in c.drawn_rows():
		if r["kind"] == "band":
			band = String(r["text"])
		if String(r.get("id", "")).begins_with("civ."):
			out.append("%s [%s]%s" % [r["text"], band, "" if r["kind"] == "action" else " {" + String(r["kind"]) + "}"])
	return out


func _run() -> void:
	var bridge = app.bridge
	var ov: Control = app.viewport.overlay
	var rect: Rect2 = ov._displayed_rect()
	var inter: Rect2 = ov._interior_rect(rect)
	_ok("MODE the broker has a card presenter", app.get("context_broker") != null, true)

	var sets: Array = ov.get("_settlements")
	for i in sets.size():
		var p: Vector2 = ov._cell_to_screen(Vector2(sets[i]["x"], sets[i]["y"]), rect)
		if inter.grow(-80.0).has_point(p) and ov._hit_test_settlement(p, inter, rect) == i:
			_k = i
			break
	_ok("a settlement to click exists", _k >= 0, true)
	if _k < 0:
		return
	_sname = String(bridge.settlements()[_k].get("name", "(unnamed)"))
	print("FIXTURE settlement k=%d name=%s" % [_k, _sname])

	# -- R: on release, not on press ---------------------------------------------
	app.select_domain("civilization")
	await _frames(4)
	var sp := _pos_of(ov, _k)
	_rmb_press(ov, sp)
	await _frames(4)
	_ok("R press alone opens nothing", _open(), false)
	_rmb_release(ov, sp)
	await _frames(4)
	_ok("R the release opens the card", _open(), true)
	await _close()
	_rmb_press(ov, sp)
	var mv := InputEventMouseMotion.new()
	mv.position = sp + Vector2(20, 0)
	mv.button_mask = MOUSE_BUTTON_MASK_RIGHT
	ov._gui_input(mv)
	_rmb_release(ov, sp + Vector2(20, 0))
	await _frames(4)
	_ok("R a 20 px drag opens nothing (the ring's gesture, CM-3)", _open(), false)
	_rmb_press(ov, sp)
	ov._gui_input(mv)
	_rmb_release(ov, sp)
	await _frames(4)
	_ok("R a drag that returns to its start is still a drag", _open(), false)
	_rmb_press(ov, sp)
	var small := mv.duplicate()
	small.position = sp + Vector2(5, 0)
	ov._gui_input(small)
	_rmb_release(ov, sp + Vector2(5, 0))
	await _frames(4)
	_ok("R 5 px of wobble is still a click", _open(), true)
	await _close()

	# -- A ------------------------------------------------------------------------
	await _rmb(ov, _pos_of(ov, _k))
	_dump("A_civ_settlement")
	## CX-01's original five rows, plus CM-7's CM-2-residual additions
	## (`MAP_CONTEXT_SCOPE.md` §11 CM-7): Open city layout… (object, right
	## after Move viewer to) and Start way/route here (place, right after
	## Drop settlement here). CX-01's own five are unchanged in text, order
	## and section among themselves -- this list only has three more rows
	## interleaved, which is this milestone's own regression test for CM-1/
	## CM-2, not a new one.
	var want_a := [
		"Edit %s [OBJECT]" % _sname, "Move viewer to %s [OBJECT]" % _sname,
		"Open city layout for %s… [OBJECT]" % _sname,
		"Delete %s [OBJECT]" % _sname, "Drop settlement here [PLACE HERE]",
		"Start way here [PLACE HERE]", "Start route here [PLACE HERE]",
		"Info here (settlement & ecology) [INFO]"]
	var rows_a := _civ_rows()
	_ok("A CX-01's five rows + CM-7's three: text, order, section, enabled", rows_a, want_a)
	var titles: Array = _card().drawn_rows().filter(func(r): return r["kind"] == "header")
	var cls := String(bridge.settlements()[_k].get("kind", ""))
	_ok("A the header names the settlement and its class",
		String(titles[0]["text"]) if not titles.is_empty() else "", "%s · %s" % [_sname, cls])
	_ok("A the header's second line is a readout (has 'cell')",
		String(titles[0].get("subtitle", "")).contains("cell ") if not titles.is_empty() else false, true)
	## Row pitch by form: the canvases' ROWH, 28 (Main.dc.html) and 44
	## (Tablet.dc.html). Measured on the laid row, both axes against the card.
	var tablet := DccTheme.is_tablet()
	var edit_row := _row("Edit ")
	var want_h := 44.0 if tablet else 28.0
	print("FORM %s  row=%s  panel=%s" % ["tablet" if tablet else "desktop",
		edit_row.size if edit_row else Vector2.ZERO, _card().panel_rect()])
	_ok("A a row is %d px tall (%s)" % [int(want_h), "tablet" if tablet else "desktop"],
		edit_row != null and edit_row.size.y >= want_h and edit_row.size.y <= want_h + 1.0, true)
	_ok("A a row spans the card's width", edit_row != null and edit_row.size.x >= _card().panel_rect().size.x - 3.0, true)
	var danger_after_rule := false
	var prev := {}
	for r in _card().drawn_rows():
		if String(r.get("id", "")) == "civ.delete":
			danger_after_rule = prev.get("kind", "") == "rule"
		prev = r
	_ok("A Delete sits behind a rule (danger, last in its section)", danger_after_rule, true)
	await _close()

	# -- B ------------------------------------------------------------------------
	var epos := _empty_pos(ov)
	_ok("B an empty land cell exists", epos != Vector2.ZERO, true)
	await _rmb(ov, epos)
	_dump("B_civ_empty")
	_ok("B CIVIL on an empty cell", _civ_rows(),
		["Drop settlement here [PLACE HERE]", "Start way here [PLACE HERE]",
			"Start route here [PLACE HERE]", "Info here (settlement & ecology) [INFO]"])
	var hb: Array = _card().drawn_rows().filter(func(r): return r["kind"] == "header")
	_ok("B a bare cell's header is 'Here'", String(hb[0]["text"]) if not hb.is_empty() else "", "Here")
	await _close()

	# -- C ------------------------------------------------------------------------
	app.arm_tool("settlement")
	await _frames(2)
	var n_before: int = bridge.settlements().size()
	var hold := InputEventMouseButton.new()
	hold.button_index = MOUSE_BUTTON_LEFT
	hold.pressed = true
	hold.position = _pos_of(ov, _k)
	hold.device = InputEvent.DEVICE_ID_EMULATION
	ov._gui_input(hold)
	await get_tree().create_timer(0.8).timeout
	await _frames(2)
	_dump("C_civ_touch_hold")
	## Settlement is armed for this leg (to reach PH-02's touch-hold path on
	## the settlement tool below), so CM-7's settlement-class Tool param row
	## (`MAP_CONTEXT_SCOPE.md` §11 CM-7's CM-2-residual addition) is also on
	## screen here, ahead of `want_a`'s own rows -- A itself runs with no
	## tool armed and does not see it.
	var want_c: Array = ["Class [TOOL · SETTLEMENT] {param}"] + want_a
	_ok("C the hold opened A's rows", _civ_rows(), want_c)
	var lift := hold.duplicate()
	lift.pressed = false
	ov._gui_input(lift)
	await _frames(2)
	_ok("C the withheld press placed nothing", bridge.settlements().size(), n_before)
	await _close()
	app.arm_tool("inspect")
	await _frames(2)

	# -- X: each row runs its action -----------------------------------------------
	## Move viewer to
	var cam: Control = app.viewport.get("_camera")
	await _rmb(ov, _pos_of(ov, _k))
	cam.position = Vector2(-12345, -6789)
	var row := _row("Move viewer to")
	_ok("X Move viewer to: row found", row != null, true)
	if row != null:
		await _click_row(row)
	var got_cam: Vector2 = cam.position
	_ok("X the click closed the card", _open(), false)
	var s_k: Dictionary = bridge.settlements()[_k]
	cam.position = Vector2(-1, -1)
	app.viewport.move_view_to(float(int(s_k.get("x", 0))), float(int(s_k.get("y", 0))))
	_ok("X Move viewer to lands where move_view_to puts it", got_cam.is_equal_approx(cam.position), true)
	await _frames(4)
	## Edit
	await _rmb(ov, _pos_of(ov, _k))
	row = _row("Edit ")
	if row != null:
		await _click_row(row)
	await _frames(4)
	var pew: Window = app.place_editor_window
	_ok("X Edit opens the place editor", pew != null and pew.visible, true)
	if pew != null:
		pew.hide()
	await _frames(4)
	## Delete -> the confirmation, which is cancelled
	await _rmb(ov, _pos_of(ov, _k))
	row = _row("Delete ")
	if row != null:
		await _click_row(row)
	await _frames(4)
	var conf := _find_visible_dialog(app, "Delete place?")
	_ok("X Delete asks first (the existing confirmation)", conf != null, true)
	if conf != null:
		conf.hide()
	_ok("X nothing was deleted", bridge.settlements().size(), n_before)
	await _frames(2)
	## Info here
	await _rmb(ov, _pos_of(ov, _k))
	row = _row("Info here")
	if row != null:
		await _click_row(row)
	await _frames(2)
	var hint: Label = app.get("_status_labels").get("hint")
	_ok("X Info here pins the settlement (status says so)",
		hint != null and hint.text.begins_with("Pinned in the right dock"), true)
	## Drop settlement here (on the empty land cell)
	epos = _empty_pos(ov, true)
	_ok("X a cell no settlement claims exists", epos != Vector2.ZERO, true)
	var n_drop: int = bridge.settlements().size()
	print("X drop: count before=%d (n_before=%d)" % [n_drop, n_before])
	await _rmb(ov, epos)
	row = _row("Drop settlement here")
	if row != null:
		await _click_row(row)
	await _frames(6)
	print("X drop: row=%s card_open=%s status='%s' count=%d" % [row != null, _open(), hint.text if hint else "", bridge.settlements().size()])
	_ok("X Drop settlement here placed one", bridge.settlements().size(), n_drop + 1)
	await _close()

	# -- S: Select ▸ over a stack ---------------------------------------------------
	s_k = bridge.settlements()[_k]
	var li: int = bridge.label_create(float(s_k["x"]) + 0.5, float(s_k["y"]) + 0.5, "CTX FIXTURE")
	var pack_path: String = ProjectSettings.globalize_path("res://") \
		+ "../crates/cartalith-assets/tests/fixtures/reference_pack.zip"
	var pack_ok: bool = bridge.world_gen.load_asset_pack(pack_path)
	var ii := -1
	if pack_ok and bridge.icon_arm("feature", 0, 1.0, 0.0, 0.0):
		ii = bridge.icon_place(float(s_k["x"]) + 0.5, float(s_k["y"]) + 0.5)
	bridge.icon_disarm()
	_ok("S fixture: a label and an icon on the settlement", li >= 0 and ii >= 0, true)
	app.viewport.refresh_annotations()
	app.select_domain("cartography")
	await _frames(6)
	var hits: Array = ov.request_at(_pos_of(ov, _k), "mouse").get("hits", [])
	print("S hits=%s" % str(hits))
	await _rmb(ov, _pos_of(ov, _k))
	_dump("S_carto_stack")
	var sel := _row("Select — ", ["select"])
	_ok("S a Select row counts every hit", _texts(["select"]), ["Select — %d objects here" % hits.size()])
	if sel != null:
		await _click_row(sel)
	_dump("S_select_list")
	var listed := _texts(["action"])
	var want_list: Array = []
	for h in hits:
		want_list.append(String(h.get("label", "(unnamed)")))
	var all_there := hits.size() >= 3
	for w in want_list:
		var hit_one := false
		for t in listed:
			if String(t).begins_with(w + " · "):
				hit_one = true
		all_there = all_there and hit_one
	_ok("S the list names every overlapping hit (settlement, label, icon)", all_there, true)
	_ok("S the list has one row per hit", listed.size(), hits.size())
	var pick := _row("CTX FIXTURE · label", ["action"])
	if pick != null:
		await _click_row(pick)
	_dump("S_after_pick")
	var after := _texts(["action", "disabled"])
	_ok("S picking the label re-resolves for it (Edit text row)", after.has("Edit text of “CTX FIXTURE”…"), true)
	_ok("S ... and not for the settlement any more", after.has("Settlement actions in CIVIL ›"), false)
	var h2: Array = _card().drawn_rows().filter(func(r): return r["kind"] == "header")
	_ok("S the header follows the pick", String(h2[0]["text"]) if not h2.is_empty() else "", "CTX FIXTURE · label")
	await _close()

	# -- D: a disabled row shows its reason ------------------------------------------
	app.arm_tool("inspect")
	await _frames(2)
	await _rmb(ov, _empty_pos(ov))
	_dump("D_carto_empty")
	var dis: Dictionary = {}
	for r in _card().drawn_rows():
		if r["kind"] == "disabled" and String(r.get("id", "")) == "carto.stamp_icon":
			dis = r
	_ok("D Stamp armed icon here is drawn disabled", not dis.is_empty(), true)
	if not dis.is_empty():
		var reason_node: Label = (dis["node"] as Control).find_child("Reason", true, false)
		_ok("D its reason is a drawn label with the reason's text",
			reason_node != null and reason_node.visible and reason_node.text == String(dis["reason"]) and reason_node.text != "", true)
		var card_r := Rect2(Vector2.ZERO, _card().size)
		_ok("D the reason is inside the card", reason_node != null and card_r.grow(1).encloses(reason_node.get_global_rect()), true)
		print("D reason=%s rect=%s" % [dis["reason"], reason_node.get_global_rect() if reason_node else Rect2()])
	await _close()

	# -- K: keyboard ------------------------------------------------------------------
	app.select_domain("civilization")
	await _frames(4)
	## Away from the fixture stack, so there is no Select row ahead of Edit.
	var ep2 := _empty_pos(ov)
	await _rmb(ov, _pos_of(ov, _k) if _only_settlement(ov) else ep2)
	var c = _card()
	var f0: Control = c.focused_row()
	var f0_text := String(f0.get_meta("card_row", {}).get("text", "")) if f0 else ""
	await _key(KEY_DOWN)
	var f1: Control = c.focused_row()
	var f1_text := String(f1.get_meta("card_row", {}).get("text", "")) if f1 else ""
	print("K focus %s -> %s" % [f0_text, f1_text])
	_ok("K Down moves the highlight to another row", f0 != null and f1 != null and f0 != f1, true)
	await _key(KEY_UP)
	_ok("K Up moves it back", c.focused_row() == f0, true)
	## Filter: type "info"
	for ch in "info":
		await _key(KEY_A + (ch.unicode_at(0) - 97), ch.unicode_at(0))
	_dump("K_filtered")
	var filtered := _texts(["action", "disabled", "param"])
	_ok("K typing filters the rows", filtered, ["Info here (settlement & ecology)"])
	_ok("K the filter is drawn", _texts(["filter"]), ["info"])
	for _i in 4:
		await _key(KEY_BACKSPACE)
	_ok("K Backspace clears the filter", _texts(["filter"]), [])
	_ok("K ... and the rows come back", _texts(["action", "disabled", "param"]).size() > 1, true)
	## Enter on "Centre view here" (navigate to it by keyboard)
	var target := "Centre view here"
	for _i in 12:
		var fr: Control = c.focused_row()
		if fr != null and String(fr.get_meta("card_row", {}).get("text", "")) == target:
			break
		await _key(KEY_DOWN)
	var fr2: Control = c.focused_row()
	_ok("K Down reaches '%s'" % target, fr2 != null and String(fr2.get_meta("card_row", {}).get("text", "")) == target, true)
	cam.position = Vector2(-12345, -6789)
	await _key(KEY_ENTER)
	_ok("K Enter ran it (the card closed)", _open(), false)
	_ok("K Enter ran it (the camera moved)", not cam.position.is_equal_approx(Vector2(-12345, -6789)), true)
	await _rmb(ov, ep2)
	_ok("K reopened", _open(), true)
	await _key(KEY_ESCAPE)
	_ok("K Esc closes", _open(), false)

	# -- W: new rows run what they name ------------------------------------------------
	## Measure from here: Measure armed, the chain started at this cell.
	await _rmb(ov, ep2)
	row = _row("Measure from here")
	if row != null:
		await _click_row(row)
	_ok("W Measure from here arms Measure", app.armed_tool, "measure")
	_ok("W ... with its first point placed", GlobalTools.measure_points().size(), 1)
	app.arm_tool("inspect")
	await _frames(2)
	## Copy coordinate ▸: the submenu opens in place, and its first row copies.
	await _rmb(ov, ep2)
	row = _row("Copy coordinate")
	if row != null:
		await _click_row(row)
	_ok("W Copy coordinate opens its submenu in place (BACK band)", _texts(["back"]), ["Copy coordinate"])
	var km_rows := _texts(["action"])
	if not km_rows.is_empty():
		var first_row := _row(String(km_rows[0]), ["action"])
		await _click_row(first_row)
		_ok("W the km row copied what it shows", DisplayServer.clipboard_get(),
			String(km_rows[0]).split("  (")[0])
	await _close()
	## View field ▸ (CARTO): the second of the eight views becomes the drawn one.
	app.select_domain("cartography")
	await _frames(4)
	await _rmb(ov, _empty_pos(ov))
	row = _row("View field")
	if row != null:
		await _click_row(row)
	var views := _texts(["action"])
	_ok("W View field lists the Layers popover's eight hot-keyed views", views.size(), 8)
	var ids: Array = []
	for r in _card().drawn_rows():
		if r["kind"] == "action":
			ids.append(String(r.get("id", "")))
	if ids.size() >= 2:
		await _click_row(_row(String(views[1]), ["action"]))
		_ok("W picking one draws it", "carto.view." + String(app.viewport.debug_view()), String(ids[1]))
		app.viewport.set_debug_layer("off")
	await _close()
	app.select_domain("civilization")
	await _frames(4)

	# -- T: an inline Tool row ---------------------------------------------------------
	app.select_domain("world")
	await _frames(4)
	app.arm_tool("paint")
	await _frames(6)
	await _rmb(ov, _empty_pos(ov))
	_dump("T_world_paint")
	var ww := _ws("world_workspace.gd")
	var r0 := float(ww.get("_paint_brush")["radius"])
	var prow: Control = null
	for r in _card().drawn_rows():
		if r["kind"] == "param" and String(r["text"]) == "Radius":
			prow = r["node"]
	_ok("T a Radius param row is drawn under TOOL · PAINT", prow != null and _texts(["band"]).has("TOOL · PAINT"), true)
	if prow != null:
		var plus: Button = null
		for b in prow.find_children("*", "Button", true, false):
			if (b as Button).text == "+":
				plus = b
		if plus != null:
			var pr := _in_vp(plus)
			print("T + button %s" % pr.size)
			await _click_at(pr.get_center())
		var r1 := float(ww.get("_paint_brush")["radius"])
		_ok("T + stepped the brush radius by one", r1, minf(r0 + 1.0, 40.0))
		var shown := ""
		for r in _card().drawn_rows():
			if r["kind"] == "param" and String(r["text"]) == "Radius":
				shown = String(r["value"])
		_ok("T the row shows the new value", shown, "%d cells" % int(r1))
		_ok("T the card stayed open for the step", _open(), true)
	await _close()
	app.arm_tool("inspect")
	await _frames(4)

	# -- E: four edges -------------------------------------------------------------------
	app.select_domain("cartography")
	await _frames(4)
	var vis: Rect2 = app.get_viewport().get_visible_rect()
	var broker = app.context_broker
	## Real right-clicks at the map's four edges, then the card's own
	## placement driven from the viewport's four extreme corners.
	var ir: Rect2 = ov._interior_rect(ov._displayed_rect())
	var edge_pts := [
		Vector2(ir.position.x + 4, ir.get_center().y), Vector2(ir.end.x - 4, ir.get_center().y),
		Vector2(ir.get_center().x, ir.position.y + 4), Vector2(ir.get_center().x, ir.end.y - 4)]
	var names := ["left", "right", "top", "bottom"]
	for i in 4:
		await _rmb(ov, edge_pts[i])
		var cr: Rect2 = _card().panel_rect() if _open() else Rect2()
		print("E map-%s card=%s visible=%s" % [names[i], cr, vis])
		_ok("E map-%s edge: the card opens and is on screen" % names[i], _open() and vis.encloses(cr), true)
		await _close()
	var corners := [Vector2(0, vis.size.y * 0.5), Vector2(vis.size.x - 1, vis.size.y * 0.5),
		Vector2(vis.size.x * 0.5, 0), Vector2(vis.size.x * 0.5, vis.size.y - 1)]
	for i in 4:
		_card().open(broker.last_request, broker.last_actions, corners[i], broker.reselect)
		await _frames(4)
		var cr2: Rect2 = _card().panel_rect()
		print("E screen-%s anchor=%s card=%s" % [names[i], corners[i], cr2])
		_ok("E screen-%s edge: the card is inside the viewport" % names[i], vis.encloses(cr2), true)
		await _close()

	# -- P: contrast, both palettes ---------------------------------------------------------
	var grounds := {}
	for dark in [true, false]:
		var was := DccTheme.is_dark()
		if was != dark:
			DccTheme.apply_theme(dark)
			app.rebuild_theme(was)
			await _frames(8)
		_ok("P palette forced: %s" % ("dark" if dark else "light"), DccTheme.is_dark(), dark)
		## CARTO on the fixture stack: title, subtitle, Select, bands, action,
		## danger and disabled rows all in one card.
		await _rmb(ov, _pos_of(ov, _k))
		await _frames(6)
		grounds[dark] = await _measure_contrast("dark" if dark else "light")
		await _close()
	_ok("P the two palettes draw different grounds (the forcing took)",
		grounds.get(true, Color.BLACK).get_luminance() < 0.1 and grounds.get(false, Color.BLACK).get_luminance() > 0.85, true)


func _only_settlement(ov: Control) -> bool:
	return ov.request_at(_pos_of(ov, _k), "mouse").get("hits", []).size() == 1


func _ws(file: String) -> Node:
	return _find(app, func(n: Node) -> bool:
		var sc: Variant = n.get_script()
		return sc != null and String(sc.resource_path).ends_with(file))


func _find(n: Node, pred: Callable) -> Node:
	if pred.call(n):
		return n
	for ch in n.get_children():
		var h := _find(ch, pred)
		if h != null:
			return h
	return null


func _find_visible_dialog(n: Node, title: String) -> Window:
	return _find(n, func(x: Node) -> bool:
		return x is Window and (x as Window).visible and (x as Window).title == title) as Window


static func _near(a: Color, b: Color) -> bool:
	return absf(a.r - b.r) < 0.02 and absf(a.g - b.g) < 0.02 and absf(a.b - b.b) < 0.02


## WCAG relative luminance contrast of `a` on `b`.
static func _ratio(a: Color, b: Color) -> float:
	var la := _lin(a)
	var lb := _lin(b)
	return (maxf(la, lb) + 0.05) / (minf(la, lb) + 0.05)


static func _lin(c: Color) -> float:
	var f := func(v: float) -> float:
		return v / 12.92 if v <= 0.03928 else pow((v + 0.055) / 1.055, 2.4)
	return 0.2126 * f.call(c.r) + 0.7152 * f.call(c.g) + 0.0722 * f.call(c.b)


## Crops each named label out of a capture: ground = the crop's most common
## colour, ink = the pixel farthest from it in luminance (MISTAKES.md: on
## light the brightest thing is the ground). Every text pair must reach 4.5.
func _measure_contrast(tag: String) -> Color:
	var img: Image = _vp.get_texture().get_image()
	var c = _card()
	var ground := Color.BLACK
	## The ground itself, from a strip of the card's left padding under the
	## header: no glyph is ever drawn there.
	## The panel's own top padding, above the title: no glyph is drawn there.
	var pr: Rect2 = c.panel_rect()
	ground = img.get_pixelv(Vector2i(int(pr.position.x) + 4, int(pr.position.y) + 5))
	print("P %s ground=%s lum=%.3f" % [tag, ground.to_html(false), ground.get_luminance()])
	## `CTXCARD_SHOT_DIR` set: the capture is saved there, to be looked at.
	if OS.get_environment("CTXCARD_SHOT_DIR") != "":
		img.save_png(OS.get_environment("CTXCARD_SHOT_DIR").path_join("ctxcard_%s.png" % tag))
	var targets: Array = []   ## [what, Label]
	for r in c.drawn_rows():
		var n: Control = r["node"]
		match String(r["kind"]):
			"header":
				targets.append(["title", n.find_child("Title", true, false)])
				targets.append(["subtitle", n.find_child("Subtitle", true, false)])
			"select":
				if c.focused_row() == n:
					var sr := _in_vp(n)
					targets.append(["highlighted select", n.find_child("Text", true, false),
						img.get_pixelv(Vector2i(int(sr.position.x) + 3, int(sr.get_center().y)))])
				else:
					targets.append(["select", n.find_child("Text", true, false)])
			"band":
				targets.append(["band " + String(r["text"]), n.get_child(0)])
			"action":
				var id := String(r.get("id", ""))
				var what := ("danger " if id.ends_with("delete") else "row ") + String(r["text"])
				if c.focused_row() == n:
					## The highlighted row is its own pair: its ink on the
					## highlight wash, sampled in the row's left padding.
					var nr := _in_vp(n)
					targets.append(["highlighted " + what, n.find_child("Text", true, false),
						img.get_pixelv(Vector2i(int(nr.position.x) + 3, int(nr.get_center().y)))])
				else:
					targets.append([what, n.find_child("Text", true, false)])
			"disabled":
				targets.append(["disabled " + String(r["text"]), n.find_child("Text", true, false)])
				targets.append(["reason", n.find_child("Reason", true, false)])
	var worst := 99.0
	for t in targets:
		var l: Label = t[1]
		if l == null:
			continue
		var bg: Color = t[2] if t.size() > 2 else ground
		var rr := _in_vp(l)
		var ink := bg
		var best := 0.0
		for y in range(int(rr.position.y), int(rr.end.y)):
			for x in range(int(rr.position.x), int(rr.end.x)):
				if x < 0 or y < 0 or x >= img.get_width() or y >= img.get_height():
					continue
				var px := img.get_pixel(x, y)
				var d := absf(px.get_luminance() - bg.get_luminance())
				if d > best:
					best = d
					ink = px
		var ratio := _ratio(ink, bg)
		worst = minf(worst, ratio)
		print("P %s %-40s ink=%s on=%s ratio=%.2f" % [tag, t[0], ink.to_html(false), bg.to_html(false), ratio])
		_ok("P %s '%s' reaches 4.5:1 on the card's ground" % [tag, t[0]], ratio >= 4.5, true)
	## The control: a ground-only strip must read ~1:1, or the measure is not
	## measuring text at all.
	var a := img.get_pixelv(Vector2i(int(pr.position.x) + 3, int(pr.position.y) + 4))
	var b := img.get_pixelv(Vector2i(int(pr.position.x) + 60, int(pr.position.y) + 6))
	_ok("P %s control: two ground pixels read below 1.2:1" % tag, _ratio(a, b) < 1.2, true)
	## And the ground is the token the card claims (`raised`), so the pair
	## measured above is text-on-card, not text-on-whatever-is-behind-it.
	_ok("P %s the ground is the card's own `raised`" % tag, _near(ground, DccTheme.c("raised")), true)
	print("P %s worst text ratio %.2f" % [tag, worst])
	return ground


func _ready() -> void:
	## `-- --vp WxH` sizes the host viewport (default 1600x900);
	## `--force-touch` is the shell's own flag. Anything else is refused, loudly.
	var vp_size := Vector2i(1600, 900)
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
		print("### CTXCARD ABORT: unknown argument '%s' ###" % a)
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

	## The card is the desktop and tablet presenter; a phone keeps its sheet.
	## A run that booted as a phone cannot answer, and says so rather than
	## reporting a pass or a pile of unrelated failures.
	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("FORM booted as %s at %s" % [form, _vp.size])
	if form == "phone":
		print("### CTXCARD ABORT: booted as a phone -- use a tablet-sized --vp (e.g. 2560x1600) ###")
		get_tree().quit(2)
		return
	await _run()
	print("### CTXCARD %s  %d/%d checks passed ###" % [
		"GREEN" if _fails == 0 else "RED", _checks - _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
