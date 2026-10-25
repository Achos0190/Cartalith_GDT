extends Node
## Alignment probe: MEASURES, in the real shell, that every icon, symbol, highlight and header strip
## sits where it should -- the 2026-10-26 owner findings (palette icons left of centre inside their
## highlight, quick-bar chips, and the left and right dock header rules at different heights).
##
##   desktop: godot --path . --resolution 1600x1000 _alignment_probe.tscn -- --shots <dir> --tag desktop
##   tablet:  godot --path . --resolution 1280x800  _alignment_probe.tscn -- --force-touch --shots <dir> --tag tablet
##   phone:   godot --path . --resolution 393x852   _alignment_probe.tscn -- --force-touch --shots <dir> --tag phone
##   self-test (each must go RED): append  --mutate icon|highlight|header|chip|phone
##
## **Windowed, never `--headless`** (`ImageTexture.update()` and the framebuffer are no-ops there, so
## every pixel comparison would pass vacuously -- `MISTAKES.md` "Run a pixel probe"). **Run it from a
## scratch copy with its own `config/name` and a wiped `app_userdata/<name>`**: section 1 stores 24
## quick-bar pins to show the `⋯` chip, and section 0 fails if the pin store was not empty at launch.
## Arguments read: `--force-touch`, `--shots <dir>`, `--tag <name>`, `--mutate <kind>`; anything else
## aborts with exit 2.
##
## ## How a rect is measured (and why it is not read off a node)
##
## A control's own rect says where its CELL is, not where Godot DREW the icon inside it -- which is
## the whole defect (a `Button` whose `icon_alignment` is LEFT reports a perfectly centred cell and
## draws its icon at the left). So the ink is measured from the framebuffer by DIFFERENCE, which
## needs no knowledge of the palette (this machine boots light; a threshold on a colour is
## palette-bound, `MISTAKES.md`):
##   * ICON BOX: the button's icon is swapped for a transparent texture of the same size, the frame
##     is captured, then for an opaque white one, and the pixels that differ are the icon's BOX
##     exactly (not the glyph's ink, which is asymmetric by design: an arrow, a ruler).
##   * TEXT / SYMBOL INK: the button's font colours are made transparent, captured, restored,
##     captured; the differing pixels are the glyph's ink.
##   * HIGHLIGHT: the same cell captured unpressed and pressed; the differing pixels are the
##     highlight (plus the recoloured icon, which lies inside it).
##   * BORDER: the chip's outline captured with its styleboxes emptied and as drawn.
## Every measurement carries its own positive control: an empty difference is a FAILURE ("nothing
## drawn"), never a pass.
##
## ## What it asserts (each tolerance is a literal here, not a constant read back from the shell)
##   1. PALETTE CELLS (desktop and tablet): per cell, the icon box centre is within 1 px of the cell
##      centre on both axes; the pressed highlight lies fully inside the cell (inset >= 0 on all four
##      sides, <= 1 px short of it) and the icon is evenly spaced inside it (left and right gaps, top
##      and bottom gaps, differ by <= 1 px).
##   2. QUICK-BAR CHIPS (`＋`, `⋯`, pins): the glyph ink centre of `＋` and `⋯` is within 1 px of the chip
##      centre on both axes; a labelled pin's ink centre is within 1 px horizontally; each chip's
##      border is fully drawn (not clipped by `QuickBar.clip_contents`), inside its parent's rect.
##   3. ROW CENTRE: every palette cell and chip is centred on the palette bar's own centre line, within
##      1 px, and all of them share one centre line.
##   4. HEADER STRIPS (desktop and tablet): the divider under the left dock's header and the one under
##      the right dock's header are at the SAME y (<= 0.5 px), the two header strips are the same
##      height, and the title and the collapse button are each centred on their strip within 1 px.
##   5. PHONE (no palette bar, no quick bar -- CQ-6 is not built): the app bar's buttons (child icon
##      or glyph centred in its cell), the bottom bar's cells (pill and caption centred on the cell),
##      the sheet header (title and close button on one centre line, close button centred on its
##      cell), the dock's tool buttons (icon box centred horizontally, highlight inside the cell),
##      and the chips in the open sheet.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument or watchdog.
## A `MEASURE ...` line per control is printed for the evidence tables.

