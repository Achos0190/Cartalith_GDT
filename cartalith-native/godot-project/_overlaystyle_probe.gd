extends Node
## Protects: the style-preset treatment of the map overlay -- OUTSTANDING_WORK.md
## "Roads, sea lanes, borders, labels and icons ignore the style preset" (owner,
## 2026-09-27: they were "drawn on top of the style" in fixed colours). Each
## preset in `render_workspace.gd::OVERLAY_TREATMENTS` now sets the ink of the
## land ways, the sea lanes, the labels and the province lines, read live by the
## overlay (`map_overlay.gd::way_style_for` / `sea_style` / `label_inks`) and by
## the viewport (`viewport_host.gd::border_style`). This probe asserts, through
## the real shell on a real generated world:
##
##   A. IDENTITY. With "Default" (and "Natural Vibrant") applied, every resolved
##      colour and width equals the shipped constant -- written here as LITERALS
##      copied from the pre-change `WAY_STYLE` / `SEA_ROUTE_*` / `LABEL_STROKE_
##      COLOR` values, not read back from the overlay's own constants -- and the
##      province lines carry no shader material. Then Night is applied and Default
##      re-applied: the resolved values and a full-window screenshot return to the
##      first Default screenshot PIXEL-FOR-PIXEL (zero differing pixels), with a
##      positive control that Night's screenshot differs from it by a lot.
##   B. PINNED PRESETS. Night, Blueprint, Woodcut, Ink and Vintage atlas carry
##      the exact colours and width written in PINNED below -- literals computed
##      by hand from the table's intent (the mix is applied to the shipped
##      highway ochre), never from the preset's own constant, so a mutated table
##      row fails here.
##   C. EVERY PRESET. After pressing each preset's tile through the shell, the
##      overlay's live treatment is empty exactly for the two presets that
##      deliberately have none, and every other preset differs from Default in
##      its highway overlay colour, its sea dash colour, its label ink and its
##      border (a shader material is present with the preset's ink).
##   D. LEGIBILITY. For every preset with a treatment: the label ink against its
##      halo has a WCAG contrast ratio >= 4.5, and each of the five generated
##      class inks (after the preset's mix) against the halo >= 4.5 (the body-
##      text bar, raised from the large-text 3.0 after the 2026-10-05 screenshots
##      showed thin 11-14 px class labels reading grey; weakest measured 6.0). A hand-authored
##      label whose author picked a colour is untouched, keeping the shipped halo.
##   E. SCREENSHOTS. One full-window screenshot per preset (roads, a province
##      border, labels, settlement names in view) saved to --out, for looking at.
##
## Roads are asserted through `way_style_for()` -- the same function the draw
## calls -- and a screenshot control proves the draw moved (E's diff counts), so
## a resolver that was right but never reached the canvas still fails.
##
## MUST run WINDOWED (`MISTAKES.md`, "Run a pixel probe"); probes exit 0 even on
## failure, so grep the output for PROBE-FAIL and the RESULT line:
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _overlaystyle_probe.tscn -- --out DIR
##
## Arguments:
##   --out DIR   where the PNGs land (required)
##   --seed N    world seed (default 483920)
##   --grid WxH  world size (default 1024x656)
##   --before    only capture the Default screenshot (for a build that has no
##               treatment API yet); every API assertion is skipped

const PRESETS_ALL := ["Natural Vibrant", "Default", "Antique", "Ink", "Watercolor", "Print",
	"Village", "Atlas", "Imhof relief", "Blueprint", "Ink wash", "Woodcut",
	"Vintage atlas", "Nautical", "Night", "Cel / Toon"]
