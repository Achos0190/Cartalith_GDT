extends RefCounted
## Shell batch B (2026-11-23), row "`MOUSE_FILTER_STOP` controls over scrollers: triage which ones block a touch scroll"
## (`ANDROID_UI_SPEC.md` section 1, "A fourth mechanism"). Touch forms only (phone, tablet).
## Boots a small world, then in each named state (the three domains' docks or sheets, and the Asset library, the Vault and the
## Preferences-style windows that open) it walks every visible Control, takes those with `mouse_filter == STOP` that sit inside a
## vertical scroller that can scroll, groups them by class + script, and for each group MEASURES a real drag that starts on one
## or two of its members: a drag that moves the scroller is "passes the drag"; one that moves it 0 px is "blocks".
## A control that consumes a drag on purpose (a slider, a text field, a list that scrolls itself) is reported with its reason, not
## counted as a block. The positive control is a plain `Control` in a scroller the drive adds itself (STOP, no handler): it must
## block until `DccWidgets.touch_passive()` has run on it.
## Run: `python C:/cw/shellB_run.py phone|tablet res://tools/shell_batch_b_census_drive.gd <out>`.
## Every line `CENSUS group ...` is a measurement; a line starting `FAIL` fails the run (the controls this batch fixed).

var _fails := 0
var _app: Node
var _groups := {}   ## key -> {count, tested, moved, blocked, example}


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _find(ctx, t: String) -> Dictionary:
	for e in DriveCtx.dump_ui(ctx, t):
		return e
	return {}


func _wait_for(ctx, t: String, secs: float = 20.0) -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		var e := _find(ctx, t)
		if not e.is_empty():
			return e
		await ctx.wait(0.25)
	return {}


func _click_entry(ctx, e: Dictionary) -> void:
	var r: Array = e["rect"]
	await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))


## The nearest ScrollContainer above `c` that can scroll vertically right now, or null.
func _scroller(c: Control) -> ScrollContainer:
	var p := c.get_parent()
	while p != null:
		if p is ScrollContainer:
			var sc := p as ScrollContainer
			if sc.vertical_scroll_mode != ScrollContainer.SCROLL_MODE_DISABLED and sc.get_v_scroll_bar().max_value > sc.get_v_scroll_bar().page + 40.0:
				return sc
		p = p.get_parent()
	return null


func _walk(n: Node, out: Array) -> void:
	for ch in n.get_children(true):
		if ch is Control:
			out.append(ch)
		_walk(ch, out)


## Why a STOP control is allowed to keep a vertical drag, or "" when it is not.
func _owns_drag(c: Control) -> String:
	if c is ScrollBar:
		return "scrollbar"
	if c is TextEdit:
		return "multi-line text (its own touch scroll)"
	if c is ItemList or c is Tree:
		return "list"
	if c is ScrollBar:
		return "scrollbar"
	if c is ScrollContainer:
		return "scroller"
	if c is RichTextLabel and (c as RichTextLabel).scroll_active and not (c as RichTextLabel).fit_content:
		return "scrolling text"
	return ""


func _key(c: Control) -> String:
	var s: Script = c.get_script() as Script
	return "%s|%s" % [c.get_class(), s.resource_path.get_file() if s != null else "-"]


## A finger the way the engine delivers it with `emulate_mouse_from_touch` on: the touch event AND its emulated mouse event (the
## shared touch scroll reads `InputEventScreenTouch` / `InputEventScreenDrag`; a mouse-only drag would measure the wrong thing).
func _finger(ctx, at: Vector2, kind: String, rel: Vector2 = Vector2.ZERO) -> void:
	var vp := _app.get_viewport()
	if kind == "move":
		var d := InputEventScreenDrag.new()
		d.index = 0
		d.position = at
		d.relative = rel
		vp.push_input(d)
		DriveCtx._motion(ctx, at)
		return
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.position = at
	t.pressed = kind == "down"
	vp.push_input(t)
	DriveCtx._button(ctx, at, kind == "down")


