extends Node
## Ruling FD, `PIN_PALETTE_SCOPE.md` PP-6: the pin palette on the two touch forms. Windowed (refuses --headless), under a wiped
## user directory (`APPDATA=<empty dir>`). The form decides the legs, so the same file serves both:
##
##   TABLET (>= 900 dp):  `-- --vp 1700x1000 --force-touch --density=1.0`
##   PHONE  (<  900 dp):  `-- --vp 1080x2340 --force-touch --density 2.625`
##   common:              `--out <dir>` (frames), `--tag <name>` (a prefix for the frame names, so a mutant run never
##                        overwrites the frames under review), `--legs <letters>` (a mutation run needs only its leg)
##
## Every touch here is a left-button event stamped `device = DEVICE_ID_EMULATION`, which is what Godot's touch emulation sends and
## what the host (`viewport_host.gd::_is_touch_mouse`) and the overlay (`map_overlay.gd::_gui_input`) treat as a finger.
## Sizes are read from LAID-OUT rects (`get_global_rect()`) after two settled frames, never from minimum sizes.
##
## Tablet legs. T1 every icon is listed (62, once each) and search finds by substring. T2 every tile, category head, the search
## field and the free-label buttons are at least 44 dp tall (and tiles 44 wide) as laid out. T3 tap-to-arm then tap-to-drop: the
## pin's tip lands within 1 px of the tap, as one undo step. T4 a long-press drag from a tile drops the pin at the finger's lift,
## and the same drag WITHOUT the hold drops nothing. T5 a long-press drag on a placed pin moves it (one undo step, the card does
## not open) and the same drag without the hold only pans the map. T6 Change kind from the card reopens the palette in change mode.
## Phone legs. P1 at the sheet's PEEK detent the tool strip above it shows the recents row (the sheet's own body is hidden there). P2 at half the sheet lists every icon, the chips narrow it to one
## category and All brings it back, search finds by substring, chips/tiles/search/heads are at least 44 dp (115 px at this scale)
## after a rebuild too. P3 tap an icon, then tap the map: the tip lands within 1 px, one undo step. P4 Change kind from the phone
## card opens the sheet at half in change mode (the twelve-kind sheet is gone) and a tile pick is one undo step.
## Landscape phone (`-- --vp 2340x1080 --force-touch --density 2.625`, leg L; the form is the phone, the viewport wider than tall).
## The tool sheet docks on the RIGHT edge instead of the foot and the tool strip (recents) stays on the map's left, so the
## portrait checks that assume "above the sheet" are replaced: L1 the recents sit on screen and clear of the sheet, L2 every icon
## is listed and each tile sits inside the sheet's own width (none clipped sideways), the chips narrow and All restores, search
## finds by substring, and chips/tiles/search/heads are at least 115 px laid out, L3 tap-to-arm then a tap on a map point that no
## control covers drops the pin with its tip within 1 px of the tap, one undo step, L4 Change kind from the card.
## Exit 0 = held, 1 = a check failed, 2 = could not run.

const Pin := preload("res://shell/pin_marker.gd")
const Pal := preload("res://shell/pin_palette.gd")

var app: Node
var _vp: SubViewport
var _bridge
var _ov: Control
var _ws
var _checks := 0
var _fails := 0
var _out := ""
var _tag := "pinpal_touch"
var _legs := "TPL"
var _phone := false
var _last_mouse := Vector2.ZERO


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("PINTOUCH %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _p(s: String) -> void:
	print("PINTOUCH %s" % s)


func _finish() -> void:
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)


# ---- touch events ------------------------------------------------------------------------------------------------

func _host(local: Vector2) -> Vector2:
	return _ov.get_global_transform_with_canvas() * local


## One finger event. `down == null` is motion (with `mask` the held button), otherwise a press or lift. `relative` carries the
## travel since the last event: Godot starts a GUI drag only once it has accumulated past 10 px.
func _touch(at: Vector2, down: Variant = null, mask: int = 0) -> void:
	if down == null:
		var mm := InputEventMouseMotion.new()
		mm.device = InputEvent.DEVICE_ID_EMULATION
		mm.position = at
		mm.global_position = at
		mm.relative = at - _last_mouse
		_last_mouse = at
		mm.button_mask = mask
		_vp.push_input(mm)
		return
	_last_mouse = at
	var ev := InputEventMouseButton.new()
	ev.device = InputEvent.DEVICE_ID_EMULATION
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = bool(down)
	ev.position = at
	ev.global_position = at
	ev.button_mask = MOUSE_BUTTON_MASK_LEFT if bool(down) else 0
	_vp.push_input(ev)


