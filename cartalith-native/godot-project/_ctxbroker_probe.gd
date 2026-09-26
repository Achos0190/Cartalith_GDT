extends Node
## CM-1 (`MAP_CONTEXT_SCOPE.md` §11): the map right-click through
## `context_requested` → `shell/context_broker.gd` → every provider's
## `context_actions`. CM-1 presented them in the same `PopupMenu` CX-01 always
## used; **since CM-2 the desktop presenter is `shell/context_card.gd`**, so
## this probe reads the card, and asserts CM-1's own contract on it: the rows
## CM-1 routed (`civ.*`, and CARTO's `carto.settlement_in_civil`) arrive with
## the same text, order and enabled state, and a `----` still falls wherever
## the section changes. The card's other rows (CM-2's WORLD, CARTO and global
## rows) are printed on the `ROW` lines but are `_ctxcard_probe.gd`'s to
## assert, not this file's. Legs D and F read "none of CM-1's rows" for that
## reason: since CM-2 a right-click there opens a card of other rows.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _ctxbroker_probe.tscn
##
## **Windowed, not `--headless`** -- the card is a `Window`, and whether it is
## on screen is part of what is asserted. No pixel is read, so no palette is
## assumed; the assertions are on row text, ids, enabled state, section
## changes, card visibility and engine selection sets. Rows are pressed by a
## click pushed through the SubViewport, the path a pointer takes.
##
## Before CM-2 this file read `PopupMenu` items; its `ROW` lines from that run
## (the CM-1 rows, in order, with `----` at section changes) are the
## before/after comparison for CM-2: the CM-1 rows must be identical.
##
## Legs:
##   A  CIVIL, RMB on a settlement      -- CX-01's five rows, in order, enabled
##   B  CIVIL, RMB on an empty cell     -- the two rows with no settlement
##   C  CIVIL, PH-02 touch hold on the settlement -- the same five rows
##   D  WORLD, RMB on the settlement    -- none of CM-1's rows
##   E  CARTO, RMB on the settlement    -- Ruling AX F4's one "…in CIVIL" row
##   F  CARTO, RMB on an empty cell     -- none of CM-1's rows
##   G  the overlap fixture: a label and an icon placed on the settlement;
##      `hits[]` must hold all three, and the RMB must not move either
##      selection set
##   H  behaviour: "Move viewer to" moves the camera where `_on_ctx_id(1)` does
##   I  F4's row, pressed: the domain becomes CIVIL and CX-01's rows open

const SEED := 483920

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


func _find(n: Node, pred: Callable) -> Node:
	if pred.call(n):
		return n
	for c in n.get_children():
		var h := _find(c, pred)
		if h != null:
			return h
	return null


func _card():
	var b = app.get("context_broker")
	return b.card if b != null else null


func _civ_ws() -> Node:
	return _find(app, func(n: Node) -> bool:
		var sc: Variant = n.get_script()
		return sc != null and String(sc.resource_path).ends_with("civilization_workspace.gd"))


## Whether the card is up -- the one presenter a desktop right-click opens.
func _visible_popups() -> Array:
	var c = _card()
	return [c] if c != null and c.visible else []


func _hide_popups() -> void:
	var c = _card()
	if c != null:
		c.hide()
	await _frames(2)


const CM1_IDS := ["civ.", "carto.settlement_in_civil"]

static func _is_cm1(id: String) -> bool:
	for p in CM1_IDS:
		if id.begins_with(p):
			return true
	return false


## CM-1's rows as the card drew them, in the format this probe always used:
## the row text, ` {disabled}` when it is drawn disabled, and `----` wherever
## the section changes between two of them -- which is where CX-01's menu had
## its separators.
func _rows(c) -> Array:
	var out: Array = []
	var band := ""
	var last_band := ""
	for r in c.drawn_rows():
		if r["kind"] == "band":
			band = String(r["text"])
		if not _is_cm1(String(r.get("id", ""))):
			continue
		if not out.is_empty() and band != last_band:
			out.append("----")
		last_band = band
		out.append("%s%s" % [r["text"], " {disabled}" if r["kind"] == "disabled" else ""])
	return out