const NO_TREATMENT := ["Natural Vibrant", "Default"]
const ROAD_TYPES := ["highway", "regional", "road", "track", "ancient"]
## `map_overlay.gd`'s pre-change `WAY_STYLE` values (RGB, alpha, width) as
## literals: [under rgba, under_w, over rgba, over_w]. Copied from the table's
## own doc comment (reference lines 15515-15534), so a change to the constants
## fails identity instead of being compared against itself.
const OLD_WAY := {
	"highway": [[0.078, 0.039, 0.020, 0.55], 2.3, [0.824, 0.569, 0.216, 0.98], 1.45],
	"regional": [[0.098, 0.055, 0.020, 0.45], 1.8, [0.698, 0.463, 0.204, 0.88], 1.15],
	"road": [[0.118, 0.078, 0.039, 0.40], 1.2, [0.627, 0.392, 0.235, 0.75], 0.7],
	"track": [[0.118, 0.078, 0.039, 0.35], 1.1, [0.392, 0.471, 0.235, 0.75], 0.6],
	"ancient": [[0.078, 0.039, 0.020, 0.35], 1.1, [0.471, 0.431, 0.392, 0.65], 0.65],
}
const OLD_SEA_UNDER := [0.039, 0.118, 0.235, 0.4]
const OLD_SEA_DASH := [0.118, 0.510, 0.784, 0.7]
const OLD_HALO := [0.031, 0.024, 0.016, 0.8]
const OLD_NAME_FILL := [0.965, 0.925, 0.831, 1.0]
const OLD_CREAM_HEX := "#f0e4c8"
## The five generated class inks (`labels.rs::LABEL_TYPOGRAPHY_DEFAULTS`).
const CLASS_INKS := ["#e0a34a", "#c8cbcd", "#a9adb0", "#6f9fb5", "#8d9296"]
## Section B. Per preset: highway overlay rgb (the mix applied to the shipped
## ochre, by hand), highway overlay width, road underlayer rgb, sea dash rgb,
## label ink rgb, label halo rgba, border ink rgb, border alpha.
const PINNED := {
	"Night": {"over": [0.9122, 0.7307, 0.3798], "over_w": 1.45, "under": [0.02, 0.02, 0.04],
		"sea": [0.55, 0.75, 1.0], "ink": [0.93, 0.93, 0.98], "halo": [0.02, 0.02, 0.05, 0.95],
		"border": [0.85, 0.75, 0.55], "border_a": 0.8},
	"Blueprint": {"over": [0.8924, 0.9119, 0.9216], "over_w": 1.305, "under": [0.03, 0.10, 0.22],
		"sea": [0.75, 0.88, 1.0], "ink": [0.92, 0.96, 1.0], "halo": [0.02, 0.08, 0.20, 0.95],
		"border": [0.90, 0.95, 1.0], "border_a": 0.9},
	"Woodcut": {"over": [0.1724, 0.1289, 0.0756], "over_w": 1.74, "under": [0.05, 0.04, 0.03],
		"sea": [0.15, 0.13, 0.12], "ink": [0.10, 0.07, 0.04], "halo": [0.93, 0.87, 0.72, 0.95],
		"border": [0.10, 0.08, 0.06], "border_a": 1.0,
		"gen_ink": [0.1227, 0.0952, 0.0674]},
	"Ink": {"over": [0.2688, 0.2418, 0.2272], "over_w": 1.2325, "under": [0.08, 0.09, 0.12],
		"sea": [0.16, 0.22, 0.34], "ink": [0.10, 0.12, 0.18], "halo": [0.97, 0.95, 0.90, 0.95],
		"border": [0.13, 0.16, 0.23], "border_a": 1.0,
		"gen_ink": [0.1227, 0.1427, 0.2004]},
	"Vintage atlas": {"over": [0.5456, 0.3776, 0.1944], "over_w": 1.45, "under": [0.20, 0.14, 0.09],
		"sea": [0.25, 0.33, 0.40], "ink": [0.20, 0.14, 0.09], "halo": [0.93, 0.87, 0.73, 0.95],
		"border": [0.40, 0.22, 0.16], "border_a": 0.85},
}
const EPS := 0.006
var GRID := Vector2i(1024, 656)

var _out := ""
var _seed := 483920
var _before := false
var _fail := 0
var _checks := 0
var _app: Node
var _vh: Control
var _br: Node
var _ov: Control
var _tiles: Array = []
var _cam := Vector2.ZERO
var _shots := {}


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if cond:
		print("OVERLAYSTYLE  ok    " + what)
	else:
		_fail += 1
		print("PROBE-FAIL OVERLAYSTYLE  " + what)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _find(n: Node, script_file: String) -> Node:
	if n.get_script() != null and String(n.get_script().resource_path).ends_with(script_file):
		return n
	for c in n.get_children(true):
		var r := _find(c, script_file)
		if r != null:
			return r
	return null


func _parse_args() -> bool:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out":
				_out = args[i + 1]; i += 1
			"--seed":
				_seed = int(args[i + 1]); i += 1
			"--grid":
				var p := args[i + 1].split("x"); GRID = Vector2i(int(p[0]), int(p[1])); i += 1
			"--before":
				_before = true
			_:
				printerr("PROBE-CANNOT-RUN: unknown argument %s" % args[i])
				return false
		i += 1
	if _out == "":
		printerr("PROBE-CANNOT-RUN: --out DIR is required")
		return false
	DirAccess.make_dir_recursive_absolute(_out)
	return true


