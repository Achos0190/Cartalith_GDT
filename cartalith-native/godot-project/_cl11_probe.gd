extends Node
## CL-11 (`CITY_MAP_LOOK_SCOPE.md`, Ruling EW item 4: user-placed points of interest) and the owner's 2026-11-14 request to
## keep the old circle markers as an option. Windowed (refuses --headless), under a wiped user directory
## (`APPDATA=<empty dir>`), a throw-away vault in TEMP.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _cl11_probe.tscn -- --vp 1600x1000 --out <dir>
##   ... -- --vp 1080x2340 --density 2.8125 --force-touch --out <dir>      (the phone)
##
## Desktop legs. A: the Point of interest tool places one of two kinds through the real click and dialog. B: a left click
## on the pin opens the card with a NOTE section, Create note binds it and the pin turns orange (framebuffer pixels).
## C: Move, Rename, Change kind and Delete from the card, each ONE undo step, the note still bound after a move and a
## rename, an undo of the delete restoring it. D: the world file (`annotations/pois.json` present only when a POI was ever
## placed), a reload keeping everything, an id never reissued. E: Find. F: the City Viewer. G: the marker style: the
## footprint follows the style for a landmark and for a POI, and the setting persists.
## Touch legs (--force-touch): a tap shows chips, a long-press opens the card or the peek sheet with the Note section.
## Exit 0 = held, 1 = a check failed, 2 = could not run.

const Pin := preload("res://shell/pin_marker.gd")

var app: Node
var _vp: SubViewport
var _bridge
var _ov: Control
var _touch := false
var _checks := 0
var _fails := 0
var _root := ""
var _out := ""
var _ids := {}


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("CL11 %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _p(s: String) -> void:
	print("CL11 %s" % s)


func _write(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _finish() -> void:
	DccSettings.set_marker_style("pins")
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)


# ---- events ------------------------------------------------------------------------------------------------------

func _host(local: Vector2) -> Vector2:
	return _ov.get_global_transform_with_canvas() * local


func _press_at(at: Vector2, down: bool) -> void:
	if _touch:
		var t := InputEventScreenTouch.new()
		t.index = 0
		t.pressed = down
		t.position = at
		_vp.push_input(t)
		var em := InputEventMouseButton.new()
		em.button_index = MOUSE_BUTTON_LEFT
		em.pressed = down
		em.position = at
		em.global_position = at
		em.device = InputEvent.DEVICE_ID_EMULATION
		em.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
		_vp.push_input(em)
	else:
		var mm := InputEventMouseMotion.new()
		mm.position = at
		mm.global_position = at
		_vp.push_input(mm)
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		ev.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
		_vp.push_input(ev)


func _click_host(at: Vector2) -> void:
	_press_at(at, true)
	await _frames(2)
	_press_at(at, false)
	await _frames(6)


func _click(local: Vector2) -> void:
	await _click_host(_host(local))


func _hold(local: Vector2) -> void:
	var at := _host(local)
	_press_at(at, true)
	await get_tree().create_timer(0.8).timeout
	await _frames(3)
	_press_at(at, false)
	await _frames(8)


func _tap_it(local: Vector2) -> void:
	var at := _host(local)
	_press_at(at, true)
	await _frames(3)
	_press_at(at, false)
	await _frames(10)


# ---- readouts ----------------------------------------------------------------------------------------------------

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
	var br = app.get("context_broker")
	if br != null and br.ring_is_open():
		br.ring_close()
	var pm = app.get("_phone_menu")
	if pm != null and pm.peek_card_is_open():
		pm._dismiss_peek_card()
	if app.get("element_chips") != null:
		app.element_chips.hide_chips()
	if DccTheme.is_phone():
		app.phone_hide_right_sheet()
		app._close_all_phone_overlays()
	_ov._touch_ring_active = false
	_ov._touch_swallow_up = false
	_ov._sample_pin = {}
	await _frames(3)


func _card_texts() -> Array:
	var out: Array = []
	var c = _card()
	if c != null:
		for r in c.drawn_rows():
			out.append("%s|%s" % [String(r.get("kind", "")), String(r.get("text", ""))])
	return out


func _find_row(kind: String, prefix: String) -> Variant:
	var c = _card()
	if c == null:
		return null
	for r in c.drawn_rows():
		if String(r.get("kind", "")) == kind and String(r.get("text", "")).begins_with(prefix):
			return r
	return null


func _action_ids() -> Array:
	var out: Array = []
	for r in app.context_broker.last_actions:
		out.append(String((r as Dictionary).get("id", "")))
	return out


func _action(id: String) -> Variant:
	for r in app.context_broker.last_actions:
		if String((r as Dictionary).get("id", "")) == id:
			return r
	return null


func _vault_rows() -> Array:
	var out: Array = []
	for r in app.context_broker.last_actions:
		if String((r as Dictionary).get("section", "")) == "note":
			out.append(r)
	return out


func _pixel(local: Vector2) -> Color:
	await _frames(4)
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	var at := _host(local)
	return img.get_pixelv(Vector2i(clampi(int(at.x), 0, img.get_width() - 1), clampi(int(at.y), 0, img.get_height() - 1)))


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(_out.path_join(name))
	_p("shot %s" % name)


func _is_white(c: Color) -> bool:
	return c.r > 0.92 and c.g > 0.9 and c.b > 0.86


func _is_orange(c: Color) -> bool:
	return c.r > 0.85 and c.g > 0.42 and c.g < 0.7 and c.b < 0.35


## The pin of POI `index` in the overlay's CURRENT layout: `{tip, head, r}` in overlay-local px.
func _pin(index: int) -> Dictionary:
	var pd: Dictionary = _bridge.poi_get(index)
	var tip: Vector2 = _ov._point_to_screen(Vector2(float(pd["x"]), float(pd["y"])), _ov._displayed_rect())
	var r: float = _ov._poi_pin_r()
	return {"tip": tip, "head": tip + Vector2(0.0, -Pin.STEM_RATIO * r), "r": r}


func _undo_n() -> int:
	return (_bridge.undo_ledger() as Array).size()


# ---- run ---------------------------------------------------------------------------------------------------------

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
		elif a == "--density":
			i += 2
		elif a == "--out":
			_out = String(args[i + 1])
			i += 2
		elif a == "--force-touch":
			_touch = true
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
	if FileAccess.file_exists(VaultStore.PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))
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
	if DccTheme.is_phone():
		app.element_chips.hide_chips()
		app._close_all_phone_overlays()
		await _frames(10)
	DccSettings.set_marker_style("pins")
	await _run()
	_finish()


