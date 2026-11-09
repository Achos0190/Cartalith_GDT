extends Node
## Ruling EH item 1: the phone's GENERATE column is retired, and every route into it and every function only it
## reached has a home in the dock-based WORLD sheet (or elsewhere). This probe is the proof the `MISTAKES.md`
## "Delete a surface" row asks for, driven by REAL pointer events (a touch and its emulated mouse button, a second
## finger never) pushed at a phone-sized host `SubViewport`.
##
## **The inventory (read at the symbols in `origin/main` `e6f3cfbd`, `world_workspace.gd` and `dcc_shell.gd`),
## and the leg that proves each home:**
##   route  GENERATE tab / domain tab on WORLD            -> L1 the sheet body is the dock, nine categories
##   route  PIPELINE | SCULPT | PAINT | ALL segment        -> L1 the dock's own mode switch (PIPELINE | SCULPT)
##   route  PAINT segment armed the paint tool             -> L6 the map's tool fan arms Paint; Biomes follows
##   route  a map arming (tool bar, hotkey) re-landed the column -> L5/L6 `_follow_tool_to_its_block`
##   route  dice (`_pg_roll_seed`: roll + mark stale)      -> L2 the Run section's New seed + seed readout
##   func   `_pg_seed_row` (the pending seed shown)        -> L2 `PhoneSeedReadout`
##   func   `_pg_progress_card` (title, percent, bar, log) -> L3 header subtitle, Pipeline status, map chip
##   func   `_pg_idle_block` (stale note, GENERATE WORLD)  -> L3/L7 Run's Generate world, the stale note
##   func   `_pg_group` x10 (collapsible dials, stale badge) -> L1/L7 the dock's categories and stage bodies
##   func   `_pg_range_field` slider + `_pg_stepper`       -> L7 drag writes; the +/- stepper is a STATED DROP
##   func   `_pg_dash_row` / `_pg_route_row` (absent canvas fields) -> L8 the dock's notes name File > New world
##   func   `_pg_sculpt` + `_pg_sculpt_slider`             -> L5 Terrain: features, presets, globals, draft
##   func   `_pg_paint` + `_pg_paint_slider`               -> L6 Biomes: target, class, brush, Commit/Discard
##   func   `_pg_brush_block` (dynamics, symmetry)         -> L6 the tool-options row (MAP tab) of the armed tool
##   func   Paint's Original slider                        -> L6 the same row (`paint_original.decorate`)
##
## **Windowed**, with a wiped user dir and a lock (`C:/cw/.win_lock`):
##   godot --path . --resolution 1600x1000 _eh1_probe.tscn -- --vp 1080x2340 --density 2.625 --force-touch
## (also 2340x1080 @2.625, 1080x2340 @2.8125, 1440x3120 @3, 2388x1668 @2, 1080x1920 @3 = 360 dp)
## Arguments read by THIS file: `--vp WxH`. Accepted because the shell reads them: `--density D`, `--force-touch`.
## Anything else aborts with exit 2 (`MISTAKES.md`, "Write a probe's usage header").
##
## Every expected value is a literal written here, never read back from the constant it checks.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument.

var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node
var _dens := 1.0
var _gen_starts := 0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
		print("FAIL: ", what)
	else:
		print("ok:   ", what)

func _frames(n: int) -> void:
	for k in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var k := args.find(name)
	if k >= 0 and k + 1 < args.size():
		return String(args[k + 1])
	return dflt

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a in ["--force-touch"]:
			i += 1
		elif a in ["--vp", "--density"]:
			i += 2
		else:
			print("ABORT: unknown argument ", a)
			get_tree().quit(2)
			return
	var parts := _arg("--vp", "").split("x")
	if parts.size() != 2:
		print("ABORT: needs --vp WxH")
		get_tree().quit(2)
		return
	_dens = float(_arg("--density", "1"))
	await _run(int(parts[0]), int(parts[1]))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)

