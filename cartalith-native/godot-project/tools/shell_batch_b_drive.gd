extends RefCounted
## Shell batch B (2026-11-23): the scripted session that checks the phone rows of `OUTSTANDING_WORK.md` "Desktop, tablet and
## phone shell" fixed in that batch, against the REAL shell, with real input events. PHONE ONLY (it prints a skip elsewhere).
## Run: `python -I cartalith-native/tools/fennara_drive.py GODOT --shell phone res://tools/shell_batch_b_drive.gd --dll DLL --out DIR`
## with `SHELLB_SHOTS=<dir>` set for frames. Each check prints `ok` or `FAIL`; a `FAIL` fails the run. Literal expectations only.
## Every check that proves a fix has a named literal mutant in `tools/shell_batch_b_mutants.py`.
##
##   A  Asset library, SLOTS pane: a tile lifted by a long press shows the collection drop strip, a drop on its row adds the tile to
##      that collection, and the strip is hidden again afterwards (it was hidden before: the positive control).
##   U  the undo chip answers every step Edit > Undo takes, not only the height stack: a recorded year (`year_history`) and a
##      grouped step (`undo_groups`) with the look and height stacks empty. Hidden before each (positive control).

var _fails := 0
var _shots := ""
var _app: Node


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _find(ctx, t: String, cls: String = "") -> Dictionary:
	for e in DriveCtx.dump_ui(ctx, t):
		if cls == "" or e["class"] == cls:
			return e
	return {}


func _wait_for(ctx, t: String, secs: float = 20.0, cls: String = "") -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		var e := _find(ctx, t, cls)
		if not e.is_empty():
			return e
		await ctx.wait(0.25)
	return {}


func _click_entry(ctx, e: Dictionary) -> void:
	var r: Array = e["rect"]
	await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))


func _shot(ctx, label: String) -> void:
	var img: Image = await ctx.frame(4096)
	if img != null and _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
		img.save_png("%s/%s.png" % [_shots, label])


func _small_world() -> void:
	_app.bridge.generate({"seed": 4242, "width_km": 400.0, "grid_w": 256, "grid_h": 160,
		"archetype": "", "villages": true, "sea_level": 0.45}, true)
	while _app.bridge.generating:
		await _app.get_tree().create_timer(0.25).timeout
	await _app.get_tree().create_timer(0.5).timeout


func _png() -> PackedByteArray:
	var img := Image.create(40, 40, false, Image.FORMAT_RGBA8)
	img.fill(Color(0.5, 0.3, 0.2))
	return img.save_png_to_buffer()


## The SlotCell of `uid` in the open library window (a `Control` with a `uid` field and the lift meta), or null.
func _cell(win: Node, uid: String) -> Control:
	for c in win.find_children("*", "Control", true, false):
		if c.get("uid") != null and String(c.get("uid")) == uid and c.has_meta(DccWidgets.LIFT_META):
			return c as Control
	return null


func _collection_uids(name: String) -> Array:
	for c in _app.bridge.world_gen.as_collections():
		if String(c["name"]) == name:
			return Array(c["uids"])
	return []


func _asset_drop(ctx) -> void:
	_app.bridge.as_import_item("icons:mountain", "a.png", _png())
	_app.bridge.as_import_item("icons:hill", "b.png", _png())
	_app.bridge.as_batch_collect(PackedStringArray(["icons:mountain"]), "Harbour")
	_app.open_asset_library("icons")
	await ctx.wait(1.2)
	var win = _app.asset_library_window
	_check(ctx, "A the library opened on the SLOTS pane", String(win._phone_pane) == "slots", String(win._phone_pane))
	var strip: Control = win.get("_phone_drop_strip")
	_check(ctx, "A the drop strip exists on the phone", strip != null)
	_check(ctx, "A control: the strip is hidden before any drag", strip != null and not strip.visible)
	var cell := _cell(win, "icons:hill")
	_check(ctx, "A the Hill tile is on screen", cell != null and cell.is_visible_in_tree())
	if cell == null or strip == null:
		return
	var sc := cell.get_parent()
	while sc != null and not (sc is ScrollContainer):
		sc = sc.get_parent()
	if sc != null:
		(sc as ScrollContainer).ensure_control_visible(cell)
	await ctx.wait(0.3)
	_check(ctx, "A control: Hill is not in Harbour yet", not _collection_uids("Harbour").has("icons:hill"))
	var at := DriveCtx.rect_of(cell).get_center()
	DriveCtx._motion(ctx, at)
	await DriveCtx._frames(ctx, 2)
	_finger(ctx, at, "down")
	await ctx.wait(0.8)   # the long press (`TouchScroll.LIFT_HOLD_SEC` 0.4 s) lifts the tile
	_check(ctx, "A a lifted tile shows the drop strip", strip.visible and strip.get_child_count() >= 1,
		"visible=%s children=%d" % [str(strip.visible), strip.get_child_count()])
	await _shot(ctx, "assets_phone_lifted")
	var row: Control = strip.get_child(0) as Control if strip.get_child_count() > 0 else null
	_check(ctx, "A the strip row is on screen", row != null and get_visible(ctx).encloses(DriveCtx.rect_of(row)))
	if row != null:
		var to := DriveCtx.rect_of(row).get_center()
		for i in 5:
			_finger(ctx, at.lerp(to, float(i + 1) / 5.0), "move", (to - at) / 5.0)
			await DriveCtx._frames(ctx, 2)
		_finger(ctx, to, "up")
		await DriveCtx._frames(ctx, 6)
	_check(ctx, "A dropping on the row adds the tile to its collection", _collection_uids("Harbour").has("icons:hill"),
		str(_collection_uids("Harbour")))
	_check(ctx, "A the strip is hidden again after the drop", not strip.visible)
	await _shot(ctx, "assets_phone_dropped")
	win.hide()