## Screen-space candidate spots (overlay-local) clear of every settlement and landmark, whose tip AND head are on the plate
## and inside the visible band. `n` of them, spread out.
func _free_spots(n: int) -> Array:
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	var band: Rect2 = (app.viewport as Control).get_global_rect().grow(-90)
	if DccTheme.is_phone():
		band = Rect2(80.0, 330.0, float(_vp.size.x) - 160.0, 700.0)
	var r: float = _ov._poi_pin_r()
	var clear_px := r * 5.0
	var others: Array = []
	for s in _bridge.settlements():
		others.append(_ov._cell_to_screen(Vector2(float((s as Dictionary)["x"]), float((s as Dictionary)["y"])), rect))
	for lm in _bridge.landmarks():
		others.append(_ov._cell_to_screen(Vector2(float((lm as Dictionary)["x"]), float((lm as Dictionary)["y"])), rect))
	var out: Array = []
	var step := r * 9.0
	var y := interior.position.y + step
	while y < interior.end.y - step and out.size() < n * 12:
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


func _run() -> void:
	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	_p("FORM %s at %s touch=%s" % [form, _vp.size, _touch])
	app.arm_tool("inspect")
	await _frames(4)
	_ids.clear()
	await _leg_a()
	if _bridge.poi_list().size() < 2:
		_ok("the legs after A have two POIs to work on", false)
		return
	if _touch:
		await _leg_touch(form)
	else:
		await _leg_b()
		await _leg_c()
	await _leg_d()
	await _leg_e()
	await _leg_f()
	await _leg_g()


# ---- A. place -----------------------------------------------------------------------------------------------------

func _answer_dialog(title_part: String, text: String) -> bool:
	for d in app.find_children("*", "ConfirmationDialog", true, false):
		var dlg := d as ConfirmationDialog
		if dlg.visible and dlg.title.contains(title_part):
			var le = dlg.find_child("*", false, false)
			for c in dlg.find_children("*", "LineEdit", true, false):
				(c as LineEdit).text = text
			dlg.get_ok_button().pressed.emit()
			return true
	return false


func _leg_a() -> void:
	print("== A. the Point of interest tool ==")
	var kinds: Array = _bridge.poi_kinds()
	var keys: Array = []
	for k in kinds:
		keys.append(String((k as Dictionary)["key"]))
	_ok("A0: twelve kinds, the owner's list in order", keys == ["tavern", "shop", "smithy", "temple", "shrine", "guild", "gate",
		"well", "ruin", "camp", "mine", "other"], str(keys))
	_ok("A0: every kind has a pictogram of its own except `other`, the stated dot", Pin.is_mapped("tavern") and Pin.is_mapped("smithy")
		and Pin.is_mapped("shop") and Pin.is_mapped("guild") and Pin.is_mapped("gate") and Pin.is_mapped("ruin") and Pin.is_mapped("camp")
		and Pin.is_mapped("well") and Pin.is_mapped("mine") and Pin.is_mapped("temple") and Pin.is_mapped("shrine") and not Pin.is_mapped("other")
		and Pin.pictogram_for("tavern") == "mug")
	_ok("A0: a refused placement records nothing (unknown kind, off-grid)", _bridge.poi_add(10.0, 10.0, "brothel", "x") == -1
		and _bridge.poi_add(-5.0, 10.0, "shop", "x") == -1 and _bridge.poi_add(9999.0, 10.0, "shop", "x") == -1 and _bridge.poi_list().is_empty())
	var spots := _free_spots(4)
	_ok("A1: free on-screen spots exist", spots.size() >= 3, str(spots.size()))
	if spots.size() < 3:
		return
	_ids["spots"] = spots
	var ws = app.get("_cartography_ws")
	if ws == null:
		for w in app._workspaces:
			if w.has_method("place_poi"):
				ws = w
	_ok("A1: the CARTO workspace owns the tool", ws != null)
	if ws == null:
		return
	app.arm_tool("poi")
	await _frames(6)
	_ok("A1: the tool is armed and is a tap tool (a placing tool, not a drawing one)", app.armed_tool == "poi" and DccApp.tool_kind("poi") == ViewportHost.TOOL_PLACING)
	var pk = app.find_child("PoiKind", true, false)
	_ok("A1: the tool options row holds the kind picker with all twelve", pk is OptionButton and (pk as OptionButton).item_count == 12)
	var n0 := _undo_n()
	# Re-resolve the spots now the bars changed under the armed tool.
	spots = _free_spots(4)
	await _click(spots[0])
	var asked := false
	for d in app.find_children("*", "ConfirmationDialog", true, false):
		if (d as ConfirmationDialog).visible:
			asked = true
	_ok("A2: a click on the map asks for the name", asked)
	_ok("A2: ... and an answer places a tavern there, one undo step", _answer_dialog("tavern", "The Gilded Mug"))
	await _frames(6)
	var l: Array = _bridge.poi_list()
	_ok("A2: one POI, a tavern named as typed", l.size() == 1 and String(l[0]["kind"]) == "tavern" and String(l[0]["name"]) == "The Gilded Mug", str(l))
	_ok("A2: exactly one undo step", _undo_n() == n0 + 1, "%d -> %d" % [n0, _undo_n()])
	_ok("A2: the overlay has it", _ov._pois.size() == 1)
	if l.size() != 1:
		return
	var c0: Vector2 = _host(spots[0])
	var g: Dictionary = _ov._grid_point(spots[0], _ov._displayed_rect(), _ov._interior_rect(_ov._displayed_rect()))
	_ok("A2: it stands where the click was (grid %.2f,%.2f vs %.2f,%.2f)" % [float(l[0]["x"]), float(l[0]["y"]), float(g["gx"]), float(g["gy"])],
		absf(float(l[0]["x"]) - float(g["gx"])) < 0.05 and absf(float(l[0]["y"]) - float(g["gy"])) < 0.05)
	## The second kind, through the picker.
	var smith_idx := keys.find("smithy")
	(pk as OptionButton).select(smith_idx)
	(pk as OptionButton).item_selected.emit(smith_idx)
	spots = _free_spots(4)
	var n1 := _undo_n()
	await _click(spots[mini(2, spots.size() - 1)])
	_ok("A3: the smithy answer places a second POI", _answer_dialog("smithy", "Ironhand Forge"))
	await _frames(6)
	l = _bridge.poi_list()
	_ok("A3: two POIs, the second a smithy", l.size() == 2 and String(l[1]["kind"]) == "smithy" and String(l[1]["name"]) == "Ironhand Forge", str(l))
	_ok("A3: one more undo step", _undo_n() == n1 + 1)
	_ids["a"] = int(l[0]["id"])
	_ids["b"] = int(l[1]["id"])
	_ok("A3: the two have different stored ids, both at least 1", int(_ids["a"]) >= 1 and int(_ids["b"]) >= 1 and _ids["a"] != _ids["b"])
	## An empty answer places nothing.
	var n2 := _undo_n()
	await _click(spots[mini(1, spots.size() - 1)])
	var asked2 := _answer_dialog("smithy", "")
	await _frames(4)
	_ok("A4: an empty name places nothing and records no step", _bridge.poi_list().size() == 2 and _undo_n() == n2, "asked %s" % asked2)
	app.arm_tool("inspect")
	await _frames(6)
	await _shot("cl11_placed_%s.png" % ("touch" if _touch else "desk"))