## Centre-offset tolerance in px: the owner's "<= 1 px".
const TOL := 1.0
## Header rule equality tolerance in px: two rules that are "at the same height" are the same pixel row.
const RULE_TOL := 0.5
## Per-channel difference that counts as a changed pixel (of 1.0). 0.006 is about 1.5/255: below the
## 8-bit step of anything the shell draws, above the zero a re-rendered identical pixel gives.
const DIFF := 0.006

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _tag := "run"
var _mutate := ""
var _app: Node
var _qb: DccQuickBar


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _finish() -> void:
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


## One frame, drawn, read back. The frame is the whole viewport.
func _grab() -> Image:
	await get_tree().process_frame
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()


func _shot(name: String) -> void:
	if _shots == "":
		return
	var img := await _grab()
	var path := _shots.path_join("al_%s_%s.png" % [_tag, name])
	img.save_png(path)
	print("SHOT %s" % path)


func _crop_shot(name: String, r: Rect2, scale: int = 4) -> void:
	if _shots == "":
		return
	var img := await _grab()
	var rr := Rect2i(r.grow(6)).intersection(Rect2i(Vector2i.ZERO, img.get_size()))
	var c := img.get_region(rr)
	c.resize(c.get_width() * scale, c.get_height() * scale, Image.INTERPOLATE_NEAREST)
	var path := _shots.path_join("al_%s_%s.png" % [_tag, name])
	c.save_png(path)
	print("SHOT %s" % path)


## The bounding box of the pixels that differ between `a` and `b` inside `r` (viewport px), or an
## empty Rect2 when none do.
func _diff_bbox(a: Image, b: Image, r: Rect2) -> Rect2:
	var ri := Rect2i(r).intersection(Rect2i(Vector2i.ZERO, a.get_size()))
	var x0 := 1 << 30
	var y0 := 1 << 30
	var x1 := -1
	var y1 := -1
	for y in range(ri.position.y, ri.end.y):
		for x in range(ri.position.x, ri.end.x):
			var p := a.get_pixel(x, y)
			var q := b.get_pixel(x, y)
			if absf(p.r - q.r) > DIFF or absf(p.g - q.g) > DIFF or absf(p.b - q.b) > DIFF:
				x0 = mini(x0, x)
				y0 = mini(y0, y)
				x1 = maxi(x1, x)
				y1 = maxi(y1, y)
	if x1 < 0:
		return Rect2()
	return Rect2(x0, y0, x1 - x0 + 1, y1 - y0 + 1)


func _centre(r: Rect2) -> Vector2:
	return r.position + r.size * 0.5


## Insets of `inner` inside `outer`: left, top, right, bottom (negative = pokes out).
func _insets(inner: Rect2, outer: Rect2) -> Array:
	return [inner.position.x - outer.position.x, inner.position.y - outer.position.y,
		outer.end.x - inner.end.x, outer.end.y - inner.end.y]


func _fmt(r: Rect2) -> String:
	return "(%d,%d %dx%d)" % [r.position.x, r.position.y, r.size.x, r.size.y]


# -- ink measurement ------------------------------------------------------------------------

const _FONT_COLOURS := ["font_color", "font_hover_color", "font_pressed_color", "font_disabled_color",
		"font_focus_color", "font_hover_pressed_color"]


## The solid white and fully transparent textures of `size`, for the icon swap.
func _tex(size: Vector2i, solid: bool) -> ImageTexture:
	var img := Image.create(maxi(1, size.x), maxi(1, size.y), false, Image.FORMAT_RGBA8)
	img.fill(Color(1, 1, 1, 1.0 if solid else 0.0))
	return ImageTexture.create_from_image(img)


## The ICON BOX of each button in `buttons` in global viewport px (an empty Rect2 where nothing
## differed). Layout is not disturbed: both textures have the icon's own size.
func _icon_boxes(buttons: Array) -> Array:
	var saved: Array = []
	for b in buttons:
		saved.append((b as Button).icon)
	for b in buttons:
		var ic: Texture2D = (b as Button).icon
		(b as Button).icon = _tex(Vector2i(ic.get_size()) if ic != null else Vector2i(1, 1), false)
	var a := await _grab()
	for b in buttons:
		var ic: Texture2D = saved[buttons.find(b)]
		(b as Button).icon = _tex(Vector2i(ic.get_size()) if ic != null else Vector2i(1, 1), true)
	var s := await _grab()
	var out: Array = []
	for b in buttons:
		out.append(_diff_bbox(a, s, (b as Button).get_global_rect().grow(2)))
	for b in buttons:
		(b as Button).icon = saved[buttons.find(b)]
	return out