func _tap(at: Vector2) -> void:
	_touch(at, true)
	await _frames(3)
	_touch(at, false)
	await _frames(8)


## A drag with the finger held down `hold_s` seconds first (0 = none), then moved in steps and lifted.
func _drag(from: Vector2, to: Vector2, hold_s: float, steps: int = 10) -> void:
	_touch(from, true)
	await _frames(2)
	if hold_s > 0.0:
		await get_tree().create_timer(hold_s).timeout
		await _frames(2)
	for i in range(1, steps + 1):
		_touch(from.lerp(to, float(i) / float(steps)), null, MOUSE_BUTTON_MASK_LEFT)
		await _frames(1)
	await _frames(2)
	_touch(to, false)
	await _frames(8)


func _undo_n() -> int:
	return (_bridge.undo_ledger() as Array).size()


func _card_open() -> bool:
	var b = app.get("context_broker")
	return b != null and b.card != null and b.card.visible


func _phone_card_open() -> bool:
	var m = app.get("_phone_menu")
	return m != null and m.peek_card_is_open()


func _name_prompt() -> ConfirmationDialog:
	for d in app.find_children("*", "ConfirmationDialog", true, false):
		var dlg := d as ConfirmationDialog
		if dlg.visible and dlg.title == "Name this pin":
			return dlg
	return null


func _dismiss_overlays() -> void:
	var dlg := _name_prompt()
	if dlg != null:
		dlg.canceled.emit()
		dlg.hide()
	await _frames(3)
	var b = app.get("context_broker")
	if b != null and b.card != null and b.card.visible:
		b.card.hide()
	var m = app.get("_phone_menu")
	if m != null and m.peek_card_is_open():
		m.go_back()
	await _frames(4)


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(_out.path_join(_tag + "_" + name))
	_p("shot %s_%s" % [_tag, name])


func _palette() -> Node:
	return app.find_child("PinPalette", true, false)


func _tile(prefix: String, kind: String) -> Button:
	var pal := _palette()
	return pal.find_child(prefix + kind, true, false) as Button if pal != null else null


## The kinds of the tiles under the palette's grid (a chip or a search builds only the categories it shows).
func _grid_kinds() -> Array:
	var out: Array = []
	var pal := _palette()
	if pal == null:
		return out
	var host := pal.find_child("Categories", true, false)
	if host == null:
		return out
	for b in host.find_children("Pin_*", "Button", true, false):
		out.append(String(b.get_meta("pin_kind", "")))
	return out


func _folder_union() -> Array:
	var u := {}
	for set in ["glyph", "painted"]:
		for f in DirAccess.get_files_at("res://icons/pins/%s" % set):
			if f.ends_with(".png"):
				u[f.trim_suffix(".png")] = true
	var out: Array = u.keys()
	out.sort()
	return out


## Scrolls whatever scroller holds `c` so it is on screen, then waits for the layout.
func _reveal(c: Control) -> void:
	## Every scroller up the chain, innermost first: the phone's chips scroll sideways inside the sheet's own vertical scroller.
	var n: Node = c.get_parent()
	while n != null:
		if n is ScrollContainer:
			(n as ScrollContainer).ensure_control_visible(c)
			await _frames(2)
		n = n.get_parent()
	await _frames(4)


func _centre(c: Control) -> Vector2:
	return c.get_global_rect().get_center()


## Free spots on the plate (host coordinates of the pin's tip), clear of settlements, landmarks and pins, with the tip and the
## head on screen and the tip above `max_y` (the phone's sheet covers the foot of the map) and outside every rect of `excl`
## (host coordinates: the landscape sheet, strip and buttons cover the sides, not the foot).
func _free_spots(n: int, max_y: float, excl: Array = []) -> Array:
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	var band: Rect2 = (app.viewport as Control).get_global_rect().grow(-60)
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
			var ht := _host(tip)
			for er in excl:
				if (er as Rect2).has_point(ht):
					near = true
			if not near and interior.has_point(tip) and interior.has_point(head) and band.has_point(ht) and band.has_point(_host(head)) and ht.y < max_y:
				out.append(ht)
			x += step
		y += step
	var picked: Array = []
	if out.is_empty():
		return picked
	var stride := maxi(1, out.size() / n)
	for k in n:
		picked.append(out[mini(k * stride, out.size() - 1)])
	return picked


