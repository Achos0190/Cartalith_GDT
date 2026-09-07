extends Node
## The numeric field ON GLASS, in the two places the owner named: the New World
## dialog (Seed, Width) and the Journey planner's party form (Group size,
## Hours/day, Trade cargo, Supplies).
##
##   Godot_v4.7.1-stable_win64_console.exe --rendering-driver vulkan \
##     --resolution 1600x950 _numglass_probe.tscn -- --shot <dir>
##
## `--shot <dir>` is optional and writes one PNG per surface; without it the
## probe only measures. **Not `--headless`.** It reads the framebuffer back, so
## what it claims about a pixel is a pixel -- which is exactly what `e830112`'s
## stylebox-only proof could not do.
##
## ### What it asserts, and why it is a relationship rather than a column
##
## `e830112` gave `DccWidgets.number()`'s field `HORIZONTAL_ALIGNMENT_RIGHT`,
## borrowed from `ENV:351` -- a **52 px fixed-width readout span**. The field
## is `SIZE_EXPAND_FILL` and measures 388 px in the New World dialog, so the
## borrowed declaration pushed a 36 px number 343 px from its label into an
## otherwise empty chip. Its sibling on the row above, a `choice()` dropdown,
## is the same `DccTheme.field_box()` chip at the same `chip_pad_x` and puts
## its value at the LEFT.
##
## So the assertion is: **a number starts where the dropdown beside it
## starts.** That survives any width change -- including the ones this lane and
## the other three are making to `dcc_widgets.gd` this batch -- where an
## absolute pixel column would not.

var app: Node
var _img: Image
var _shot_dir := ""
var fails := 0

func _ok(cond: bool, what: String) -> void:
	if cond: print("  PASS  ", what)
	else:
		fails += 1
		print("  FAIL  ", what)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _hex(c: Color) -> String:
	return "#%02x%02x%02x" % [int(round(c.r * 255)), int(round(c.g * 255)),
		int(round(c.b * 255))]

func _l(c: Color) -> float:
	return c.r * 0.2126 + c.g * 0.7152 + c.b * 0.0722

func _cr(a: Color, b: Color) -> float:
	return (maxf(_l(a), _l(b)) + 0.05) / (minf(_l(a), _l(b)) + 0.05)

func _modal(r: Rect2i) -> Color:
	var counts := {}
	for y in range(r.position.y, r.position.y + r.size.y):
		for x in range(r.position.x, r.position.x + r.size.x):
			if x < 0 or y < 0 or x >= _img.get_width() or y >= _img.get_height():
				continue
			var k := _hex(_img.get_pixel(x, y))
			counts[k] = int(counts.get(k, 0)) + 1
	var top := "#000000"
	var topn := -1
	for k in counts:
		if int(counts[k]) > topn:
			topn = int(counts[k]); top = String(k)
	return Color(top)

## Ink bbox strictly inside `r`, ignoring the rounded corners: inset by
## `inset` on every side and count anything far enough from `field`.
func _ink(r: Rect2i, field: Color, inset: int) -> Rect2i:
	var x0 := 99999; var y0 := 99999; var x1 := -1; var y1 := -1
	for y in range(r.position.y + inset, r.position.y + r.size.y - inset):
		for x in range(r.position.x + inset, r.position.x + r.size.x - inset):
			if x < 0 or y < 0 or x >= _img.get_width() or y >= _img.get_height():
				continue
			var p := _img.get_pixel(x, y)
			if absf(p.r - field.r) + absf(p.g - field.g) + absf(p.b - field.b) < 0.06:
				continue
			x0 = mini(x0, x); y0 = mini(y0, y); x1 = maxi(x1, x); y1 = maxi(y1, y)
	if x1 < 0:
		return Rect2i(0, 0, 0, 0)
	return Rect2i(x0, y0, x1 - x0 + 1, y1 - y0 + 1)

var _origin := Vector2.ZERO

func _rect_of(ctl: Control) -> Rect2i:
	var g := ctl.get_global_rect()
	return Rect2i(Vector2i(g.position + _origin), Vector2i(g.size))

## **A control can be `is_visible_in_tree()` and still not be on the screen.**
## The New World dialog's `Factions` field sits below its `ScrollContainer`'s
## fold: the node is visible, its `get_global_rect()` is a perfectly ordinary
## rect, and the pixels at that rect belong to whatever is drawn *outside* the
## dialog. Measured before this guard existed, it reported a `#fbfaf7` ground
## and ink 104 px in -- a reading of the app behind it. Any ancestor that
## clips is a fold, so every one of them has to contain the rect.
func _on_screen(ctl: Control) -> bool:
	var r := ctl.get_global_rect()
	var n: Node = ctl.get_parent()
	while n != null and n is Control:
		var a := n as Control
		if a.clip_contents:
			var ar := a.get_global_rect()
			if not (ar.position.x <= r.position.x + 1 and ar.position.y <= r.position.y + 1
					and ar.end.x >= r.end.x - 1 and ar.end.y >= r.end.y - 1):
				return false
		n = n.get_parent()
	return true