## The glyph INK of each button's text in global viewport px.
func _text_inks(buttons: Array) -> Array:
	var kept: Array = []
	for b in buttons:
		var d := {}
		for n in _FONT_COLOURS:
			var has: bool = (b as Button).has_theme_color_override(n)
			d[n] = [has, (b as Button).get_theme_color(n) if has else Color()]
			(b as Button).add_theme_color_override(n, Color(0, 0, 0, 0))
		kept.append(d)
	var a := await _grab()
	for i in buttons.size():
		for n in _FONT_COLOURS:
			var rec: Array = kept[i][n]
			if rec[0]:
				(buttons[i] as Button).add_theme_color_override(n, rec[1])
			else:
				(buttons[i] as Button).remove_theme_color_override(n)
	var s := await _grab()
	var out: Array = []
	for b in buttons:
		out.append(_diff_bbox(a, s, (b as Button).get_global_rect().grow(2)))
	return out


## The border/fill ink of each button (its normal stylebox as drawn vs emptied), global viewport px.
func _box_inks(buttons: Array) -> Array:
	var kept: Array = []
	for b in buttons:
		var d := {}
		for n in ["normal", "disabled", "hover", "pressed"]:
			d[n] = (b as Button).get_theme_stylebox(n)
			# Transparent, NOT empty: an emptied box has zero content margins, which would re-lay
			# the chip out and make the difference measure the layout change, not the border.
			var ghost: StyleBox = (d[n] as StyleBox).duplicate()
			if ghost is StyleBoxFlat:
				(ghost as StyleBoxFlat).bg_color.a = 0.0
				(ghost as StyleBoxFlat).border_color.a = 0.0
				(ghost as StyleBoxFlat).shadow_color.a = 0.0
			(b as Button).add_theme_stylebox_override(n, ghost)
		kept.append(d)
	var a := await _grab()
	for i in buttons.size():
		for n in ["normal", "disabled", "hover", "pressed"]:
			(buttons[i] as Button).add_theme_stylebox_override(n, kept[i][n])
	var s := await _grab()
	var out: Array = []
	for b in buttons:
		out.append(_diff_bbox(a, s, (b as Button).get_global_rect().grow(2)))
	return out


## The pressed HIGHLIGHT of one toggle button: pixels that change when it is pressed, inside its
## cell grown by 8 px so a highlight that overflows the cell is seen, not clipped by the region.
func _highlight(b: Button) -> Rect2:
	var was := b.button_pressed
	b.set_pressed_no_signal(false)
	var a := await _grab()
	b.set_pressed_no_signal(true)
	var s := await _grab()
	b.set_pressed_no_signal(was)
	return _diff_bbox(a, s, b.get_global_rect().grow(8))


# -- checks ---------------------------------------------------------------------------------

func _check_centre(label: String, inner: Rect2, cell: Rect2, axes: String = "xy") -> void:
	if inner.size == Vector2.ZERO:
		_ok(false, "%s: nothing was drawn to measure (positive control)" % label)
		return
	var d := _centre(inner) - _centre(cell)
	print("MEASURE %s | %s | cell %s | ink %s | centre offset (%.1f, %.1f)" % [_tag, label, _fmt(cell), _fmt(inner), d.x, d.y])
	if axes.contains("x"):
		_ok(absf(d.x) <= TOL, "%s: horizontal centre offset %.1f px (<= %.0f)" % [label, d.x, TOL])
	if axes.contains("y"):
		_ok(absf(d.y) <= TOL, "%s: vertical centre offset %.1f px (<= %.0f)" % [label, d.y, TOL])


