extends Node
## MS-3 remainder probe (`MOBILE_SHELL_PROPOSAL.md` section 6, MS-3: "no control reachable only through Window >
## Left dock (walk every category, count 0)"). Drives REAL pointer events against the real shell as a phone in a
## `--vp`-sized host `SubViewport` and checks the four parts MS-3's first step left:
##   1. WORLD: the sheet's fourth segment ALL (a real tap) mounts the dock's own nine categories
##      (Generate, Planet, Geology, Hydrology, Climate, Ecology, World data; then Terrain, Biomes after a real tap
##      on the dock's SCULPT pill), the column is hidden, a back row returns to the column;
##   2. CIVIL: the Journeys row (a real tap) opens STORY > Journeys (the planner row is on screen);
##   3. MAP: its own row (Layers & fields, Fit world, 2D / 3D) at the 44 dp floor; a real tap on Layers & fields
##      opens the Layers popover;
##   4. the fit hook: a button added under `_phone_dom_host` is sized by `_on_phone_node_added()`'s
##      `p == _phone_dom_host` clause (a control added late is brought to the 44 dp floor).
##
## Windowed, with a wiped user dir:
##   godot --path . --resolution 1600x1000 _ms3b_probe.tscn -- --vp 1080x2340 --density 2.625 --force-touch
##   (also 2340x1080 @2.625, 1080x2340 @2.8125, 1440x3120 @3, 2388x1668 @2, 1080x1920 @3 = 360 dp)
## Arguments read by THIS file: `--vp WxH`. Accepted because the shell reads them: `--density D`, `--force-touch`.
## Anything else aborts with exit 2 (`MISTAKES.md`, "Write a probe's usage header").
##
## **Every expected category list is a literal written by hand from `RAIL_NODES`' `owns` lists**; nothing is
## asserted against the constant it came from. Mutants (run by hand, listed in the commit): the ALL segment
## removed, `world_all` forced false, the `p == _phone_dom_host` clause dropped, the journeys row's callback
## emptied, the MAP row never shown.
##
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument.

var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node
var _dens := 1.0
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

## One finger as the project delivers it with mouse emulation on: a touch and its emulated mouse button.
func _press(p: Vector2, both: bool) -> void:
	var m := InputEventMouseButton.new()
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = true
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	if both:
		var t := InputEventScreenTouch.new()
		t.index = 0
		t.pressed = true
		t.position = p
		_vp.push_input(t)
	_vp.push_input(m)
	await _frames(2)

func _move(p: Vector2, from: Vector2, both: bool) -> void:
	var m := InputEventMouseMotion.new()
	m.position = p
	m.global_position = p
	m.relative = p - from
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	if both:
		var d := InputEventScreenDrag.new()
		d.index = 0
		d.position = p
		d.relative = p - from
		_vp.push_input(d)
	_vp.push_input(m)
	await _frames(2)

func _release(p: Vector2, both: bool) -> void:
	var m := InputEventMouseButton.new()
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = false
	m.position = p
	m.global_position = p
	if both:
		var t := InputEventScreenTouch.new()
		t.index = 0
		t.pressed = false
		t.position = p
		_vp.push_input(t)
	_vp.push_input(m)
	await _frames(3)

func _tap(p: Vector2, both: bool = false) -> void:
	await _press(p, both)
	await _release(p, both)

## Hold on `p` for `ms`, slide in three steps to `to`, and (if `lift`) release there.
func _hold_slide(p: Vector2, to: Vector2, ms: int, lift: bool, both: bool = false) -> void:
	await _press(p, both)
	await get_tree().create_timer(ms / 1000.0).timeout
	var cur := p
	for s in 3:
		var nxt := p.lerp(to, float(s + 1) / 3.0)
		await _move(nxt, cur, both)
		cur = nxt
	if lift:
		await _release(to, both)

func _cell(id: String) -> Button:
	return (_app._phone_tab_cells.get(id, {}) as Dictionary).get("button") as Button

func _centre(id: String) -> Vector2:
	var b := _cell(id)
	return b.get_global_rect().get_center() if b != null else Vector2(-1, -1)

func _caption(id: String) -> String:
	var l := (_app._phone_tab_cells.get(id, {}) as Dictionary).get("label") as Label
	return l.text if l != null else "<none>"