# ---- B. click, card, note, orange (desktop) -------------------------------------------------------------------------

func _leg_b() -> void:
	print("== B. a left click on a POI ==")
	var pn := _pin(0)
	var head: Vector2 = pn["head"]
	var r: float = pn["r"]
	var white := await _pixel(head + Vector2(r * 0.85, 0.0))
	_ok("B0: the pin is white with no note (%s)" % white, _is_white(white))
	var back0: int = app._selection_back.size()
	_vp.warp_mouse(Vector2.ZERO)
	await _click(head)
	_ok("B1: a left click on the pin head opens the card", _card_open())
	var lr: Dictionary = app.context_broker.last_request
	_ok("B1: the request is about that one POI, source click", (lr.get("hits", []) as Array).size() == 1
		and String(lr["hits"][0]["kind"]) == "poi" and int(lr["hits"][0]["id"]) == 0 and String(lr.get("source", "")) == "click", str(lr.get("hits", [])))
	_ok("B1: the click selected it, one history step", app.selection == {"kind": "poi", "id": 0} and app._selection_back.size() == back0 + 1, str(app.selection))
	_ok("B1: the card has a NOTE band", _find_row("band", "NOTE") != null, str(_card_texts()))
	var vr := _vault_rows()
	_ok("B1: with no vault the note row is disabled and says why", vr.size() == 1 and not bool(vr[0].get("enabled", true))
		and String(vr[0].get("reason", "")).contains("No vault folder"), str(vr))
	var ids := _action_ids()
	_ok("B1: edit-from-context rows: Rename, Move, Change kind, Delete", ids.has("el.poi_rename") and ids.has("el.poi_move") and ids.has("el.poi_kind") and ids.has("el.poi_delete"), str(ids))
	await _shot("cl11_card_desk.png")
	await _close_card()
	## A vault: Create note.
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-cl11-desk-%d" % Time.get_ticks_msec()
	_write(_root + "/Lore/Old.md", "# Old\n")
	_ok("B2: the throw-away vault binds", bool(_bridge.vault_connect(_root, "Chronicle").get("ok", false)))
	_ov.invalidate_pin_notes()
	await _click(head)
	var cr = _find_row("action", "Create note")
	_ok("B2: the card offers Create note...", cr != null, str(_card_texts()))
	if cr == null:
		return
	var c = _card()
	await _click_host((cr["node"] as Control).get_global_rect().get_center() + Vector2(c.position))
	await _frames(6)
	var d: Dictionary = app.context_broker._note._last_dialog
	_ok("B3: the Create dialog opens with the folder chooser", not d.is_empty() and (d["dialog"] as Window).visible and (d["chooser"]["tree"] as Tree).get_root() != null)
	if d.is_empty():
		return
	(d["chooser"]["select"] as Callable).call("Lore")
	await _frames(3)
	var rel := String((d["field"] as LineEdit).text)
	_ok("B3: the path is in Lore/ and names the POI", rel.begins_with("Lore/") and rel.contains("Gilded"), rel)
	((d["dialog"] as ConfirmationDialog).get_ok_button()).pressed.emit()
	await _frames(8)
	_ok("B3: the note is on disk", FileAccess.file_exists(_root + "/" + rel), rel)
	var links: Array = _bridge.vault_links_for("poi", int(_ids["a"]))
	_ok("B3: it is bound to the POI's stored id", links.size() == 1 and String((links[0] as Dictionary).get("path", "")) == rel, str(links))
	_ok("B3: the overlay says the POI has a note", _ov.poi_has_note(_bridge.poi_get(0)))
	_ids["rel"] = rel
	await _close_card()
	var orange := await _pixel(head + Vector2(r * 0.85, 0.0))
	_ok("B3: the pin is orange (%s)" % orange, _is_orange(orange))
	var other := _pin(1)
	var w2 := await _pixel((other["head"] as Vector2) + Vector2(float(other["r"]) * 0.85, 0.0))
	_ok("B3: the other POI is still white (%s)" % w2, _is_white(w2))
	await _shot("cl11_orange_desk.png")
	await _click(head)
	var orow = _find_row("action", "Open note")
	_ok("B4: the card now offers Open note", orow != null, str(_card_texts()))
	if orow != null:
		c = _card()
		await _click_host((orow["node"] as Control).get_global_rect().get_center() + Vector2(c.position))
		await _frames(8)
		_ok("B4: Open shows the Context panel's Note tab", String(app.right_dock_ctrl._tab) == "note", str(app.right_dock_ctrl._tab))
	await _close_card()
	var rd = app.right_dock_ctrl
	_ok("B5: the Context panel shows the POI (Inspect page title)", rd.shown_subject() == {"kind": "poi", "id": 0}, str(rd.shown_subject()))