## `a` is an array literal [r, g, b(, a)]; compares channel by channel.
func _near(c: Color, a: Array, with_alpha: bool = false) -> bool:
	if absf(c.r - float(a[0])) > EPS or absf(c.g - float(a[1])) > EPS or absf(c.b - float(a[2])) > EPS:
		return false
	return not with_alpha or absf(c.a - float(a[3])) <= EPS


## WCAG relative luminance and contrast ratio (sRGB, no alpha).
func _lum(c: Color) -> float:
	var ch := func(v: float) -> float:
		return v / 12.92 if v <= 0.03928 else pow((v + 0.055) / 1.055, 2.4)
	return 0.2126 * ch.call(c.r) + 0.7152 * ch.call(c.g) + 0.0722 * ch.call(c.b)


func _contrast(a: Color, b: Color) -> float:
	var la := _lum(a)
	var lb := _lum(b)
	return (maxf(la, lb) + 0.05) / (minf(la, lb) + 0.05)


func _tile_index(pname: String) -> int:
	for i in RenderWorkspace.STYLE_PRESETS.size():
		if String(RenderWorkspace.STYLE_PRESETS[i][0]) == pname:
			return i
	return -1


func _press(pname: String) -> void:
	var idx := _tile_index(pname)
	_ok(idx >= 0, "preset %s is in STYLE_PRESETS" % pname)
	if idx < 0:
		return
	(_tiles[idx]["button"] as Button).emit_signal("pressed")
	await _frames(6)
	await get_tree().create_timer(0.4).timeout
	await _frames(3)


func _grab() -> Image:
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	img.convert(Image.FORMAT_RGB8)
	return img


func _settle() -> void:
	await _frames(10)
	var t0 := Time.get_ticks_msec()
	while _vh.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 20000:
		await get_tree().process_frame
	await get_tree().create_timer(0.6).timeout
	await _frames(4)


## Count pixels that differ at all between two same-size images.
func _diff_count(a: Image, b: Image) -> int:
	if a.get_size() != b.get_size():
		return -1
	var da := a.get_data()
	var db := b.get_data()
	var n := 0
	var px := a.get_width() * a.get_height()
	for i in px:
		var o := i * 3
		if da[o] != db[o] or da[o + 1] != db[o + 1] or da[o + 2] != db[o + 2]:
			n += 1
	return n


