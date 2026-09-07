extends Node
## VERIFIER probe -- the selected Preferences chip, as DRAWN PIXELS, and its
## advance. Independent of `_chipink_probe.gd`: the selected chip is identified
## from the PIXELS (the one in its group whose fill differs from its siblings),
## never from `font_color == DccTheme.c("accent")`, which would assert the claim
## against itself.
##
##   ... _vfychip_probe.tscn -- --force-touch --vp 1080x2400     WINDOWED
##
## Canvas literals, pinned with their ENV lines and NOT read from DccTheme:
##   AND:1356  chip(on) -> col: var(--acc)   bg: var(--wash)   bord: var(--acc)
##   AND:31    dark  --acc:#e0a34a
##   AND:1469  light --acc:#a4650f
## Theme preference is restored to light at the end.

const ACC_LIGHT := "a4650f"
const ACC_DARK := "e0a34a"

var app: Node
var pm: Node
var _fail := 0

func _log(s: String) -> void:
	print("[vfychip] %s" % s)

func _chk(ok: bool, w: String) -> void:
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", w])

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _a(n: String, d: String) -> String:
	var g := OS.get_cmdline_user_args()
	var i := g.find(n)
	return String(g[i + 1]) if (i >= 0 and i + 1 < g.size()) else d

func _shot(r: Rect2i) -> Image:
	await _frames(3)
	await RenderingServer.frame_post_draw
	var t := get_viewport().get_texture()
	if t == null:
		return null
	var im := t.get_image()
	if im == null:
		return null
	var b := Rect2i(Vector2i.ZERO, im.get_size())
	var c := r.intersection(b)
	if c.size.x < 8 or c.size.y < 8:
		return null
	var out := im.get_region(c)
	out.convert(Image.FORMAT_RGBA8)
	return out

## Modal colour of a horizontal band across the vertical middle, inset well
## clear of the rounded corners AND of the 1 px accent border -- reading the
## border instead of the glyph is the trap this inset guards.
func _fill(im: Image) -> Color:
	var counts := {}
	var best := Color(0, 0, 0, 0)
	var bn := 0
	var h := im.get_height()
	var w := im.get_width()
	for y in range(maxi(0, h / 2 - 2), mini(h, h / 2 + 3)):
		for x in range(8, maxi(9, w - 8)):
			var p := im.get_pixel(x, y)
			var k := p.to_html(false)
			var n: int = int(counts.get(k, 0)) + 1
			counts[k] = n
			if n > bn:
				bn = n
				best = p
	return best

## The glyph colour: the pixel inside the middle half whose luminance is
## furthest from the fill. Inset 8 px so the border is never a candidate.
func _ink(im: Image, fill: Color) -> Color:
	var best := fill
	var bd := -1.0
	var h := im.get_height()
	var w := im.get_width()
	for y in range(maxi(0, h / 4), mini(h, h - h / 4)):
		for x in range(8, maxi(9, w - 8)):
			var p := im.get_pixel(x, y)
			var d: float = absf(p.get_luminance() - fill.get_luminance())
			if d > bd:
				bd = d
				best = p
	return best

## **Canvas space is not framebuffer space.** The shell content-scales its root
## on a handset, so `get_global_rect()` is in ~412 dp units while
## `get_texture()` is 1080 physical px. `get_screen_transform()` is the
## engine own answer, so no arithmetic here can drift from what it drew.
func _rect_of(c: Control) -> Rect2i:
	var r: Rect2 = c.get_viewport().get_screen_transform() * c.get_global_rect()
	return Rect2i(Vector2i(r.position.round()), Vector2i(r.size.round()))

func _chips_under(n: Node, out: Array) -> Array:
	if n is Button and (n as Control).is_visible_in_tree():
		var b := n as Button
		if b.size.y > 0.0 and b.size.x > 0.0:
			out.append(b)
	for ch in n.get_children():
		_chips_under(ch, out)
	return out

func _force(dark: bool) -> void:
	if DccTheme.is_dark() == dark:
		return
	var was := DccTheme.is_dark()
	DccTheme.apply_theme(dark)
	app.rebuild_theme(was)