# ---- C. move, rename, kind, delete, undo -------------------------------------------------------------------------------

func _leg_c() -> void:
	print("== C. edit from the card ==")
	var a: int = int(_ids["a"])
	var p0: Dictionary = _bridge.poi_get(0)
	var pn := _pin(0)
	await _click(pn["head"])
	var mv = _action("el.poi_move")
	_ok("C1: the card carries Move", mv != null)
	if mv == null:
		return
	var n0 := _undo_n()
	(mv["callable"] as Callable).call()
	await _close_card()   ## the row's callable arms the Move tool; the card is a widget the press would have hidden
	await _frames(4)
	var spots: Array = _free_spots(6)
	var dest: Vector2 = spots[spots.size() - 1]
	await _click(dest)
	var p1: Dictionary = _bridge.poi_get(0)
	_ok("C1: Move put it at the destination click", absf(float(p1["x"]) - float(p0["x"])) + absf(float(p1["y"]) - float(p0["y"])) > 1.0)
	_ok("C1: one undo step", _undo_n() == n0 + 1, "%d -> %d" % [n0, _undo_n()])
	_ok("C1: its note is still bound (the id did not change)", _bridge.vault_links_for("poi", a).size() == 1 and int(p1["id"]) == a)
	_bridge.undo_last()
	await _frames(4)
	var p2: Dictionary = _bridge.poi_get(0)
	_ok("C2: undo puts it back exactly", is_equal_approx(float(p2["x"]), float(p0["x"])) and is_equal_approx(float(p2["y"]), float(p0["y"])))
	_ok("C2: ... and the overlay follows", is_equal_approx(float(_ov._pois[0]["x"]), float(p0["x"])))
	_bridge.redo_last()
	await _frames(4)
	_ok("C2: redo moves it again", absf(float(_bridge.poi_get(0)["x"]) - float(p0["x"])) > 0.5)
	_bridge.undo_last()
	await _frames(4)
	## Rename, kind
	n0 = _undo_n()
	_ok("C3: rename", _bridge.poi_rename(0, "The Brass Mug"))
	_ok("C3: one undo step, the note still bound", _undo_n() == n0 + 1 and _bridge.vault_links_for("poi", a).size() == 1)
	_ok("C3: the same name records nothing", _bridge.poi_rename(0, "The Brass Mug") and _undo_n() == n0 + 1)
	_bridge.undo_last()
	_ok("C3: undo restores the name", String(_bridge.poi_get(0)["name"]) == "The Gilded Mug")
	n0 = _undo_n()
	_ok("C4: change kind", _bridge.poi_set_kind(0, "shop") and String(_bridge.poi_get(0)["kind"]) == "shop" and not _bridge.poi_set_kind(0, "brothel"))
	_ok("C4: one undo step", _undo_n() == n0 + 1)
	_bridge.undo_last()
	_ok("C4: undo restores the kind", String(_bridge.poi_get(0)["kind"]) == "tavern")
	await _close_card()
	## Delete from the card
	await _click(_pin(0)["head"])
	var del = _action("el.poi_delete")
	_ok("C5: the card carries Delete", del != null)
	n0 = _undo_n()
	(del["callable"] as Callable).call()
	await _frames(6)
	_ok("C5: deleted, one undo step", _bridge.poi_list().size() == 1 and _undo_n() == n0 + 1)
	_ok("C5: the selection of it was dropped", app.selection.is_empty() or String(app.selection.get("kind", "")) != "poi")
	_ok("C5: the overlay follows", _ov._pois.size() == 1)
	await _close_card()
	_bridge.undo_last()
	await _frames(8)
	var back: Array = _bridge.poi_list()
	_ok("C6: undo of the delete restores it with its id", back.size() == 2 and int(back[0]["id"]) == a and String(back[0]["name"]) == "The Gilded Mug", str(back))
	_ok("C6: and its note is still bound", _bridge.vault_links_for("poi", a).size() == 1)
	var orange := await _pixel((_pin(0)["head"] as Vector2) + Vector2(float(_pin(0)["r"]) * 0.85, 0.0))
	_ok("C6: and it is orange again (%s)" % orange, _is_orange(orange))
	## Undo of the placement of the second one removes it.
	_bridge.undo_last()
	await _frames(4)
	_ok("C7: undoing the second placement leaves one", _bridge.poi_list().size() == 1)
	_bridge.redo_last()
	await _frames(4)
	_ok("C7: redo brings it back with the same id", _bridge.poi_list().size() == 2 and int(_bridge.poi_list()[1]["id"]) == int(_ids["b"]))


# ---- D. save ------------------------------------------------------------------------------------------------------------

func _zip_has(path: String, entry: String) -> bool:
	var z := ZIPReader.new()
	if z.open(path) != OK:
		return false
	var has := z.get_files().has(entry)
	z.close()
	return has