## Host position of pin `idx`'s tip.
func _tip_host(idx: int) -> Vector2:
	var pd: Dictionary = _bridge.poi_get(idx)
	return _host(_ov._point_to_screen(Vector2(float(pd["x"]), float(pd["y"])), _ov._displayed_rect()))


func _ready() -> void:
	var vp_size := Vector2i(1700, 1000)
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
		elif a == "--tag":
			_tag = String(args[i + 1])
			i += 2
		elif a == "--density":
			i += 2
		elif a == "--force-touch" or a.begins_with("--density="):
			i += 1
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
	_phone = DccTheme.is_phone()
	if not DccTheme.is_touch() or not (_phone or DccTheme.is_tablet()):
		_p("ABORT needs a touch form (--force-touch, a tablet- or phone-sized --vp, --density)")
		get_tree().quit(2)
		return
	_p("form: %s, viewport %s" % ["PHONE" if _phone else "TABLET", str(vp_size)])
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
	app.select_domain("cartography")
	await _frames(8)
	if _phone:
		await _run_phone()
	else:
		await _run_tablet()
	DccSettings.set_marker_style("pins")
	_finish()


# ---- tablet --------------------------------------------------------------------------------------------------------

func _run_tablet() -> void:
	_ok("T0: the tablet form at unit 1 (a dp is a layout pixel)", DccTheme.is_tablet() and is_equal_approx(float(app._unit_factor()), 1.0), str(app._unit_factor()))
	app.arm_tool("inspect")
	await _frames(4)
	app.arm_tool("poi")
	await _frames(8)
	if _legs.contains("T"):
		await _leg_t1()
		await _leg_t2()
		await _leg_t3()
		await _leg_t4()
		await _leg_t5()
		await _leg_t6()


func _leg_t1() -> void:
	print("== T1. every icon, search ==")
	var pal := _palette()
	## The palette is taller than the screen (it scrolls inside the dock): judge its width, its left edge and where it starts.
	var pr: Rect2 = pal.get_global_rect() if pal != null else Rect2()
	_ok("T1: the palette is in the right dock and on screen (starts inside the viewport, fits its width)", pal != null and pal.is_visible_in_tree() and pr.position.x >= 0.0 and pr.end.x <= float(_vp.size.x) and pr.position.y >= 0.0 and pr.position.y < float(_vp.size.y), str(pr))
	var kinds := _grid_kinds()
	var union := _folder_union()
	var sorted: Array = kinds.duplicate()
	sorted.sort()
	_ok("T1: the grid lists every icon of both folders, once each (62)", sorted == union and union.size() == 62, "%d tiles, %d in the folders" % [kinds.size(), union.size()])
	_search("fort")
	await _frames(4)
	var f := _grid_kinds()
	_ok("T1: search 'fort' finds by substring (fort, fortified_pass, fortified_crossing, and nothing unrelated)", f.has("fort") and f.has("fortified_pass") and f.has("fortified_crossing") and not f.has("tavern"), str(f))
	_search("zzzz")
	await _frames(4)
	_ok("T1: a search with no match says so and lists no tile", _grid_kinds().is_empty() and _palette().find_child("NoMatch", true, false) != null)
	_search("")
	await _frames(4)
	_ok("T1: clearing the search lists all 62 again", _grid_kinds().size() == 62)


func _search(t: String) -> void:
	var le := _palette().find_child("PinSearch", true, false) as LineEdit
	le.text = t
	le.text_changed.emit(t)


## The touch floor: every control a finger must hit, as laid out.
func _floor_check(tag: String, min_px: float, ctls: Array) -> void:
	var bad: Array = []
	for c in ctls:
		var r: Rect2 = (c as Control).get_global_rect()
		if r.size.y < min_px - 0.5:
			bad.append("%s %s" % [(c as Control).name, str(r.size)])
	_ok("%s: %d controls, each at least %d px tall as laid out" % [tag, ctls.size(), int(min_px)], bad.is_empty() and not ctls.is_empty(), "; ".join(bad.slice(0, 4)))


