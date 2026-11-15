extends Node
## Ruling FD item 3: Change kind for a GENERATED landmark, through the same pin palette as a user pin's. Windowed (refuses
## --headless), under a wiped user directory (`APPDATA=<empty dir>`). One file serves all three forms:
##
##   DESKTOP: `-- --vp 1600x1000`
##   TABLET:  `-- --vp 1700x1000 --force-touch --density=1.0`
##   PHONE:   `-- --vp 1080x2340 --force-touch --density 2.625`
##   common:  `--out <dir>` (frames) and `--tag <name>` (a prefix for the frame names, so a mutant run never overwrites the
##            frames under review).
##
## What it holds. K1 the landmark's card has Change kind (one row, no submenu) and no Restore row while nothing is overridden.
## K2 Change kind opens the palette in change mode (banner, Cancel) WITHOUT the free-text group; on the phone it is the sheet's
## body at half. K3 a tile pick is ONE undo step, sets `kind_override` and leaves the generated `kind`, class, cell, score and
## the landmark names untouched (display only). K4 the landmark's drawn icon changes (a pixel crop of its head differs) and the
## card's words read the new kind. K5 hit-testing still picks the landmark at its head. K6 one undo restores the generated kind,
## the icon pixels exactly and the words. K7 Restore generated kind (shown only while overridden) clears it in one step, pixels
## identical to the generated ones; undoing it brings the choice back. K8 Cancel changes nothing.
## Exit 0 = held, 1 = a check failed, 2 = could not run.

const Pin := preload("res://shell/pin_marker.gd")