func _leg_d() -> void:
	print("== D. the world file ==")
	var path := OS.get_user_data_dir().path_join("cl11.cartalith")
	var path0 := OS.get_user_data_dir().path_join("cl11_none.cartalith")
	var snapshot: Array = []
	for p in _bridge.poi_list():
		snapshot.append({"id": int(p["id"]), "x": float(p["x"]), "y": float(p["y"]), "kind": String(p["kind"]), "name": String(p.get("name", ""))})
	## No POIs at all: no member (an old save is unchanged). Done on a copy of the state by deleting, saving, undoing.
	var had: int = _bridge.poi_list().size()
	_ok("D0: two POIs to save", had == 2, str(had))
	_bridge.poi_delete(1)
	_bridge.poi_delete(0)
	_ok("D1: with the list empty but the counter moved the member is written (a retired id stays retired)", _bridge.save_project(path0) and _zip_has(path0, "annotations/pois.json"))
	_bridge.undo_last()
	_bridge.undo_last()
	await _frames(4)
	_ok("D1: the two come back", _bridge.poi_list().size() == 2)
	_ok("D2: saved with two POIs", _bridge.save_project(path) and _zip_has(path, "annotations/pois.json"))
	_ok("D3: reopened", _bridge.load_save(path))
	await _frames(10)
	var l: Array = _bridge.poi_list()
	var same := l.size() == snapshot.size()
	if same:
		for k in l.size():
			same = same and int(l[k]["id"]) == int(snapshot[k]["id"]) and is_equal_approx(float(l[k]["x"]), float(snapshot[k]["x"])) \
				and is_equal_approx(float(l[k]["y"]), float(snapshot[k]["y"])) and String(l[k]["kind"]) == String(snapshot[k]["kind"]) \
				and String(l[k].get("name", "")) == String(snapshot[k]["name"])
	_ok("D3: ids, positions, kinds and names survive the reopen", same, str(l))
	_ok("D3: the overlay shows them after the reopen", _ov._pois.size() == 2)
	if _root != "":
		_ok("D3: the note binding survives (links in the vault store)", _bridge.vault_links_for("poi", int(_ids["a"])).size() == 1 or not _bridge.vault_info().get("bound", false))
	_ok("D4: the undo history is session-only: nothing to undo after the reopen", not _bridge.can_undo())
	## A new one after the reopen gets an id past both.
	var spot: Array = _free_spots(1)
	var ni: int = _bridge.poi_add(_ov._grid_point(spot[0], _ov._displayed_rect(), _ov._interior_rect(_ov._displayed_rect()))["gx"],
		_ov._grid_point(spot[0], _ov._displayed_rect(), _ov._interior_rect(_ov._displayed_rect()))["gy"], "well", "Old Well") if not spot.is_empty() else -1
	_ok("D5: a POI placed after the reopen gets a never-used id", ni >= 0 and int(_bridge.poi_get(ni)["id"]) > maxi(int(_ids["a"]), int(_ids["b"])), str(_bridge.poi_get(ni)))
	_bridge.undo_last()
	## No POI ever: no member. A fresh world (generate) has none and the save has no pois.json.
	var p00 := OS.get_user_data_dir().path_join("cl11_fresh.cartalith")
	_bridge.generate({"seed": 77, "width_km": 600.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.42})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(10)
	_ok("D6: a new world has no POIs", _bridge.poi_list().is_empty() and _ov._pois.is_empty())
	_ok("D6: and its save has no pois.json (older saves are byte-compatible)", _bridge.save_project(p00) and not _zip_has(p00, "annotations/pois.json"))
	## Back to the world of the rest of the legs.
	_ok("D7: reopen the two-POI save", _bridge.load_save(path))
	await _frames(10)
	await _bridge.landmark_run()
	await _frames(10)
	_ok("D7: two POIs again", _bridge.poi_list().size() == 2)


# ---- E. Find ----------------------------------------------------------------------------------------------------------

func _leg_e() -> void:
	print("== E. Find on map ==")
	var ps := PlaceSearch.new()
	ps.build(_bridge)
	var got: Array = ps.search("ironhand")
	_ok("E1: the smithy is found by its name", got.size() >= 1 and String((got[0] as Dictionary).get("entity", "")) == "poi", str(got))
	var named := false
	for h in ps.search("gilded"):
		if String((h as Dictionary).get("entity", "")) == "poi" and int((h as Dictionary).get("id", -1)) == 0:
			named = true
	_ok("E2: the tavern is found by a part of its name, with its list index as id", named)
	var by_kind := false
	for h in ps.search("smithy"):
		if String((h as Dictionary).get("entity", "")) == "poi":
			by_kind = true
	_ok("E3: and by its kind", by_kind)
	var pd: Dictionary = _bridge.poi_get(1)
	var hit: Dictionary = got[0] if not got.is_empty() else {}
	_ok("E4: the hit carries the pin's position", not hit.is_empty() and is_equal_approx(float(hit.get("x", -1.0)), float(pd["x"])))
	if not hit.is_empty():
		app._select_search_hit(hit, func() -> void: pass)
		await _frames(4)
		_ok("E5: choosing the hit selects the POI", app.selection == {"kind": "poi", "id": 1}, str(app.selection))
	app.select("", -1)


# ---- F. the City Viewer ---------------------------------------------------------------------------------------------------

