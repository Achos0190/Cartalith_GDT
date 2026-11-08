extends Node
## MS-3 probe (`MOBILE_SHELL_PROPOSAL.md` section 6, MS-3: "no control reachable only through Window > Left
## dock (walk every category, count 0)"). Boots the real shell as a phone in a `--vp`-sized host `SubViewport`
## and drives REAL pointer events: the domain tab's swipe picks CIVIL and CARTO, and the probe then walks the
## domain sheet -- the dock's own category scroller, reparented into the phone tool sheet by
## `DccShell._mount_dock_in_sheet()` -- counting every category of the domain's `RAIL_NODES` `owns` lists that
## is NOT drawn inside the sheet. Also: a real tap on a category header opens its body, Window > Left dock
## still gets the dock back, WORLD keeps its GENERATE column, and the touch floor of the header rows.
##
## Windowed, with a wiped user dir (the shell remembers coach marks and layout in the user dir):
##   godot --path . --resolution 1600x1000 _ms3_probe.tscn -- --vp 1080x2340 --density 2.625 --force-touch
##   godot --path . --resolution 1600x1000 _ms3_probe.tscn -- --vp 2340x1080 --density 2.625 --force-touch
##   godot --path . --resolution 1600x1000 _ms3_probe.tscn -- --vp 1440x3120 --density 3 --force-touch
##   godot --path . --resolution 1600x1000 _ms3_probe.tscn -- --vp 2388x1668 --density 2 --force-touch
##   godot --path . --resolution 1600x1000 _ms3_probe.tscn -- --vp 1080x1920 --density 3 --force-touch   (360 dp)
## Arguments read by THIS file: `--vp WxH`. Accepted because the shell reads them: `--density D`, `--force-touch`.
## Anything else aborts with exit 2 (`MISTAKES.md`, "Write a probe's usage header").
##
## **Every expected category list is a literal written by hand from `RAIL_NODES`' `owns` lists as the
## proposal names them** (CIVIL ▸ Factions: ten categories; CARTO ▸ Style: four); nothing is asserted against
## the constant it came from. Mutants run by hand and listed in the commit: `_mount_dock_in_sheet` made a
## no-op (the sheet stays the one-line tool row), the `_active_domain != "world"` clause dropped (WORLD loses
## its GENERATE column), `_set_sheet_open`'s hand-back removed (Window > Left dock opens empty).
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

## Hand-written from `RAIL_NODES`' `owns` lists (proposal section 2.1): every category the domain's nodes own.
const CIVIL_CATS := ["Landmarks", "Populate", "Settlements", "Factions", "Territories", "Relationships",
	"Military", "Culture", "Religion", "Economy", "Timeline", "Routes & ways", "Travel"]
const CARTO_CATS := ["Style", "Relief & light", "Colours", "Feature style", "Layers", "Labels", "Icons"]

func _dock_scroll() -> ScrollContainer:
	return _app._left_dock_scroll as ScrollContainer

func _in_sheet() -> bool:
	return _dock_scroll().get_parent() == _app._phone_dom_host

func _cat_entry(ws: Node, title: String) -> Dictionary:
	for e in ws.categories:
		if String((e as Dictionary).get("title", "")) == title:
			return e
	return {}

