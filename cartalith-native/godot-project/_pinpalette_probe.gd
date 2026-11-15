extends Node
## Ruling FD (`PIN_PALETTE_SCOPE.md` PP-2..PP-5): the pin palette and the Pins tool. Windowed (refuses --headless), under a wiped
## user directory (`APPDATA=<empty dir>`). Desktop form only (the tablet and phone forms are `_pinpalette_touch_probe.gd`, PP-6).
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _pinpalette_probe.tscn -- --vp 1600x1000 --out <dir>
##
## Legs. A: the palette lists every icon of both sets, once, in a category, and the engine's name list is the same union.
## B: search by substring; recents keep the last 8, newest first, per user. C: a click drops the armed pin with its tip within
## 1 px of the click, as one undo step, then the card and the Name prompt open. D: dragging an icon from the palette onto the
## map drops it (a real drag through the viewport). E: dragging a placed pin moves it, as ONE undo step. F: Change kind from the
## card reopens the palette in change mode; a pick and a free-text kind are one undo step each. G: rename and delete are one
## step each and undo restores. H: a user pin inside a town hides below Ruling FC's threshold. The old-format save is the Rust tests' (`poi_bridge.rs`), not this probe's.
## Exit 0 = held, 1 = a check failed, 2 = could not run.

const Pin := preload("res://shell/pin_marker.gd")
const Pal := preload("res://shell/pin_palette.gd")

var app: Node
var _vp: SubViewport
var _bridge
var _ov: Control
var _checks := 0
var _fails := 0
var _out := ""
var _ws
## `--legs ABC`: run only these legs (a mutation run needs only the leg that must go red). E and F share a pin, so E before F.
var _legs := "ABCDEFGH"


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("PINPAL %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _p(s: String) -> void:
	print("PINPAL %s" % s)


func _finish() -> void:
	DccSettings.set_marker_style("pins")
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)


# ---- events ------------------------------------------------------------------------------------------------------

func _host(local: Vector2) -> Vector2:
	return _ov.get_global_transform_with_canvas() * local


## Godot starts a GUI drag only after the motion events' `relative` has accumulated past 10 px (`Viewport::_gui_input_event`'s
## `drag_accum`), so a synthesised motion must carry it or no drag ever begins (the first run of leg D proved it).
var _last_mouse := Vector2.ZERO

func _mouse(at: Vector2, down: Variant = null, mask: int = 0) -> void:
	if down == null:
		var mm := InputEventMouseMotion.new()
		mm.position = at
		mm.global_position = at
		mm.relative = at - _last_mouse
		_last_mouse = at
		mm.button_mask = mask
		_vp.push_input(mm)
		return
	_last_mouse = at
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = bool(down)
	ev.position = at
	ev.global_position = at
	ev.button_mask = MOUSE_BUTTON_MASK_LEFT if bool(down) else 0
	_vp.push_input(ev)


func _click_host(at: Vector2) -> void:
	_mouse(at)
	await _frames(1)
	_mouse(at, true)
	await _frames(2)
	_mouse(at, false)
	await _frames(6)


func _click(local: Vector2) -> void:
	await _click_host(_host(local))


## A real drag: press at `from`, move in steps to `to`, release. Host (viewport) coordinates.
func _drag_host(from: Vector2, to: Vector2, steps: int = 8) -> void:
	_mouse(from)
	await _frames(1)
	_mouse(from, true)
	await _frames(2)
	for i in range(1, steps + 1):
		_mouse(from.lerp(to, float(i) / float(steps)), null, MOUSE_BUTTON_MASK_LEFT)
		await _frames(1)
	await _frames(2)
	_mouse(to, false)
	await _frames(6)


func _undo_n() -> int:
	return (_bridge.undo_ledger() as Array).size()


func _card():
	var b = app.get("context_broker")
	return b.card if b != null else null


func _card_open() -> bool:
	var c = _card()
	return c != null and c.visible