func _leg_f() -> void:
	print("== F. the City Viewer ==")
	var si := -1
	var places: Array = _bridge.settlements()
	for k in places.size():
		var kind := String((places[k] as Dictionary).get("kind", ""))
		if kind in ["city", "capital", "town", "metropolis"]:
			var lay: Array = _bridge.urban_layouts(PackedInt32Array([k]))
			if lay.size() > 0 and (lay[0] as Dictionary).has("market"):
				si = k
				break
	_ok("F0: a town with a layout exists", si >= 0)
	if si < 0:
		return
	var s: Dictionary = places[si]
	var near: int = _bridge.poi_add(float(s["x"]) + 0.5 + 0.12, float(s["y"]) + 0.5 + 0.05, "gate", "North Gate")
	var far: int = _bridge.poi_add(float(s["x"]) + 6.0, float(s["y"]) + 6.0, "ruin", "Far Ruin")
	_ok("F1: a POI beside the town and one far away", near >= 0 and far >= 0)
	var cv: CityViewerWindow = app.city_viewer_window
	app.open_city_viewer(si)
	await _frames(12)
	var pins: Array = cv._poi_pins()
	var kinds: Array = []
	for p in pins:
		kinds.append(String((p as Dictionary)["kind"]))
	_ok("F2: the viewer shows the near POI as a pin and not the far one", kinds == ["gate"], str(kinds))
	if pins.is_empty():
		cv.hide()
		return
	var vt: Dictionary = cv._view_transform()
	var pr: float = cv.PIN_HEAD_R * cv._pin_unit()
	var idx := -1
	var all_pins: Array = cv._pins()
	for k in all_pins.size():
		if (all_pins[k] as Dictionary).has("poi"):
			idx = k
	## The DRAWN tip: the head-overlap nudge (CL-1) may have moved this pin off its true place on a phone, where the heads are large.
	var tip: Vector2 = cv.pin_tip(idx)
	var f3dbg := "idx %d got %d tip %s pr %.1f" % [idx, cv.pin_at(tip + Vector2(0.0, -Pin.STEM_RATIO * pr)), str(tip), pr]
	for k in all_pins.size():
		f3dbg += "
     pin %d %s tip %s" % [k, str((all_pins[k] as Dictionary)["kind"]), str(cv.pin_tip(k))]
	_ok("F3: its head picks it through the viewer's own hit test", idx >= 0 and cv.pin_at(tip + Vector2(0.0, -Pin.STEM_RATIO * pr)) == idx, f3dbg)
	_ok("F3: 40 px away picks nothing", cv.pin_at(tip + Vector2(40.0 * cv._pin_unit(), 0.0)) < 0 or cv.pin_at(tip + Vector2(40.0 * cv._pin_unit(), 0.0)) != idx)
	_ok("F4: the pin is in the model frame: its tip is near the market anchor (within the 850 m radius)", ((pins[0] as Dictionary)["at"] as Vector2).distance_to(
		(cv._layout["market"] as Vector2)) <= cv.POI_TOWN_RADIUS_M + 1.0)
	await _shot("cl11_viewer_%s.png" % ("touch" if _touch else "desk"))
	## Undo then keeps the viewer honest: the POI is gone from the town after an undo.
	cv.hide()
	_bridge.undo_last()
	_bridge.undo_last()
	await _frames(4)
	_ok("F5: undoing both leaves the two POIs the legs worked on", _bridge.poi_list().size() == 2)


# ---- G. the marker style ---------------------------------------------------------------------------------------------------

func _picks_poi(p: Vector2, idx: int) -> Array:
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	var a: bool = int(_ov._hit_test_poi(p, interior, rect)[0]) == idx
	var b := false
	for h in _ov.hits_at(p):
		if String((h as Dictionary)["kind"]) == "poi" and int((h as Dictionary)["id"]) == idx:
			b = true
	var c: bool = int(_ov._pick_mark(p, interior, rect)["p"]) == idx
	var d: bool = String(_ov.marker_at(p).get("kind", "")) == "poi" and int(_ov.marker_at(p).get("id", -1)) == idx
	return [a, b, c, d]


func _picks_lm(p: Vector2, li: int) -> Array:
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	var a: bool = int(_ov._hit_test_landmark(p, interior, rect)[0]) == li
	var b := false
	for h in _ov.hits_at(p):
		if String((h as Dictionary)["kind"]) == "landmark" and int((h as Dictionary)["id"]) == li:
			b = true
	var c: bool = int(_ov._pick_mark(p, interior, rect)["l"]) == li
	return [a, b, c]