var app: Node
var _vp: SubViewport
var _bridge
var _ov: Control
var _checks := 0
var _fails := 0
var _out := ""
var _tag := "lmkind"
var _phone := false
var _last_mouse := Vector2.ZERO
const Pal := preload("res://shell/pin_palette.gd")
## The kind chosen. Every painted picture has the same pin footprint (measured: the head radius and stem are identical across
## the icons), so a hit test that read the generated kind instead of this one is an equivalent mutant in the painted style.
const NEW_KIND := "temple"


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("LMKIND %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _p(s: String) -> void:
	print("LMKIND %s" % s)


func _finish() -> void:
	DccSettings.set_marker_style("pins")
	_p("---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)


func _host(local: Vector2) -> Vector2:
	return _ov.get_global_transform_with_canvas() * local


func _touch(at: Vector2, down: Variant = null) -> void:
	if down == null:
		return
	_last_mouse = at
	var ev := InputEventMouseButton.new()
	if DccTheme.is_touch():
		ev.device = InputEvent.DEVICE_ID_EMULATION
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = bool(down)
	ev.position = at
	ev.global_position = at
	ev.button_mask = MOUSE_BUTTON_MASK_LEFT if bool(down) else 0
	_vp.push_input(ev)


func _tap(at: Vector2) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = at - _last_mouse
	_vp.push_input(mm)
	await _frames(1)
	_touch(at, true)
	await _frames(3)
	_touch(at, false)
	await _frames(8)


func _undo_n() -> int:
	return (_bridge.undo_ledger() as Array).size()


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(_out.path_join(_tag + "_" + name))
	_p("shot %s_%s" % [_tag, name])


func _palette() -> Node:
	return app.find_child("PinPalette", true, false)


func _tile(kind: String) -> Button:
	var pal := _palette()
	return pal.find_child("Pin_" + kind, true, false) as Button if pal != null else null


func _reveal(c: Control) -> void:
	var n: Node = c.get_parent()
	while n != null:
		if n is ScrollContainer:
			(n as ScrollContainer).ensure_control_visible(c)
			await _frames(2)
		n = n.get_parent()
	await _frames(4)


## The landmark rows the card offers (`GlobalTools._landmark_rows`, the one source the card and the phone chips both read).
func _rows(li: int) -> Array:
	return GlobalTools._landmark_rows(app, li)


func _row(li: int, id: String) -> Dictionary:
	for r in _rows(li):
		if String((r as Dictionary).get("id", "")) == id:
			return r
	return {}


## The pin's head and the crop around it, in host (viewport) pixels.
func _crop_rect(lm: Dictionary) -> Rect2i:
	var rect: Rect2 = _ov._displayed_rect()
	var tip: Vector2 = _ov._cell_to_screen(Vector2(float(lm["x"]), float(lm["y"])), rect)
	var r: float = _ov._landmark_pin_r(lm)
	var head := tip + Vector2(0.0, -Pin.STEM_RATIO * r)
	var a := _host(head)
	var b := _host(tip)
	var sc := absf(b.y - a.y) / maxf(Pin.STEM_RATIO * r, 0.001)
	var half := r * sc * 2.2
	return Rect2i(Vector2i((a - Vector2(half, half)).round()), Vector2i(Vector2(half * 2.0, half * 2.0 + Pin.STEM_RATIO * r * sc).round()))


func _grab(lm: Dictionary) -> Image:
	await _frames(4)
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	return img.get_region(_crop_rect(lm).intersection(Rect2i(Vector2i.ZERO, img.get_size())))


## The head's pixels once they stop changing: a pin's picture and its note colour load lazily (a texture bucket, the note
## state), so the first frames after a change are not the state's own. Two identical grabs in a row, at most 25 tries.
func _pixels(lm: Dictionary) -> Image:
	var prev := await _grab(lm)
	for _i in 25:
		var cur := await _grab(lm)
		if _same(prev, cur):
			return cur
		prev = cur
	return prev


func _diff(a: Image, b: Image) -> int:
	if a.get_size() != b.get_size():
		return -1
	var da := a.get_data()
	var db := b.get_data()
	var n := 0
	for i in da.size():
		if da[i] != db[i]:
			n += 1
	return n


func _same(a: Image, b: Image) -> bool:
	return a.get_size() == b.get_size() and a.get_data() == b.get_data()


## The words the card and the Inspect page give the landmark's kind.
func _words(li: int) -> String:
	return String(app.right_dock_ctrl._subject_text({"kind": "landmark", "id": li})[0])


func _lm_row(li: int) -> Dictionary:
	return (_bridge.landmarks()[li] as Dictionary)


## A landmark on screen, drawn, alone, and (on the phone) in the band between the first-use sheet tip (about 42-48% down) and the zoom buttons and sheet (70% down).
func _pick_landmark() -> int:
	var rect: Rect2 = _ov._displayed_rect()
	var interior: Rect2 = _ov._interior_rect(rect)
	var vr: Rect2 = (app.viewport as Control).get_global_rect()
	var lms: Array = _bridge.landmarks()
	var tips: Array = []
	for lm in lms:
		tips.append(_ov._cell_to_screen(Vector2(float(lm["x"]), float(lm["y"])), rect))
	for s in _bridge.settlements():
		tips.append(_ov._cell_to_screen(Vector2(float(s["x"]), float(s["y"])), rect))
	var best := -1
	var best_gap := 0.0
	for i in lms.size():
		var tip: Vector2 = tips[i]
		var head: Vector2 = tip + Vector2(0.0, -Pin.STEM_RATIO * float(_ov._landmark_pin_r(lms[i])))
		if not interior.has_point(tip) or not interior.has_point(head) or _ov._lm_hidden(i, rect) or not _ov._landmark_shown(lms[i]):
			continue
		var h := _host(head)
		if not vr.grow(-60).has_point(h) or (_phone and (h.y > vr.position.y + vr.size.y * 0.68 or h.y < vr.position.y + vr.size.y * 0.5)):
			continue
		## The most isolated eligible landmark: the one whose nearest neighbour (another landmark or a settlement) is farthest.
		var nearest := 1.0e9
		for j in tips.size():
			if j != i:
				nearest = minf(nearest, (tips[j] as Vector2).distance_to(tip))
		if nearest > best_gap:
			best_gap = nearest
			best = i
	return best


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
	var wd := get_tree().create_timer(800.0)
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
	_p("form: %s, viewport %s" % ["PHONE" if _phone else ("TABLET" if DccTheme.is_tablet() else "DESKTOP"), str(vp_size)])
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
	DccSettings.set_marker_style("painted")
	app.select_domain("cartography")
	await _frames(8)
	## The style is read at draw time but the overlay only redraws on an event, so the first frames of a run still show the
	## drawn pins; a refresh (what any annotation edit does) makes the baseline the painted state the later frames are.
	app.viewport.refresh_annotations()
	_ov.queue_redraw()
	await _frames(10)
	app.arm_tool("inspect")
	await _frames(6)
	if _phone:
		app.set_phone_detent("peek")
		await _frames(12)
	await _run()
	_finish()


func _run() -> void:
	var li := _pick_landmark()
	_ok("K0: a drawn, isolated landmark to work on", li >= 0, str(_bridge.landmarks().size()))
	if li < 0:
		return
	var before: Dictionary = _lm_row(li)
	var key := String((_bridge.landmark_names()[li] as Dictionary)["key"])
	var gen_kind := String(before["kind"])
	var names0: Array = _bridge.landmark_names()
	var gen_words := _words(li)
	_p("landmark %d key %s generated kind %s words %s" % [li, key, gen_kind, gen_words])
	## The landmark is the selection for every frame compared (a selection ring is part of its pixels), on the forms that show one
	## without opening a sheet over the map.
	if not _phone:
		app.select("landmark", li)
		await _frames(10)
	var px_gen := await _pixels(before)
	var head_local: Vector2 = _ov._cell_to_screen(Vector2(float(before["x"]), float(before["y"])), _ov._displayed_rect()) \
		+ Vector2(0.0, -Pin.STEM_RATIO * float(_ov._landmark_pin_r(before)))

	# K1
	var rows := _rows(li)
	var kind_row := _row(li, "el.landmark_kind")
	_ok("K1: the landmark's card has Change kind as one row, no submenu", not kind_row.is_empty() and not kind_row.has("children"), str(rows.map(func(r): return r.get("id"))))
	_ok("K1: and no Restore row while nothing is overridden", _row(li, "el.landmark_kind_restore").is_empty() and not before.has("kind_override"))
	if kind_row.is_empty():
		return

	# K8 Cancel first (so nothing else has changed yet)
	var n0 := _undo_n()
	(kind_row["callable"] as Callable).call()
	await _frames(14)
	var pal := _palette()
	_ok("K2: Change kind opens the palette in change mode (banner and Cancel)", pal != null and pal.find_child("ChangeBanner", true, false) != null and pal.find_child("ChangeCancel", true, false) != null)
	_ok("K2: the free-text group is NOT offered for a generated landmark", pal != null and pal.find_child("FreeKind", true, false) == null)
	if _phone:
		_ok("K2: on the phone the palette is the sheet's body, opened at half", String(app._phone_detent) == "half" and pal != null and app._phone_tool_sheet.is_ancestor_of(pal), str(app._phone_detent))
	await _shot("change_mode.png")
	if pal != null and pal.find_child("ChangeCancel", true, false) != null:
		(pal.find_child("ChangeCancel", true, false) as Button).pressed.emit()
	await _frames(8)
	_ok("K8: Cancel changes nothing and ends change mode", not _lm_row(li).has("kind_override") and _undo_n() == n0 and (_palette() == null or _palette().find_child("ChangeBanner", true, false) == null))

	# K3 pick
	(kind_row["callable"] as Callable).call()
	await _frames(14)
	var tile := _tile(NEW_KIND)
	_ok("K3: the palette has the %s tile" % NEW_KIND, tile != null)
	if tile == null:
		return
	if _phone:
		await _reveal(tile)
		await _tap(tile.get_global_rect().get_center())
	else:
		tile.pressed.emit()
	await _frames(10)
	var after := _lm_row(li)
	_ok("K3: the pick is ONE undo step", _undo_n() == n0 + 1, "%d -> %d" % [n0, _undo_n()])
	_ok("K3: it sets kind_override", String(after.get("kind_override", "")) == NEW_KIND, str(after.get("kind_override", "<absent>")))
	var same := true
	for k in ["id", "kind", "class", "x", "y", "elevation", "score", "importance", "causal"]:
		same = same and after.get(k) == before.get(k)
	_ok("K3: the generated data (kind, class, cell, score, importance, causes) is unchanged", same)
	_ok("K3: the landmark names (and so the vault keys) are unchanged", _bridge.landmark_names() == names0)
	_ok("K3: change mode ended (no banner)", _palette() == null or _palette().find_child("ChangeBanner", true, false) == null)
	if _phone:
		app.set_phone_detent("peek")
	app.arm_tool("inspect")
	await _frames(14)

	# K4 pixels and words
	var px_new := await _pixels(after)
	_ok("K4: the landmark's drawn icon changed (head pixels differ)", not _same(px_gen, px_new))
	_ok("K4: the card's kind words are the new kind", _words(li) == Pal.label_for(NEW_KIND), _words(li))
	await _shot("changed.png")
	if not _phone:
		var found := false
		var seen: Array = []
		for l in app.find_children("*", "Label", true, false):
			seen.append((l as Label).text)
			if (l as Label).text == Pal.label_for(NEW_KIND):
				found = true
		_ok("K4: the Inspect page's Kind readout says the new kind", found, str(seen.filter(func(t): return t.contains(Pal.label_for(NEW_KIND)) or t.contains("Ford") or t.begins_with("Kind")).slice(0, 20)))

	# K5 hit test
	var m: Dictionary = _ov.marker_at(head_local)
	_ok("K5: hit-testing still picks the landmark at its head", String(m.get("kind", "")) == "landmark" and int(m.get("id", -1)) == li, str(m))

	# K6 undo
	var ur: Dictionary = _row(li, "el.landmark_kind_restore")
	_ok("K7: a Restore generated kind row is offered while overridden", not ur.is_empty())
	_bridge.undo_last()
	await _frames(10)
	_ok("K6: one undo clears the override", not _lm_row(li).has("kind_override") and _undo_n() == n0, str(_lm_row(li).get("kind_override", "<absent>")))
	_ok("K6: the words are the generated words again", _words(li) == gen_words, _words(li))
	var px_undo := await _pixels(_lm_row(li))
	_ok("K6: the icon pixels are the generated ones again, exactly", _same(px_gen, px_undo), "%d of %d bytes differ" % [_diff(px_gen, px_undo), px_gen.get_data().size()])
	if _out != "":
		px_gen.save_png(_out.path_join(_tag + "_crop_generated.png"))
		px_new.save_png(_out.path_join(_tag + "_crop_changed.png"))
		px_undo.save_png(_out.path_join(_tag + "_crop_undone.png"))
	_ok("K6: the Restore row is gone again", _row(li, "el.landmark_kind_restore").is_empty())

	# K7 restore
	_bridge.redo_last()
	await _frames(10)
	_ok("K6: redo puts the choice back", String(_lm_row(li).get("kind_override", "")) == NEW_KIND)
	var rr := _row(li, "el.landmark_kind_restore")
	if rr.is_empty():
		_ok("K7: Restore generated kind row present", false)
		return
	var n1 := _undo_n()
	(rr["callable"] as Callable).call()
	await _frames(10)
	_ok("K7: Restore generated kind clears it in ONE step", not _lm_row(li).has("kind_override") and _undo_n() == n1 + 1)
	var px_rest := await _pixels(_lm_row(li))
	_ok("K7: and the icon is the generated one, pixel for pixel", _same(px_gen, px_rest))
	_ok("K7: the words are the generated words", _words(li) == gen_words)
	_bridge.undo_last()
	await _frames(10)
	_ok("K7: undoing the restore brings the choice back", String(_lm_row(li).get("kind_override", "")) == NEW_KIND)
	_bridge.undo_last()
	await _frames(6)
	_ok("K7: and one more undo is back at the generated kind", not _lm_row(li).has("kind_override") and _undo_n() == n0)