## Where to look: the settlement nearest a province-border pixel, so one frame
## holds a road end, a pin with its name, and a border line.
func _pick_view() -> Vector2:
	var setl: Array = _br.settlements()
	var tex: Texture2D = _br.province_boundary_texture()
	var img: Image = tex.get_image() if tex != null else null
	if img == null or setl.is_empty():
		return Vector2(GRID) * 0.5
	var best := Vector2(GRID) * 0.5
	var best_d := 1.0e9
	var lim: int = mini(setl.size(), 60)
	for si in lim:
		var s: Dictionary = setl[si]
		var sx := int(s["x"])
		var sy := int(s["y"])
		for r in range(2, 26, 2):
			var hit := false
			for dy in range(-r, r + 1, 2):
				for dx in range(-r, r + 1, 2):
					if maxi(absi(dx), absi(dy)) != r:
						continue
					var x := sx + dx
					var y := sy + dy
					if x < 0 or y < 0 or x >= img.get_width() or y >= img.get_height():
						continue
					if img.get_pixel(x, y).a > 0.3:
						hit = true
						break
				if hit:
					break
			if hit:
				if float(r) < best_d:
					best_d = float(r)
					best = Vector2(sx, sy)
				break
	print("OVERLAYSTYLE  view centre %s (nearest border %s cells)" % [best, best_d])
	return best


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless -- run windowed.")
		get_tree().quit(2)
		return
	if not _parse_args():
		get_tree().quit(2)
		return
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		printerr("PROBE-CANNOT-RUN: watchdog"); get_tree().quit(2))
	DisplayServer.window_set_size(Vector2i(1600, 1000))
	await _frames(2)
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_vh = _app.viewport
	_br = _app.bridge
	_ov = _vh.overlay
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	_br.generate({"seed": _seed, "width_km": 1200.0, "grid_w": GRID.x, "grid_h": GRID.y,
		"archetype": "", "villages": true, "sea_level": 0.42})
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await get_tree().create_timer(1.0).timeout
	_br.labels_generate({"cull": {"on": false}})
	_vh.refresh_annotations()
	for l in ["settlements", "roads", "sea_routes", "provinces"]:
		_vh.set_layer_visible(l, true)
	_vh.set_layer_visible("rivers", false)
	print("OVERLAYSTYLE  world: %d settlements, %d roads, %d sea routes, %d provinces, %d labels" % [
		_br.settlements().size(), _br.roads().size(), _br.sea_routes().size(),
		_br.provinces().size(), _br.labels_render_list().size()])

	_app.select_domain_category("cartography", "Style")
	await _frames(8)
	var rw := _find(_app, "render_workspace.gd")
	_ok(rw != null, "the RenderWorkspace is reachable")
	if rw == null:
		_finish()
		return
	_tiles = rw.get("_preset_tiles")

	## One view for every screenshot.
	_vh.reset_view()
	await _frames(3)
	_cam = _pick_view()
	_vh.zoom_step(3.0 / _vh.zoom())
	_vh.move_view_to(_cam.x, _cam.y)
	await _settle()

	## ---- the screenshot sweep, in an order that makes the identity leg ----
	await _press("Default")
	var first_default := await _grab_stable()
	first_default.save_png("%s/overlaystyle_Default_first.png" % _out)
	if _before:
		_finish()
		return
	_leg_identity("Default")
	await _press("Natural Vibrant")
	_leg_identity("Natural Vibrant")

	var diffs := {}
	for pname in PRESETS_ALL:
		await _press(pname)
		var img := await _grab_stable()
		img.save_png("%s/overlaystyle_%s.png" % [_out, pname.replace(" / ", "_").replace(" ", "_")])
		_shots[pname] = img
		diffs[pname] = _diff_count(first_default, img)
		_leg_every(pname)
		if PINNED.has(pname):
			_leg_pinned(pname)
		_leg_contrast(pname)
		if pname == "Blueprint":
			await _leg_border_pixels(pname)
	print("OVERLAYSTYLE  differing pixels vs the first Default screenshot: %s" % [diffs])

	## ---- A: identity round trip, with its positive control ----
	_ok(int(diffs["Night"]) > 2000, "positive control: Night's screenshot differs from Default's in more than 2000 pixels (%d)" % int(diffs["Night"]))
	_ok(int(diffs["Blueprint"]) > 2000, "positive control: Blueprint's screenshot differs from Default's in more than 2000 pixels (%d)" % int(diffs["Blueprint"]))
	await _press("Night")
	await _press("Default")
	var again := await _grab_stable()
	again.save_png("%s/overlaystyle_Default_again.png" % _out)
	_ok(_diff_count(first_default, again) == 0, "Default re-applied after Night is pixel-identical to the first Default screenshot (%d differing)" % _diff_count(first_default, again))
	_leg_identity("Default (after Night)")
	_finish()


## Two grabs a few frames apart must agree before the image is trusted, so a
## label still fading in or a tile still settling is not read as a style effect.
func _grab_stable() -> Image:
	await _settle()
	var a := await _grab()
	await _frames(4)
	var b := await _grab()
	var n := _diff_count(a, b)
	if n != 0:
		print("OVERLAYSTYLE  note: two grabs differed by %d px, settling again" % n)
		await _settle()
		b = await _grab()
	return b