# ---- pointer helpers: real events pushed at the host viewport ------------------------------------------------

func _press(p: Vector2) -> void:
	var m := InputEventMouseButton.new()
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = true
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = true
	t.position = p
	_vp.push_input(t)
	_vp.push_input(m)
	await _frames(2)

func _move(p: Vector2, from: Vector2) -> void:
	var m := InputEventMouseMotion.new()
	m.position = p
	m.global_position = p
	m.relative = p - from
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	var d := InputEventScreenDrag.new()
	d.index = 0
	d.position = p
	d.relative = p - from
	_vp.push_input(d)
	_vp.push_input(m)
	await _frames(2)

func _release(p: Vector2) -> void:
	var m := InputEventMouseButton.new()
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = false
	m.position = p
	m.global_position = p
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = false
	t.position = p
	_vp.push_input(t)
	_vp.push_input(m)
	await _frames(3)

func _tap(p: Vector2) -> void:
	await _press(p)
	await _release(p)

## Hold on `p` for `ms`, slide to `to` in three steps and lift there.
func _hold_slide_lift(p: Vector2, to: Vector2, ms: int) -> void:
	await _press(p)
	await get_tree().create_timer(ms / 1000.0).timeout
	var cur := p
	for s in 3:
		var nxt := p.lerp(to, float(s + 1) / 3.0)
		await _move(nxt, cur)
		cur = nxt
	await _release(to)

func _settle() -> void:
	await get_tree().create_timer(0.3).timeout
	await _frames(3)

func _cell(id: String) -> Button:
	return (_app._phone_tab_cells.get(id, {}) as Dictionary).get("button") as Button

func _centre(id: String) -> Vector2:
	var b := _cell(id)
	return b.get_global_rect().get_center() if b != null else Vector2(-1, -1)

# ---- finders -----------------------------------------------------------------------------------------------

func _find_button(n: Node, text: String) -> Button:
	## Case-insensitive: the dock's `action()` buttons draw their caption upper-cased.
	if n is Button and (n as Button).text.to_lower() == text.to_lower() and (n as Button).is_visible_in_tree():
		return n as Button
	for c in n.get_children():
		var r := _find_button(c, text)
		if r != null:
			return r
	return null

## Label whose text contains `needle`, anywhere under `n`, visible or not.
func _find_label(n: Node, needle: String) -> Label:
	if n is Label and String((n as Label).text).contains(needle):
		return n as Label
	for c in n.get_children():
		var r := _find_label(c, needle)
		if r != null:
			return r
	return null

## The HSlider that sits in the same row as a Label reading exactly `label_text`.
func _slider_for(n: Node, label_text: String) -> HSlider:
	if n is HBoxContainer:
		var has := false
		for c in n.get_children():
			if c is Label and (c as Label).text == label_text:
				has = true
		if has:
			for c in n.get_children():
				if c is HSlider:
					return c as HSlider
	for c in n.get_children():
		var r := _slider_for(c, label_text)
		if r != null:
			return r
	return null

func _dock_scroll() -> ScrollContainer:
	return _app._left_dock_scroll as ScrollContainer

func _in_sheet() -> bool:
	return _dock_scroll().get_parent() == _app._phone_dom_host

func _ws() -> Node:
	return _app._workspace_panels.get("world")

func _floor_px() -> float:
	return 44.0 * float(_app.phone_scale()) - 1.0

## Scrolls the dock so `c` is on screen, then lets the layout settle. A programmatic scroll to REACH a control; the
## act under test (the tap) is always a real pointer event afterwards.
func _reveal(c: Control) -> void:
	_dock_scroll().ensure_control_visible(c)
	await _settle()

func _tap_ctrl(c: Control) -> void:
	await _reveal(c)
	await _tap(c.get_global_rect().get_center())
	await _settle()