func _close_card() -> void:
	var c = _card()
	if c != null and c.visible:
		c.hide()
	await _frames(3)


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(_out.path_join(name))
	_p("shot %s" % name)


## The visible Name prompt (a ConfirmationDialog titled "Name this pin"), or null.
func _name_prompt() -> ConfirmationDialog:
	for d in app.find_children("*", "ConfirmationDialog", true, false):
		var dlg := d as ConfirmationDialog
		if dlg.visible and dlg.title == "Name this pin":
			return dlg
	return null


func _answer_name(text: String) -> bool:
	var dlg := _name_prompt()
	if dlg == null:
		return false
	for c in dlg.find_children("*", "LineEdit", true, false):
		(c as LineEdit).text = text
	dlg.get_ok_button().pressed.emit()
	await _frames(4)
	return true


func _dismiss_name() -> void:
	var dlg := _name_prompt()
	if dlg != null:
		dlg.canceled.emit()
		dlg.hide()
	await _frames(3)


func _palette() -> Node:
	return app.find_child("PinPalette", true, false)


func _tile(prefix: String, kind: String) -> Button:
	var pal := _palette()
	return pal.find_child(prefix + kind, true, false) as Button if pal != null else null


func _tiles(prefix: String) -> Array:
	var out: Array = []
	var pal := _palette()
	if pal == null:
		return out
	for b in pal.find_children(prefix + "*", "Button", true, false):
		out.append(String(b.get_meta("pin_kind", "")))
	return out


## Scrolls the dock so `c` is on screen (the grid is longer than the dock), then waits for the layout.
func _reveal(c: Control) -> void:
	var n: Node = c.get_parent()
	while n != null and not (n is ScrollContainer):
		n = n.get_parent()
	if n != null:
		(n as ScrollContainer).ensure_control_visible(c)
	await _frames(4)


func _folder_union() -> Array:
	var u := {}
	for set in ["glyph", "painted"]:
		for f in DirAccess.get_files_at("res://icons/pins/%s" % set):
			if f.ends_with(".png"):
				u[f.trim_suffix(".png")] = true
	var out: Array = u.keys()
	out.sort()
	return out


## Free spots on the plate (overlay-local tips), clear of settlements and landmarks.
func _free_spots(n: int) -> Array:
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	var band: Rect2 = (app.viewport as Control).get_global_rect().grow(-90)
	var r: float = _ov._poi_pin_r()
	var clear_px := r * 5.0
	var others: Array = []
	for s in _bridge.settlements():
		others.append(_ov._cell_to_screen(Vector2(float((s as Dictionary)["x"]), float((s as Dictionary)["y"])), rect))
	for lm in _bridge.landmarks():
		others.append(_ov._cell_to_screen(Vector2(float((lm as Dictionary)["x"]), float((lm as Dictionary)["y"])), rect))
	for p in _bridge.poi_list():
		others.append(_ov._point_to_screen(Vector2(float(p["x"]), float(p["y"])), rect))
	var out: Array = []
	var step := r * 9.0
	var y := interior.position.y + step
	while y < interior.end.y - step:
		var x := interior.position.x + step
		while x < interior.end.x - step:
			var tip := Vector2(x, y)
			var head := tip + Vector2(0.0, -Pin.STEM_RATIO * r)
			var near := false
			for o in others:
				if (o as Vector2).distance_to(tip) < clear_px or (o as Vector2).distance_to(head) < clear_px:
					near = true
					break
			if not near and interior.has_point(tip) and interior.has_point(head) and band.has_point(_host(tip)) and band.has_point(_host(head)):
				out.append(tip)
			x += step
		y += step
	var picked: Array = []
	if out.is_empty():
		return picked
	var stride := maxi(1, out.size() / n)
	for k in n:
		picked.append(out[mini(k * stride, out.size() - 1)])
	return picked