func _leg_t2() -> void:
	print("== T2. 44 dp touch targets, laid out ==")
	var pal := _palette()
	_search("")          # a rebuild, so freshly built tiles are measured too
	await _frames(4)
	await _frames(2)
	var tiles: Array = pal.find_children("Pin_*", "Button", true, false)
	var heads: Array = pal.find_children("Cat_*", "Button", true, false)
	_floor_check("T2 tiles", 44.0, tiles)
	var narrow: Array = []
	for t in tiles:
		if (t as Control).get_global_rect().size.x < 43.5:
			narrow.append(t.name)
	_ok("T2: every tile is at least 44 wide", narrow.is_empty(), str(narrow.slice(0, 4)))
	_floor_check("T2 category heads", 44.0, heads)
	var others: Array = [pal.find_child("PinSearch", true, false), pal.find_child("FreeUse", true, false),
		pal.find_child("Mark_other", true, false), pal.find_child("Mark_notice", true, false)]
	_floor_check("T2 search, Use, ?, !", 44.0, others)
	await _shot("tablet_dock.png")


func _drop_free(n: int) -> Array:
	return _free_spots(n, 1.0e9)


func _leg_t3() -> void:
	print("== T3. tap to arm, tap to drop ==")
	var spots := _drop_free(6)
	_ok("T3: free spots exist", spots.size() >= 4, str(spots.size()))
	var tile := _tile("Pin_", "castle")
	await _reveal(tile)
	await _tap(_centre(tile))
	_ok("T3: a tap on a tile arms it (no pin yet)", String(_ws._pin_state["kind"]) == "castle" and _bridge.poi_list().is_empty(), str(_ws._pin_state["kind"]))
	var n0 := _undo_n()
	var at: Vector2 = spots[0]
	await _tap(at)
	var pl: Array = _bridge.poi_list()
	_ok("T3: a tap on the map drops one pin of that kind, one undo step", pl.size() == 1 and String(pl[0]["kind"]) == "castle" and _undo_n() == n0 + 1, "%d pins, %d steps" % [pl.size(), _undo_n() - n0])
	if pl.size() == 1:
		var d := _tip_host(0).distance_to(at)
		_ok("T3: the tip lands within 1 px of the tap (%.2f px)" % d, d < 1.0)
	await _shot("tablet_drop.png")
	await _dismiss_overlays()
	await _shot("tablet_dropped_pin.png")


func _leg_t4() -> void:
	print("== T4. long-press drag from a tile ==")
	var spots := _drop_free(8)
	var tile := _tile("Pin_", "temple")
	await _reveal(tile)
	var n0: int = _bridge.poi_list().size()
	var u0 := _undo_n()
	## Control: the same drag with no hold is a scroll flick, not a drag. Nothing is dropped.
	await _drag(_centre(tile), spots[2], 0.0)
	_ok("T4: without the hold the drag drops nothing", _bridge.poi_list().size() == n0 and _undo_n() == u0, "%d pins" % _bridge.poi_list().size())
	await _dismiss_overlays()
	await _reveal(tile)
	var to: Vector2 = spots[3]
	await _drag(_centre(tile), to, 0.62)
	var pl: Array = _bridge.poi_list()
	_ok("T4: with a long press the drag drops a temple, one undo step", pl.size() == n0 + 1 and String(pl[pl.size() - 1]["kind"]) == "temple" and _undo_n() == u0 + 1, "%d pins, %d steps" % [pl.size() - n0, _undo_n() - u0])
	if pl.size() == n0 + 1:
		var d := _tip_host(pl.size() - 1).distance_to(to)
		_ok("T4: the tip lands within 1 px of the lift (%.2f px)" % d, d < 1.0)
	await _dismiss_overlays()


var _mv_idx := -1