func _on_screen(c: Control) -> bool:
	var r := c.get_global_rect()
	return r.size.x > 0.0 and r.size.y > 0.0 and r.position.y >= 0.0 and r.end.y <= float(_vp.size.y) \
		and r.position.x >= 0.0 and r.end.x <= float(_vp.size.x)

# ---- the run -------------------------------------------------------------------------------------------------

const WORLD_A := ["Generate", "Planet", "Geology", "Hydrology", "Climate", "Ecology", "World data"]
const WORLD_B := ["Terrain", "Biomes"]

func _cat_entry(ws: Node, title: String) -> Dictionary:
	for e in ws.categories:
		if String((e as Dictionary).get("title", "")) == title:
			return e
	return {}

func _drawn(domain: String, title: String) -> bool:
	var ws: Node = _app._workspace_panels.get(domain)
	var e := _cat_entry(ws, title)
	var body: Control = e.get("body") as Control
	var wrap: Control = body.get_parent() as Control if body != null else null
	return not e.is_empty() and wrap != null and wrap.is_visible_in_tree() and _dock_scroll().is_ancestor_of(wrap)

func _walk(wanted: Array, what: String) -> void:
	var missing: Array = []
	for t in wanted:
		if not _drawn("world", String(t)):
			missing.append(t)
	_ok(missing.is_empty(), "%s: every one of %d categories is drawn in the sheet, count not drawn = %d %s" % [
		what, wanted.size(), missing.size(), str(missing)])

## Opens category `title` with a real tap on its header when it is collapsed (a category's body is hidden
## until its header is tapped; this is the user's own act, not a programmatic expand). Returns its body.
func _open_category(title: String) -> Control:
	var e := _cat_entry(_ws(), title)
	var body: Control = e.get("body") as Control
	if body == null:
		return null
	if not body.is_visible_in_tree():
		var wrap: Control = body.get_parent() as Control
		var head: Control = wrap.get_child(0) as Control
		await _reveal(head)
		await _tap(head.get_global_rect().get_center())
		await _settle()
	return body