func _pass(dark: bool) -> void:
	_force(dark)
	await _frames(10)
	var key := "dark" if dark else "light"
	var want := ACC_DARK if dark else ACC_LIGHT
	pm.open()
	pm._push_screen("prefs")
	await _frames(14)
	var groups := {}
	for b in _chips_under(pm, []):
		var p := (b as Node).get_parent()
		if not groups.has(p):
			groups[p] = []
		groups[p].append(b)
	var tested := 0
	var matched := 0
	var first_sel: Button = null
	var first_fill := Color(0, 0, 0, 0)
	for p in groups:
		var g: Array = groups[p]
		if g.size() < 2:
			continue
		var fills := []
		var imgs := []
		var ok_grp := true
		for b in g:
			var im: Image = await _shot(_rect_of(b as Control))
			if im == null:
				ok_grp = false
				break
			imgs.append(im)
			fills.append(_fill(im))
		if not ok_grp:
			continue
		var tally := {}
		for f in fills:
			var k: String = (f as Color).to_html(false)
			tally[k] = int(tally.get(k, 0)) + 1
		var odd := -1
		for i in fills.size():
			if int(tally[(fills[i] as Color).to_html(false)]) == 1:
				odd = i if odd < 0 else -2
		if odd < 0:
			continue
		tested += 1
		var ink := _ink(imgs[odd], fills[odd])
		var hex := ink.to_html(false)
		if hex == want:
			matched += 1
		else:
			_log("  [%s] MISMATCH group of %d: selected=%s fill=#%s ink=#%s want #%s"
				% [key, g.size(), (g[odd] as Button).text,
					(fills[odd] as Color).to_html(false), hex, want])
		if first_sel == null:
			first_sel = g[odd]
			first_fill = fills[odd]
			_log("  [%s] sample group of %d: selected=%s fill=#%s ink=#%s (canvas --acc #%s)"
				% [key, g.size(), (g[odd] as Button).text,
					(fills[odd] as Color).to_html(false), hex, want])
	_chk(tested >= 3, "[%s] at least three chip groups were readable from pixels (%d)" % [key, tested])
	_chk(tested > 0 and matched == tested,
		"[%s] every pixel-identified selected chip INK is the canvas --acc #%s (%d of %d)"
			% [key, want, matched, tested])

	if first_sel != null:
		first_sel.add_theme_color_override("font_color", Color(1, 0, 0, 1))
		await _frames(8)
		var im2: Image = await _shot(_rect_of(first_sel))
		var got := _ink(im2, first_fill).to_html(false) if im2 != null else "none"
		_chk(got == "ff0000", "[%s] positive control: forcing the ink red MEASURES red (got #%s)" % [key, got])
		first_sel.remove_theme_color_override("font_color")
		await _frames(6)

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		_log("ABORT headless -- get_image() is vacuous here. exit 2")
		get_tree().quit(2)
		return
	var p: PackedStringArray = _a("--vp", "1080x2400").split("x")
	DisplayServer.window_set_size(Vector2i(int(p[0]), int(p[1])))
	await _frames(6)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	## The phone entry screen is a SEPARATE full-screen Control, and hiding the
	## desktop dialog does not touch it. Without this the prefs chips exist,
	## report real rects, and are painted over by the world gallery -- which is
	## what the first cut of this probe measured, and why its positive control
	## failed: it was reading terrain.
	if app.get("phone_project_picker") != null:
		app.phone_project_picker.hide()
	await _frames(4)
	if not DccTheme.is_phone():
		_log("ABORT not phone -- pass --force-touch. exit 2")
		get_tree().quit(2)
		return
	pm = app.get("_phone_menu")
	if pm == null:
		_log("ABORT no phone menu. exit 2")
		get_tree().quit(2)
		return
	var fbs := get_viewport().get_texture().get_size()
	_log("phone=true scale=%.4f fb=%s window=%s vp_visible=%s root_csf=%.4f screen_xf=%s started_dark=%s"
		% [app.phone_scale(), fbs, DisplayServer.window_get_size(),
			get_viewport().get_visible_rect().size,
			get_tree().root.content_scale_factor,
			get_viewport().get_screen_transform(), DccTheme.is_dark()])

	pm.open()
	pm._push_screen("prefs")
	await _frames(12)
	## DIAGNOSTIC: where does a chip actually live, and whose framebuffer is it in?
	var all := _chips_under(pm, [])
	_log("  chips found under pm: %d" % all.size())
	for i in mini(4, all.size()):
		var b := all[i] as Button
		var anc: Node = b
		var win := "<none>"
		while anc != null:
			if anc is Window:
				win = "%s(%s)" % [anc.name, anc.get_class()]
				break
			anc = anc.get_parent()
		_log("    chip[%d] %-18s rect=%s  vp_same_as_probe=%s  window=%s vis_in_tree=%s"
			% [i, b.text, b.get_global_rect(), b.get_viewport() == get_viewport(), win,
				b.is_visible_in_tree()])
	var med := DccTheme.mono(0, true)
	var reg := DccTheme.mono(0, false)
	var fs := 0
	var moved := 0
	var n := 0
	for b in _chips_under(pm, []):
		var btn := b as Button
		if btn.text == "":
			continue
		fs = btn.get_theme_font_size("font_size")
		var aw: float = med.get_string_size(btn.text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x
		var cw: float = reg.get_string_size(btn.text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x
		n += 1
		if absf(aw - cw) > 0.01:
			moved += 1
	_log("  advance: %d labels at font_size %d, Medium vs Regular -- %d differ" % [n, fs, moved])
	_chk(n >= 20, "enough chip labels were measured (%d)" % n)
	_chk(moved == 0, "Plex Mono Medium advance equals Regular on every chip label (%d moved)" % moved)

	await _pass(false)
	await _pass(true)
	_force(false)
	await _frames(6)
	_log("restored: dark=%s (preference must end light)" % DccTheme.is_dark())
	_log("RESULT vfychip fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