## The highlight must lie inside the cell, fill it to within 1 px, and sit evenly around the icon.
func _check_highlight(label: String, hl: Rect2, cell: Rect2, icon: Rect2) -> void:
	if hl.size == Vector2.ZERO:
		_ok(false, "%s: pressing it changed no pixel -- nothing to measure (positive control)" % label)
		return
	var ins := _insets(hl, cell)
	print("MEASURE %s | %s highlight | cell %s | highlight %s | inset l,t,r,b = %d,%d,%d,%d" % [_tag, label, _fmt(cell), _fmt(hl), ins[0], ins[1], ins[2], ins[3]])
	_ok(ins[0] >= 0 and ins[1] >= 0 and ins[2] >= 0 and ins[3] >= 0,
		"%s: highlight fully inside its cell (insets %d,%d,%d,%d >= 0)" % [label, ins[0], ins[1], ins[2], ins[3]])
	_ok(ins[0] <= 1 and ins[1] <= 1 and ins[2] <= 1 and ins[3] <= 1,
		"%s: highlight fills its cell to within 1 px (insets %d,%d,%d,%d)" % [label, ins[0], ins[1], ins[2], ins[3]])
	if icon.size != Vector2.ZERO:
		var gi := _insets(icon, hl)
		print("MEASURE %s | %s icon inside highlight | gaps l,t,r,b = %d,%d,%d,%d" % [_tag, label, gi[0], gi[1], gi[2], gi[3]])
		_ok(absi(gi[0] - gi[2]) <= 1 and absi(gi[1] - gi[3]) <= 1,
			"%s: icon evenly spaced in its highlight (left/right %d/%d, top/bottom %d/%d)" % [label, gi[0], gi[2], gi[1], gi[3]])


func _name_of(b: Control) -> String:
	if b is Button:
		var t := (b as Button).text.strip_edges()
		if t != "":
			return t
		var tid := String(b.get_meta(DccWidgets.TOOL_ID_META, ""))
		if tid != "":
			return tid
		return (b as Button).tooltip_text.get_slice("\n", 0).substr(0, 18)
	return String(b.name)


# -- main -----------------------------------------------------------------------------------

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
		elif a == "--tag" and i + 1 < args.size():
			i += 1
			_tag = args[i]
		elif a == "--mutate" and i + 1 < args.size():
			i += 1
			_mutate = args[i]
		else:
			print("ABORT: unrecognised argument %s -- this probe reads --force-touch, --shots <dir>, --tag <name>, --mutate <kind>" % a)
			get_tree().quit(2)
			return
		i += 1
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	_ok(DccSettings.quick_bar_value("bar") == [] and not DccSettings.has_quick_bar_value("bar"), "the pin store is empty at launch (the user dir was wiped)")
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	## The phone boots into its full-screen entry picker, which covers everything under test.
	if _app.phone_project_picker != null and _app.phone_project_picker.visible:
		_app.phone_project_picker.hide()
	await _settle(8)
	_mode = "phone" if _app.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	var vp := get_viewport().get_visible_rect().size
	print("== mode %s, viewport %s ==" % [_mode, vp])
	var probe_img := await _grab()
	_ok(Vector2(probe_img.get_size()) == vp, "the capture is 1:1 with the viewport (%s vs %s)" % [probe_img.get_size(), vp])
	# Park the real cursor on the status bar, away from every control measured, so no hover state
	# can differ between the two frames of a difference.
	var mm := InputEventMouseMotion.new()
	mm.position = Vector2(vp.x * 0.5, vp.y - 3)
	mm.global_position = mm.position
	Input.parse_input_event(mm)
	await _settle(3)
	if _mode == "phone":
		await _phone()
	else:
		await _desktop_tablet()
	_finish()


# -- desktop and tablet ----------------------------------------------------------------------