## A drag of 120 px down from the centre of `c`; the scroller's movement in px.
func _drag(ctx, c: Control, sc: ScrollContainer) -> float:
	var at := DriveCtx.rect_of(c).get_center()
	var before := sc.scroll_vertical
	DriveCtx._motion(ctx, at)
	await DriveCtx._frames(ctx, 2)
	_finger(ctx, at, "down")
	await DriveCtx._frames(ctx, 2)
	for i in 8:
		_finger(ctx, at + Vector2(0, 15.0 * float(i + 1)), "move", Vector2(0, 15))
		await DriveCtx._frames(ctx, 1)
	_finger(ctx, at + Vector2(0, 120), "up")
	await DriveCtx._frames(ctx, 4)
	return absf(float(sc.scroll_vertical - before))


func _census(ctx, state: String) -> void:
	await ctx.wait(0.6)
	var all := []
	_walk(_app.get_tree().root, all)
	var screen := Rect2(Vector2.ZERO, _app.get_viewport().get_visible_rect().size)
	var members := {}   ## key -> Array[Control]
	var n_stop := 0
	for c in all:
		var ctl := c as Control
		if ctl.mouse_filter != Control.MOUSE_FILTER_STOP or not ctl.is_visible_in_tree() or ctl.size.x < 4.0 or ctl.size.y < 4.0:
			continue
		var sc := _scroller(ctl)
		if sc == null:
			continue
		n_stop += 1
		var k := _key(ctl)
		if not members.has(k):
			members[k] = []
		(members[k] as Array).append(ctl)
	ctx.log("CENSUS state %s: %d visible STOP controls inside a live vertical scroller, %d groups" % [state, n_stop, members.size()])
	for k in members:
		var list: Array = members[k]
		var g: Dictionary = _groups.get(k, {"count": 0, "tested": 0, "moved": 0, "blocked": 0, "example": "", "why": "", "states": ""})
		g["count"] = int(g["count"]) + list.size()
		g["states"] = (String(g["states"]) + "," + state) if String(g["states"]).find(state) < 0 else g["states"]
		var why := _owns_drag(list[0] as Control)
		g["why"] = why
		# measure up to 2 members that are fully on screen inside their scroller's own rect, once per state
		var tested := 0
		for ctl_v in list:
			if tested >= 2 or why != "":
				break
			var ctl := ctl_v as Control
			var sc := _scroller(ctl)
			var r := DriveCtx.rect_of(ctl)
			var vis := DriveCtx.rect_of(sc)
			if not (screen.encloses(r) and vis.encloses(r)):
				continue
			# start mid-scroll so the drag has room in both directions
			sc.scroll_vertical = int(sc.get_v_scroll_bar().max_value / 3.0)
			await DriveCtx._frames(ctx, 3)
			r = DriveCtx.rect_of(ctl)
			if not (screen.encloses(r) and vis.encloses(r)) or r.get_center().y + 130.0 > screen.size.y:
				continue
			var v0 := float((ctl as Range).value) if ctl is Range else 0.0
			var moved := await _drag(ctx, ctl, sc)
			var edited: bool = ctl is Range and not is_equal_approx(float((ctl as Range).value), v0)
			if edited:
				g["edited"] = int(g.get("edited", 0)) + 1
			tested += 1
			g["tested"] = int(g["tested"]) + 1
			if moved > 4.0:
				g["moved"] = int(g["moved"]) + 1
			else:
				g["blocked"] = int(g["blocked"]) + 1
				g["example"] = "%s (%s) in %s" % [ctl.get_path(), str(ctl.mouse_filter), state]
		_groups[k] = g


func _report(ctx) -> void:
	var keys := _groups.keys()
	keys.sort()
	for k in keys:
		var g: Dictionary = _groups[k]
		var verdict := "owns the drag (%s)" % String(g["why"]) if String(g["why"]) != "" else ("BLOCKS" if int(g["blocked"]) > 0 else ("passes" if int(g["tested"]) > 0 else "not on screen to measure"))
		ctx.log("CENSUS group %-44s x%-4d tested %d, scrolled %d, blocked %d, value edited %d  => %s  %s" % [k, int(g["count"]), int(g["tested"]), int(g["moved"]), int(g["blocked"]), int(g.get("edited", 0)), verdict, String(g["example"])])


