extends Node
## Phone landscape probe (owner report 2026-11-13, `OUTSTANDING_WORK.md` "Phone landscape: the shell fills only part of
## the screen"). The phone shell in a REAL root window (not a `--vp` host), through a cold start in landscape and through
## window resizes (a rotation), with real `Window.size` changes the shell hears on `root.size_changed`.
##
## What the report was, found by this probe: the shell's root, its `_phone_root` and the map host DID fill the window after a
## rotation; what did not was the docked landscape tool sheet. Rotating a phone that sat at the `peek` detent (the cold-start
## detent) into landscape left every child of the 46 %-wide sheet hidden (`_phone_sheet_peeked()` ignored the orientation and
## nothing re-ran it on a rotation): an empty panel, with the map showing through the gesture band under it. Checks:
##   1. THE WINDOW IS FILLED: the shell, `_phone_root` and the map host are exactly the window's rect;
##   2. THE BAND: the map's own area is what is left of the rail and the sheet (`phone_content_insets()`), top 0 in landscape;
##   3. THE SHEET HAS A BODY: at least one tall visible control inside it, and it ends at the window's right edge, below the
##      header and above the gesture band;
##   4. THE HEADER: top-right corner (right edge on the window's, top 0, right half), floating: an EMPTY panel style, no opaque
##      backdrop; back in portrait it spans the width again with the same empty style;
##   5. THE GRIP: on the right edge of the band, wholly on screen, vertically in the band; with a display cut-out on the right
##      (`cutouts_probe`, a `Rect2` list the shell reads where the platform has none) it is centred on the cut-out's height;
##   6. THE BOTTOM EDGE: the two strips under the rail and the sheet exist in landscape and are hidden in portrait.
## Usage (windowed, wiped user dir; `--resolution` is the START size, a rotation swaps it):
##   godot --path . --resolution 1170x540 _phoneland_probe.tscn -- --force-touch --density 1.40625 --leg cold
##   godot --path . --resolution 540x1000 _phoneland_probe.tscn -- --force-touch --density 1.40625 --leg rotate
## 1170x540 at 1.40625 is 832 x 384 dp, the shape of the 6T's 2340x1080 at 2.8125; the same dp at any scale.
## Legs: `cold` (boot in landscape, checks 1 to 6), `rotate` (boot portrait, rotate to landscape, back, to landscape again),
## `plain` / `plainrot` (screenshots only, no pins, no assertions beyond the phone form), `dump` (prints the sheet's tree).
## Reads: `--leg`, `--shots DIR`; the shell reads `--force-touch`, `--density`. Anything else aborts with exit 2.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument, 3 watchdog.
var _fails := 0
var _checks := 0
var _app: Node
var _shots := ""

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1

func _frames(n: int) -> void:
	for k in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var k := args.find(name)
	return String(args[k + 1]) if k >= 0 and k + 1 < args.size() else dflt

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 200.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch":
			i += 1
		elif a in ["--leg", "--shots", "--density"]:
			i += 2
		else:
			print("ABORT: unknown argument ", a)
			get_tree().quit(2)
			return
	_shots = _arg("--shots", "")
	await _run(_arg("--leg", "cold"))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)

func _shot(tag: String) -> void:
	if _shots == "":
		return
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	img.save_png("%s/%s.png" % [_shots, tag])

func _settle() -> void:
	await get_tree().create_timer(0.6).timeout
	await _frames(4)

func _store_pins() -> void:
	## Two pins so the quick drawer mounts (an empty bar draws no drawer): the same keys `_ms11_probe.gd` stores.
	for k in DccSettings.quick_bar_pins_for("bar", "").duplicate():
		DccSettings.quick_bar_remove("bar", String(k["k"]))
	var idx := CommandIndex.new()
	idx.build(_app, _app.bridge)
	for k in ["menu:726", "tool:measure"]:
		var row := idx.find_pin(String(k))
		DccSettings.quick_bar_add("bar", String(k), String(row.get("title", "")) if not row.is_empty() else String(k))
	_app.phone_quick.refresh()
	await _frames(6)