func _dump(tag: String) -> Array:
	var vis := _visible_popups()
	print("ROW %s visible_popups=%d" % [tag, vis.size()])
	var rows: Array = []
	for c in vis:
		print("ROW %s card rect=%s" % [tag, c.panel_rect()])
		for r in c.drawn_rows():
			if r["kind"] in ["action", "disabled", "param", "select"]:
				print("ROW %s   card-row %s | %s" % [tag, String(r.get("id", "")), r["text"]])
		rows = _rows(c)
		for i in rows.size():
			print("ROW %s [%d] %s" % [tag, i, rows[i]])
	return rows


func _rmb(ov: Control, pos: Vector2) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_RIGHT
	ev.pressed = true
	ev.position = pos
	ov._gui_input(ev)
	var up := ev.duplicate()
	up.pressed = false
	ov._gui_input(up)
	await _frames(4)


## PH-02's hold: an emulated-from-touch left press, held past `_TOUCH_HOLD_MS`
## without moving, then lifted.
func _hold(ov: Control, pos: Vector2) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = true
	ev.position = pos
	ev.device = InputEvent.DEVICE_ID_EMULATION
	ov._gui_input(ev)
	await get_tree().create_timer(0.8).timeout
	await _frames(2)


func _lift(ov: Control, pos: Vector2) -> void:
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = pos
	up.device = InputEvent.DEVICE_ID_EMULATION
	ov._gui_input(up)
	await _frames(2)


## A left click on the card row whose text starts with `prefix`, pushed
## through the SubViewport at the row's centre.
func _press_row(c, prefix: String) -> bool:
	for r in c.drawn_rows():
		if r["kind"] == "action" and String(r["text"]).begins_with(prefix):
			var n: Control = r["node"]
			var p: Vector2 = n.get_global_rect().get_center() + Vector2(c.position)
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
			await _frames(4)
			return true
	return false