func _desktop_tablet() -> void:
	_qb = _app._quick_bar
	var bar: Control = _app.tool_palette_bar
	_ok(bar != null and bar.is_visible_in_tree(), "the tool palette bar exists and is visible")
	var strip: Control = _app._tool_strip_row
	var cells: Array = []
	for b in strip.find_children("*", "Button", true, false):
		cells.append(b)
	print("== 1. palette cells (%d) ==" % cells.size())
	_ok(cells.size() >= 4, "at least the four global cells are present (%d)" % cells.size())
	if _mutate == "icon" and cells.size() > 0:
		# Self-test: shift the first cell's icon by 3 px by widening its content margin on the left.
		for st in ["normal", "hover", "pressed", "disabled"]:
			var sb: StyleBox = (cells[0] as Button).get_theme_stylebox(st).duplicate()
			sb.content_margin_left = 6
			(cells[0] as Button).add_theme_stylebox_override(st, sb)
		await _settle(2)
	if _mutate == "highlight":
		# Self-test: widen the pressed highlight of the lit cell by 4 px on the right.
		for b in cells:
			if (b as Button).toggle_mode:
				var sb2: StyleBox = (b as Button).get_theme_stylebox("pressed").duplicate()
				sb2.expand_margin_right = 4
				(b as Button).add_theme_stylebox_override("pressed", sb2)
		await _settle(2)
	var boxes := await _icon_boxes(cells)
	var cell_y: Array = []
	for k in cells.size():
		var b: Button = cells[k]
		var nm := "palette cell %d '%s'" % [k, _name_of(b)]
		_check_centre(nm + " icon", boxes[k], b.get_global_rect())
		cell_y.append(b.get_global_rect().get_center().y)
		if b.toggle_mode:
			var hl := await _highlight(b)
			_check_highlight(nm, hl, b.get_global_rect(), boxes[k])
	await _crop_shot("palette_cells", bar.get_global_rect(), 3)

	print("== 2. quick-bar chips ==")
	# A default list shows only the plus chip; 24 pins shows the pins and the overflow chip.
	var idx := CommandIndex.new()
	idx.build(_app, _app.bridge)
	var pins: Array = []
	for r in idx.all():
		if bool(r.get("pinnable", false)) and pins.size() < 24:
			pins.append({"k": String(r["pin_key"]), "t": String(r["title"])})
	_ok(pins.size() == 24, "24 pinnable commands found (%d)" % pins.size())
	_ok(DccSettings.set_quick_bar_value("bar", pins), "stored 24 pins")
	_qb.refresh()
	await _settle(6)
	_ok(_qb.more_chip().visible, "the overflow chip is shown with 24 pins at this width")
	var chips: Array = []
	for c in _qb.chips():
		if c.visible:
			chips.append(c)
	chips.append(_qb.more_chip())
	chips.append(_qb.plus_chip())
	if _mutate == "chip":
		var sbc: StyleBox = (_qb.plus_chip() as Button).get_theme_stylebox("normal").duplicate()
		sbc.content_margin_left = 6 + sbc.content_margin_left
		for st in ["normal", "hover", "pressed", "disabled"]:
			(_qb.plus_chip() as Button).add_theme_stylebox_override(st, sbc)
		await _settle(2)
	var inks := await _text_inks(chips)
	var bords := await _box_inks(chips)
	var bar_r := bar.get_global_rect()
	for k in chips.size():
		var c: Button = chips[k]
		var is_symbol := c == _qb.plus_chip() or c == _qb.more_chip()
		var nm2 := "chip %d '%s'" % [k, _name_of(c)]
		_check_centre(nm2 + " ink", inks[k], c.get_global_rect(), "xy" if is_symbol else "x")
		var bi := _insets(bords[k], c.get_global_rect())
		_ok(bords[k].size != Vector2.ZERO and bi[0] >= 0 and bi[1] >= 0 and bi[2] >= 0 and bi[3] >= 0 and bi[0] <= 1 and bi[1] <= 1 and bi[2] <= 1 and bi[3] <= 1,
			"%s: border fully drawn in its cell (insets %s)" % [nm2, bi])
		var pr := _insets(c.get_global_rect(), _qb.get_global_rect())
		_ok(pr[0] >= 0 and pr[1] >= 0 and pr[2] >= 0 and pr[3] >= 0, "%s: inside the QuickBar's clip rect (insets %s)" % [nm2, pr])
		cell_y.append(c.get_global_rect().get_center().y)
	await _crop_shot("quickbar_chips", _qb.get_global_rect().merge(chips[chips.size() - 1].get_global_rect()), 3)

	print("== 3. one centre line ==")
	var bar_cy := bar_r.get_center().y
	var lo := INF
	var hi := -INF
	for y in cell_y:
		lo = minf(lo, y)
		hi = maxf(hi, y)
	print("MEASURE %s | row centre lines %.1f..%.1f, bar centre %.1f (%d controls)" % [_tag, lo, hi, bar_cy, cell_y.size()])
	_ok(hi - lo <= TOL, "every cell and chip shares one centre line (spread %.1f px)" % (hi - lo))
	_ok(absf(lo - bar_cy) <= TOL and absf(hi - bar_cy) <= TOL, "that line is the palette bar's centre (bar %.1f, cells %.1f..%.1f)" % [bar_cy, lo, hi])

	# restore the store so a later probe in the same user dir starts clean
	DccSettings.set_quick_bar_value("bar", [])
	_qb.refresh()

	print("== 4. dock header strips ==")
	await _headers()
	await _shot("full")