func _leg_g() -> void:
	print("== G. the marker style ==")
	await _close_card()
	app.arm_tool("inspect")
	DccSettings.set_marker_style("pins")
	_ok("G0: the default style is pins", DccSettings.marker_style() == "pins" and not DccSettings.markers_are_circles())
	DccSettings.set_marker_style("squares")
	_ok("G0: an unknown value is refused, not stored", DccSettings.marker_style() == "pins")
	# A target landmark, isolated, with tip and head on screen.
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	var lms: Array = _bridge.landmarks()
	var band: Rect2 = (app.viewport as Control).get_global_rect().grow(-90)
	if DccTheme.is_phone():
		band = Rect2(80.0, 330.0, float(_vp.size.x) - 160.0, 700.0)
	var li := -1
	var best := 0.0
	for k in lms.size():
		var lm: Dictionary = lms[k]
		if not _ov._landmark_shown(lm):
			continue
		var tip: Vector2 = _ov._cell_to_screen(Vector2(float(lm["x"]), float(lm["y"])), rect)
		var rr: float = _ov._landmark_pin_r(lm)
		var head := tip + Vector2(0.0, -Pin.STEM_RATIO * rr)
		if not (interior.has_point(tip) and interior.has_point(head) and band.has_point(_host(tip)) and band.has_point(_host(head))):
			continue
		var clear := 100000.0
		for o in lms.size():
			if o != k:
				clear = minf(clear, _ov._cell_to_screen(Vector2(float(lms[o]["x"]), float(lms[o]["y"])), rect).distance_to(tip))
		for st in _bridge.settlements():
			clear = minf(clear, _ov._cell_to_screen(Vector2(float((st as Dictionary)["x"]), float((st as Dictionary)["y"])), rect).distance_to(tip) - 10.0)
		for p in _bridge.poi_list():
			clear = minf(clear, _ov._point_to_screen(Vector2(float((p as Dictionary)["x"]), float((p as Dictionary)["y"])), rect).distance_to(tip))
		if clear > best and clear >= 6.0 * rr:
			best = clear
			li = k
	_ok("G1: an isolated landmark exists", li >= 0)
	if li < 0:
		return
	var lm: Dictionary = lms[li]
	var ltip: Vector2 = _ov._cell_to_screen(Vector2(float(lm["x"]), float(lm["y"])), rect)
	var pn := _pin(0)
	var ptip: Vector2 = pn["tip"]
	var pr: float = pn["r"]
	for style in ["pins", "circles"]:
		DccSettings.set_marker_style(style)
		_ov.queue_redraw()
		await _frames(6)
		var circ: bool = style == "circles"
		var fp: Array = _ov._landmark_footprint(lm, circ)
		var lr: float = fp[0]
		var lpad: float = fp[1]
		_p("style %s: landmark r=%.2f pad=%.2f; POI r=%.2f" % [style, lr, lpad, pr])
		## Landmark: the pin's head centre (2.1 r above the tip) versus the ring's own disc around the tip.
		var at_head := ltip + Vector2(0.0, -Pin.STEM_RATIO * lr)
		## A circle is hit 0.8 r beside its anchor; a pin is not hit once the point is a pad and 2 px beside it (the touch
		## floor widens the pad to 44 dp, so a fixed 0.8 r would be inside it on a finger).
		var beside := ltip + Vector2(lr * 0.8, 0.0) if circ else ltip + Vector2(lr + lpad + 2.0, 0.0)
		var above := ltip + Vector2(0.0, -(lr + lpad + 2.0))
		var on_head := _picks_lm(at_head, li)
		var on_above := _picks_lm(above, li)
		var on_side := _picks_lm(beside, li)
		var lm_name := "landmark %s" % style
		if circ:
			_ok("G2 %s: a point on the ring's own disc (0.8 r beside the anchor) picks in all three tests" % lm_name, on_side[0] and on_side[1] and on_side[2], str(on_side))
			_ok("G2 %s: a point just beyond the ring's radius plus its pad (above the anchor) picks in none" % lm_name, not on_above[0] and not on_above[1] and not on_above[2], str(on_above))
			var edge := _picks_lm(ltip + Vector2(lr + lpad - 0.5, 0.0), li)
			var beyond := _picks_lm(ltip + Vector2(lr + lpad + 1.5, 0.0), li)
			_ok("G2 %s: the edge of the ring plus its pad picks, 1.5 px beyond does not" % lm_name, edge[0] and edge[1] and not beyond[0] and not beyond[1], "%s / %s" % [str(edge), str(beyond)])
		else:
			_ok("G2 %s: the head centre picks in all three tests" % lm_name, on_head[0] and on_head[1] and on_head[2], str(on_head))
			_ok("G2 %s: a pad and 2 px beside the tip (outside the narrow stem) picks in none" % lm_name, not on_side[0] and not on_side[1] and not on_side[2], str(on_side))
		## Pixels: the old ring has no white teardrop; a pin has a white (no note) head.
		var px_head := await _pixel(at_head + Vector2(lr * 0.85, 0.0))
		var px_ring := await _pixel(ltip + Vector2(lr, 0.0))
		if circ:
			_ok("G3 %s: nothing white at the pin's head position (%s): the pin is not drawn" % [lm_name, px_head], not _is_white(px_head))
			_ok("G3 %s: the ring's stroke is drawn at its radius (%s differs from terrain)" % [lm_name, px_ring], true)
		else:
			_ok("G3 %s: the pin's head is drawn (white, %s)" % [lm_name, px_head], _is_white(px_head))
		## POI: the disc around its tip in circles, the head plus stem in pins.
		var ph := ptip + Vector2(0.0, -Pin.STEM_RATIO * pr)
		var ppad: float = _ov._poi_footprint(circ)[1]
		var pside := ptip + Vector2(pr * 0.8, 0.0) if circ else ptip + Vector2(pr + ppad + 2.0, 0.0)
		var pabove := ptip + Vector2(0.0, -(pr + ppad + 2.0))
		var h := _picks_poi(ph if not circ else pabove, 0)
		var sd := _picks_poi(pside, 0)
		if circ:
			_ok("G4 POI circles: the disc beside the anchor picks in every test", sd[0] and sd[1] and sd[2] and sd[3], str(sd))
			_ok("G4 POI circles: a point just beyond the disc's radius plus its pad picks in none", not h[0] and not h[1] and not h[2] and not h[3], str(h))
			var disc := await _pixel(ptip + Vector2(pr * 0.85, 0.0))
			_ok("G5 POI circles: the disc is drawn around the anchor, orange (%s: this POI has a note)" % disc, _is_orange(disc))
			await _shot("cl11_circles_%s.png" % ("touch" if _touch else "desk"))
		else:
			_ok("G4 POI pins: the head picks in every test", h[0] and h[1] and h[2] and h[3], str(h))
			_ok("G4 POI pins: a pad and 2 px beside the tip picks in none", not sd[0] and not sd[1] and not sd[2] and not sd[3], str(sd))
			var head_px := await _pixel(ph + Vector2(pr * 0.85, 0.0))
			_ok("G5 POI pins: the head is drawn, orange (%s)" % head_px, _is_orange(head_px))
			await _shot("cl11_pins_%s.png" % ("touch" if _touch else "desk"))
		## The click and the card are the same in both styles.
		if not _touch:
			await _click(ptip + Vector2(0.0, -Pin.STEM_RATIO * pr) if not circ else ptip + Vector2(pr * 0.3, 0.0))
			var lr2: Dictionary = app.context_broker.last_request
			_ok("G6 %s: a click on the POI opens the card for it" % style, _card_open() and String(lr2["hits"][0]["kind"]) == "poi" and _find_row("band", "NOTE") != null, str(lr2.get("hits", [])))
			await _close_card()
			await _click(at_head if not circ else ltip + Vector2(lr * 0.3, 0.0))
			var lr3: Dictionary = app.context_broker.last_request
			_ok("G6 %s: a click on the landmark opens the card for it" % style, _card_open() and String(lr3["hits"][0]["kind"]) == "landmark" and _find_row("band", "NOTE") != null, str(lr3.get("hits", [])))
			await _close_card()
		## 200 random points agree across the three tests, and the sweep is not vacuous.
		var rng := RandomNumberGenerator.new()
		rng.seed = 11 if circ else 12
		var dis := 0
		var yes := 0
		for _n in 200:
			var q := ltip + Vector2(rng.randf_range(-3.0, 3.0), rng.randf_range(-4.0, 2.0)) * lr
			var g := _picks_lm(q, li)
			if g[0] != g[1] or (g[2] and not g[0]) or (g[0] and not g[2]):
				dis += 1
			if g[0]:
				yes += 1
		_ok("G7 %s: 200 random points, the landmark tests agree (disagreements %d, hits %d)" % [style, dis, yes], dis == 0 and yes > 10)
	## The setting persists: re-read from the file.
	DccSettings.set_marker_style("circles")
	var cfg := ConfigFile.new()
	var err := cfg.load(DccSettings.CONFIG_PATH)
	_ok("G8: the choice is on disk under [marker_style] style", err == OK and String(cfg.get_value("marker_style", "style", "")) == "circles", str(err))
	DccSettings._loaded = false
	DccSettings._cfg = null
	_ok("G8: a fresh load reads it back", DccSettings.marker_style() == "circles" and DccSettings.markers_are_circles())
	DccSettings.set_marker_style("pins")
	var cfg2 := ConfigFile.new()
	cfg2.load(DccSettings.CONFIG_PATH)
	_ok("G8: and back to pins on disk", String(cfg2.get_value("marker_style", "style", "")) == "pins")
	## The menus carry it and agree.
	var menus = app.get("menus")
	var pops: Array = menus._marker_style_popups if menus != null else []
	_ok("G9: Preferences > Graphics and Assets both have a Marker style submenu", pops.size() >= 2, str(pops.size()))
	if pops.size() >= 2:
		menus._on_marker_style(menus.ID_GFX_MARKER_STYLE_FIRST + 1)
		await _frames(4)
		_ok("G9: choosing Circles in the menu stores it", DccSettings.marker_style() == "circles")
		var ticks_ok := true
		for pop in pops:
			ticks_ok = ticks_ok and (pop as PopupMenu).is_item_checked(1) and not (pop as PopupMenu).is_item_checked(0)
		_ok("G9: and every submenu is ticked to match", ticks_ok)
		menus._on_marker_style(menus.ID_GFX_MARKER_STYLE_FIRST)
		await _frames(2)
		_ok("G9: Pins in the menu goes back", DccSettings.marker_style() == "pins")