func _run(leg: String) -> void:
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.6).timeout
	DccSettings.set_dominant_hand("right")
	var bridge = _app.bridge
	bridge.generate({"seed": 24601, "width_km": 1200.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	_ok(DccTheme.is_phone(), "the shell is the phone form")
	if not DccTheme.is_phone():
		return
	if leg == "cold" or leg == "rotate":
		await _store_pins()
	await _settle()
	var w0 := Vector2i(get_window().size)
	if leg == "dump":
		_dump(_app._phone_tool_sheet, 0, 4)
	_report("boot")
	await _shot("boot")
	if leg == "cold":
		_check_landscape("cold boot")
		await _check_grip()
	elif leg == "rotate":
		_check_portrait("boot portrait")
		get_window().size = Vector2i(w0.y, w0.x)
		await _settle()
		_report("rotated")
		await _shot("rotated")
		_check_landscape("rotated to landscape")
		await _check_grip()
		get_window().size = w0
		await _settle()
		_report("back")
		await _shot("back")
		_check_portrait("rotated back to portrait")
		get_window().size = Vector2i(w0.y, w0.x)
		await _settle()
		await _shot("rotated_again")
		_check_landscape("rotated to landscape a second time")
	elif leg == "plainrot":
		get_window().size = Vector2i(w0.y, w0.x)
		await _settle()
		_report("rotated")
		await _shot("rotated")
		get_window().size = w0
		await _settle()
		await _shot("back")

# ---- checks -----------------------------------------------------------------------------------------------------------

func _near(a: Rect2, b: Rect2, tol: float = 1.5) -> bool:
	return a.position.distance_to(b.position) <= tol and a.size.distance_to(b.size) <= tol

func _filled(tag: String) -> void:
	var win := Rect2(Vector2.ZERO, Vector2(get_window().size))
	_ok(_near(_app.get_global_rect(), win), "%s: the shell is the window's rect (%s vs %s)" % [tag, str(_app.get_global_rect()), str(win)])
	_ok(_near(_app._phone_root.get_global_rect(), win), "%s: the phone root is the window's rect" % tag)
	_ok(_near(_app.viewport.get_global_rect(), win), "%s: the map host is the window's rect (no bare strip)" % tag)

## True when a visible BUTTON at least `min_h` tall is under `c`: the dock's Generate and tool buttons, i.e. a body, not an empty
## panel (a plain "any visible control" test passes on the empty scroll containers, which the first version did: mutant M1).
func _has_tall_body(c: Control, min_h: float) -> bool:
	for n in c.find_children("*", "BaseButton", true, false):
		var k := n as Control
		if k.is_visible_in_tree() and k.size.y >= min_h and k.size.x >= 40.0:
			return true
	return false

func _check_landscape(tag: String) -> void:
	_ok(_app._landscape, "%s: the shell reads landscape" % tag)
	_filled(tag)
	var win := Vector2(get_window().size)
	var rail: Rect2 = _app._phone_menu_bar.get_global_rect()
	var sheet: Rect2 = _app._phone_tool_sheet.get_global_rect()
	var bar: Rect2 = _app._phone_app_bar.get_global_rect()
	var ins: Dictionary = _app.phone_content_insets()
	_ok(absf(float(ins["left"]) - rail.end.x) <= 1.5, "%s: the map band starts where the rail ends (%.1f vs %.1f)" % [tag, float(ins["left"]), rail.end.x])
	_ok(absf(float(ins["right"]) - sheet.size.x) <= 1.5 and absf(sheet.end.x - win.x) <= 1.5, "%s: the band ends where the sheet begins, the sheet at the window's right edge (%.1f)" % [tag, sheet.end.x])
	_ok(float(ins["top"]) == 0.0, "%s: no top reserve in the band (the header is in the corner, not over the map)" % tag)
	_ok(_app._phone_tool_sheet.is_visible_in_tree() and _has_tall_body(_app._phone_tool_sheet, 40.0), "%s: the docked sheet has a visible body (not an empty panel)" % tag)
	_ok(absf(sheet.position.y - bar.end.y) <= 1.5, "%s: the sheet starts under the header (%.1f vs %.1f)" % [tag, sheet.position.y, bar.end.y])
	_ok(sheet.end.y < win.y and win.y - sheet.end.y >= 4.0, "%s: and stops above the gesture band (%.1f of %.1f)" % [tag, sheet.end.y, win.y])
	_ok(absf(bar.end.x - win.x) <= 1.5 and bar.position.y == 0.0 and bar.position.x >= win.x * 0.4, "%s: the header is in the TOP-RIGHT corner (%s)" % [tag, str(bar)])
	_ok(_app._phone_app_bar.get_theme_stylebox("panel") is StyleBoxEmpty, "%s: the header has no backdrop (an empty panel style)" % tag)
	var strip: Control = _app._phone_land_strip
	_ok(strip != null and strip.is_visible_in_tree() and strip.get_global_rect().end.y >= win.y - 1.5, "%s: the bottom-edge strips are drawn" % tag)
	if strip != null:
		var l: Rect2 = (strip.get_node("Left") as Control).get_global_rect()
		var r: Rect2 = (strip.get_node("Right") as Control).get_global_rect()
		_ok(l.size.y > 4.0 and l.end.x >= rail.end.x - 1.5 and r.size.y > 4.0 and r.position.x <= sheet.position.x + 1.5, "%s: one under the rail and one under the sheet, both with height" % tag)

func _check_portrait(tag: String) -> void:
	_ok(not _app._landscape, "%s: the shell reads portrait" % tag)
	_filled(tag)
	var win := Vector2(get_window().size)
	var bar: Rect2 = _app._phone_app_bar.get_global_rect()
	_ok(_app._phone_app_bar.get_parent() == _app._phone_chrome_col, "%s: the header is back in the chrome column" % tag)
	_ok(absf(bar.size.x - win.x) <= 1.5 and bar.position.y > 0.0, "%s: it spans the width under the status row (%s)" % [tag, str(bar)])
	_ok(_app._phone_app_bar.get_theme_stylebox("panel") is StyleBoxEmpty, "%s: still no backdrop" % tag)
	_ok(_app._phone_land_strip == null or not _app._phone_land_strip.visible, "%s: the landscape strips are hidden" % tag)
	var sheet: Rect2 = _app._phone_tool_sheet.get_global_rect()
	_ok(sheet.size.x >= win.x - 1.5 and sheet.size.y < win.y * 0.3, "%s: the sheet is the bottom strip again (%s)" % [tag, str(sheet)])
	_ok(not _app._phone_dom_host.visible or _app._phone_detent != "peek", "%s: at peek the sheet's body is hidden in portrait" % tag)

func _check_grip() -> void:
	var d = _app.phone_quick.drawer()
	_ok(d != null and d.visible, "the quick drawer is mounted (two pins)")
	if d == null:
		return
	var win := Rect2(Vector2.ZERO, Vector2(get_window().size))
	var ins: Dictionary = _app.phone_content_insets()
	var band := Rect2(float(ins["left"]), float(ins["top"]), win.size.x - float(ins["left"]) - float(ins["right"]),
		win.size.y - float(ins["top"]) - float(ins["bottom"]) - float(_app.phone_tool_pill_reserve()))
	var g: Rect2 = d.grip().get_global_rect()
	_ok(absf(g.end.x - band.end.x) <= 1.5, "the grip is on the RIGHT edge of the band, right-handed too (end %.1f, band end %.1f)" % [g.end.x, band.end.x])
	_ok(win.encloses(g), "the grip is wholly on screen")
	_ok(absf(g.get_center().y - band.get_center().y) <= 2.0, "with no cut-out it is at the middle of the band's height (%.1f vs %.1f)" % [g.get_center().y, band.get_center().y])
	## A cut-out on the right half of the screen at y 300 to 360: the grip centres on it.
	_app.set_meta("cutouts_probe", [Rect2(win.size.x - 40.0, 300.0, 40.0, 60.0)])
	d.relayout()
	await _frames(2)
	g = d.grip().get_global_rect()
	_ok(absf(g.get_center().y - 330.0) <= 1.5, "with a cut-out on the right at y 300..360 the grip centres on it (%.1f)" % g.get_center().y)
	_ok(absf(g.end.x - band.end.x) <= 1.5, "and stays on the band's right edge")
	## A cut-out on the LEFT half does not move it.
	_app.set_meta("cutouts_probe", [Rect2(0.0, 300.0, 40.0, 60.0)])
	d.relayout()
	await _frames(2)
	_ok(absf(d.grip().get_global_rect().get_center().y - band.get_center().y) <= 2.0, "a cut-out on the left half is ignored")
	_app.remove_meta("cutouts_probe")
	d.relayout()

func _report(tag: String) -> void:
	var w := Vector2(get_window().size)
	print("== ", tag, " window ", w, " landscape ", _app._landscape)
	var names := {"app": _app, "phone_root": _app._phone_root, "viewport_host": _app.viewport,
		"tool_sheet": _app._phone_tool_sheet, "app_bar": _app._phone_app_bar, "menu_bar": _app._phone_menu_bar}
	for n in names:
		var c: Control = names[n]
		print("   ", n, " ", c.get_global_rect(), " vis ", c.is_visible_in_tree())

func _dump(n: Node, depth: int, maxd: int) -> void:
	if not n is Control or depth > maxd:
		return
	var c := n as Control
	print("   ".repeat(depth + 1), c.name, " ", c.get_class(), " vis=", c.visible, " ", c.get_global_rect())
	for k in c.get_children():
		_dump(k, depth + 1, maxd)