func _leg_t5() -> void:
	print("== T5. long-press drag on a placed pin ==")
	var spots := _drop_free(8)
	_mv_idx = _bridge.poi_add(0.0, 0.0, "tavern", "Mover")
	var at: Vector2 = spots[4]
	var rect: Rect2 = _ov._displayed_rect()
	var g := _grid_of_host(at)
	_bridge.poi_move(_mv_idx, g.x, g.y)
	app.viewport.refresh_annotations()
	await _frames(6)
	_ws._pin_state["kind"] = "tavern"
	var tip := _tip_host(_mv_idx)
	var grab := tip + Vector2(0.0, -10.0)
	var cam0: Vector2 = app.viewport._camera.position
	var u0 := _undo_n()
	## Control: a quick finger drag on the pin pans the map (the Pins tool is a tap tool) and does not move the pin.
	await _drag(grab, grab + Vector2(40.0, 30.0), 0.0)
	_ok("T5: without the hold the drag pans the camera and leaves the pin", (app.viewport._camera.position as Vector2).distance_to(cam0) > 20.0 and _undo_n() == u0, "camera moved %.1f" % (app.viewport._camera.position as Vector2).distance_to(cam0))
	await _dismiss_overlays()
	tip = _tip_host(_mv_idx)
	grab = tip + Vector2(0.0, -10.0)
	var delta := Vector2(36.0, 24.0)
	await _drag(grab, grab + delta, 0.7)
	var tip2 := _tip_host(_mv_idx)
	var d := tip2.distance_to(tip + delta)
	_ok("T5: a long press then drag moves the pin by the drag (%.2f px off)" % d, d < 1.0, "%s -> %s" % [str(tip), str(tip2)])
	_ok("T5: the whole drag is ONE undo step", _undo_n() == u0 + 1, "%d" % (_undo_n() - u0))
	_ok("T5: the hold did not open the card or the ring", not _card_open() and not _phone_card_open())
	_bridge.undo_last()
	await _frames(4)
	_ok("T5: undo puts the pin back", _tip_host(_mv_idx).distance_to(tip) < 1.0)
	await _dismiss_overlays()


## Inverse of `_host(_ov._point_to_screen(...))` at the current camera: the grid point under host position `h`.
func _grid_of_host(h: Vector2) -> Vector2:
	var rect: Rect2 = _ov._displayed_rect()
	var local: Vector2 = _ov.get_global_transform_with_canvas().affine_inverse() * h
	return Vector2((local.x - rect.position.x) / rect.size.x * float(_ov._gw), (local.y - rect.position.y) / rect.size.y * float(_ov._gh))


func _leg_t6() -> void:
	print("== T6. Change kind from the card ==")
	var idx := _mv_idx
	app.open_poi_card(idx)
	await _frames(6)
	var row := _kind_row()
	_ok("T6: the card has one Change kind row, no submenu", not row.is_empty() and not row.has("children"))
	await _dismiss_overlays()
	if row.is_empty():
		return
	(row["callable"] as Callable).call()
	await _frames(8)
	_ok("T6: the palette reopens in change mode (banner and Cancel)", _palette() != null and _palette().find_child("ChangeBanner", true, false) != null and _palette().find_child("ChangeCancel", true, false) != null)
	var u0 := _undo_n()
	var tile := _tile("Pin_", "harbour")
	await _reveal(tile)
	await _tap(_centre(tile))
	_ok("T6: a tile tap changes the kind, one undo step", String(_bridge.poi_get(idx)["kind"]) == "harbour" and _undo_n() == u0 + 1)
	_bridge.undo_last()
	await _frames(4)


func _kind_row() -> Dictionary:
	var b = app.get("context_broker")
	for r in b.last_actions:
		if String((r as Dictionary).get("id", "")) == "el.poi_kind":
			return r
	return {}


# ---- phone ---------------------------------------------------------------------------------------------------------

func _sheet_rect() -> Rect2:
	return (app._phone_tool_sheet as Control).get_global_rect()


func _landscape() -> bool:
	return _vp.size.x > _vp.size.y


func _run_phone() -> void:
	_ok("P0: the phone form", DccTheme.is_phone())
	if _landscape():
		if _legs.contains("L"):
			await _run_landscape()
		return
	if not _legs.contains("P"):
		return
	## Recents are per user and live in the settings; seed three so the sheet's peek has a row to show.
	for k in ["tavern", "harbour", "castle"]:
		DccSettings.pin_recent_push(k)
	app.arm_tool("inspect")
	await _frames(4)
	app.set_phone_detent("peek")
	await _frames(20)
	app.call_arming_from_map(func() -> void: app.arm_tool("poi"))
	await _frames(24)
	await _leg_p1()
	await _leg_p2()
	await _leg_p3()
	await _leg_p4()


## Every visible Control that eats a finger over the map: the sheet, the strip, buttons, the rail, the header. A Control that
## holds the overlay, or is most of the screen, is the map's own ancestry and is skipped.
func _blockers() -> Array:
	var out: Array = []
	var vis := Rect2(Vector2.ZERO, Vector2(_vp.size))
	var stack: Array = [app]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children():
			stack.append(c)
		if not (n is Control) or n == _ov or not (n as Control).is_visible_in_tree():
			continue
		var ctl := n as Control
		if ctl.mouse_filter != Control.MOUSE_FILTER_STOP or ctl.is_ancestor_of(_ov):
			continue
		var r := ctl.get_global_rect()
		if r.size.x * r.size.y > vis.size.x * vis.size.y * 0.5 or r.size.x < 2.0:
			continue
		out.append(r.grow(8.0))
	return out