## The measured guard of the fix: on the tablet a vertical drag that starts on a SpinBox in a window scrolls it and edits nothing.
func _guard_spinbox(ctx) -> void:
	var tested := 0
	var blocked := 0
	var edited := 0
	for k in _groups:
		if String(k).begins_with("SpinBox|"):
			tested += int(_groups[k]["tested"])
			blocked += int(_groups[k]["blocked"])
			edited += int(_groups[k].get("edited", 0))
	_check(ctx, "control: a SpinBox was measured (tablet)", tested >= 1, str(tested))
	_check(ctx, "T1 a drag that starts on a tablet window's SpinBox scrolls the window", tested >= 1 and blocked == 0, "tested %d blocked %d" % [tested, blocked])
	_check(ctx, "T1 ... and does not change the number", edited == 0, "edited %d" % edited)


## A stock ProgressBar is STOP, and `DccWidgets.touch_passive()` (what the shell's window walk calls on every control) turns it
## PASS so a drag that starts on it reaches the scroller. This is a state check, not a measured drag: the engine bubbles a touch
## drag past a non-STOP control (`_touchscroll_probe.gd` measures that for cards, rects and labels), and a bare-bones bed here did
## not scroll from synthetic events (the filler control scrolled 0 px too), so a measured drag from a ProgressBar was not obtainable.
func _progress_state(ctx) -> void:
	var sc := ScrollContainer.new()
	var pb := ProgressBar.new()
	sc.add_child(pb)
	_app.add_child(sc)
	var was_stop: bool = pb.mouse_filter == Control.MOUSE_FILTER_STOP
	var changed := DccWidgets.touch_passive(pb)
	_check(ctx, "control: a stock ProgressBar is STOP", was_stop)
	_check(ctx, "P1 touch_passive() turns a ProgressBar PASS", changed and pb.mouse_filter == Control.MOUSE_FILTER_PASS, "changed %s filter %d" % [str(changed), pb.mouse_filter])
	sc.queue_free()
	var left := 0
	for k in _groups:
		if String(k).begins_with("ProgressBar|"):
			left += int(_groups[k]["count"])
	_check(ctx, "P1 no STOP ProgressBar is left in any state the census visited", left == 0, str(left))


func _png() -> PackedByteArray:
	var img := Image.create(40, 40, false, Image.FORMAT_RGBA8)
	img.fill(Color(0.5, 0.3, 0.2))
	return img.save_png_to_buffer()


func run(ctx) -> void:
	_app = ctx.get_scene_root()
	var size: Vector2 = _app.get_viewport().get_visible_rect().size
	var phone := size.x < 700.0
	if phone:
		await _click_entry(ctx, await _wait_for(ctx, "+ NEW WORLD"))
		await _click_entry(ctx, await _wait_for(ctx, "CANCEL"))
	else:
		await _click_entry(ctx, await _wait_for(ctx, "Continue without a world"))
	await ctx.wait(1.0)
	_app.bridge.generate({"seed": 4242, "width_km": 400.0, "grid_w": 256, "grid_h": 160, "archetype": "", "villages": true, "sea_level": 0.45}, true)
	while _app.bridge.generating:
		await _app.get_tree().create_timer(0.25).timeout
	await ctx.wait(1.0)
	ctx.log("form: %s canvas %s" % ["phone" if phone else "tablet", str(size)])
	for dom in ["world", "civilization", "cartography"]:
		_app.select_domain(dom)
		if phone:
			_app._pick_phone_tab("domain", false)
		await _census(ctx, dom)
	_app.bridge.as_import_item("icons:mountain", "a.png", _png())
	_app.open_asset_library("icons")
	await _census(ctx, "assets")
	_app.asset_library_window.hide()
	await ctx.wait(0.4)
	_app.open_vault_overview()
	await _census(ctx, "vault")
	_app.vault_window.hide()
	await ctx.wait(0.4)
	_app.data_manager_window.open()
	await _census(ctx, "data manager")
	_app.data_manager_window.hide()
	await ctx.wait(0.4)
	_app.open_travel_library()
	await _census(ctx, "travel library")
	_app.travel_library_window.hide()
	await ctx.wait(0.4)
	await DriveCtx.menu_open(ctx, "File", "New world")
	await _census(ctx, "new world")
	_app.new_world_dialog.hide()
	await ctx.wait(0.4)
	_report(ctx)
	await _progress_state(ctx)
	if not phone:
		_guard_spinbox(ctx)
	ctx.log("shell batch B census drive: %d failure(s)" % _fails)