func _leg_identity(tag: String) -> void:
	var t: Dictionary = _ov.style_treatment()
	_ok(t.is_empty(), "%s: the overlay carries the empty treatment" % tag)
	for ty in ROAD_TYPES:
		var row: Dictionary = _ov.way_style_for(ty)
		var old: Array = OLD_WAY[ty]
		_ok(_near(row["under"], old[0], true) and absf(float(row["under_w"]) - float(old[1])) < 1e-6
			and _near(row["over"], old[2], true) and absf(float(row["over_w"]) - float(old[3])) < 1e-6,
			"%s: %s road equals the shipped WAY_STYLE literals" % [tag, ty])
	var sea: Dictionary = _ov.sea_style()
	_ok(_near(sea["under"], OLD_SEA_UNDER, true) and _near(sea["dash"], OLD_SEA_DASH, true)
		and absf(float(sea["under_w"]) - 1.5) < 1e-6 and absf(float(sea["dash_w"]) - 0.85) < 1e-6,
		"%s: the sea lane equals the shipped SEA_ROUTE literals" % tag)
	var cream := {"color": OLD_CREAM_HEX, "generated": false}
	var gen := {"color": CLASS_INKS[0], "generated": true}
	var ci: Array = _ov.label_inks(cream)
	var gi: Array = _ov.label_inks(gen)
	_ok(_near(ci[0], [0.941, 0.894, 0.784], false) and _near(ci[1], OLD_HALO, true),
		"%s: a default-ink hand label keeps the cream fill and the shipped halo" % tag)
	_ok(_near(gi[0], [0.878, 0.639, 0.290], false) and _near(gi[1], OLD_HALO, true),
		"%s: a generated label keeps its class ink and the shipped halo" % tag)
	var ni: Array = _ov.settlement_name_inks()
	_ok(_near(ni[0], OLD_NAME_FILL, true) and _near(ni[1], OLD_HALO, true),
		"%s: the settlement name keeps the shipped fill and halo" % tag)
	_ok(not bool(_vh.border_style()["material"]), "%s: the province lines carry no shader material" % tag)
	## Protects the identity of the halo width: an untreated overlay multiplies a
	## label's halo by exactly 1.0 (the shipped width).
	_ok(_ov._label_halo_k() == 1.0, "%s: the label halo width multiplier is exactly 1.0" % tag)


func _leg_every(pname: String) -> void:
	var t: Dictionary = _ov.style_treatment()
	if NO_TREATMENT.has(pname):
		_ok(t.is_empty(), "%s: no treatment, by design" % pname)
		return
	_ok(not t.is_empty(), "%s: a treatment is live on the overlay" % pname)
	var hw: Dictionary = _ov.way_style_for("highway")
	var old_over: Array = OLD_WAY["highway"][2]
	_ok(not _near(hw["over"], old_over, false), "%s: the highway overlay colour moved off the shipped ochre" % pname)
	if t.has("sea_dash"):
		_ok(not _near(_ov.sea_style()["dash"], OLD_SEA_DASH, false), "%s: the sea dash colour moved off the shipped blue" % pname)
	var ink: Color = _ov.label_inks({"color": OLD_CREAM_HEX, "generated": false})[0]
	_ok(not _near(ink, [0.941, 0.894, 0.784], false), "%s: the default-ink label colour moved off the cream" % pname)
	var b: Dictionary = _vh.border_style()
	_ok(bool(b["material"]), "%s: the province lines carry the preset's shader material" % pname)


func _leg_pinned(pname: String) -> void:
	var p: Dictionary = PINNED[pname]
	var hw: Dictionary = _ov.way_style_for("highway")
	_ok(_near(hw["over"], p["over"], false), "%s: highway overlay rgb %s (got %s)" % [pname, p["over"], hw["over"]])
	_ok(absf(float(hw["over_w"]) - float(p["over_w"])) < 0.002, "%s: highway overlay width %s (got %s)" % [pname, p["over_w"], hw["over_w"]])
	_ok(absf(float(hw["over"].a) - 0.98) < EPS, "%s: highway overlay keeps its own alpha 0.98" % pname)
	_ok(_near(hw["under"], p["under"], false), "%s: highway underlayer rgb %s (got %s)" % [pname, p["under"], hw["under"]])
	_ok(_near(_ov.sea_style()["dash"], p["sea"], false), "%s: sea dash rgb %s" % [pname, p["sea"]])
	var inks: Array = _ov.label_inks({"color": OLD_CREAM_HEX, "generated": false})
	_ok(_near(inks[0], p["ink"], false) and _near(inks[1], p["halo"], true), "%s: label ink %s and halo %s (got %s / %s)" % [pname, p["ink"], p["halo"], inks[0], inks[1]])
	## Protects the halo-width key: every pinned preset widens the halo 1.5x
	## (render_workspace.gd's label_halo_k), a literal worked by hand, not read back.
	_ok(absf(_ov._label_halo_k() - 1.5) < 1e-6, "%s: label halo width multiplier 1.5 (got %s)" % [pname, _ov._label_halo_k()])
	if p.has("gen_ink"):
		## Protects the generated-label mix: the region class ink "#8d9296" lerped
		## to the preset ink by the table's label_mix (0.95), worked by hand.
		var gi: Array = _ov.label_inks({"color": "#8d9296", "generated": true})
		_ok(_near(gi[0], p["gen_ink"], false), "%s: generated region ink %s (got %s)" % [pname, p["gen_ink"], gi[0]])
	var ni: Array = _ov.settlement_name_inks()
	_ok(_near(ni[0], p["ink"], false) and _near(ni[1], p["halo"], true), "%s: the settlement name takes the same ink pair" % pname)
	var b: Dictionary = _vh.border_style()
	_ok(bool(b["material"]) and _near(b["ink"], p["border"], false) and absf(float(b["alpha"]) - float(p["border_a"])) < EPS,
		"%s: border ink %s alpha %s (got %s)" % [pname, p["border"], p["border_a"], b])