func _run_landscape() -> void:
	for k in ["tavern", "harbour", "castle"]:
		DccSettings.pin_recent_push(k)
	app.arm_tool("inspect")
	await _frames(4)
	app.set_phone_detent("peek")
	await _frames(20)
	app.call_arming_from_map(func() -> void: app.arm_tool("poi"))
	await _frames(24)
	await _leg_l1()
	await _leg_p2(true)
	await _leg_p3(true)
	await _leg_p4()


func _leg_l1() -> void:
	print("== L1. landscape: the sheet docks right, the strip's recents beside it ==")
	var pal := _palette()
	_ok("L1: the palette is the phone tool sheet's body", pal != null and app._phone_tool_sheet.is_ancestor_of(pal) and pal.is_visible_in_tree())
	var sheet := _sheet_rect()
	_ok("L1: the sheet docks on the right half (x %d of %d), not the foot" % [int(sheet.position.x), _vp.size.x], sheet.position.x > _vp.size.x * 0.4 and sheet.end.x >= _vp.size.x - 80.0)
	var strip := app.find_child("ToolStrip", true, false)
	var rec: Button = strip.find_child("Recent_harbour", true, false) as Button if strip != null else null
	_ok("L1: the strip's recents row has the three recents", rec != null and strip.find_child("Recent_castle", true, false) != null and strip.find_child("Recent_tavern", true, false) != null)
	if rec != null:
		var vis := Rect2(Vector2.ZERO, Vector2(_vp.size))
		for nm in ["castle", "harbour", "tavern"]:
			var tr := (strip.find_child("Recent_" + nm, true, false) as Control).get_global_rect()
			_ok("L1: recent %s is on screen and clear of the sheet (%s vs sheet %s)" % [nm, str(tr), str(sheet)], vis.encloses(tr) and not tr.intersects(sheet))
			var fl := _phone_floor_px()
			_ok("L1: recent %s is at least 115 x 115 px as laid out" % nm, tr.size.x >= fl - 0.5 and tr.size.y >= fl - 0.5, str(tr.size))
		await _tap(_centre(strip.find_child("Recent_harbour", true, false) as Control))
		_ok("L1: a tap on a recent arms that kind", String(_ws._pin_state["kind"]) == "harbour", str(_ws._pin_state["kind"]))
	await _shot("land_sheet_peek.png")


func _leg_p1() -> void:
	print("== P1. the sheet at peek ==")
	var pal := _palette()
	_ok("P1: the palette is the phone tool sheet's body", pal != null and app._phone_tool_sheet.is_ancestor_of(pal) and pal.is_visible_in_tree())
	_ok("P1: the sheet is at its peek detent", String(app._phone_detent) == "peek", str(app._phone_detent))
	var strip := app.find_child("ToolStrip", true, false)
	var rec: Button = strip.find_child("Recent_tavern", true, false) as Button if strip != null else null
	_ok("P1: the strip's recents row has the three recents, newest first (castle, harbour, tavern)", rec != null and strip.find_child("Recent_harbour", true, false) != null and strip.find_child("Recent_castle", true, false) != null)
	if rec != null:
		var row := strip.find_child("PinRecents", true, false)
		var first := ""
		for c in row.get_children():
			if String(c.name).begins_with("Recent_"):
				first = String(c.name)
				break
		_ok("P1: castle is first", first == "Recent_castle", first)
		var tr := rec.get_global_rect()
		var vis := Rect2(Vector2.ZERO, Vector2(_vp.size))
		var sheet := _sheet_rect()
		_ok("P1: the recents sit on screen above the peeked sheet (tile %s, sheet top %d)" % [str(tr), int(sheet.position.y)], vis.encloses(tr) and tr.end.y <= sheet.position.y + 1.0 and rec.is_visible_in_tree())
		var fl := _phone_floor_px()
		_ok("P1: each recent is at least 115 x 115 px as laid out", tr.size.x >= fl - 0.5 and tr.size.y >= fl - 0.5, str(tr.size))
		await _tap(_centre(strip.find_child("Recent_harbour", true, false) as Control))
		_ok("P1: a tap on a recent arms that kind", String(_ws._pin_state["kind"]) == "harbour", str(_ws._pin_state["kind"]))
	await _shot("phone_sheet_peek.png")


