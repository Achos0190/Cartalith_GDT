extends Node
## Lane WashAlpha's **measurement, and then the pin on what it decided.**
##
## The row it comes from said "measure whether the difference is visible before
## proposing it", so this ran *first*, against the shell as it then stood --
## `set_segment_on()` painting `accent_wash`, alpha .09 -- and answered *is the
## step from .09 to `accent_wash_2`'s .16 visible on a lit segment* with real
## framebuffer pixels rather than a composite computed from the alphas. The
## token moved only because the answer came back visible.
##
## It now runs the same comparison from the other side: the shipped fill is
## `accent_wash_2` and the **counterfactual** rendered beside it is the old
## .09. The numbers are the same numbers; `_ok()` pins the binding that
## produced them.
##
## Method, and why each part of it is that way:
##
##   1. It boots the **real shell** (`res://shell/app.tscn`), so every chip it
##      measures sits on the ground it actually sits on in the product. A
##      strip built by the probe would have had the probe choose the ground,
##      which is the half of an alpha composite that decides the answer.
##   2. It finds lit segments by the **stylebox signature `set_segment_on(b,
##      true)` leaves** -- `StyleBoxFlat`, `border_color == c("accent")`,
##      `bg_color == c(SHIPPED)` -- not by name, node path or call site.
##   3. It reads the fill as the **modal colour of the chip's interior**, so a
##      glyph pixel cannot be mistaken for the wash. `_frames()` plus
##      `RenderingServer.frame_post_draw` before every grab.
##   4. It renders the counterfactual by overriding `normal`/`pressed`/
##      `disabled` on those same chips with the same box at `accent_wash_2`'s
##      alpha and grabbing the **same pixels** again. Same chip, same ground,
##      same density -- the alpha is the only thing that moved.
##   5. `off` is captured too, as the scale bar: the ground-to-.09 step is the
##      one this design already ships and calls legible, so .09-to-.16 is
##      reported against it rather than against an opinion.
##
## **Windowed only.** `MISTAKES.md`: a texture read under `--headless` is not
## the framebuffer. It refuses to run headless rather than printing numbers
## that mean nothing.
##
## Both palettes, because this machine boots LIGHT and a one-palette answer is
## half an answer. Density comes from the command line:
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _washstep_probe.tscn \
##       --resolution 1920x1080
##   Godot_v4.7.1-stable_win64_console.exe --path . _washstep_probe.tscn \
##       --resolution 1280x800 -- --force-touch

## The shipped token and the one it replaced. Both are named so the
## counterfactual is a token lookup and not a literal this file invented.
const SHIPPED := "accent_wash_2"
const FORMER := "accent_wash"

var app: Node
var _fail := 0

func _ok(name: String, got, want) -> void:
	var good: bool = got == want
	print("  %s %-46s got=%s want=%s"
		% ["ok  " if good else "FAIL", name, got, want])
	if not good:
		_fail += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## The chip interior, inset past the 1 px border and one more pixel of padding.
func _interior(r: Rect2i) -> Rect2i:
	return Rect2i(r.position + Vector2i(2, 2), r.size - Vector2i(4, 4))

## The modal colour of a rect -- the fill, whatever the glyphs did.
func _modal(img: Image, r: Rect2i) -> Color:
	var counts := {}
	var best := Color(0, 0, 0, 0)
	var best_n := -1
	for y in range(r.position.y, r.position.y + r.size.y):
		for x in range(r.position.x, r.position.x + r.size.x):
			if x < 0 or y < 0 or x >= img.get_width() or y >= img.get_height():
				continue
			var k := img.get_pixel(x, y).to_rgba32()
			var n: int = int(counts.get(k, 0)) + 1
			counts[k] = n
			if n > best_n:
				best_n = n
				best = img.get_pixel(x, y)
	return best

func _grab() -> Image:
	await _frames(3)
	await RenderingServer.frame_post_draw
	return get_window().get_texture().get_image()