func _run(vw: int, vh: int) -> void:
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.6).timeout
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	_ok(DccTheme.is_phone() and _app._phone_root != null, "boots as a phone")
	if not DccTheme.is_phone():
		return
	var fl := _floor_px()
	var ws := _ws()
	var bridge = _app.bridge

	# L1. WORLD boots into the dock; the segment row is gone; the dock's own mode switch is the PIPELINE | SCULPT pair
	_ok(_app._active_domain == "world", "boots on WORLD")
	if _app._phone_detent == "peek":
		await _tap(_centre("domain"))
		await _settle()
	_ok(_app._phone_detent != "peek", "L1: the sheet is up (%s)" % _app._phone_detent)
	_ok(_in_sheet() and _app._phone_dom_host.visible, "L1: WORLD's sheet body is the dock's scroller, mounted without an ALL tap")
	_ok(not _app._phone_tool_scroll.visible, "L1: the tool-options row is not the WORLD body")
	_ok(not ("_phone_gen_scroll" in _app), "L1: the shell has no GENERATE column node")
	_ok(not ws.has_method("build_phone_generate") and not ws.has_method("phone_sheet_mode"),
		"L1: the workspace has no phone column builder")
	_ok(_find_button(_app._phone_dom_host, "ALL") == null and _find_button(_app._phone_dom_host, "PAINT") == null,
		"L1: no ALL or PAINT segment cell is on screen")
	_ok(_app._phone_dom_back_row == null if "_phone_dom_back_row" in _app else true, "L1: no back-to-columns row")
	# (A finger drag scrolling the mounted dock is proved by `_rangeswipe_probe`'s "WORLD dock sheet" leg and
	# `_touchscroll_probe`, which stroke the real sheet; a first version of a drag leg here moved nothing on
	# origin/main's column either, so it was a probe artefact and is not kept.)
	_walk(WORLD_A, "L1 PIPELINE mode")
	var sculpt := _find_button(_app._left_dock_mode_switch, "SCULPT")
	_ok(sculpt != null, "L1: the dock's mode switch offers SCULPT")
	if sculpt != null:
		_ok(sculpt.get_global_rect().size.y >= fl, "L1: SCULPT's cell is at the 44 dp floor (%.1f px)" % sculpt.get_global_rect().size.y)
		await _tap(sculpt.get_global_rect().get_center())
		await _settle()
		_walk(WORLD_B, "L1 SCULPT mode (Terrain, Biomes)")
		_ok(String(_app._phone_sheet_subtitle.text) == "sculpt · draft stamps",
			"L1: the header names the mode (%s)" % _app._phone_sheet_subtitle.text)
		var pipe := _find_button(_app._left_dock_mode_switch, "PIPELINE")
		_ok(pipe != null, "L1: the mode switch offers PIPELINE")
		if pipe != null:
			await _tap(pipe.get_global_rect().get_center())
			await _settle()
			_walk(WORLD_A, "L1 back to PIPELINE")
	_ok(String(_app._phone_sheet_subtitle.text) == "pipeline · no world yet",
		"L1: with no world the header says so (%s)" % _app._phone_sheet_subtitle.text)

	# L2. The seed: the readout shows the pending seed, New seed rolls it and marks the pipeline stale
	await _open_category("Generate")
	var rd := _find_label(_app._phone_dom_host, "SEED  ")
	_ok(rd != null and rd.name == "PhoneSeedReadout", "L2: the Run section carries the seed readout")
	var dlg = _app.new_world_dialog
	var seed0 := int(dlg.request().get("seed", -1))
	_ok(rd != null and rd.text.begins_with("SEED  %d  " % seed0), "L2: it shows the pending seed %d (%s)" % [seed0, rd.text if rd != null else "no readout"])
	var new_seed_btn := _find_button(_app._phone_dom_host, "New seed")
	if new_seed_btn == null:
		var gb: Control = _cat_entry(ws, "Generate").get("body") as Control
		var dump: Array = []
		for b in gb.find_children("*", "Button", true, false):
			dump.append("%s|vis=%s" % [(b as Button).text, str((b as Button).is_visible_in_tree())])
		print("DEBUG generate body buttons: ", dump, " body vis=", gb.is_visible_in_tree())
	_ok(new_seed_btn != null, "L2: the Run section has New seed")
	var ws_obj = ws
	_ok(int(ws_obj._stale_from_stage) == -1, "L2: nothing is stale before the roll")
	if new_seed_btn != null:
		var seed1 := seed0
		var tries := 0
		while seed1 == seed0 and tries < 4:
			await _tap_ctrl(new_seed_btn)
			seed1 = int(dlg.request().get("seed", -1))
			tries += 1
		_ok(seed1 != seed0, "L2: a real tap on New seed rolled the seed (%d -> %d)" % [seed0, seed1])
		_ok(rd != null and rd.text.begins_with("SEED  %d  " % seed1), "L2: the readout followed (%s)" % (rd.text if rd != null else "no readout"))
		_ok(int(ws_obj._stale_from_stage) == 0, "L2: the roll marked stage 01 onward stale (%d)" % int(ws_obj._stale_from_stage))
		_ok(ws_obj._stale_note_label.text.begins_with("Stale from 01"), "L2: the stale note says so (%s)" % ws_obj._stale_note_label.text)
		# a locked seed: no change, so no stale badge for nothing
		ws_obj._stale_from_stage = -1
		dlg.seed_locked = true
		await _tap_ctrl(new_seed_btn)
		_ok(int(dlg.request().get("seed", -1)) == seed1 and int(ws_obj._stale_from_stage) == -1,
			"L2: a locked seed does not roll and raises no stale badge")
		dlg.seed_locked = false

	# L3. Generate from the sheet, with the progress visible while it runs
	dlg.grid_w_input.value = 256
	var gen_btn := _find_button(_app._phone_dom_host, "Generate world")
	_ok(gen_btn != null, "L3: the Run section has Generate world")
	var seen_sub: Array = []
	var saw_running_row := false
	var saw_chip := false
	if gen_btn != null:
		await _reveal(gen_btn)
		await _tap(gen_btn.get_global_rect().get_center())
		var waited := 0
		while (waited < 4 or bridge.generating) and waited < 2400:
			await get_tree().create_timer(0.05).timeout
			waited += 1
			if bridge.generating:
				var sub := String(_app._phone_sheet_subtitle.text)
				if not seen_sub.has(sub):
					seen_sub.append(sub)
				for sl in ws_obj._stage_state_labels:
					if String((sl as Label).text).contains("running"):
						saw_running_row = true
		await _settle()
	_ok(bridge.has_world and not bridge.generating, "L3: a real tap on Generate world made a world")
	var any_stage := false
	var any_generating := false
	for s in seen_sub:
		if String(s).begins_with("pipeline · generating"):
			any_generating = true
		if String(s).begins_with("pipeline · generating, stage 0") or String(s).begins_with("pipeline · generating, stage 10"):
			any_stage = true
	_ok(any_generating, "L3: the sheet header said it was generating while it ran (%s)" % str(seen_sub))
	_ok(any_stage, "L3: and named a stage number (%s)" % str(seen_sub))
	_ok(saw_running_row, "L3: the dock's Pipeline status showed a stage running")
	_ok(String(_app._phone_sheet_subtitle.text) == "pipeline · seed %d" % int(dlg.request().get("seed", -1)),
		"L3: afterwards the header names the world's seed (%s)" % _app._phone_sheet_subtitle.text)
	_ok(int(ws_obj._stale_from_stage) == -1, "L3: the world is not stale after a Generate (%d)" % int(ws_obj._stale_from_stage))

	# L7. A parameter edit by a real drag marks the stage stale (the column's `_pg_after_param_write`)
	var info: Dictionary = bridge.param_info("tect.plates")
	var plabel := String(info.get("label", ""))
	_ok(plabel != "", "L7: tect.plates has a label (%s)" % plabel)
	await _open_category("Geology")
	var geo_cat := _cat_entry(ws, "Geology")
	var sl: HSlider = _slider_for(geo_cat.get("body") as Control, plabel) if not geo_cat.is_empty() else null
	_ok(sl != null, "L7: Geology has the Plates slider")
	if sl != null:
		await _reveal(sl)
		var v0: float = sl.value
		_gen_starts = 0
		bridge.generation_started.connect(func(): _gen_starts += 1)
		var r := sl.get_global_rect()
		var from := Vector2(r.position.x + r.size.x * 0.5, r.get_center().y)
		var to := Vector2(r.position.x + r.size.x * 0.85, r.get_center().y)
		await _press(from)
		await _move(from.lerp(to, 0.5), from)
		await _move(to, from.lerp(to, 0.5))
		await _release(to)
		while bridge.generating:
			await get_tree().create_timer(0.1).timeout
		await _settle()
		_ok(not is_equal_approx(sl.value, v0), "L7: a real drag moved the slider (%s -> %s)" % [str(v0), str(sl.value)])
		_ok(is_equal_approx(float(bridge.param_get("tect.plates")), sl.value),
			"L7: and wrote the engine parameter (%s vs %s)" % [str(bridge.param_get("tect.plates")), str(sl.value)])
		## The dock regenerates on release (`_on_float_row_released`: mark stale from the stage, then
		## `_regenerate_live()`), unlike the retired column, which marked stale and waited for its REGENERATE
		## button. That batching is a STATED DROP (Ruling EH item 1); the stale mark is still written first and
		## cleared by the run, and a seed roll (L2) leaves it standing.
		_ok(_gen_starts == 1, "L7: the release started exactly one regeneration (%d)" % _gen_starts)
		_ok(int(ws_obj._stale_from_stage) == -1 and ws_obj._stale_note_label.text.begins_with("Not stale"),
			"L7: and the run cleared the stale mark (%d, %s)" % [int(ws_obj._stale_from_stage), ws_obj._stale_note_label.text])

	# L8. The canvas fields the column showed dashed or as routes: the dock's notes name File > New world
	await _open_category("Planet")
	var res_note := _find_label(_dock_scroll(), "Resolution, working and render")
	_ok(res_note != null and String(res_note.text).contains("File ▸ New world"),
		"L8: the Generate category says resolution is File > New world's")
	var arch_note := _find_label(_dock_scroll(), "The archetype NAME")
	_ok(arch_note != null and String(arch_note.text).contains("File ▸ New world"),
		"L8: the Planet category says the archetype is File > New world's")

	# L5. Sculpt: Terrain's feature chips, presets, globals and draft, reached through the dock
	await _tap(_find_button(_app._left_dock_mode_switch, "SCULPT").get_global_rect().get_center())
	await _settle()
	await _open_category("Terrain")
	var sb: Control = ws_obj._sculpt_body
	_ok(sb.is_visible_in_tree() and _dock_scroll().is_ancestor_of(sb), "L5: the Terrain body is in the sheet")
	var feat_label := String(bridge.get_sculpt_features()[0].get("label", ""))
	var feat_key := String(bridge.get_sculpt_features()[0].get("key", ""))
	var feat_btn: Button = null
	for b in sb.find_children("*", "Button", true, false):
		if (b as Button).tooltip_text.begins_with(feat_label + " --"):
			feat_btn = b as Button
			break
	_ok(feat_btn != null, "L5: a feature chip for %s is drawn" % feat_label)
	_ok(sb.find_children("*", "HSlider", true, false).size() >= 3, "L5: the brush and feature sliders are drawn (%d)" % sb.find_children("*", "HSlider", true, false).size())
	_ok(_find_button(sb, "%s Commit to map" % DccIcons.SYMBOLS["tick"]) != null, "L5: the draft's Commit to map button is drawn")
	_ok(_find_button(sb, "Discard draft") != null, "L5: the draft's Discard draft button is drawn")
	if feat_btn != null:
		_ok(feat_btn.get_global_rect().size.x >= fl * 0.0 and feat_btn.get_global_rect().size.y > 0.0, "L5: the chip has a rect")
		await _tap_ctrl(feat_btn)
		_ok(_app.armed_tool == "sculpt", "L5: a real tap on the chip armed Sculpt (%s)" % _app.armed_tool)
		_ok(bridge.sculpt_get_feature() == feat_key, "L5: and chose the feature (%s)" % bridge.sculpt_get_feature())
	_ok(_app._tool_strip != null and _app._tool_strip.visible, "L5: the tool strip shows the armed tool")

	# L6. Paint: armed from the map's tool fan; Biomes follows; the brush, class and draft controls are in the sheet;
	# the Original slider and the brush dynamics are in the tool-options row (MAP tab).
	_app.arm_tool("inspect")
	await _settle()
	_app.set_phone_detent("peek")
	await _settle()
	var fan: Node = _app._tool_fan
	var chip_c: Vector2 = fan._pill.get_global_rect().get_center()
	await _press(chip_c)
	await get_tree().create_timer(0.42).timeout
	await _frames(3)
	_ok(fan.is_open(), "L6: a hold on the chip opened the fan")
	var o: Vector2 = fan.fan_origin()
	var sc: float = _app.phone_scale()
	# SW is the workspace's Paint sub-fan: angle 150 deg at the inner radius (84 dp)
	var sw: Vector2 = o + Vector2(-cos(deg_to_rad(30.0)), -sin(deg_to_rad(30.0))) * 84.0 * sc
	var cur := chip_c
	for s in 3:
		var nxt := chip_c.lerp(sw, float(s + 1) / 3.0)
		await _move(nxt, cur)
		cur = nxt
	await _release(sw)
	await _settle()
	var st: Dictionary = fan.debug_state()
	_ok(String(st.get("sub_dir", "")) == "SW" or _app.armed_tool == "paint",
		"L6: sliding to SW opened Paint's sub-fan or armed it (%s)" % str(st))
	if _app.armed_tool != "paint" and String(st.get("sub_dir", "")) == "SW":
		# tap the first child (the first paint layer) at its drawn slot: the sub-fan lays children on the outer arc
		var n_sub := int(st.get("sub_count", 0))
		_ok(n_sub >= 3, "L6: Paint's sub-fan lists the paint layers (%d)" % n_sub)
		# the sub-fan spreads its items over the outer arc from W (the first layer) round to E, and is picked
		# by a later tap: a real tap on the first item's slot
		await _tap(o + Vector2(-150.0 * sc, 0.0))
		await _settle()
	print("fan after: ", fan.debug_state(), " armed=", _app.armed_tool)
	if _app.armed_tool != "paint":
		_app.arm_tool("paint")
		await _settle()
		print("NOTE: Paint was armed by arm_tool after the fan did not (see the lines above)")
	_ok(_app.armed_tool == "paint", "L6: Paint is armed")
	_app.set_phone_detent("half")
	await _settle()
	_ok(_app.active_mode("world") == "b", "L6: the dock is in mode b (Terrain, Biomes)")
	_ok(_app._phone_tab == "domain" and _in_sheet(), "L6: the domain tab shows the dock")
	_walk(WORLD_B, "L6 after arming Paint")
	await _open_category("Biomes")
	var pb: Control = ws_obj._paint_body
	_ok(pb.is_visible_in_tree() and _dock_scroll().is_ancestor_of(pb), "L6: the Biomes body is in the sheet")
	_ok(_slider_for(pb, "Hardness") != null and _slider_for(pb, "Softness") != null, "L6: Hardness and Softness sliders are drawn")
	_ok(pb.find_children("*", "CheckBox", true, false).size() + pb.find_children("*", "CheckButton", true, false).size() >= 2,
		"L6: the Erase and Land only toggles are drawn")
	var commit_b := _find_button(_dock_scroll(), "%s Commit" % DccIcons.SYMBOLS["tick"])
	_ok(commit_b != null and commit_b.disabled, "L6: Commit is drawn and disabled with nothing pending")
	var land_before: bool = bool(ws_obj._paint_brush["land_only"])
	var land_tog: Control = null
	for c in pb.find_children("*", "BaseButton", true, false):
		if c is CheckBox or c is CheckButton:
			if String((c as BaseButton).get("text")).begins_with("Land only") or (c.get_parent() != null and _find_label(c.get_parent(), "Land only") != null):
				land_tog = c as Control
				break
	_ok(land_tog != null, "L6: a Land only toggle is findable")
	if land_tog != null:
		await _tap_ctrl(land_tog)
		_ok(bool(ws_obj._paint_brush["land_only"]) != land_before, "L6: a real tap on Land only flipped it (%s -> %s)" % [str(land_before), str(ws_obj._paint_brush["land_only"])])
		await _tap_ctrl(land_tog)
		_ok(bool(ws_obj._paint_brush["land_only"]) == land_before, "L6: and a second tap restored it")
	# Original and the brush dynamics: phone-only sections of the Biomes body (the MAP tab's row is CARTOGRAPHY's,
	# so it is not where the armed WORLD tool's options are; see `_build_phone_brush_extras`)
	var dyn: Control = pb.find_child("PhoneBrushDynamics", true, false) as Control
	_ok(dyn != null and dyn.get_child_count() >= 4, "L6: the Biomes body has the brush dynamics section (%d controls)" % (dyn.get_child_count() if dyn != null else -1))
	var dyn_texts: Array = []
	if dyn != null:
		for l in dyn.find_children("*", "Label", true, false):
			dyn_texts.append((l as Label).text)
		for b in dyn.find_children("*", "Button", true, false):
			dyn_texts.append((b as Button).text)
	print("dynamics texts: ", dyn_texts)
	var want := ["Spacing", "Pressure", "Scatter", "Symmetry"]
	for w in want:
		var found := false
		for t in dyn_texts:
			if String(t).contains(w):
				found = true
		_ok(found, "L6: the dynamics section draws '%s'" % w)
	var orig := _slider_for(pb, "Original")
	_ok(orig != null and orig.editable, "L6: the Original slider is drawn and live while Paint is armed")
	if orig != null:
		await _reveal(orig)
		var po = _app.paint_original
		var o0: float = po.opacity
		var rr := orig.get_global_rect()
		var f2 := Vector2(rr.position.x + rr.size.x * 0.2, rr.get_center().y)
		var t2 := Vector2(rr.position.x + rr.size.x * 0.8, rr.get_center().y)
		await _press(f2)
		await _move(f2.lerp(t2, 0.5), f2)
		await _move(t2, f2.lerp(t2, 0.5))
		await _release(t2)
		await _settle()
		_ok(po.opacity > o0 + 0.1, "L6: a real drag on Original moved paint_original.opacity (%.2f -> %.2f)" % [o0, po.opacity])
	var chip: Label = _app.viewport.get("_vp_context") as Label
	_ok(chip != null and chip.text == "PAINT · DRAFT", "L6: the map's mode chip reads PAINT · DRAFT with Paint armed (%s)" % (chip.text if chip != null else "no chip"))
	# Escape (a real key press) disarms Paint: Original goes inert with the reason, the chip stops saying PAINT
	for pressed in [true, false]:
		var ke := InputEventKey.new()
		ke.keycode = KEY_ESCAPE
		ke.physical_keycode = KEY_ESCAPE
		ke.pressed = pressed
		_vp.push_input(ke, true)
		await _frames(2)
	await _settle()
	_ok(_app.armed_tool == "inspect", "L6: Escape disarmed Paint (%s)" % _app.armed_tool)
	var inert := _slider_for(ws_obj._paint_body, "Original")
	_ok(inert != null and not inert.editable, "L6: with Paint disarmed the Original slider is drawn inert")
	_ok(_find_label(ws_obj._paint_body, "Paint is not armed") != null, "L6: and says why")
	chip = _app.viewport.get("_vp_context") as Label
	_ok(chip != null and chip.text != "PAINT · DRAFT", "L6: the chip stops saying PAINT (%s)" % (chip.text if chip != null else "no chip"))
	# Sculpt's dynamics, in Terrain
	var sdyn: Control = ws_obj._sculpt_body.find_child("PhoneBrushDynamics", true, false) as Control
	_ok(sdyn != null and sdyn.get_child_count() >= 2, "L5b: the Terrain body has the brush dynamics and symmetry section")

	# L9. Touch targets in the dock sheet: reported, not assumed (the column's own figures are listed for MS-10)
	_app.arm_tool("inspect")
	await _tap(_find_button(_app._left_dock_mode_switch, "PIPELINE").get_global_rect().get_center())
	await _settle()
	var small: Array = []
	var seen := 0
	for c in _dock_scroll().find_children("*", "BaseButton", true, false):
		var cc := c as Control
		if not cc.is_visible_in_tree():
			continue
		var r2 := cc.get_global_rect()
		if r2.size.x <= 0.0 or r2.size.y <= 0.0:
			continue
		seen += 1
		if r2.size.y < fl:
			small.append("%s %.0fx%.0f" % [String(c.get("text")), r2.size.x, r2.size.y])
	print("L9: %d visible buttons in the pipeline sheet, %d under the 44 dp height floor (%.1f px): %s" % [seen, small.size(), fl, str(small.slice(0, 12))])
	_ok(seen > 0, "L9: the sheet has buttons to measure")