func _texts(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	for c in n.get_children():
		_texts(c, out)

func _menu_has(text: String) -> bool:
	var out: Array = []
	_texts(_app._phone_menu, out)
	for t in out:
		if String(t).contains(text):
			return true
	return false

func _settle() -> void:
	await get_tree().create_timer(0.3).timeout
	await _frames(3)


# ---- the run -------------------------------------------------------------------------------------------------

const WORLD_A := ["Generate", "Planet", "Geology", "Hydrology", "Climate", "Ecology", "World data"]
const WORLD_B := ["Terrain", "Biomes"]

func _dock_scroll() -> ScrollContainer:
	return _app._left_dock_scroll as ScrollContainer

func _in_sheet() -> bool:
	return _dock_scroll().get_parent() == _app._phone_dom_host

func _cat_entry(ws: Node, title: String) -> Dictionary:
	for e in ws.categories:
		if String((e as Dictionary).get("title", "")) == title:
			return e
	return {}

func _walk(domain: String, wanted: Array) -> void:
	var ws: Node = _app._workspace_panels.get(domain)
	var missing: Array = []
	for t in wanted:
		var e := _cat_entry(ws, String(t))
		var body: Control = e.get("body") as Control
		var wrap: Control = body.get_parent() as Control if body != null else null
		if e.is_empty() or wrap == null or not wrap.is_visible_in_tree() \
				or not _dock_scroll().is_ancestor_of(wrap):
			missing.append(t)
	_ok(missing.is_empty(), "%s: every one of %d categories is drawn in the sheet, count not drawn = %d %s" % [
		domain, wanted.size(), missing.size(), str(missing)])

func _find_button(n: Node, text: String) -> Button:
	if n is Button and (n as Button).text == text and (n as Button).is_visible_in_tree():
		return n as Button
	for c in n.get_children():
		var r := _find_button(c, text)
		if r != null:
			return r
	return null

## 44 dp on the phone is 44 x phone_scale() px, rounded down (see `_ms3_probe.gd`).
func _floor_px() -> float:
	return 44.0 * float(_app.phone_scale()) - 1.0

func _swipe_to(dom: String, cell_c: Vector2, dx: float) -> void:
	var guard := 0
	while _app._active_domain != dom and guard < 4:
		await _hold_slide(cell_c, cell_c + Vector2(-dx, 0), 0, true, true)
		await _settle()
		guard += 1
	_ok(_app._active_domain == dom, "swipe selects %s (got %s)" % [dom, _app._active_domain])
	if _app._phone_detent == "peek":
		await _tap(_centre("domain"), true)
		await _settle()

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
	var cell_c := _centre("domain")
	var dx: float = 48.0 * float(_app.phone_scale()) + 24.0
	var fl := _floor_px()

	# 1. WORLD: the column, then ALL
	_ok(_app._active_domain == "world" and _app._phone_gen_scroll.visible and not _in_sheet(),
		"WORLD boots with the GENERATE column and the dock in its own column")
	if _app._phone_detent == "peek":
		await _tap(_centre("domain"), true)
		await _settle()
	_ok(_app._phone_detent != "peek", "WORLD: the sheet is up (%s)" % _app._phone_detent)
	var all_btn := _find_button(_app._phone_gen_col, "ALL")
	_ok(all_btn != null, "WORLD: the segment row has an ALL cell")
	if all_btn == null:
		return
	_ok(all_btn.get_global_rect().size.y >= fl,
		"WORLD: ALL is at the 44 dp floor (%s px, floor %.1f)" % [str(all_btn.get_global_rect().size), fl])
	_ok(all_btn.get_global_rect().end.y <= float(_vp.size.y) and all_btn.get_global_rect().position.y >= 0.0,
		"WORLD: ALL is on screen (%s)" % str(all_btn.get_global_rect()))
	await _tap(all_btn.get_global_rect().get_center(), true)
	await _settle()
	_ok(_in_sheet() and _app._phone_dom_host.visible, "WORLD ALL: the dock's scroller sits in the domain host")
	_ok(not _app._phone_gen_scroll.visible and not _app._phone_tool_scroll.visible,
		"WORLD ALL: the GENERATE column and the tool row are not the body")
	_ok(_app._phone_dom_back_row != null and _app._phone_dom_back_row.visible, "WORLD ALL: the back row is shown")
	_walk("world", WORLD_A)
	_ok(String(_app._phone_sheet_subtitle.text).contains("all categories"),
		"WORLD ALL: the sheet subtitle says so (%s)" % _app._phone_sheet_subtitle.text)
	# the dock's own mode switch reaches Terrain and Biomes
	var sculpt := _find_button(_app._left_dock_mode_switch, "SCULPT")
	_ok(sculpt != null, "WORLD ALL: the dock's mode switch has SCULPT")
	if sculpt != null:
		await _tap(sculpt.get_global_rect().get_center(), true)
		await _settle()
		_walk("world", WORLD_B)
		var pipe := _find_button(_app._left_dock_mode_switch, "PIPELINE")
		if pipe != null:
			await _tap(pipe.get_global_rect().get_center(), true)
			await _settle()
	# the fit hook: a control added late under the host is sized by the hook, not at creation
	var late := Button.new()
	late.text = "late"
	_app._phone_dom_host.add_child(late)
	await _settle()
	_ok(late.custom_minimum_size.y >= fl or late.size.y >= fl,
		"fit hook: a button added under _phone_dom_host reaches the 44 dp floor (min %.1f, size %.1f, floor %.1f)" % [
			late.custom_minimum_size.y, late.size.y, fl])
	late.queue_free()
	# back to the columns
	var back: Button = _app._phone_dom_back_row as Button
	_ok(back.get_global_rect().size.y >= fl, "WORLD ALL: the back row is at the 44 dp floor (%.1f px)" % back.get_global_rect().size.y)
	await _tap(back.get_global_rect().get_center(), true)
	await _settle()
	_ok(not _in_sheet() and _app._phone_gen_scroll.visible and not _app._phone_dom_back_row.visible,
		"WORLD: the back row returns the column and hands the dock back")

	# 2. CIVIL: Journeys opens STORY > Journeys
	await _swipe_to("civilization", cell_c, dx)
	_ok(_in_sheet(), "CIVIL: the dock is in the sheet")
	var jr: Button = _app._phone_dom_journeys_row as Button
	_ok(jr != null and jr.visible, "CIVIL: the Journeys row is shown")
	if jr != null:
		_ok(jr.get_global_rect().size.y >= fl, "CIVIL: the Journeys row is at the 44 dp floor (%.1f px)" % jr.get_global_rect().size.y)
		await _tap(jr.get_global_rect().get_center(), true)
		await _settle()
		_ok(_app._phone_menu.is_open() and _app._phone_tab == "story", "CIVIL Journeys: STORY is open (tab %s)" % _app._phone_tab)
		_ok(_menu_has("Journey planner"), "CIVIL Journeys: the planner row is on screen (STORY > Journeys)")
		_ok(not _in_sheet(), "CIVIL Journeys: the dock went back to its own column")
		await _tap(_centre("story"), true)
		await _settle()
		_ok(not _app._phone_menu.is_open(), "STORY closes on a second tap")

	# 3. MAP: its own row
	await _tap(_centre("map"), true)
	await _settle()
	_ok(_app._phone_map_row != null and _app._phone_map_row.visible, "MAP: its row is shown")
	if _app._phone_map_row != null:
		var texts: Array = []
		for c in _app._phone_map_row.get_children():
			texts.append((c as Button).text)
			_ok((c as Button).get_global_rect().size.y >= fl, "MAP: '%s' is at the 44 dp floor (%.1f px)" % [
				(c as Button).text, (c as Button).get_global_rect().size.y])
		_ok(texts == ["LAYERS & FIELDS", "FIT WORLD", "2D / 3D"], "MAP: the row reads %s" % str(texts))
		var lf := _find_button(_app._phone_map_row, "LAYERS & FIELDS")
		await _tap(lf.get_global_rect().get_center(), true)
		await _settle()
		_ok(_app.layers_popover.visible, "MAP: a tap on Layers & fields opens the Layers popover")
		_app.layers_popover.hide()
		await _settle()
		var before_zoom: float = _app.viewport._zoom
		_app.viewport._zoom = before_zoom * 2.0
		var fw := _find_button(_app._phone_map_row, "FIT WORLD")
		await _tap(fw.get_global_rect().get_center(), true)
		await _settle()
		_ok(is_equal_approx(_app.viewport._zoom, before_zoom), "MAP: Fit world resets the view (zoom %.3f, was %.3f)" % [
			_app.viewport._zoom, before_zoom])
	# the row leaves with the MAP tab
	await _tap(_centre("domain"), true)
	await _settle()
	_ok(_app._phone_map_row == null or not _app._phone_map_row.visible, "the MAP row is hidden off the MAP tab")