## The strip, rule and children of one dock header, found from its title label.
func _header_parts(title: Label) -> Dictionary:
	var head: Control = title.get_parent()
	var pad: Control = head.get_parent()
	var col: Control = pad.get_parent()
	var rule: Control = col.get_child(pad.get_index() + 1)
	return {"head": head, "pad": pad, "rule": rule}


func _headers() -> void:
	if _mutate == "header":
		# Self-test: the left strip loses its fix and is 6 px taller than the right.
		var lp := _header_parts(_app.left_dock_title)
		(lp["head"] as Control).custom_minimum_size.y += 6
		await _settle(3)
	var res := {}
	for side in ["left", "right"]:
		var title: Label = _app.left_dock_title if side == "left" else _app.right_dock_title
		_ok(title != null and title.is_visible_in_tree(), "%s dock header is on screen" % side)
		if title == null or not title.is_visible_in_tree():
			return
		var p := _header_parts(title)
		var head_r: Rect2 = (p["pad"] as Control).get_global_rect()
		var rule_y: float = (p["rule"] as Control).get_global_rect().position.y
		var btn: Button = _app._collapse_buttons[side]
		var tink := await _label_ink(title)
		var bink: Rect2 = (await _text_inks([btn]))[0]
		print("MEASURE %s | %s header strip %s | rule y %.1f | title ink %s | collapse button %s" % [_tag, side, _fmt(head_r), rule_y, _fmt(tink), _fmt(btn.get_global_rect())])
		res[side] = {"rect": head_r, "rule": rule_y}
		# The strip's own content row; the vertical centre line the title and the button must share.
		var row_r: Rect2 = (p["head"] as Control).get_global_rect()
		_check_centre("%s header collapse button glyph" % side, bink, row_r, "y")
		_check_centre("%s header title" % side, tink, row_r, "y")
		_check_centre("%s collapse button cell" % side, btn.get_global_rect(), row_r, "y")
	if res.size() == 2:
		var dy: float = res["left"]["rule"] - res["right"]["rule"]
		_ok(absf(dy) <= RULE_TOL, "the left and right header dividers are at the same y (left %.1f, right %.1f, offset %.1f px)" % [res["left"]["rule"], res["right"]["rule"], dy])
		var dh: float = (res["left"]["rect"] as Rect2).size.y - (res["right"]["rect"] as Rect2).size.y
		_ok(absf(dh) <= RULE_TOL, "the two header strips are the same height (left %.1f, right %.1f)" % [(res["left"]["rect"] as Rect2).size.y, (res["right"]["rect"] as Rect2).size.y])
		var dt: float = (res["left"]["rect"] as Rect2).position.y - (res["right"]["rect"] as Rect2).position.y
		_ok(absf(dt) <= RULE_TOL, "the two header strips start at the same y (offset %.1f)" % dt)
		var lrow := _header_parts(_app.left_dock_title)
		await _crop_shot("headers_left", Rect2((lrow["pad"] as Control).get_global_rect().position, Vector2(300, 60)), 3)
		var rrow := _header_parts(_app.right_dock_title)
		await _crop_shot("headers_right", Rect2((rrow["pad"] as Control).get_global_rect().position, Vector2(300, 60)), 3)


## The ink of a Label's text (colour made transparent, then restored).
func _label_ink(l: Label) -> Rect2:
	var had := l.has_theme_color_override("font_color")
	var prev := l.get_theme_color("font_color")
	l.add_theme_color_override("font_color", Color(0, 0, 0, 0))
	var a := await _grab()
	if had:
		l.add_theme_color_override("font_color", prev)
	else:
		l.remove_theme_color_override("font_color")
	var s := await _grab()
	return _diff_bbox(a, s, l.get_global_rect().grow(2))


# -- phone -----------------------------------------------------------------------------------

