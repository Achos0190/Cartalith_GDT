extends Node
## Phone map-view probe (2026-10-26 owner findings, OnePlus 6T): the zoom buttons were drawn over and behind the
## armed-tool pill (INSPECT), and a map re-centred on a place near a world edge showed black void.
##
##   portrait: godot --path . --resolution 393x852 _phonemap_probe.tscn -- --force-touch --shots <dir>
##             godot --path . --resolution 412x915 _phonemap_probe.tscn -- --force-touch
##   landscape: godot --path . --resolution 852x393 _phonemap_probe.tscn -- --force-touch
##
## **Windowed, never `--headless`.** Run from a worktree with its own `config/name` and a wiped user dir. Arguments
## read: `--force-touch`, `--shots <dir>`; anything else aborts with exit 2. It refuses to run off the phone form.
##
## ## What it protects (each check names its own number; a mutation of the fix turns the named ones red)
##   1. NO OVERLAP: at the peek, half and full detents (portrait; the one landscape sheet), no two of {the zoom
##      rocker's halves, pan, fit, 3D, the armed-tool pill, the layers button} overlap, and none of the zoom
##      row or pill reaches into the tool sheet.
##   2. TOPMOST AT THE CENTRE: the topmost mouse-reachable control at each button's centre (walking the live
##      tree in draw order) is that button (or one of its own children), and the pill's centre is the pill.
##   3. 44 DP: every one of those controls is at least 44 dp on both axes.
##   4. THE BUTTONS WORK: a real mouse click at each centre does what its button does (zoom in grows the zoom,
##      zoom out shrinks it, 3D toggles and toggles back, a click on the pill does not zoom) -- clicks, not calls.
##   5. NO VOID AFTER A RE-CENTRE: zoomed in, `move_view_to()` on each of the four world corners leaves the map
##      covering the whole viewport node (no black strip), and on an interior point still centres it exactly.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.

const SEED := 483920
const FLOOR_DP := 44.0

var _fails := 0
var _checks := 0
var _shots := ""
var _app: Node
var _vh
var _tag := "p"


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _shot(name_: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	var path := _shots.path_join("pm_%s_%s.png" % [_tag, name_])
	get_viewport().get_texture().get_image().save_png(path)
	print("SHOT %s" % path)


func _click_at(at: Vector2) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 2)
	Input.parse_input_event(mm)
	await _settle(2)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(3)