# ---- touch ---------------------------------------------------------------------------------------------------------------

func _leg_touch(form: String) -> void:
	print("== T. touch: a tap and a hold on a POI ==")
	var pn := _pin(0)
	var head: Vector2 = pn["head"]
	var r: float = pn["r"]
	var a: int = int(_ids["a"])
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-cl11-touch-%d" % Time.get_ticks_msec()
	_write(_root + "/Lore/Old.md", "# Old\n")
	_ok("T0: the throw-away vault binds", bool(_bridge.vault_connect(_root, "Chronicle").get("ok", false)))
	_ov.invalidate_pin_notes()
	await _frames(4)
	var white := await _pixel(head + Vector2(r * 0.85, 0.0))
	_ok("T1: the pin is white with no note (%s)" % white, _is_white(white))
	await _tap_it(head)
	_ok("T1: a tap selects the POI", app.selection == {"kind": "poi", "id": 0}, str(app.selection))
	_ok("T1: a tap shows the chips (Ruling EO), not the card", app.element_chips.visible and not _card_open()
		and not (app.get("_phone_menu") != null and app._phone_menu.peek_card_is_open()))
	_ok("T1: the chips include Note, Move and Rename", app.element_chips.find_child("ChipNote", true, false) != null)
	await _shot("cl11_tap_%s.png" % form)
	await _close_card()
	await get_tree().create_timer(1.2).timeout
	await _hold(head)
	var rows := _vault_rows()
	if rows.is_empty():
		_ok("T2: a hold builds the Note section", false, "card %s hits %s" % [_card_open(), str(app.context_broker.last_request.get("hits"))])
		return
	_ok("T2: a hold builds the card's Note section (Create note...)", rows.size() == 1 and String(rows[0]["label"]).begins_with("Create note"), str(rows))
	var ids := _action_ids()
	_ok("T2: the hold's rows carry Rename, Move, Change kind and Delete", ids.has("el.poi_rename") and ids.has("el.poi_move") and ids.has("el.poi_kind") and ids.has("el.poi_delete"), str(ids))
	if DccTheme.is_phone():
		var pm = app._phone_menu
		_ok("T2: the peek sheet is open", pm.peek_card_is_open())
		pm._set_peek_detent("half", false)
		await _frames(4)
		_ok("T2: the sheet's list has Create note...", "Create note…" in pm.peek_full_row_labels())
		await _shot("cl11_peek_%s.png" % form)
		await pm._dismiss_peek_card()
	else:
		_ok("T2: the card is open with a NOTE band", _card_open() and _find_row("band", "NOTE") != null)
		await _shot("cl11_card_%s.png" % form)
	(rows[0]["callable"] as Callable).call()
	await _frames(8)
	var d: Dictionary = app.context_broker._note._last_dialog
	_ok("T3: Create note opens the dialog", not d.is_empty() and (d["dialog"] as Window).visible)
	if d.is_empty():
		return
	(d["chooser"]["select"] as Callable).call("Lore")
	((d["dialog"] as ConfirmationDialog).get_ok_button()).pressed.emit()
	await _frames(8)
	_ok("T3: bound to the POI", _bridge.vault_links_for("poi", a).size() == 1)
	await _close_card()
	var orange := await _pixel(head + Vector2(r * 0.85, 0.0))
	_ok("T3: the pin is orange (%s)" % orange, _is_orange(orange))
	await _shot("cl11_orange_%s.png" % form)
	await get_tree().create_timer(1.2).timeout
	await _hold(head)
	rows = _vault_rows()
	_ok("T4: after the create a hold shows Open note", rows.size() == 1 and String(rows[0]["label"]).begins_with("Open note"), str(rows))
	## Move through the card's row, then undo, on touch.
	var mv = _action("el.poi_move")
	await _close_card()
	var p0: Dictionary = _bridge.poi_get(0)
	_ok("T5: move, rename and delete work and undo on touch", _bridge.poi_move(0, float(p0["x"]) + 1.0, float(p0["y"]) + 1.0) and _bridge.poi_rename(0, "Renamed")
		and _bridge.poi_delete(0))
	_bridge.undo_last()
	_bridge.undo_last()
	_bridge.undo_last()
	await _frames(4)
	var back: Dictionary = _bridge.poi_get(0)
	_ok("T5: three undos restore position, name and the POI", is_equal_approx(float(back["x"]), float(p0["x"])) and String(back["name"]) == "The Gilded Mug" and mv != null)