func _phone() -> void:
	_ok(_app._tool_palette_row == null, "the phone has no tool palette bar (CQ-6 is not built)")
	var listed: Array = []
	print("== 5a. app bar ==")
	var bar: Control = _app._phone_app_bar
	_ok(bar != null and bar.is_visible_in_tree(), "the phone app bar is on screen")
	var abtns: Array = []
	for b in bar.find_children("*", "Button", true, false):
		if (b as Button).is_visible_in_tree():
			abtns.append(b)
	listed.append("app bar buttons (%d)" % abtns.size())
	if _mutate == "phone" and abtns.size() > 0:
		# Self-test: nudge the first app-bar button's glyph/child icon 3 px off its centre.
		var first: Button = abtns[0]
		if first.get_child_count() > 0 and first.get_child(0) is Control:
			(first.get_child(0) as Control).position.x += 3
		else:
			var sbp: StyleBox = first.get_theme_stylebox("normal").duplicate()
			sbp.content_margin_left = 6
			for st in ["normal", "hover", "pressed"]:
				first.add_theme_stylebox_override(st, sbp)
		await _settle(2)
	for b in abtns:
		await _measure_phone_button("app bar", b)
	await _crop_shot("phone_app_bar", bar.get_global_rect(), 2)

	print("== 5b. bottom bar ==")
	var nav: Control = _app._phone_menu_bar
	var nbtns: Array = []
	for b in nav.find_children("*", "Button", true, false):
		if (b as Button).is_visible_in_tree():
			nbtns.append(b)
	listed.append("bottom bar cells (%d)" % nbtns.size())
	_ok(nbtns.size() >= 4, "the bottom bar has its cells (%d)" % nbtns.size())
	for b in nbtns:
		var cell: Rect2 = (b as Button).get_global_rect()
		for ch in (b as Button).find_children("*", "Control", true, false):
			if ch is Label or ch is PanelContainer:
				if ch is PanelContainer and ch.get_child_count() == 0:
					continue
				var r: Rect2 = (ch as Control).get_global_rect()
				_check_centre("bottom bar cell '%s' %s" % [_name_of(b), ch.get_class()], r, cell, "x")
				var ins := _insets(r, cell)
				_ok(ins[0] >= 0 and ins[2] >= 0 and ins[1] >= 0 and ins[3] >= 0, "bottom bar cell '%s' %s inside its cell (insets %s)" % [_name_of(b), ch.get_class(), ins])
	await _crop_shot("phone_bottom_bar", nav.get_global_rect(), 2)

	print("== 5c. sheet header ==")
	var grab: Control = _app._phone_sheet_grab
	var title: Label = _app._phone_sheet_title
	var close: Button = null
	for b in grab.find_children("*", "Button", true, false):
		close = b
	listed.append("sheet header strip (title, subtitle, close button)")
	_ok(title != null and title.is_visible_in_tree(), "the sheet header title is on screen")
	if close != null:
		var row_r: Rect2 = close.get_parent().get_global_rect() if close.get_parent() != grab else grab.get_global_rect()
		# The title row: the close button's row (it is the tallest cell in it).
		var tink := await _label_ink(title)
		var cink: Rect2 = (await _text_inks([close]))[0]
		_check_centre("sheet header close glyph vs its button", cink, close.get_global_rect())
		var sub: Label = _app._phone_sheet_subtitle
		var tr := title.get_global_rect()
		var sr := sub.get_global_rect() if sub.is_visible_in_tree() else tr
		var text_block := tr.merge(sr)
		var d := text_block.get_center().y - close.get_global_rect().get_center().y
		print("MEASURE %s | sheet header: title+subtitle block %s, close button %s, centre offset %.1f" % [_tag, _fmt(text_block), _fmt(close.get_global_rect()), d])
		_ok(absf(d) <= TOL, "sheet header: the title block and the close button share one centre line (offset %.1f px)" % d)
		_ok(tink.size != Vector2.ZERO, "sheet header title drew ink")
		await _crop_shot("phone_sheet_header", row_r.merge(grab.get_global_rect()), 2)

	print("== 5d. chips in the open sheet ==")
	# At the peek detent the sheet's body is clipped away; raise it so its chips are really on screen.
	_app.set_phone_detent("full")
	await _settle(30)
	var chips: Array = []
	for b in _app.find_children("*", "Button", true, false):
		var bb: Button = b
		if not bb.is_visible_in_tree() or bb.text.strip_edges() == "" or bb.has_meta(DccWidgets.TOOL_GLYPH_META):
			continue
		var sb := bb.get_theme_stylebox("normal")
		if not (sb is StyleBoxFlat) or (sb as StyleBoxFlat).border_width_left < 1:
			continue
		var g2 := bb.get_global_rect()
		if g2.size.x < 8 or not Rect2(Vector2.ZERO, get_viewport().get_visible_rect().size).encloses(g2):
			continue
		if bb.alignment != HORIZONTAL_ALIGNMENT_CENTER:
			continue
		chips.append(bb)
	listed.append("outline chips in the open sheet (%d visible)" % chips.size())
	var cinks := await _text_inks(chips)
	var cb := await _box_inks(chips)
	for k in chips.size():
		var c: Button = chips[k]
		var nm3 := "phone chip '%s'" % _name_of(c)
		_check_centre(nm3 + " text", cinks[k], c.get_global_rect(), "x")
		var bi := _insets(cb[k], c.get_global_rect())
		_ok(cb[k].size != Vector2.ZERO and bi[0] >= 0 and bi[1] >= 0 and bi[2] >= 0 and bi[3] >= 0, "%s: border fully inside its cell (insets %s)" % [nm3, bi])
	print("== 5e. dock tool buttons ==")
	# The TOOLS block lives in the left dock, which the phone presents as a sheet.
	_app._set_sheet_open("left", true)
	await _settle(30)
	var tools: Array = []
	for b in _app.find_children("*", "Button", true, false):
		if (b as Button).has_meta(DccWidgets.TOOL_GLYPH_META) and (b as Button).is_visible_in_tree() and (b as Button).get_global_rect().size.x > 4:
			var g: Rect2 = (b as Button).get_global_rect()
			if Rect2(Vector2.ZERO, get_viewport().get_visible_rect().size).encloses(g):
				tools.append(b)
	listed.append("dock tool buttons (%d visible)" % tools.size())
	print("phone tool buttons visible: %d" % tools.size())
	var tboxes := await _icon_boxes(tools)
	var tinks := await _text_inks(tools)
	for k in tools.size():
		var tb: Button = tools[k]
		var nm := "phone tool '%s'" % _name_of(tb)
		_check_centre(nm + " icon", tboxes[k], tb.get_global_rect(), "x")
		# The icon is stacked ABOVE its caption (`phone_fit`), so the thing that must be centred in
		# the cell and its highlight, vertically, is the GROUP of icon box and caption ink.
		var group: Rect2 = tboxes[k].merge(tinks[k]) if tinks[k].size != Vector2.ZERO else tboxes[k]
		# Vertical offset of the group is MEASURED AND REPORTED, not asserted: it cannot be fixed without
		# growing the cell (see `_phone_fit_tool_button`), so a bar here would be a bar nobody meets.
		_check_centre(nm + " icon+caption group", group, tb.get_global_rect(), "x")
		if tb.toggle_mode:
			var hl := await _highlight(tb)
			_check_highlight(nm, hl, tb.get_global_rect(), Rect2())
	if tools.size() > 0:
		await _crop_shot("phone_tools", (tools[0] as Control).get_global_rect().merge((tools[tools.size() - 1] as Control).get_global_rect()), 2)
	_app._set_sheet_open("left", false)
	await _settle(10)

	await _shot("full")
	print("PHONE CONTROLS MEASURED: " + "; ".join(listed))


## One phone app-bar button: a child icon (rect compared with the cell) or a glyph (ink compared).
func _measure_phone_button(family: String, b: Button) -> void:
	var cell := b.get_global_rect()
	var nm := "%s '%s'" % [family, _name_of(b)]
	var child_icon: Control = null
	for ch in b.get_children():
		if ch is Control and (ch as Control).visible:
			child_icon = ch
	if b.text.strip_edges() == "" and child_icon != null:
		_check_centre(nm + " child icon box", child_icon.get_global_rect(), cell)
	elif b.text.strip_edges() != "":
		var ink: Rect2 = (await _text_inks([b]))[0]
		_check_centre(nm + " glyph", ink, cell)
	else:
		_ok(false, "%s: neither a child icon nor text to measure" % nm)
	# The press/hover washes are rounded pills drawn in the cell: they must not poke out of it.
	for st in ["hover", "pressed"]:
		var sb := b.get_theme_stylebox(st)
		var dr: Rect2 = sb.get_draw_rect(Rect2(Vector2.ZERO, cell.size))
		_ok(Rect2(Vector2.ZERO, cell.size).encloses(dr), "%s: %s wash drawn inside the cell (%s in %s)" % [nm, st, dr, cell.size])