## Protects the border shader's two outputs, which the uniform read-back in
## `_leg_pinned` cannot see (a shader that ignored `border_alpha` still carries the
## right uniform): the live `province_view` material is applied to a TextureRect of
## one baked-style texel -- the engine's own ink `[35, 24, 9]` at alpha 200 -- and
## rendered into a transparent SubViewport, then the pixel is read back.
## Expected alpha is 200/255 * the preset's `border_alpha`, worked by hand
## (Blueprint 0.9 -> 0.706); the **positive control** is the same texel with no
## material, which must come back at 0.784 and still brown, so the read-back can
## tell a recoloured/dimmed texel from an untouched one.
func _leg_border_pixels(pname: String) -> void:
	var live := _vh.province_view.material as ShaderMaterial
	_ok(live != null, "%s: border pixel leg has a live shader material to render" % pname)
	if live == null:
		return
	var styled := await _texel_through(live)
	var control := await _texel_through(null)
	_ok(absf(control.a - 200.0 / 255.0) < 0.03 and control.r < 0.3,
		"positive control: the untreated baked texel reads back brown at alpha 0.784 (got %s)" % control)
	_ok(absf(styled.a - 200.0 / 255.0 * 0.9) < 0.03,
		"%s: the border shader scales the baked alpha by border_alpha 0.9 -> 0.706 (got %.3f)" % [pname, styled.a])
	_ok(styled.r > 0.5 and styled.b > 0.5,
		"%s: the border shader swaps the baked brown for the preset's pale ink (got %s)" % [pname, styled])


## One 8x8 baked-style texel drawn through `mat` (or none) into a transparent
## SubViewport and read back at its centre. Local to the probe; frees what it makes.
func _texel_through(mat: Material) -> Color:
	var sv := SubViewport.new()
	sv.size = Vector2i(8, 8)
	sv.transparent_bg = true
	sv.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	var img := Image.create(8, 8, false, Image.FORMAT_RGBA8)
	img.fill(Color8(35, 24, 9, 200))
	var tr := TextureRect.new()
	tr.texture = ImageTexture.create_from_image(img)
	tr.size = Vector2(8, 8)
	tr.material = mat
	sv.add_child(tr)
	add_child(sv)
	await _frames(3)
	await RenderingServer.frame_post_draw
	var out := sv.get_texture().get_image().get_pixel(4, 4)
	sv.queue_free()
	return out


func _leg_contrast(pname: String) -> void:
	if NO_TREATMENT.has(pname):
		return
	var inks: Array = _ov.label_inks({"color": OLD_CREAM_HEX, "generated": false})
	var r: float = _contrast(inks[0], Color(inks[1].r, inks[1].g, inks[1].b))
	_ok(r >= 4.5, "%s: label ink vs halo contrast %.2f >= 4.5" % [pname, r])
	var worst := 99.0
	var worst_hex := ""
	for hex in CLASS_INKS:
		var gi: Array = _ov.label_inks({"color": hex, "generated": true})
		var cr: float = _contrast(gi[0], Color(gi[1].r, gi[1].g, gi[1].b))
		if cr < worst:
			worst = cr
			worst_hex = hex
	_ok(worst >= 4.5, "%s: weakest generated class ink (%s) vs halo contrast %.2f >= 4.5" % [pname, worst_hex, worst])
	var own: Array = _ov.label_inks({"color": "#ff3366", "generated": false})
	_ok(_near(own[0], [1.0, 0.2, 0.4], false) and _near(own[1], OLD_HALO, true),
		"%s: an author-picked label colour keeps its own colour and the shipped halo" % pname)


func _finish() -> void:
	print("OVERLAYSTYLE  RESULT %d checks, %d failed" % [_checks, _fail])
	get_tree().quit(0 if _fail == 0 else 1)