func _phone_floor_px() -> float:
	return float(int(round(44.0 * float(app.phone_scale()))))


func _leg_p2(landscape: bool = false) -> void:
	print("== P2. the sheet at half: every icon, chips, search, 115 px targets ==")
	app.set_phone_detent("half")
	await _frames(24)
	var pal := _palette()
	var floor_px := _phone_floor_px()
	_ok("P2: 44 dp is %d px at this scale" % int(floor_px), floor_px >= 100.0)
	var kinds := _grid_kinds()
	var union := _folder_union()
	var sorted: Array = kinds.duplicate()
	sorted.sort()
	_ok("P2: the grid lists every icon of both folders, once each (62)", sorted == union and union.size() == 62, "%d tiles" % kinds.size())
	var chips: Array = pal.find_children("Chip_*", "Button", true, false)
	_ok("P2: nine chips (All and the eight categories)", chips.size() == 9, str(chips.size()))
	## Chips narrow to a category and All brings the rest back.
	await _press_chip("water")
	var w := _grid_kinds()
	_ok("P2: the Water chip narrows the grid to the 10 water pins", w.size() == 10 and w.has("harbour") and not w.has("castle"), str(w.size()))
	await _press_chip("all")
	_ok("P2: the All chip lists all 62 again", _grid_kinds().size() == 62)
	_search("fort")
	await _frames(6)
	var f := _grid_kinds()
	_ok("P2: search 'fort' finds by substring", f.has("fort") and f.has("fortified_pass") and f.has("fortified_crossing") and not f.has("tavern"), str(f))
	_search("zzzz")
	await _frames(4)
	_ok("P2: a search with no match says so", _grid_kinds().is_empty() and _palette().find_child("NoMatch", true, false) != null)
	_search("")
	await _frames(6)
	## Laid-out sizes, after rebuilds (a chip press and two searches), so freshly built nodes are measured.
	await _frames(2)
	pal = _palette()
	_floor_check("P2 chips", floor_px, pal.find_children("Chip_*", "Button", true, false))
	var narrowc: Array = []
	for c in pal.find_children("Chip_*", "Button", true, false):
		if (c as Control).get_global_rect().size.x < floor_px - 0.5:
			narrowc.append(c.name)
	_ok("P2: every chip is at least 115 px wide", narrowc.is_empty(), str(narrowc))
	var tiles: Array = pal.find_children("Pin_*", "Button", true, false)
	_floor_check("P2 tiles", floor_px, tiles)
	var narrow: Array = []
	for t in tiles:
		if (t as Control).get_global_rect().size.x < floor_px - 0.5:
			narrow.append(t.name)
	_ok("P2: every tile is at least 115 px wide", narrow.is_empty(), str(narrow.slice(0, 4)))
	_floor_check("P2 search and category heads", floor_px, [pal.find_child("PinSearch", true, false)] + pal.find_children("Cat_*", "Button", true, false))
	if landscape:
		## Every tile must lie inside the sheet's own width after a reveal: a grid wider than the right-docked sheet clips sideways.
		var sheet := _sheet_rect()
		var clipped: Array = []
		for t in tiles:
			var r := (t as Control).get_global_rect()
			if r.position.x < sheet.position.x - 1.0 or r.end.x > sheet.end.x + 1.0:
				clipped.append("%s %s" % [t.name, str(r)])
		_ok("L2: every tile lies inside the sheet's width (none clipped sideways)", clipped.is_empty(), "; ".join(clipped.slice(0, 3)))
		var last := tiles[tiles.size() - 1] as Control
		await _reveal(last)
		_ok("L2: the last icon scrolls into the sheet's view", (app._phone_tool_scroll as Control).get_global_rect().encloses(last.get_global_rect()), str(last.get_global_rect()))
		await _shot("land_sheet_last.png")
		app._phone_tool_scroll.scroll_vertical = 0
		await _frames(4)
	await _shot("land_sheet_half.png" if landscape else "phone_sheet_half.png")


func _press_chip(id: String) -> void:
	var b := _palette().find_child("Chip_" + id, true, false) as Button
	await _reveal(b)
	await _tap(_centre(b))
	await _frames(4)