func _is_lit_segment(b: Button) -> bool:
	var sb := b.get_theme_stylebox("normal")
	if not (sb is StyleBoxFlat):
		return false
	var f := sb as StyleBoxFlat
	return f.border_color == DccTheme.c("accent") \
		and f.bg_color == DccTheme.c(SHIPPED) \
		and f.border_width_left == 1

func _walk_buttons(n: Node, out: Array) -> void:
	if n is Button and (n as Button).is_visible_in_tree():
		out.append(n)
	for ch in n.get_children(true):
		_walk_buttons(ch, out)

func _hex(c: Color) -> String:
	return "#%02x%02x%02x" % [int(round(c.r * 255.0)), int(round(c.g * 255.0)),
		int(round(c.b * 255.0))]

func _d(a: Color, b: Color) -> String:
	return "dR=%+d dG=%+d dB=%+d  maxΔ=%d" % [
		int(round(b.r * 255.0)) - int(round(a.r * 255.0)),
		int(round(b.g * 255.0)) - int(round(a.g * 255.0)),
		int(round(b.b * 255.0)) - int(round(a.b * 255.0)),
		maxi(maxi(absi(int(round(b.r * 255.0)) - int(round(a.r * 255.0))),
			absi(int(round(b.g * 255.0)) - int(round(a.g * 255.0)))),
			absi(int(round(b.b * 255.0)) - int(round(a.b * 255.0))))]