func _ready() -> void:
	var vp_size := Vector2i(1600, 1000)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--vp":
			var wh := String(args[i + 1]).split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
		elif a == "--legs":
			_legs = String(args[i + 1]).to_upper()
			i += 2
		elif a == "--out":
			_out = String(args[i + 1])
			i += 2
		else:
			_p("ABORT unknown argument %s" % a)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		_p("ABORT run windowed")
		get_tree().quit(2)
		return
	var wd := get_tree().create_timer(900.0)
	wd.timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(3))
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(4)
	_vp.notification(Node.NOTIFICATION_VP_MOUSE_ENTER)
	_bridge = app.bridge
	_bridge.generate({"seed": 1200, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if not _bridge.has_world:
		_ok("a world generated", false)
		_finish()
		return
	_ov = app.viewport.overlay
	await _bridge.landmark_run()
	await _frames(6)
	var t0 := Time.get_ticks_msec()
	while app.busy_overlay != null and app.busy_overlay.visible and Time.get_ticks_msec() - t0 < 30000:
		await _frames(10)
	await _frames(20)
	for w in app._workspaces:
		if w.has_method("build_pin_palette"):
			_ws = w
	_ok("the CARTO workspace owns the Pins tool", _ws != null)
	DccSettings.set_marker_style("painted")
	await _run()
	_finish()


func _run() -> void:
	app.arm_tool("inspect")
	await _frames(4)
	app.arm_tool("poi")
	await _frames(6)
	if _legs.contains("A"):
		await _leg_a()
	if _legs.contains("B"):
		await _leg_b()
	if _legs.contains("C"):
		await _leg_c()
	if _legs.contains("D"):
		await _leg_d()
	if _legs.contains("E"):
		await _leg_e()
	if _legs.contains("F"):
		await _leg_f()
	if _legs.contains("G"):
		await _leg_g()
	if _legs.contains("H"):
		await _leg_h()


# ---- A. the palette lists every icon ------------------------------------------------------------------------------

func _leg_a() -> void:
	print("== A. every icon, once ==")
	var union := _folder_union()
	_ok("A0: the two icon folders hold 62 distinct names", union.size() == 62, str(union.size()))
	var table: Array = Pal.all_kinds()
	var sorted_table := table.duplicate()
	sorted_table.sort()
	_ok("A1: the category table is exactly that union, no name twice", sorted_table == union, "table %d, union %d" % [table.size(), union.size()])
	var names: PackedStringArray = _bridge.poi_kind_names()
	_ok("A2: the engine's name list is that union", Array(names) == union, "engine %d" % names.size())
	_ok("A2: the engine refuses a name that is no icon", _bridge.poi_add(10.0, 10.0, "brothel", "x") == -1 and _bridge.poi_list().is_empty())
	app.arm_tool("poi")
	await _frames(8)
	var pal := _palette()
	_ok("A3: arming the Pins tool shows the palette in the right dock", pal != null and pal.is_visible_in_tree())
	if pal == null:
		return
	var shown := _tiles("Pin_")
	shown.sort()
	_ok("A3: the grid shows all 62, once each", shown == union, "%d tiles" % shown.size())
	for style in ["painted", "icons", "pins", "circles"]:
		DccSettings.set_marker_style(style)
		app.viewport.refresh_annotations()
		app.right_dock_ctrl.show_pins()
		await _frames(6)
		var miss := 0
		for k in union:
			var b := _tile("Pin_", k)
			if b == null or b.icon == null:
				miss += 1
		_ok("A4: style %s draws a picture on every tile" % style, miss == 0, "%d without" % miss)
		var note := _palette().find_child("StyleNote", true, false) as Label
		_ok("A4: and the caption names the style", note != null and note.text.to_lower().contains(style) or (style == "icons" and note != null), note.text if note != null else "none")
	DccSettings.set_marker_style("painted")
	app.right_dock_ctrl.show_pins()
	await _frames(6)
	await _shot("pinpal_palette.png")


# ---- B. search and recents ----------------------------------------------------------------------------------------------

func _leg_b() -> void:
	print("== B. search, recents ==")
	var union := _folder_union()
	var se := _palette().find_child("PinSearch", true, false) as LineEdit
	_ok("B0: a search field", se != null)
	se.text = "fort"
	se.text_changed.emit("fort")
	await _frames(3)
	var got := _tiles("Pin_")
	got.sort()
	var want: Array = []
	for k in union:
		if Pal.label_for(k).to_lower().contains("fort") or k.contains("fort"):
			want.append(k)
	_ok("B1: 'fort' finds exactly the names holding it", got == want and want.size() >= 3, "%s" % str(got))
	## A substring that is NOT a prefix of any name (a prefix-only search finds nothing).
	se.text = "pass"
	se.text_changed.emit("pass")
	await _frames(3)
	got = _tiles("Pin_")
	got.sort()
	_ok("B1: 'pass' (inside the names, never at the start) finds mountain_pass and fortified_pass", got == ["fortified_pass", "mountain_pass"], str(got))
	se.text = "WATER"
	se.text_changed.emit("WATER")
	await _frames(3)
	got = _tiles("Pin_")
	_ok("B2: a category title matches case-insensitively (WATER lists the Water category)", got.has("lake") and got.has("harbour") and got.has("waterfall") and not got.has("tavern"), str(got))
	se.text = "zzzz"
	se.text_changed.emit("zzzz")
	await _frames(3)
	_ok("B3: no match says so and draws no tile", _tiles("Pin_").is_empty() and _palette().find_child("NoMatch", true, false) != null)
	se.text = ""
	se.text_changed.emit("")
	await _frames(3)
	## Recents: ten picks, only the last eight stay, newest first.
	var picks := ["tavern", "inn", "market", "smithy", "guild", "mine", "quarry", "salt_works", "castle", "fort"]
	for k in picks:
		(_tile("Pin_", k) as Button).pressed.emit()
		await _frames(2)
	var rec: Array = DccSettings.pin_recents()
	_ok("B4: recents keep the last 8, newest first", rec == ["fort", "castle", "salt_works", "quarry", "mine", "guild", "smithy", "market"], str(rec))
	await _frames(3)
	var shown_rec := _tiles("Recent_")
	_ok("B4: the dock's row shows those 8", shown_rec == rec, str(shown_rec))
	(_tile("Pin_", "castle") as Button).pressed.emit()
	await _frames(2)
	_ok("B5: re-picking a recent moves it to the front without a duplicate", DccSettings.pin_recents() == ["castle", "fort", "salt_works", "quarry", "mine", "guild", "smithy", "market"], str(DccSettings.pin_recents()))
	pass


# ---- C. a click drops the armed pin -------------------------------------------------------------------------------------

func _leg_c() -> void:
	print("== C. click to drop ==")
	(_tile("Pin_", "waterfall") as Button).pressed.emit()
	await _frames(3)
	_ok("C0: the armed readout names the pin", ((_palette().find_child("ArmedPin", true, false)) as Label).text.contains("Waterfall"))
	var spots := _free_spots(6)
	_ok("C1: free spots exist", spots.size() >= 4, str(spots.size()))
	var n0 := _undo_n()
	var at: Vector2 = spots[0]
	await _click(at)
	var l: Array = _bridge.poi_list()
	_ok("C2: one pin, a waterfall, unnamed", l.size() == 1 and String(l[0]["kind"]) == "waterfall" and not l[0].has("name"), str(l))
	_ok("C2: one undo step", _undo_n() == n0 + 1, "%d -> %d" % [n0, _undo_n()])
	if l.size() != 1:
		return
	var tip: Vector2 = _ov.poi_tip(0)
	_ok("C3: the tip is within 1 px of the click (%.2f px)" % tip.distance_to(at), tip.distance_to(at) <= 1.0)
	_ok("C4: the pin is selected and its card is open", app.selection == {"kind": "poi", "id": 0} and _card_open(), str(app.selection))
	var lr: Dictionary = app.context_broker.last_request
	_ok("C4: the card is about that one pin and carries a NOTE band", String((lr.get("hits", [{}]) as Array)[0].get("kind", "")) == "poi"
		and app.context_broker.last_actions.any(func(r): return String((r as Dictionary).get("section", "")) == "note"))
	_ok("C5: the Name prompt is open with the field focused", _name_prompt() != null)
	await _shot("pinpal_drop_card.png")
	var undo_before := _undo_n()
	_ok("C6: naming it is its own step", await _answer_name("Silver Falls") and String(_bridge.poi_get(0)["name"]) == "Silver Falls" and _undo_n() == undo_before + 1)
	await _close_card()
	_ok("C7: the tool is still armed", app.armed_tool == "poi")
	## A second drop with no pin armed does nothing but say so (never a plausible default).
	_ws._pin_state["kind"] = ""
	var n1 := _undo_n()
	await _click(spots[1])
	_ok("C8: nothing armed: nothing placed, no step", _bridge.poi_list().size() == 1 and _undo_n() == n1)
	## A free-text kind.
	var free_le := _palette().find_child("FreeLabel", true, false) as LineEdit
	free_le.text = "Beehives"
	(_palette().find_child("Mark_notice", true, false) as Button).pressed.emit()
	(_palette().find_child("FreeUse", true, false) as Button).pressed.emit()
	await _frames(3)
	var n2 := _undo_n()
	await _click(spots[2])
	l = _bridge.poi_list()
	_ok("C9: a free-text pin: the exclamation mark with the user's words", l.size() == 2 and String(l[1]["kind"]) == "notice" and String(l[1].get("free_label", "")) == "Beehives"
		and String(l[1]["kind_label"]) == "Beehives" and _undo_n() == n2 + 1, str(l))
	await _dismiss_name()
	await _close_card()
	## Undo each: the free pin, then the name, then the first pin.
	_bridge.undo_last()
	await _frames(3)
	_ok("C10: undo removes the free-text pin", _bridge.poi_list().size() == 1)
	_bridge.undo_last()
	await _frames(3)
	_ok("C10: undo reverts the name", not _bridge.poi_get(0).has("name"))
	_bridge.undo_last()
	await _frames(3)
	_ok("C10: undo removes the dropped pin", _bridge.poi_list().is_empty())
	_ws._pin_state["kind"] = "waterfall"
	_ws._pin_state["free_label"] = ""


# ---- D. drag an icon from the palette --------------------------------------------------------------------------------------

func _leg_d() -> void:
	print("== D. drag from the palette ==")
	app.right_dock_ctrl.show_pins()
	await _frames(6)
	var b := _tile("Pin_", "lake")
	_ok("D0: the lake tile exists", b != null)
	var spots := _free_spots(6)
	var at: Vector2 = spots[3]
	_ok("D0: the tile is on screen before the drag", get_viewport() != null and Rect2(Vector2.ZERO, Vector2(_vp.size)).encloses(b.get_global_rect()), str(b.get_global_rect()))
	await _reveal(b)
	var from := b.get_global_rect().get_center()
	var n0 := _undo_n()
	await _drag_host(from, _host(at), 12)
	var l: Array = _bridge.poi_list()
	await _shot("pinpal_after_drag.png")
	_ok("D1: a real drag from the tile drops a lake pin", l.size() == 1 and String(l[0]["kind"]) == "lake", str(l))
	if l.size() == 1:
		var tip: Vector2 = _ov.poi_tip(0)
		_ok("D2: its tip is within 1 px of the drop point (%.2f px)" % tip.distance_to(at), tip.distance_to(at) <= 1.0)
		_ok("D3: one undo step", _undo_n() == n0 + 1)
	await _dismiss_name()
	await _close_card()
	await _shot("pinpal_dropped_pin.png")
	_bridge.undo_last()
	await _frames(3)
	_ok("D4: undo removes it", _bridge.poi_list().is_empty())
	## The drop handler takes nothing but a pin payload.
	_ok("D5: a foreign payload is not accepted", not _ov._can_drop_data(at, {"other": 1}) and not _ov._can_drop_data(at, "text"))
	_ok("D5: a pin payload over bare map is", _ov._can_drop_data(at, {"pin_kind": "lake"}))


# ---- E. drag a placed pin -----------------------------------------------------------------------------------------------------

func _leg_e() -> void:
	print("== E. drag a placed pin to move it ==")
	var spots := _free_spots(8)
	var g0: Dictionary = _ov._grid_point(spots[0], _ov._displayed_rect(), _ov._interior_rect(_ov._displayed_rect()))
	var idx: int = _bridge.poi_add(float(g0["gx"]), float(g0["gy"]), "tavern", "Mover")
	app.viewport.refresh_annotations()
	await _frames(6)
	var n0 := _undo_n()
	var tip: Vector2 = _ov.poi_tip(idx)
	var r: float = _ov._poi_pin_r()
	var grab := tip + Vector2(0.0, -Pin.STEM_RATIO * r)   # on the pin's head
	var dest: Vector2 = spots[5]
	var delta := dest - tip
	await _drag_host(_host(grab), _host(grab + delta), 10)
	var tip2: Vector2 = _ov.poi_tip(idx)
	_ok("E1: the pin moved by the drag (%.1f px off)" % tip2.distance_to(tip + delta), tip2.distance_to(tip + delta) <= 1.5)
	_ok("E2: ONE undo step for the whole drag", _undo_n() == n0 + 1, "%d -> %d" % [n0, _undo_n()])
	_ok("E3: no second pin was dropped", _bridge.poi_list().size() == 1)
	_bridge.undo_last()
	await _frames(3)
	var back: Vector2 = _ov.poi_tip(idx)
	_ok("E4: undo puts it back exactly (%.2f px)" % back.distance_to(tip), back.distance_to(tip) <= 0.5)
	## A press and release with no travel opens the card instead of moving it.
	await _close_card()
	await _click(grab)
	_ok("E5: a click on a pin opens its card, moves nothing", _card_open() and _undo_n() == n0)
	await _close_card()


# ---- F. change kind -----------------------------------------------------------------------------------------------------------------

func _leg_f() -> void:
	print("== F. Change kind ==")
	var ids: Array = []
	await _click(_ov.poi_tip(0) + Vector2(0.0, -Pin.STEM_RATIO * _ov._poi_pin_r()))
	for r in app.context_broker.last_actions:
		ids.append(String((r as Dictionary).get("id", "")))
	_ok("F0: the card has Change kind as one row, no submenu", ids.has("el.poi_kind") and not app.context_broker.last_actions.any(func(r): return (r as Dictionary).has("children") and String(r.get("id", "")) == "el.poi_kind"), str(ids))
	var row: Dictionary = {}
	for r in app.context_broker.last_actions:
		if String((r as Dictionary).get("id", "")) == "el.poi_kind":
			row = r
	await _close_card()
	(row["callable"] as Callable).call()
	await _frames(8)
	_ok("F1: the palette reopens in change mode with its banner", _palette() != null and _palette().find_child("ChangeBanner", true, false) != null)
	await _shot("pinpal_change_kind.png")
	var n0 := _undo_n()
	(_tile("Pin_", "temple") as Button).pressed.emit()
	await _frames(4)
	_ok("F2: a pick changes the kind, one undo step", String(_bridge.poi_get(0)["kind"]) == "temple" and _undo_n() == n0 + 1)
	_ok("F2: and ends change mode (no banner)", _palette() == null or _palette().find_child("ChangeBanner", true, false) == null)
	_bridge.undo_last()
	await _frames(3)
	_ok("F3: undo restores the tavern", String(_bridge.poi_get(0)["kind"]) == "tavern")
	## Free-text change.
	(row["callable"] as Callable).call()
	await _frames(8)
	var le := _palette().find_child("FreeLabel", true, false) as LineEdit
	le.text = "Old oak"
	(_palette().find_child("Mark_other", true, false) as Button).pressed.emit()
	(_palette().find_child("FreeUse", true, false) as Button).pressed.emit()
	await _frames(4)
	var pd: Dictionary = _bridge.poi_get(0)
	_ok("F4: a free-text kind: the question mark with the user's words, one undo step", String(pd["kind"]) == "other" and String(pd.get("free_label", "")) == "Old oak" and _undo_n() == n0 + 1, str(pd))
	_bridge.undo_last()
	await _frames(3)
	_ok("F5: undo restores the tavern and clears the label", String(_bridge.poi_get(0)["kind"]) == "tavern" and not _bridge.poi_get(0).has("free_label"))
	## Cancel leaves it.
	(row["callable"] as Callable).call()
	await _frames(8)
	(_palette().find_child("ChangeCancel", true, false) as Button).pressed.emit()
	await _frames(4)
	_ok("F6: Cancel changes nothing and ends change mode", String(_bridge.poi_get(0)["kind"]) == "tavern" and _palette().find_child("ChangeBanner", true, false) == null and _undo_n() == n0)


# ---- G. rename and delete -----------------------------------------------------------------------------------------------------------------

func _leg_g() -> void:
	print("== G. rename, delete, undo ==")
	var n0 := _undo_n()
	_ok("G1: rename is one step", _bridge.poi_rename(0, "Renamed") and _undo_n() == n0 + 1)
	_bridge.undo_last()
	_ok("G1: undo restores the name", String(_bridge.poi_get(0)["name"]) == "Mover")
	_ok("G2: delete is one step", _bridge.poi_delete(0) and _undo_n() == n0 + 1 and _bridge.poi_list().is_empty())
	_bridge.undo_last()
	await _frames(3)
	_ok("G2: undo restores it", _bridge.poi_list().size() == 1 and String(_bridge.poi_get(0)["name"]) == "Mover")


# ---- H. Ruling FC ---------------------------------------------------------------------------------------------------------------------------

func _leg_h() -> void:
	print("== H. a user pin inside a town (Ruling FC) ==")
	var s: Dictionary = _bridge.settlements()[0]
	var idx: int = _bridge.poi_add(float(s["x"]) + 0.5, float(s["y"]) + 0.5, "well", "Town well")
	app.viewport.refresh_annotations()
	await _frames(6)
	var rect: Rect2 = _ov._displayed_rect()
	_ok("H1: at the fit view the town layout is not drawn", not _ov._town_layer_zoom_reached(rect))
	_ok("H2: so the pin inside the town is hidden and not pickable", _ov._poi_town_hidden(idx, rect) and _ov.poi_at(_ov.poi_tip(idx) + Vector2(0.0, -10.0)) != idx)
	var others: int = _bridge.poi_list().size()
	_ok("H3: the pin outside any town is untouched", others == 2 and not _ov._poi_town_hidden(0, rect))
	## Positive control: deep enough that the town layout draws, the same pin is no longer hidden.
	var z0: float = _ov._camera_zoom
	_ov._camera_zoom = 400.0
	var deep: Rect2 = _ov._displayed_rect()
	_ok("H4: zoomed in to the town layout, the same pin is shown (the gate follows the view)", _ov._town_layer_zoom_reached(deep) and not _ov._poi_town_hidden(idx, deep))
	_ov._camera_zoom = z0
	_bridge.undo_last()
	await _frames(3)