## The topmost visible Control that would take a mouse event at `p` (mouse_filter other than IGNORE), by a
## pre-order walk of the live tree from the root: a node later in that order draws over an earlier one.
func _topmost_at(p: Vector2) -> Control:
	var best: Control = null
	var stack: Array = [get_tree().root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is Window and n != get_tree().root and not (n as Window).visible:
			continue   # a closed popup or dialog keeps its Controls `visible` but draws nothing
		if n is Control:
			var c := n as Control
			if not c.visible:
				continue
			if c.mouse_filter != Control.MOUSE_FILTER_IGNORE and c.get_global_rect().has_point(p):
				best = c
		elif n is CanvasItem and not (n as CanvasItem).visible:
			continue
		var kids := n.get_children()
		kids.reverse()
		for k in kids:
			stack.append(k)
	return best


func _is_or_inside(c: Control, owner_: Control) -> bool:
	var n: Node = c
	while n != null:
		if n == owner_:
			return true
		n = n.get_parent()
	return false


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := args[i]
		if a == "--force-touch":
			pass
		elif a == "--shots" and i + 1 < args.size():
			i += 1
			_shots = args[i]
		else:
			print("ABORT: unrecognised argument %s -- this probe reads --force-touch and --shots <dir> only" % a)
			get_tree().quit(2)
			return
		i += 1
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	if _app.phone_project_picker != null and _app.phone_project_picker.visible:
		_app.phone_project_picker.hide()   # the phone's full-screen entry picker covers everything under test
	await _settle(6)
	if not _app.is_phone():
		print("ABORT: this probe is for the phone form (--resolution 393x852 -- --force-touch)")
		get_tree().quit(2)
		return
	var land: bool = _app._landscape
	var vs := get_viewport().get_visible_rect().size
	_tag = "land" if land else "%dx%d" % [int(vs.x), int(vs.y)]
	print("== phone %s, viewport %s, phone_scale %.4f ==" % ["landscape" if land else "portrait", vs, _app.phone_scale()])
	var bridge = _app.bridge
	bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	if _app.phone_project_picker != null and _app.phone_project_picker.visible:
		_app.phone_project_picker.hide()
	await _settle(8)
	_vh = _app.viewport
	_app._pick_phone_tab("map")
	await _settle(10)
	if land:
		await _controls("landscape")
	else:
		for det in ["peek", "half", "full"]:
			_app.set_phone_detent(det)
			await _settle(30)
			await _controls(det)
		_app.set_phone_detent("half")
		await _settle(30)
	await _functional()
	await _no_void()
	_finish()


func _nav_buttons() -> Array:
	var nav: Control = _vh._navpad
	if nav == null or not nav.is_visible_in_tree():
		return []
	var out: Array = []
	for b in nav.find_children("*", "Button", true, false):
		if (b as Button).is_visible_in_tree():
			out.append(b)
	return out


## Sections 1-3 at the current detent.
func _controls(label: String) -> void:
	print("== controls at %s ==" % label)
	await _shot(label)
	var sc: float = _app.phone_scale()
	var items: Array = []   # [name, Control]
	var nav_btns := _nav_buttons()
	for b in nav_btns:
		var bb := b as Button
		items.append(["navpad %s" % (bb.text if bb.text != "" else bb.tooltip_text), bb])
	var pill: Control = _app._tool_fan._pill
	if pill != null and pill.is_visible_in_tree():
		items.append(["tool pill", pill])
	var layers: Control = _vh._layers_btn
	# At `full` the sheet covers the whole map, the layers button with it: that is the sheet's job, not a defect.
	if layers != null and layers.is_visible_in_tree() and label != "full":
		items.append(["layers button", layers])
	print("controls: ", items.map(func(e): return e[0]))
	if label != "full":
		_ok(nav_btns.size() == 5, "%s: the zoom row is shown with its five buttons (%d)" % [label, nav_btns.size()])
	_ok(pill != null and pill.is_visible_in_tree(), "%s: the armed-tool pill is shown" % label)
	# 1. pairwise overlap
	for a in items.size():
		for b in range(a + 1, items.size()):
			var ra: Rect2 = (items[a][1] as Control).get_global_rect()
			var rb: Rect2 = (items[b][1] as Control).get_global_rect()
			_ok(not ra.intersects(rb), "%s: '%s' %s and '%s' %s do not overlap" % [label, items[a][0], ra, items[b][0], rb])
	# ... and none reaches into the tool sheet
	var sheet: Control = _app._phone_tool_sheet
	if sheet != null and sheet.is_visible_in_tree() and not _app._landscape:
		var top: float = sheet.get_global_rect().position.y
		for e in items:
			if e[0] == "layers button":
				continue
			var r: Rect2 = (e[1] as Control).get_global_rect()
			_ok(r.end.y <= top + 0.5, "%s: '%s' bottom %.1f is above the sheet top %.1f" % [label, e[0], r.end.y, top])
	# 2. topmost at the centre, 3. 44 dp
	for e in items:
		var c: Control = e[1]
		var r: Rect2 = c.get_global_rect()
		var top_c := _topmost_at(r.get_center())
		_ok(top_c != null and _is_or_inside(top_c, c), "%s: topmost control at the centre of '%s' is it (got %s)" % [label, e[0], str(top_c.get_path()) if top_c != null else "none"])
		if e[0] != "layers button":   # the layers button's size is not in this finding (it is its own row)
			_ok(r.size.x / sc >= FLOOR_DP - 0.5 and r.size.y / sc >= FLOOR_DP - 0.5, "%s: '%s' is at least 44 dp (%.1f x %.1f dp)" % [label, e[0], r.size.x / sc, r.size.y / sc])


## Section 4: real clicks.
func _functional() -> void:
	print("== real clicks ==")
	var btns := _nav_buttons()
	if btns.is_empty():
		_ok(false, "the zoom row is not shown at the half detent, so nothing can be clicked")
		return
	var by_tip := {}
	for b in btns:
		by_tip[(b as Button).tooltip_text] = b
	print("tooltips: ", by_tip.keys())
	var z0: float = _vh._zoom
	var zin = by_tip.get("Zoom in")
	var zout = by_tip.get("Zoom out")
	_ok(zin != null and zout != null, "both zoom halves are found by tooltip (Zoom in / Zoom out)")
	if zin != null:
		await _click_at((zin as Control).get_global_rect().get_center())
		var z1: float = _vh._zoom
		_ok(z1 > z0 + 0.001, "a real click on Zoom in grows the zoom (%.3f -> %.3f)" % [z0, z1])
		if zout != null:
			await _click_at((zout as Control).get_global_rect().get_center())
			_ok(absf(_vh._zoom - z0) < 0.01, "a real click on Zoom out brings it back (%.3f)" % _vh._zoom)
	var v3: Button = _vh._view3d_btn
	if v3 != null:
		var before: bool = _vh.is_3d()
		await _click_at(v3.get_global_rect().get_center())
		_ok(_vh.is_3d() != before, "a real click on 3D toggles the view (3D now %s)" % _vh.is_3d())
		await _click_at(v3.get_global_rect().get_center())
		_ok(_vh.is_3d() == before, "a second click toggles it back")
	var pill: Control = _app._tool_fan._pill
	if pill != null:
		var zc: float = _vh._zoom
		await _click_at(pill.get_global_rect().get_center())
		_ok(absf(_vh._zoom - zc) < 0.0001, "a click on the pill's centre does not zoom the map")


## Section 5.
func _no_void() -> void:
	print("== no void after re-centring ==")
	_vh.reset_view()
	await _settle(4)
	for _k in 6:
		_vh.zoom_step(1.35)
	await _settle(4)
	var g: Vector2i = _vh._bridge.grid_size()
	var sz: Vector2 = _vh.size
	var corners := [Vector2(0, 0), Vector2(g.x - 1, 0), Vector2(0, g.y - 1), Vector2(g.x - 1, g.y - 1)]
	for cpt in corners:
		_vh.move_view_to(cpt.x, cpt.y)
		await _settle(3)
		var rect: Rect2 = _vh.overlay.displayed_rect()
		var z: float = _vh._zoom
		var tl: Vector2 = _vh._camera.position + rect.position * z
		var br: Vector2 = tl + rect.size * z
		_ok(z > 1.5, "zoomed in enough to need the clamp (zoom %.2f)" % z)
		_ok(tl.x <= 0.5 and tl.y <= 0.5 and br.x >= sz.x - 0.5 and br.y >= sz.y - 0.5,
			"re-centring on corner %s leaves the map covering the view (map %s .. %s in %s)" % [cpt, tl, br, sz])
	var mid := Vector2(g.x * 0.5, g.y * 0.5)
	_vh.move_view_to(mid.x, mid.y)
	await _settle(3)
	var rect2: Rect2 = _vh.overlay.displayed_rect()
	var z2: float = _vh._zoom
	var pt: Vector2 = _vh._camera.position + (rect2.position + Vector2((mid.x + 0.5) / g.x, (mid.y + 0.5) / g.y) * rect2.size) * z2
	_ok(pt.distance_to(sz * 0.5) < 0.5, "an interior point is still centred exactly (offset %.2f px)" % pt.distance_to(sz * 0.5))
	await _shot("after_recentre")