## A finger the way the engine delivers it with `emulate_mouse_from_touch` on: the touch event AND its emulated mouse event
## (the shared touch scroll and the long-press lift read `InputEventScreenTouch` / `InputEventScreenDrag`).
func _finger(ctx, at: Vector2, kind: String, rel: Vector2 = Vector2.ZERO) -> void:
	var vp := _app.get_viewport()
	match kind:
		"down":
			var t := InputEventScreenTouch.new()
			t.index = 0
			t.position = at
			t.pressed = true
			vp.push_input(t)
			DriveCtx._button(ctx, at, true)
		"move":
			var d := InputEventScreenDrag.new()
			d.index = 0
			d.position = at
			d.relative = rel
			vp.push_input(d)
			DriveCtx._motion(ctx, at)
		"up":
			var t2 := InputEventScreenTouch.new()
			t2.index = 0
			t2.position = at
			t2.pressed = false
			vp.push_input(t2)
			DriveCtx._button(ctx, at, false)


func get_visible(ctx) -> Rect2:
	return Rect2(Vector2.ZERO, _app.get_viewport().get_visible_rect().size)


func _undo_chip(ctx) -> void:
	var chip: Button = _app.get("_phone_undo_chip")
	_check(ctx, "U the phone undo chip exists", chip != null)
	if chip == null:
		return
	_check(ctx, "U control: the chip is hidden with nothing to undo", not chip.visible and not bool(_app.can_undo_any()))
	# (1) a recorded year
	var y := 1111
	_app.bridge.civ_add_year(y)
	_app.year_history.note_recorded(y, "Record year %d" % y)
	await ctx.wait(0.3)
	_check(ctx, "U a recorded year shows the chip", chip.visible and bool(_app.can_undo_any()))
	await _shot(ctx, "undo_chip_year")
	await DriveCtx.click(ctx, DriveCtx.rect_of(chip).get_center())
	await ctx.wait(0.5)
	_check(ctx, "U tapping the chip removed the recorded year", not Array(_app.bridge.get_civ_timeline_years()).has(y))
	_check(ctx, "U the chip hides again once the year is undone", not chip.visible)
	# (2) a grouped step with the look and height stacks empty
	var ug = _app.undo_groups
	var before = _app.bridge.param_get("sea_level")
	var err: String = ug.begin("Test group")
	_check(ctx, "U a group opened", err == "", err)
	var ok: bool = ug.write_param("sea_level", float(before) + 0.02)
	var res: Dictionary = ug.commit()
	await ctx.wait(0.3)
	_check(ctx, "U setup: the group holds a parameter member, and the height stack and look history are empty",
		ok and bool(res.get("ok", false)) and not _app.bridge.can_undo() and not _app.look_history.can_undo() and ug.wants_undo(),
		str(res))
	_check(ctx, "U a grouped step shows the chip", chip.visible)
	await DriveCtx.click(ctx, DriveCtx.rect_of(chip).get_center())
	await ctx.wait(0.5)
	_check(ctx, "U tapping the chip undid the group (the setting is back)", is_equal_approx(float(_app.bridge.param_get("sea_level")), float(before)),
		"%s vs %s" % [str(_app.bridge.param_get("sea_level")), str(before)])
	_check(ctx, "U the chip hides again once the group is undone", not chip.visible and not ug.wants_undo())


func run(ctx) -> void:
	_app = ctx.get_scene_root()
	_shots = OS.get_environment("SHELLB_SHOTS")
	var size: Vector2 = _app.get_viewport().get_visible_rect().size
	if size.x >= 700.0:
		ctx.log("skip: this drive is phone only")
		return
	var nw := await _wait_for(ctx, "+ NEW WORLD")
	_check(ctx, "boot: the worlds list shows + NEW WORLD", not nw.is_empty())
	await _click_entry(ctx, nw)
	var cancel := await _wait_for(ctx, "CANCEL")
	await _click_entry(ctx, cancel)
	await ctx.wait(1.0)
	await _small_world()
	_check(ctx, "setup: a small world exists", _app.bridge.has_world)
	await _undo_chip(ctx)
	await _asset_drop(ctx)
	ctx.log("shell batch B phone drive: %d failure(s)" % _fails)