func _run() -> void:
	var bridge = app.bridge
	var ov: Control = app.viewport.overlay
	var rect: Rect2 = ov._displayed_rect()
	var inter: Rect2 = ov._interior_rect(rect)
	var has_broker: bool = app.get("context_broker") != null
	print("MODE broker=%s" % has_broker)

	## A settlement the overlay's own hit test resolves to itself, well inside
	## the plate, so the fixture legs below are not fighting an edge.
	var sets: Array = ov.get("_settlements")
	var k := -1
	var spos := Vector2.ZERO
	for i in sets.size():
		var s: Dictionary = sets[i]
		var p: Vector2 = ov._cell_to_screen(Vector2(s["x"], s["y"]), rect)
		if not inter.grow(-60.0).has_point(p):
			continue
		if ov._hit_test_settlement(p, inter, rect) == i:
			k = i
			spos = p
			break
	_ok("a settlement to click exists", k >= 0, true)
	if k < 0:
		return
	var sname := String(bridge.settlements()[k].get("name", "(unnamed)"))
	print("FIXTURE settlement k=%d name=%s screen=%s" % [k, sname, spos])
	## An empty cell: inside the plate, no settlement pin within reach.
	var epos := Vector2.ZERO
	for tries in 400:
		var c := inter.position + Vector2(fmod(37.0 * tries, inter.size.x), fmod(53.0 * tries, inter.size.y))
		if inter.grow(-40.0).has_point(c) and ov._hit_test_settlement(c, inter, rect) == -1:
			epos = c
			break
	_ok("an empty cell exists", epos != Vector2.ZERO, true)

	# -- A ------------------------------------------------------------------
	await _hide_popups()
	app.select_domain("civilization")
	await _frames(4)
	await _rmb(ov, _pos_of(ov, k))
	var rows_a := _dump("A_civ_settlement")
	## CX-01's five rows plus CM-7's CM-2-residual additions (`MAP_CONTEXT_
	## SCOPE.md` §11 CM-7): Open city layout… (object) and Start way/route
	## here (place). Ten rows, three separators.
	_ok("A eight rows plus two separators", rows_a.size(), 10)
	_ok("A row order", rows_a, [
		"Edit %s" % sname, "Move viewer to %s" % sname,
		"Open city layout for %s…" % sname, "Delete %s" % sname, "----",
		"Drop settlement here", "Start way here", "Start route here", "----",
		"Info here (settlement & ecology)"])

	# -- B ------------------------------------------------------------------
	await _hide_popups()
	await _rmb(ov, _empty_pos(ov))
	var rows_b := _dump("B_civ_empty")
	_ok("B row order", rows_b, ["Drop settlement here", "Start way here",
		"Start route here", "----", "Info here (settlement & ecology)"])

	# -- C ------------------------------------------------------------------
	await _hide_popups()
	app.arm_tool("settlement")
	await _frames(2)
	var n_before: int = bridge.settlements().size()
	await _hold(ov, _pos_of(ov, k))
	var rows_c := _dump("C_civ_touch_hold")
	## Settlement is armed for this leg, so CM-7's settlement-class Tool
	## param row is also on screen, ahead of A's own rows -- A itself runs
	## with no tool armed and does not see it.
	_ok("C the hold opened the same rows as A, plus the armed Class row",
		rows_c, ["Class", "----"] + rows_a)
	await _lift(ov, _pos_of(ov, k))
	_ok("C the withheld press never dropped a settlement", bridge.settlements().size(), n_before)
	app.arm_tool("inspect")
	await _frames(2)

	# -- D ------------------------------------------------------------------
	await _hide_popups()
	app.select_domain("world")
	await _frames(4)
	await _rmb(ov, _pos_of(ov, k))
	_ok("D WORLD shows none of CM-1's rows", _dump("D_world_settlement").size(), 0)

	# -- E / F ---------------------------------------------------------------
	await _hide_popups()
	app.select_domain("cartography")
	await _frames(4)
	await _rmb(ov, _pos_of(ov, k))
	var rows_e := _dump("E_carto_settlement")
	await _hide_popups()
	await _rmb(ov, _empty_pos(ov))
	var rows_f := _dump("F_carto_empty")
	if has_broker:
		_ok("E CARTO: one row for the settlement, in CIVIL", rows_e, ["Settlement actions in CIVIL ›"])
	else:
		_ok("E (HEAD) CARTO opens nothing", rows_e.size(), 0)
	_ok("F CARTO empty cell shows none of CM-1's rows", rows_f.size(), 0)

	# -- H: behaviour of one CX-01 row ----------------------------------------
	await _hide_popups()
	app.select_domain("civilization")
	await _frames(4)
	await _rmb(ov, _pos_of(ov, k))
	var cam: Control = app.viewport.get("_camera")
	cam.position = Vector2(-12345, -6789)
	var vis := _visible_popups()
	if vis.size() == 1:
		await _press_row(vis[0], "Move viewer to")
	var s_k: Dictionary = bridge.settlements()[k]
	var want_cam: Vector2 = cam.position
	cam.position = Vector2(-1, -1)
	app.viewport.move_view_to(float(int(s_k.get("x", 0))), float(int(s_k.get("y", 0))))
	var direct: Vector2 = cam.position
	_ok("H Move viewer to lands where move_view_to puts it", want_cam.is_equal_approx(direct), true)

	if not has_broker:
		return

	# -- G: the overlap fixture -------------------------------------------------
	await _hide_popups()
	var cell: Dictionary = {"x": float(s_k["x"]), "y": float(s_k["y"])}
	var li: int = bridge.label_create(cell["x"] + 0.5, cell["y"] + 0.5, "CTX FIXTURE")
	_ok("G a label was placed on the settlement", li >= 0, true)
	var pack_path: String = ProjectSettings.globalize_path("res://") \
		+ "../crates/cartalith-assets/tests/fixtures/reference_pack.zip"
	var pack_ok: bool = bridge.world_gen.load_asset_pack(pack_path)
	_ok("G the fixture pack loaded", pack_ok, true)
	var ii := -1
	if pack_ok and bridge.icon_arm("feature", 0, 1.0, 0.0, 0.0):
		ii = bridge.icon_place(cell["x"] + 0.5, cell["y"] + 0.5)
	_ok("G an icon was placed on the settlement", ii >= 0, true)
	## Both sets emptied, so "unchanged" is a real comparison: the selecting
	## hit tests (`label_hit_test` / `icon_hit_test`, plain-click Replace) would
	## put the fixture label and icon into them.
	bridge.label_select_set(PackedInt64Array())
	bridge.icon_select_set(PackedInt64Array())
	await _frames(6)
	var lsel0: PackedInt64Array = bridge.label_get_selection()
	var isel0: PackedInt64Array = bridge.icon_get_selection()

	spos = _pos_of(ov, k)
	rect = ov._displayed_rect()
	inter = ov._interior_rect(rect)
	var req: Dictionary = ov.request_at(spos, "mouse")
	var kinds: Array = []
	for h in req.get("hits", []):
		kinds.append("%s:%d" % [h["kind"], h["id"]])
		print("HIT %s" % str(h))
	_ok("G hits holds the settlement", kinds.has("settlement:%d" % k), true)
	_ok("G hits holds the label", kinds.has("label:%d" % li), true)
	_ok("G hits holds the icon", kinds.has("icon:%d" % ii), true)
	var sorted_ok := true
	var hits: Array = req.get("hits", [])
	for j in range(1, hits.size()):
		if float(hits[j - 1]["dist"]) > float(hits[j]["dist"]):
			sorted_ok = false
	_ok("G hits are nearest first", sorted_ok, true)
	for h in hits:
		_ok("G hit %s:%d has kind/id/dist, and no blank label" % [h["kind"], h["id"]],
			h.has("kind") and h.has("id") and h.has("dist")
				and (not h.has("label") or String(h["label"]) != ""), true)
	for h in hits:
		if h["kind"] == "settlement" and int(h["id"]) == k:
			_ok("G the settlement hit carries its name", h.get("label"), sname)
		if h["kind"] == "label" and int(h["id"]) == li:
			_ok("G the label hit carries its text", h.get("label"), "CTX FIXTURE")
	_ok("G the first settlement hit is _hit_test_settlement's",
		_first_settlement(hits), ov._hit_test_settlement(spos, inter, rect))

	## The live gesture, in CIVIL and CARTO, over the fixture.
	app.select_domain("civilization")
	await _frames(4)
	await _rmb(ov, _pos_of(ov, k))
	_ok("G CIVIL rows are A's with a label and icon on top", _dump("G_civ_fixture"), rows_a)
	_ok("G the RMB left the label selection alone", bridge.label_get_selection(), lsel0)
	_ok("G the RMB left the icon selection alone", bridge.icon_get_selection(), isel0)
	await _hide_popups()
	app.select_domain("cartography")
	await _frames(4)
	await _rmb(ov, _pos_of(ov, k))
	_ok("G CARTO rows over the fixture", _dump("G_carto_fixture"), ["Settlement actions in CIVIL ›"])
	_ok("G the CARTO RMB left the label selection alone", bridge.label_get_selection(), lsel0)
	_ok("G the CARTO RMB left the icon selection alone", bridge.icon_get_selection(), isel0)

	# -- I: F4's row, pressed ----------------------------------------------------
	vis = _visible_popups()
	if vis.size() == 1:
		await _press_row(vis[0], "Settlement actions in CIVIL")
	await _frames(4)
	_ok("I the domain is CIVIL", app.active_domain(), "civilization")
	_ok("I CX-01's rows are open at the same spot", _dump("I_after_f4"), rows_a)
	await _hide_popups()


func _first_settlement(hits: Array) -> int:
	for h in hits:
		if h["kind"] == "settlement":
			return int(h["id"])
	return -1


## Settlement `k`'s pin and an empty cell, in the overlay's local space *now*:
## the displayed rect moves whenever a domain switch resizes the docks, so a
## position taken before `select_domain()` misses after it.
func _pos_of(ov: Control, k: int) -> Vector2:
	var rect: Rect2 = ov._displayed_rect()
	var s: Dictionary = ov.get("_settlements")[k]
	return ov._cell_to_screen(Vector2(s["x"], s["y"]), rect)


func _empty_pos(ov: Control) -> Vector2:
	var rect: Rect2 = ov._displayed_rect()
	var inter: Rect2 = ov._interior_rect(rect)
	for tries in 400:
		var c := inter.position + Vector2(fmod(37.0 * tries, inter.size.x), fmod(53.0 * tries, inter.size.y))
		if inter.grow(-40.0).has_point(c) and ov._hit_test_settlement(c, inter, rect) == -1:
			return c
	return Vector2.ZERO


func _ready() -> void:
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
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

	await _run()
	print("### CTXBROKER %s  %d/%d checks passed ###" % [
		"GREEN" if _fails == 0 else "RED", _checks - _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