## `{left, right, w, h, ground, ink}` for one chip, or an empty Dictionary if
## the control is not laid out. `left`/`right` are the ink's inset from the
## chip's own edges, so nothing here is an absolute screen column.
func _chip(ctl: Control) -> Dictionary:
	if not _on_screen(ctl):
		return {}
	var r := _rect_of(ctl)
	if r.size.x <= 8 or r.size.y <= 8:
		return {}
	var ground := _modal(Rect2i(r.position + Vector2i(3, 3), r.size - Vector2i(6, 6)))
	var ink := _ink(r, ground, 3)
	if ink.size.x <= 0:
		return {}
	return {
		"left": ink.position.x - r.position.x,
		"right": (r.position.x + r.size.x) - (ink.position.x + ink.size.x),
		"w": r.size.x, "h": r.size.y, "ground": ground, "ink": ink,
	}

func _label_of(ctl: Control) -> String:
	var row := ctl.get_parent()
	if row != null and row.get_child_count() > 0 and row.get_child(0) is Label:
		return (row.get_child(0) as Label).text
	return "?"

func _walk(n: Node, want_spin: bool, out: Array) -> Array:
	for c in n.get_children(true):
		if (want_spin and c is SpinBox) or (not want_spin and c is OptionButton):
			if (c as Control).is_visible_in_tree():
				out.append(c)
		if c is Node:
			_walk(c, want_spin, out)
	return out

func _shot(name: String, vp: Viewport) -> void:
	if _shot_dir == "":
		return
	var im := vp.get_texture().get_image()
	if im != null:
		im.save_png(_shot_dir.path_join(name + ".png"))
		print("  [shot] ", _shot_dir.path_join(name + ".png"))

## Measure one surface: every visible `number()` field against every visible
## `choice()` dropdown in the same container.
func _surface(tag: String, root: Node, vp: Viewport) -> void:
	print("\n[", tag, "]")
	await RenderingServer.frame_post_draw
	_img = vp.get_texture().get_image()
	if _img == null:
		print("  [FATAL] no image -- run WITHOUT --headless"); fails += 1; return
	var spins := _walk(root, true, [])
	var drops := _walk(root, false, [])
	## The reference inset: what a dropdown's value does on this same chip.
	var drop_left := -1
	for d in drops:
		var m := _chip(d)
		if m.is_empty():
			continue
		drop_left = int(m["left"])
		print("  reference chip [", (d as OptionButton).text, "]  ink starts ",
			drop_left, " px in, chip is ", m["w"], "x", m["h"])
		break
	if drop_left < 0:
		## No dropdown on screen here. Fall back to the canvas literal the
		## chip's padding comes from -- `ENV:522` is
		## `min-height:var(--ctl);border-radius:8px;background:var(--ins);
		## padding:3px 9px` -- rather than to `role_px("chip_pad_x")`, which
		## is the same number the shipping code reads and would assert the
		## constant against itself.
		drop_left = 9
		print("  no dropdown on screen; reference is the canvas literal",
			" padding-left:9px (ENV:522)")
	var seen := 0
	for s in spins:
		var m := _chip(s)
		if m.is_empty():
			continue
		seen += 1
		var name := _label_of(s)
		print("  [", name, "]  chip ", m["w"], "x", m["h"], "  ground ",
			_hex(m["ground"]), "  ink starts ", m["left"],
			" px in, ends ", m["right"], " px from the right")
		_ok(absf(float(m["left"]) - drop_left) <= 3.0,
			"%s: the number starts where the dropdown's value starts (%d vs %d)"
				% [name, int(m["left"]), drop_left])
	_ok(seen > 0, "%s: at least one numeric field was laid out and measured" % tag)

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	for i in args.size():
		if args[i] == "--shot" and i + 1 < args.size():
			_shot_dir = args[i + 1]
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.8).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	## Force the palette this machine boots, through the shell's own path.
	if DccTheme.is_dark():
		app.toggle_theme()
		await _frames(4)
	print("[BOOT] palette dark=", DccTheme.is_dark(), "  touch=", DccTheme.is_touch())

	## ---- Surface 1: the New World dialog -------------------------------
	var dlg = app.new_world_dialog
	dlg.popup_centered()
	await _frames(14)
	var embedded: bool = dlg.is_embedded()
	var dvp: Viewport = get_tree().root if embedded else (dlg as Viewport)
	_origin = Vector2(dlg.position) if embedded else Vector2.ZERO
	await _surface("New World dialog", dlg, dvp)
	_shot("new_world", dvp)
	dlg.hide()
	await _frames(4)

	## ---- Surface 2: the Journey planner's party form -------------------
	_origin = Vector2.ZERO
	app.open_journey_planner()
	await _frames(20)
	## The planner builds INTO the left dock, not into its own subtree, so the
	## walk starts at the shell.
	await _surface("Journey planner", app, get_tree().root)
	_shot("planner", get_tree().root)

	print("\n[RESULT] fails=", fails)
	get_tree().quit(1 if fails > 0 else 0)