func _leg_p3(landscape: bool = false) -> void:
	print("== P3. tap an icon, tap the map ==")
	var tile := _tile("Pin_", "castle")
	await _reveal(tile)
	await _tap(_centre(tile))
	_ok("P3: a tap on a tile arms it (no pin yet)", String(_ws._pin_state["kind"]) == "castle" and _bridge.poi_list().is_empty(), str(_ws._pin_state["kind"]))
	var spots := _free_spots(6, 1.0e9, _blockers()) if landscape else _free_spots(6, _sheet_rect().position.y - 40.0)
	_ok("P3: free spots above the sheet exist", spots.size() >= 3, str(spots.size()))
	if spots.is_empty():
		return
	var at: Vector2 = spots[0]
	var n0 := _undo_n()
	await _tap(at)
	var pl: Array = _bridge.poi_list()
	_ok("P3: a tap on the map drops one pin of that kind, one undo step", pl.size() == 1 and String(pl[0]["kind"]) == "castle" and _undo_n() == n0 + 1, "%d pins, %d steps" % [pl.size(), _undo_n() - n0])
	if pl.size() == 1:
		var d := _tip_host(0).distance_to(at)
		_ok("P3: the tip lands within 1 px of the tap (%.2f px)" % d, d < 1.0)
	await _shot("land_drop_name_prompt.png" if landscape else "phone_drop_name_prompt.png")
	await _dismiss_overlays()
	await _shot("land_dropped_pin.png" if landscape else "phone_dropped_pin.png")
	## The floating undo chip is up now (a drop is undoable); it must not cover a recent.
	var chip := app.get("_phone_undo_chip") as Control
	var strip := app.find_child("ToolStrip", true, false)
	if chip != null and chip.visible and strip != null:
		var over: Array = []
		for r in strip.find_children("Recent_*", "Button", true, false):
			if (r as Control).get_global_rect().intersects(chip.get_global_rect()):
				over.append(r.name)
		_ok("P3: the undo chip (up after the drop) covers no recent pin tile", over.is_empty(), str(over))
	else:
		_ok("P3: the undo chip is up after the drop", false)


func _leg_p4() -> void:
	print("== P4. Change kind from the phone card ==")
	if _bridge.poi_list().is_empty():
		_ok("P4: a pin exists to change", false)
		return
	app.set_phone_detent("peek")
	await _frames(20)
	app.select("poi", 0)
	app.open_poi_card(0)
	await _frames(10)
	var row := _kind_row()
	_ok("P4: the phone card has one Change kind row, no submenu and no twelve-kind sheet", not row.is_empty() and not row.has("children") and not row.has("phone_jump"))
	if row.is_empty():
		return
	await _shot("phone_card.png")
	var chips: Array = app._phone_menu.get("_peek_chip_row").get_children().filter(func(c): return c is Button and not c.is_queued_for_deletion() and (c as Button).text.begins_with("Change kind"))
	_ok("P4: the card shows a Change kind chip of at least 115 px", chips.size() == 1 and (chips[0] as Control).get_global_rect().size.y >= _phone_floor_px() - 0.5, str(chips.size()))
	if chips.size() == 1:
		await _tap(_centre(chips[0] as Control))
	else:
		(row["callable"] as Callable).call()
	await _frames(20)
	_ok("P4: the sheet opens at half", String(app._phone_detent) == "half", str(app._phone_detent))
	var pal := _palette()
	_ok("P4: the palette is in change mode in the sheet (banner and Cancel)", pal != null and app._phone_tool_sheet.is_ancestor_of(pal) and pal.find_child("ChangeBanner", true, false) != null and pal.find_child("ChangeCancel", true, false) != null)
	_ok("P4: the old twelve-kind sheet is not shown", app.find_children("PoiKind_*", "Button", true, false).is_empty())
	var banner := _palette().find_child("ChangeBanner", true, false) as Control
	_ok("P4: the banner is scrolled into view in the sheet", banner != null and (app._phone_tool_scroll as Control).get_global_rect().intersects(banner.get_global_rect()), str(banner.get_global_rect()) if banner != null else "")
	await _shot("phone_change_kind.png")
	var u0 := _undo_n()
	var tile := _tile("Pin_", "temple")
	if tile != null:
		await _reveal(tile)
		await _tap(_centre(tile))
	_ok("P4: a tile tap changes the kind, one undo step", String(_bridge.poi_get(0)["kind"]) == "temple" and _undo_n() == u0 + 1)
	_ok("P4: and ends change mode (no banner)", _palette() == null or _palette().find_child("ChangeBanner", true, false) == null)
	_bridge.undo_last()
	await _frames(4)
	_ok("P4: undo restores the castle", String(_bridge.poi_get(0)["kind"]) == "castle")