func _measure(tag: String) -> void:
	## Every visible lit segment in the booted shell, found by signature.
	var all: Array = []
	_walk_buttons(app, all)
	var lit: Array = []
	for b in all:
		if _is_lit_segment(b):
			var r := Rect2i((b as Button).get_global_rect())
			if r.size.x >= 8 and r.size.y >= 8:
				lit.append(b)
	## An unlit sibling, for the ground the wash composites onto.
	var off: Array = []
	for b in all:
		var sb = (b as Button).get_theme_stylebox("normal")
		if sb is StyleBoxFlat and (sb as StyleBoxFlat).bg_color.a == 0.0 \
				and (sb as StyleBoxFlat).border_color == DccTheme.c("border"):
			off.append(b)

	## The shell lights few segments at rest, and one sample is not a
	## measurement. So the **unlit** segments on screen are lit through the
	## product's own call -- `DccWidgets.set_segment_on(b, true)`, the thing
	## under test -- rather than by a box this probe builds. Same widget, same
	## ground, same density; only the user's click is simulated.
	## Grabbed BEFORE the promotion below, or the "ground" the wash composites
	## onto would be sampled from a chip this probe had already washed.
	var img_pre := await _grab()
	var ground: Array = []
	for b in off:
		ground.append(_modal(img_pre, _interior(Rect2i((b as Button).get_global_rect()))))
	var promoted := 0
	for b in off:
		DccWidgets.set_segment_on(b as Button, true)
		promoted += 1
	if promoted > 0:
		await _frames(3)
		for b in off:
			if _is_lit_segment(b as Button):
				var rr := Rect2i((b as Button).get_global_rect())
				if rr.size.x >= 8 and rr.size.y >= 8 and not lit.has(b):
					lit.append(b)
	print("[%s] visible Buttons=%d  lit segments=%d (of which %d lit by "
		% [tag, all.size(), lit.size(), promoted]
		+ "set_segment_on for the measurement)")
	if lit.is_empty():
		print("[%s] NO LIT SEGMENT ON SCREEN -- no measurement made" % tag)
		_fail += 1
		return

	## -- the assertion, and the mutation test for the one line this lane
	## moved. Revert `set_segment_on()` to `accent_wash` and `_is_lit_segment()`
	## matches nothing, so the branch above fires and this probe goes red. It
	## cannot pass against either binding.
	var pin := (lit[0] as Button).get_theme_stylebox("normal") as StyleBoxFlat
	_ok("lit fill is " + SHIPPED, pin.bg_color, DccTheme.c(SHIPPED))
	_ok("lit fill alpha is .16", snappedf(pin.bg_color.a, 0.001), 0.16)
	_ok("lit fill is NOT " + FORMER + " (.09, row-selection weight)",
		pin.bg_color == DccTheme.c(FORMER), false)
	_ok("lit fill is NOT the accent slab",
		pin.bg_color == DccTheme.c("accent"), false)
	_ok("lit border is accent", pin.border_color, DccTheme.c("accent"))

	var img_a := await _grab()
	img_a.save_png("user://washstep_%s_shipped_016.png" % tag)
	var before: Array = []
	for b in lit:
		before.append(_modal(img_a, _interior(Rect2i((b as Button).get_global_rect()))))

	## The counterfactual: same chips, same ground, `accent_wash_2`'s alpha.
	## The originals are kept and put back below -- without that, the second
	## palette's pass finds no lit segment at all, because this probe's own
	## override has already destroyed the signature it searches by.
	var orig: Array = []
	for b in lit:
		var src := (b as Button).get_theme_stylebox("normal") as StyleBoxFlat
		orig.append(src)
		var alt := src.duplicate() as StyleBoxFlat
		alt.bg_color = DccTheme.c(FORMER)
		for sb_name in ["normal", "pressed", "disabled"]:
			(b as Button).add_theme_stylebox_override(sb_name, alt)
	var img_b := await _grab()
	img_b.save_png("user://washstep_%s_former_009.png" % tag)

	var worst := 0
	for i in lit.size():
		var b: Button = lit[i]
		var r := Rect2i(b.get_global_rect())
		var former := _modal(img_b, _interior(r))
		var now: Color = before[i]
		var step := _d(former, now)
		worst = maxi(worst, int(step.get_slice("maxΔ=", 1)))
		print("  %-26s %3dx%-3d px  .09=%s  .16=%s  %s"
			% [b.text.substr(0, 26), r.size.x, r.size.y, _hex(former), _hex(now), step])
	## The scale bar: the ground-to-.09 step is the one this design already
	## ships and calls legible, so .09-to-.16 is reported against it.
	for i in off.size():
		var ob: Button = off[i]
		var j := lit.find(ob)
		if j >= 0:
			print("  -- scale bar %-18s ground=%s -> lit=%s  %s"
				% [ob.text.substr(0, 18), _hex(ground[i]), _hex(before[j]),
				   _d(ground[i], before[j])])
	for i in lit.size():
		for sb_name in ["normal", "pressed", "disabled"]:
			(lit[i] as Button).add_theme_stylebox_override(sb_name, orig[i])
	await _frames(2)
	print("[%s] WORST .09 -> .16 channel step over %d chips = %d/255"
		% [tag, lit.size(), worst])

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("WASHSTEP: refusing to run headless -- a texture read is not the "
			+ "framebuffer. Re-run windowed.")
		get_tree().quit(2)
		return
	var vp := get_viewport().get_visible_rect().size
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	## A real world, because the welcome screen lights exactly one segment and
	## one sample is not a measurement. Same call `_rdappend_shot.gd` uses.
	app.bridge.generate({
		"seed": 483920, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while app.bridge.generating:
		await get_tree().create_timer(0.2).timeout
	await _frames(20)
	var dens := "tablet" if DccTheme.is_tablet() else \
		("laptop" if DccTheme.is_laptop() else "desktop")
	print("=== washstep  viewport=%dx%d  density=%s  boot theme=%s ==="
		% [int(vp.x), int(vp.y), dens, "dark" if DccTheme.is_dark() else "light"])
	print("  token alphas: %s=%.2f  %s=%.2f"
		% [FORMER, DccTheme.c(FORMER).a, SHIPPED, DccTheme.c(SHIPPED).a])
	await _measure("%s_%s" % [dens, "dark" if DccTheme.is_dark() else "light"])
	## The other palette, through the shell's own repaint path.
	var was_dark := DccTheme.is_dark()
	DccTheme.apply_theme(not was_dark)
	(app as DccShell).rebuild_theme(was_dark)
	await _frames(30)
	await _measure("%s_%s" % [dens, "dark" if DccTheme.is_dark() else "light"])
	print("=== ", "PASS" if _fail == 0 else "FAIL (%d)" % _fail, " ===")
	get_tree().quit(0 if _fail == 0 else 1)