## Counts the domain's categories that are NOT drawn inside the open domain sheet (the bar is zero).
func _walk(domain: String, wanted: Array) -> void:
	var ws: Node = _app._workspace_panels.get(domain)
	_ok(ws != null, "%s: its workspace panel exists" % domain)
	if ws == null:
		return
	var missing: Array = []
	for t in wanted:
		var e := _cat_entry(ws, String(t))
		var body: Control = e.get("body") as Control
		var wrap: Control = body.get_parent() as Control if body != null else null
		if e.is_empty() or wrap == null or not wrap.is_visible_in_tree() \
				or not _dock_scroll().is_ancestor_of(wrap):
			missing.append(t)
	_ok(missing.is_empty(), "%s: every one of %d categories is in the domain sheet, count not drawn = %d %s" % [
		domain, wanted.size(), missing.size(), str(missing)])

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

	# 1. WORLD: keeps its GENERATE column; the dock stays in its own left dock
	_ok(_app._active_domain == "world", "boots on WORLD")
	_ok(not _in_sheet() and not _app._phone_dom_host.visible, "WORLD: the domain host is empty and hidden")
	_ok(_app._phone_gen_scroll.visible, "WORLD: the phone GENERATE column is still the sheet's body")

	# 2. a real swipe (left = next) picks CIVIL; the sheet then hosts the dock
	for dom in ["civilization", "cartography"]:
		await _hold_slide(cell_c, cell_c + Vector2(-dx, 0), 0, true, true)
		await _settle()
		_ok(_app._active_domain == dom, "swipe selects %s (got %s)" % [dom, _app._active_domain])
		if _app._phone_detent == "peek":
			await _tap(_centre("domain"), true)
			await _settle()
		_ok(_app._phone_detent != "peek", "%s: the sheet is up (%s)" % [dom, _app._phone_detent])
		_ok(_in_sheet() and _app._phone_dom_host.visible, "%s: the dock's scroller sits in the domain host" % dom)
		_ok(not _app._phone_gen_scroll.visible and not _app._phone_tool_scroll.visible,
			"%s: neither the GENERATE column nor the one-line tool row is the body" % dom)
		var sc := _dock_scroll()
		_ok(sc.is_visible_in_tree(), "%s: the scroller is visible" % dom)
		var r := sc.get_global_rect()
		_ok(r.size.y / _dens >= 120.0 and r.end.y <= vh + 0.5 and r.end.x <= vw + 0.5 and r.position.x >= -0.5,
			"%s: the scroller is at least 120 dp tall and on screen (%s)" % [dom, str(r)])
		_walk(dom, CIVIL_CATS if dom == "civilization" else CARTO_CATS)
		# a real tap on a closed category's header opens its body
		var ws: Node = _app._workspace_panels.get(dom)
		var picked := false
		for e in ws.categories:
			var body: Control = (e as Dictionary).get("body") as Control
			if body == null or body.visible or not body.get_parent().visible:
				continue
			var wrap: Control = body.get_parent() as Control
			sc.ensure_control_visible(wrap)
			await _frames(3)
			var head: Control = wrap.get_child(0) as Control
			## The phone sizes a 44 dp target as 44 x phone_scale() px (short side / 412), rounded down to whole px, so at
			## 1080 px / 2.625 density it is 115 px = 43.8 dp by the DEVICE's density: the floor is 44 x phone_scale() less
			## one px of rounding, not 44 x density (ms2 passes only because its cells are taller than the floor).
			_ok(head.size.y >= 44.0 * float(_app.phone_scale()) - 1.0, "%s: header '%s' meets the 44 dp floor (%.1f px, %.1f dp)" % [
				dom, String(e.get("title", "")), head.size.y, head.size.y / _dens])
			await _tap(head.get_global_rect().get_center(), true)
			await _settle()
			_ok(body.visible, "%s: a tap on '%s' opens it" % [dom, String(e.get("title", ""))])
			picked = true
			break
		_ok(picked, "%s: some closed category existed to tap" % dom)
		# Window > Left dock still gets the dock
		_app._set_sheet_open("left", true)
		await _settle()
		_ok(not _in_sheet() and _app.left_dock.visible and sc.is_visible_in_tree(),
			"%s: Window > Left dock hands the dock back and shows it" % dom)
		_app._set_sheet_open("left", false)
		await _settle()
		await _tap(_centre("domain"), true)
		await _settle()
		if _app._phone_detent == "peek":
			await _tap(_centre("domain"), true)
			await _settle()
		_ok(_in_sheet(), "%s: the domain tab takes the dock back into the sheet" % dom)

	# 3. leaving the domain tab returns the dock
	await _tap(_centre("story"), true)
	await _settle()
	_ok(not _in_sheet(), "STORY open: the dock is back in its own column")
	await _tap(_centre("story"), true)
	await _settle()

	# 4. WORLD again: the GENERATE column, not the dock
	await _hold_slide(cell_c, cell_c + Vector2(-dx, 0), 0, true, true)
	await _settle()
	_ok(_app._active_domain == "world", "swipe from CARTO wraps to WORLD (got %s)" % _app._active_domain)
	_ok(not _in_sheet() and _app._phone_gen_scroll.visible, "WORLD again: the GENERATE column is the body, dock not mounted")
