extends Node
## Protects: ET-F1 (EDITING_TOOLS_SCOPE.md section 7; Ruling BX item 4), the Look sliders
## (Hue, Saturation, Brightness) in `shell/workspaces/render_workspace.gd`, end to end in the
## real shell on a generated world. They write ONE Hue/Sat/Light entry pinned at the top of
## the selected groups' stacks through `adjust_panel.gd`'s write path. What it guards, by leg
## (`--legs` letters; default all):
##   A REST. A gesture that ends at 0/0/0 leaves every stack's saved form and the map texture
##     byte-identical and pushes no history entry; a layer that already holds a user entry gets
##     the Look INSERTED ABOVE it (index 0) and gets back exactly its old text at 0/0/0.
##   B SATURATION -100 %. Every layer's pinned entry is `{hue 0, saturation -1, lightness 0}`;
##     the engine's map texture has OkLCh chroma < 1e-3 on EVERY pixel, and so has the
##     on-screen map (overlay layers hidden) in a central box -- chroma computed HERE from the
##     captured bytes (IEC 61966-2-1 transfer + Ottosson 2020 matrices written out below), never
##     by the shell's or engine's own function. Labels follow: label fills the guard did not move
##     are greys. Positive control: the same measures before the edit are > 1e-3.
##   C UNDO / REDO. Edit > Undo (`DccApp.look_undo`) restores every stack's text and the map
##     texture byte for byte; Redo restores the edited bytes; the entry's label is
##     "Look: saturation" (a literal).
##   D GROUPS. "Water" writes the water stack only; "Labels & overlays" the overlay stack only.
##   E GUARDS. With the Look at Saturation -100, Brightness -100 and Brightness +100 on all
##     groups, under Default and the 17 presets: every label / settlement-name / journey pair
##     keeps min(4.5, its unadjusted contrast) (WCAG contrast computed HERE) and every faction
##     set keeps its bars or reverts to the unadjusted set (`_etc3_probe.gd` leg D's method).
##   F PANEL. The pinned entry is listed ONCE, tagged (meta `adj_look`), its Down and the next
##     row's Up are disabled; a panel edit of it moves the Look slider (two views, one state);
##     deleting it in the panel returns the slider to 0.
##   G LOCK. While a generate runs the sliders and picker are locked and a forced write makes
##     no engine call (stderr is grepped for panics by the runner); unlocked afterwards.
##   H TOUCH (only with --force-touch): each Look slider row and track is >= 44 px tall.
## Driving: a gesture is `drag_started` + `value` + `drag_ended` on the real HSlider (the
## handler path, NOT the touch path -- MISTAKES.md "Reach a screen inside a probe").
##
## Run WINDOWED, never headless (MISTAKES.md, "Run a pixel probe"), from a scratch copy:
##   godot --path <copy> res://_etf1_probe.tscn -- [--force-touch --vp 1080x2340] [--tag T]
##       [--legs abcdefgh] [--seed N] [--grid WxH]
## Exit 0 pass, 1 a check failed, 2 could not run.
## Result: an assertion, windowed only. Command per the header: `godot --path <copy> res://_etf1_probe.tscn --
## [--force-touch --vp 1080x2340] [--tag T] [--legs abcdefgh] [--seed N] [--grid WxH]` (defaults: `--vp` 1600x900,
## `--tag` `etf1`, `--legs` all, `--grid` 384x246). Every line is prefixed `[<tag>] `, ending `RESULT <tag> checks=<n>
## fail=<n>`. Exit 0 pass, 1 a check failed, 2 could not run (an unknown flag, or `ABORT run WINDOWED`), as the header
## states.

var OA = null
var app: Node
var bridge: Node
var rw: Node
var panel: Node
var _vh: Control
var _ov: Control
var _vp: SubViewport
var _tag := "etf1"
var _checks := 0
var _fail := 0
var _seed := 808   ## the `_etc3_probe.gd` world: 228 settlements, 8 provinces at 384x246
var _grid := Vector2i(384, 246)
var _legs := "abcdefgh"

const PRESETS_ALL := ["Natural Vibrant", "Default", "Antique", "Ink", "Watercolor", "Print",
	"Village", "Atlas", "Imhof relief", "Blueprint", "Ink wash", "Woodcut",
	"Vintage atlas", "Nautical", "Night", "Cel / Toon", "Tanaka relief"]
const CLASS_INKS := ["#e0a34a", "#c8cbcd", "#a9adb0", "#6f9fb5", "#8d9296"]
## Okabe-Ito, as `_etc3_probe.gd` writes them (not read from the overlay).
const SWATCHES := [Color8(230, 159, 0), Color8(86, 180, 233), Color8(0, 158, 115),
	Color8(240, 228, 66), Color8(0, 114, 178), Color8(213, 94, 0)]
const CUSTOM := [false, false, true, false, false, false]
## The acceptance bar: OkLCh chroma "below 1e-3 on every pixel" (EDITING_TOOLS_SCOPE.md 7).
const CHROMA_BAR := 1e-3
## Ottosson 2020, "A perceptual color space for image processing": linear sRGB -> LMS and
## cube-rooted LMS -> OkLab, the published 10-digit values, typed from the post.
const M1 := [[0.4122214708, 0.5363325363, 0.0514459929],
	[0.2119034982, 0.6806995451, 0.1073969566],
	[0.0883024619, 0.2817188376, 0.6299787005]]
const M2 := [[0.2104542553, 0.7936177850, -0.0040720468],
	[1.9779984951, -2.4285922050, 0.4505937099],
	[0.0259040371, 0.7827717662, -0.8086757660]]

var _chroma_memo := {}


func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])


func _check(ok: bool, what: String) -> void:
	_checks += 1
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])


func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt


func _reject_unknown_args() -> bool:
	var known := ["--force-touch", "--vp", "--tag", "--legs", "--seed", "--grid"]
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var s := String(args[i])
		if s.begins_with("--") and not (s in known):
			_log("ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			return false
		if s in ["--vp", "--tag", "--legs", "--seed", "--grid"]:
			i += 1
		i += 1
	return true


func _finish() -> void:
	_log("RESULT %s checks=%d fail=%d" % [_tag, _checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


## `ImageTexture.update()` is a no-op headless: refuse to run there.
func _guard() -> bool:
	var img := Image.create_empty(2, 2, false, Image.FORMAT_RGBA8)
	img.fill(Color(0, 0, 0, 0))
	var t := ImageTexture.create_from_image(img)
	img.set_pixel(1, 1, Color(1, 0, 0, 1))
	t.update(img)
	return t.get_image().get_pixel(1, 1).a > 0.0


# ------------------------------------------------------------- colour maths ----

func _lin(v: float) -> float:
	return v / 12.92 if v <= 0.04045 else pow((v + 0.055) / 1.055, 2.4)


## OkLCh chroma of an 8-bit sRGB colour, computed here (memoised per colour).
func _chroma8(r: int, g: int, b: int) -> float:
	var key := (r << 16) | (g << 8) | b
	if _chroma_memo.has(key):
		return _chroma_memo[key]
	var l := [_lin(r / 255.0), _lin(g / 255.0), _lin(b / 255.0)]
	var lms: Array = []
	for row in M1:
		var v: float = row[0] * l[0] + row[1] * l[1] + row[2] * l[2]
		lms.append(signf(v) * pow(absf(v), 1.0 / 3.0))
	var a: float = M2[1][0] * lms[0] + M2[1][1] * lms[1] + M2[1][2] * lms[2]
	var bb: float = M2[2][0] * lms[0] + M2[2][1] * lms[1] + M2[2][2] * lms[2]
	var c := sqrt(a * a + bb * bb)
	_chroma_memo[key] = c
	return c


func _chroma(c: Color) -> float:
	return _chroma8(int(round(c.r * 255.0)), int(round(c.g * 255.0)), int(round(c.b * 255.0)))


## `[max chroma, pixels over CHROMA_BAR, pixels, over pixels whose channels differ by at most
## 1 LSB, a sample over pixel]` over `img` inside `rect` (whole image if empty). The fourth
## figure separates an 8-bit quantisation split of a grey (the engine's single quantiser
## truncates, `render::finish_rgb`) from a colour that kept chroma.
func _chroma_scan(img: Image, rect: Rect2i = Rect2i()) -> Array:
	var im := img.duplicate() as Image
	im.convert(Image.FORMAT_RGB8)
	var w := im.get_width()
	var r := rect if rect.size != Vector2i.ZERO else Rect2i(0, 0, w, im.get_height())
	var data := im.get_data()
	var mx := 0.0
	var over := 0
	var split1 := 0
	var sample := ""
	var n := 0
	for y in range(r.position.y, r.end.y):
		for x in range(r.position.x, r.end.x):
			var o := (y * w + x) * 3
			var c := _chroma8(data[o], data[o + 1], data[o + 2])
			n += 1
			if c > mx:
				mx = c
				sample = "(%d,%d,%d) C=%.5f" % [data[o], data[o + 1], data[o + 2], c]
			if c >= CHROMA_BAR:
				over += 1
				var hi := maxi(data[o], maxi(data[o + 1], data[o + 2]))
				var lo := mini(data[o], mini(data[o + 1], data[o + 2]))
				if hi - lo <= 1:
					split1 += 1
	return [mx, over, n, split1, sample]


## WCAG relative luminance and contrast, written out (sRGB, alpha ignored).
func _lum(c: Color) -> float:
	return 0.2126 * _lin(c.r) + 0.7152 * _lin(c.g) + 0.0722 * _lin(c.b)


func _contrast(a: Color, b: Color) -> float:
	var la := _lum(a)
	var lb := _lum(b)
	return (maxf(la, lb) + 0.05) / (minf(la, lb) + 0.05)


func _cs(c: Color) -> String:
	return "%.9f,%.9f,%.9f" % [c.r, c.g, c.b]


# ------------------------------------------------------------- shell access ----

## Saved-form text of every adjustable layer, read from the owners (engine / overlay script).
func _stacks() -> Dictionary:
	var d := {}
	for l in panel.layer_ids():
		if String(l) == "overlay":
			d[l] = OA.get_stack_json()
		else:
			var g: Dictionary = bridge.adjust_get(String(l))
			d[l] = String(g.get("stack", "<unreadable>"))
	return d


func _entries(layer: String) -> Array:
	var t: String = _stacks()[layer]
	var p = JSON.parse_string(t)
	return (p as Dictionary).get("entries", []) if p is Dictionary else []


func _tex_bytes() -> PackedByteArray:
	var t: Texture2D = _vh.map_view.texture
	if t == null:
		return PackedByteArray()
	var im := t.get_image()
	im.convert(Image.FORMAT_RGB8)
	return im.get_data()


func _tex_image() -> Image:
	return (_vh.map_view.texture as Texture2D).get_image()


func _slider(key: String) -> HSlider:
	return rw._quick_rows[key]["slider"]


## One gesture on a Look slider through its own signals.
func _look(key: String, values: Array) -> void:
	var s := _slider(key)
	s.drag_started.emit()
	for v in values:
		s.value = float(v)
		await get_tree().create_timer(0.05).timeout
	s.drag_ended.emit(true)
	await _frames(4)


func _pick_group(id: String) -> void:
	var i: int = rw._quick_ids.find(id)
	rw._quick_pick.select(i)
	rw._quick_pick.item_selected.emit(i)
	await _frames(2)


func _grab() -> Image:
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	img.convert(Image.FORMAT_RGB8)
	return img


func _settle() -> void:
	await _frames(8)
	var t0 := Time.get_ticks_msec()
	while _vh.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 20000:
		await get_tree().process_frame
	await get_tree().create_timer(0.4).timeout
	await _frames(4)


## Central box of the map as drawn: the middle 40 % of the ViewportHost's rect in the
## SubViewport (a box chosen by POSITION, never by the colour under test).
func _map_box() -> Rect2i:
	var r: Rect2 = _vh.get_global_rect()
	var c := r.get_center()
	var half := r.size * 0.2
	var box := Rect2i(Vector2i(c - half), Vector2i(half * 2.0))
	return box.intersection(Rect2i(Vector2i.ZERO, _vp.size))


func _hide_overlays(hide: bool) -> void:
	for l in ["settlements", "roads", "sea_routes", "provinces", "rivers"]:
		_vh.set_layer_visible(l, not hide)
	_ov.set_labels([] if hide else bridge.labels_render_list())
	_ov.queue_redraw()


func _context(pname: String) -> void:
	var t := RenderWorkspace.overlay_treatment_for(pname) if pname != "Default" else {}
	_vh.set_overlay_treatment(t)
	_ov.set_faction_custom(CUSTOM)
	_ov.set_faction_colors(SWATCHES)


# --------------------------------------------------------------------- main ----

func _ready() -> void:
	_tag = _arg("--tag", "etf1")
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
	if not _guard():
		_log("ABORT run WINDOWED")
		get_tree().quit(2)
		return
	_legs = _arg("--legs", "abcdefgh")
	_seed = int(_arg("--seed", str(_seed)))
	var gp := _arg("--grid", "384x246").split("x")
	_grid = Vector2i(int(gp[0]), int(gp[1]))
	var parts: PackedStringArray = _arg("--vp", "1600x900").split("x")
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)
	bridge = app.bridge
	_vh = app.viewport
	_ov = _vh.overlay
	OA = load("res://shell/overlay_adjust.gd")
	for ws in app._workspaces:
		if ws is CartographyWorkspace and ws._render != null:
			rw = ws._render
	_log("viewport %dx%d phone=%s touch=%s; DLL mtime (unix) %d" % [_vp.size.x, _vp.size.y, app.is_phone(), DccTheme.is_touch(),
		FileAccess.get_modified_time(ProjectSettings.globalize_path("res://").path_join("../target/debug/cartalith_godot.dll"))])
	_check(rw != null, "the render workspace is composed into CARTO")
	if rw == null:
		_finish()
		return
	panel = rw._adjust_panel
	var has_look: bool = rw.get("_quick_rows") != null and not (rw._quick_rows as Dictionary).is_empty()
	_check(has_look, "the Look sliders exist")
	if not has_look or panel == null:
		_finish()
		return
	var names: Array = []
	for i in rw._quick_pick.item_count:
		names.append(rw._quick_pick.get_item_text(i))
	_check(names == ["All groups", "Terrain", "Water", "Labels & overlays"], "group picker items %s" % str(names))
	_check(rw._quick_pick.selected == 0, "All groups is the default")
	_check(_slider("hue_deg").name == "Look_hue_deg" and _slider("lightness").max_value == 100.0 and _slider("saturation").min_value == -100.0,
		"three sliders: hue degrees, saturation and brightness in percent")

	bridge.generate({"seed": _seed, "width_km": 1200.0, "grid_w": _grid.x, "grid_h": _grid.y,
		"archetype": "", "villages": true, "sea_level": 0.42})
	var spins := 0
	while bridge.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await get_tree().create_timer(1.0).timeout
	_check(bridge.has_world, "a world landed (seed %d, %dx%d)" % [_seed, _grid.x, _grid.y])
	if not bridge.has_world:
		_finish()
		return
	bridge.labels_generate({"cull": {"on": false}})
	_vh.refresh_annotations()
	_hide_overlays(false)
	await _settle()

	if "a" in _legs:
		await _leg_a()
	if "b" in _legs:
		await _leg_b()
	if "c" in _legs:
		await _leg_c()
	if "d" in _legs:
		await _leg_d()
	if "e" in _legs:
		await _leg_e()
	if "f" in _legs:
		await _leg_f()
	if "g" in _legs:
		await _leg_g()
	if "h" in _legs:
		_leg_h()
	_finish()


# ===================================================================== A ====

func _leg_a() -> void:
	_log("--- A: rest is absence")
	var s0 := _stacks()
	var t0 := _tex_bytes()
	var h0: int = app.look_history.entries.size()
	_check(not t0.is_empty(), "map texture readable (%d bytes)" % t0.size())
	await _look("saturation", [30.0, 0.0])
	_check(_stacks() == s0, "a gesture ending at 0/0/0 leaves every stack's text unchanged")
	_check(_tex_bytes() == t0, "... and the map texture byte-identical")
	_check(app.look_history.entries.size() == h0, "... and pushes no history entry (%d -> %d)" % [h0, app.look_history.entries.size()])
	# A user entry already on Terrain: the Look goes ABOVE it and gives it back exactly.
	var lv := JSON.stringify({"entries": [{"kind": {"levels": {"in_black": 0.0, "in_white": 255.0, "gamma": 1.3, "out_black": 0.0, "out_white": 255.0}}, "enabled": true, "opacity": 1.0}]})
	_check(bool(bridge.adjust_set("terrain", lv).get("ok", false)), "setup: a user Levels entry on Terrain")
	rw.on_adjust_restored()
	var s1 := _stacks()
	await _look("saturation", [-50.0])
	var te := _entries("terrain")
	_check(te.size() == 2 and (te[0]["kind"] as Dictionary).has("hsl") and (te[1]["kind"] as Dictionary).has("levels"),
		"Look pinned at the TOP of Terrain, above the user's Levels (%s)" % str(te.map(func(e): return (e["kind"] as Dictionary).keys()[0])))
	for l in ["water", "overlay"]:
		var e := _entries(l)
		_check(e.size() == 1 and (e[0]["kind"] as Dictionary).has("hsl"), "all groups: %s holds one pinned Hue/Sat/Light" % l)
	await _look("saturation", [0.0])
	_check(_stacks() == s1, "back to 0: every stack's text equals its pre-Look text (the Levels kept, the Look absent)")
	bridge.adjust_set("terrain", "{\"entries\":[]}")
	rw.on_adjust_restored()


# ===================================================================== B ====

func _leg_b() -> void:
	_log("--- B: saturation -100 %")
	_hide_overlays(true)
	_vh.refresh()
	await _settle()
	var before := _chroma_scan(_tex_image())
	var scr_before := _chroma_scan(await _grab(), _map_box())
	_log("before: texture max C %.5f (%d of %d over); screen box %s max C %.5f (%d of %d over)" % [before[0], before[1], before[2], str(_map_box()), scr_before[0], scr_before[1], scr_before[2]])
	_check(float(before[0]) > CHROMA_BAR and float(scr_before[0]) > CHROMA_BAR, "positive control: the untouched map has colour on texture and screen")
	await _look("saturation", [-60.0, -100.0])
	for l in panel.layer_ids():
		var e := _entries(String(l))
		var ok: bool = e.size() >= 1 and (e[0]["kind"] as Dictionary).has("hsl")
		if ok:
			var b: Dictionary = e[0]["kind"]["hsl"]
			ok = float(b["saturation"]) == -1.0 and float(b["hue_deg"]) == 0.0 and float(b["lightness"]) == 0.0
		_check(ok, "%s: pinned entry is {hue 0, saturation -1, lightness 0}" % l)
	await _settle()
	var after := _chroma_scan(_tex_image())
	_check(int(after[1]) == 0, "map texture: OkLCh C < %s on every pixel (max %.6f at %s, %d of %d over)" % [str(CHROMA_BAR), after[0], after[4], after[1], after[2]])
	## Diagnostic for the bar above (not a substitute for it): an over-bar pixel whose three
	## channels differ by at most 1 is a grey split by the 8-bit quantiser, not kept colour.
	_check(int(after[3]) == int(after[1]), "map texture: every over-bar pixel is a <= 1 LSB channel split of a grey (%d of %d)" % [after[3], after[1]])
	var scr := _chroma_scan(await _grab(), _map_box())
	_check(int(scr[1]) == 0, "screen map box, overlays hidden: C < %s on every pixel (max %.6f at %s, %d of %d over)" % [str(CHROMA_BAR), scr[0], scr[4], scr[1], scr[2]])
	_check(int(scr[3]) == int(scr[1]), "screen map box: every over-bar pixel is a <= 1 LSB channel split (%d of %d)" % [scr[3], scr[1]])
	# Labels follow (all groups): a label fill the guard did not move is a grey.
	_context("Default")
	var grey := 0
	var guarded := 0
	var keep: String = OA.get_stack_json()
	var pre := {}
	_vh.set_overlay_adjust("{\"entries\":[]}")
	for ink in CLASS_INKS + ["#f0e4c8", "#ff3366"]:
		pre[ink] = _ov.label_inks({"color": ink, "generated": ink in CLASS_INKS})
	_vh.set_overlay_adjust(keep)
	for ink in CLASS_INKS + ["#f0e4c8", "#ff3366"]:
		var p: Array = _ov.label_inks({"color": ink, "generated": ink in CLASS_INKS})
		if _cs(p[0]) == _cs(OA.color(pre[ink][0])):
			if _chroma(p[0]) < CHROMA_BAR:
				grey += 1
			else:
				_log("  label %s fill %s chroma %.5f" % [ink, _cs(p[0]), _chroma(p[0])])
		else:
			guarded += 1
	_check(grey + guarded == 7 and grey > 0, "labels follow: %d label fills are grey, %d moved by the contrast guard, 0 coloured" % [grey, guarded])
	_hide_overlays(false)


# ===================================================================== C ====

func _leg_c() -> void:
	_log("--- C: undo / redo")
	# Start from a clean look.
	for l in panel.layer_ids():
		if String(l) == "overlay":
			_vh.set_overlay_adjust("{\"entries\":[]}")
		else:
			bridge.adjust_set(String(l), "{\"entries\":[]}")
	rw.on_adjust_restored()
	_vh.refresh()
	await _settle()
	var s0 := _stacks()
	var t0 := _tex_bytes()
	await _look("saturation", [-100.0])
	await _settle()
	var s1 := _stacks()
	var t1 := _tex_bytes()
	_check(t1 != t0, "positive control: the edit moved the map texture")
	_check(String(app.look_history.undo_label()) == "Look: saturation", "history label %s" % app.look_history.undo_label())
	app.look_undo()
	await _settle()
	_check(_stacks() == s0, "undo restores every stack's text")
	_check(_tex_bytes() == t0, "undo restores the map texture byte for byte")
	_check(_slider("saturation").value == 0.0, "undo returns the Saturation slider to 0")
	app.look_redo()
	await _settle()
	_check(_stacks() == s1 and _tex_bytes() == t1, "redo restores the edited stacks and bytes")
	_check(_slider("saturation").value == -100.0, "redo returns the slider to -100")
	app.look_undo()
	await _settle()


# ===================================================================== D ====

func _leg_d() -> void:
	_log("--- D: groups")
	var s0 := _stacks()
	await _pick_group("water")
	await _look("saturation", [-100.0])
	var s1 := _stacks()
	_check(s1["terrain"] == s0["terrain"] and s1["overlay"] == s0["overlay"] and s1["water"] != s0["water"], "Water writes the water stack only")
	await _look("saturation", [0.0])
	_check(_stacks() == s0, "Water back to 0: unchanged")
	await _pick_group("overlay")
	await _look("lightness", [40.0])
	var s2 := _stacks()
	_check(s2["terrain"] == s0["terrain"] and s2["water"] == s0["water"] and s2["overlay"] != s0["overlay"], "Labels & overlays writes the overlay stack only")
	var oe := _entries("overlay")
	_check(oe.size() == 1 and float(oe[0]["kind"]["hsl"]["lightness"]) == 0.4, "Brightness writes the entry's lightness (0.4), not exposure")
	await _look("lightness", [0.0])
	_check(_stacks() == s0, "overlay back to 0: unchanged")
	await _pick_group("all")


# ===================================================================== E ====

func _leg_e() -> void:
	_log("--- E: contrast guards over the Look")
	var settings := [["saturation", -100.0], ["lightness", -100.0], ["lightness", 100.0]]
	var label_n := 0
	var label_bad := 0
	var fac_n := 0
	var fac_bad := 0
	var modes := {}
	for st in settings:
		await _look(String(st[0]), [float(st[1])])
		_check(OA.is_live(), "Look %s %d: the overlay stack is live" % [st[0], st[1]])
		for pname in PRESETS_ALL:
			# The unadjusted references: read with the stack held aside, then put back.
			var keep: String = OA.get_stack_json()
			_vh.set_overlay_adjust("{\"entries\":[]}")
			_context(pname)
			var t: Dictionary = _ov.style_treatment()
			var lbl := {"cream": {"color": "#f0e4c8", "generated": false}, "author": {"color": "#ff3366", "generated": false}}
			for i in CLASS_INKS.size():
				lbl["gen%d" % i] = {"color": CLASS_INKS[i], "generated": true}
			var pre := {}
			for k in lbl:
				pre[k] = _ov.label_inks(lbl[k])
			var pre_name: Array = _ov.settlement_name_inks()
			var pre_f: Array = []
			for f in range(1, 7):
				pre_f.append(_ov._faction_color(f))
			var ground: Color = _ov.pin_ground(t)
			var req_de := minf(20.0, _ov.min_pair_delta_e(pre_f))
			var req_c := minf(3.0, _ov.min_contrast_against(pre_f, ground)) if _ov.has_pin_style(t) else 0.0
			_vh.set_overlay_adjust(keep)
			_context(pname)
			for k in lbl:
				var p: Array = _ov.label_inks(lbl[k])
				var req := minf(4.5, _contrast(pre[k][0], pre[k][1]))
				label_n += 1
				if _contrast(p[0], p[1]) < req - 1e-3:
					label_bad += 1
					_log("  LABEL %s / %s %d / %s: %.3f < %.3f" % [pname, st[0], st[1], k, _contrast(p[0], p[1]), req])
			var pn: Array = _ov.settlement_name_inks()
			label_n += 1
			if _contrast(pn[0], pn[1]) < minf(4.5, _contrast(pre_name[0], pre_name[1])) - 1e-3:
				label_bad += 1
				_log("  NAME %s / %s %d" % [pname, st[0], st[1]])
			var cols: Array = []
			for f in range(1, 7):
				cols.append(_ov._faction_color(f))
			var mode: String = _ov.adjusted_factions_mode()
			modes[mode] = int(modes.get(mode, 0)) + 1
			fac_n += 1
			if mode == "original":
				for i in 6:
					if _cs(cols[i]) != _cs(pre_f[i]):
						fac_bad += 1
						break
			else:
				var de: float = _ov.min_pair_delta_e(cols)
				var cc: float = _ov.min_contrast_against(cols, _ov.adjust_overlay_color(ground))
				if de < req_de - 1e-3 or cc < req_c - 1e-3:
					fac_bad += 1
					_log("  FACTION %s / %s %d (%s): de %.2f / %.2f, c %.2f / %.2f" % [pname, st[0], st[1], mode, de, req_de, cc, req_c])
		await _look(String(st[0]), [0.0])
	_context("Default")
	_check(label_bad == 0, "label / name pairs keep min(4.5, unadjusted): %d pairs, %d violations" % [label_n, label_bad])
	_check(fac_bad == 0, "faction sets keep their bars or revert: %d sets, %d violations (modes %s)" % [fac_n, fac_bad, str(modes)])
	_check(int(modes.get("guarded", 0)) + int(modes.get("clamped", 0)) + int(modes.get("original", 0)) > 0, "positive control: the faction guard fires on the Look")


# ===================================================================== F ====

func _cards() -> Array:
	var out: Array = []
	var list: Node = panel.get_node("AdjList")
	for c in list.get_children():
		if String(c.name).begins_with("AdjEntry_") and not c.is_queued_for_deletion():
			out.append(c)
	return out


func _find(n: Node, nm: String) -> Node:
	return n.find_child(nm, true, false)


func _leg_f() -> void:
	_log("--- F: the adjustment panel")
	await _pick_group("terrain")
	var lv := JSON.stringify({"entries": [{"kind": {"levels": {"in_black": 0.0, "in_white": 255.0, "gamma": 1.3, "out_black": 0.0, "out_white": 255.0}}, "enabled": true, "opacity": 1.0}]})
	bridge.adjust_set("terrain", lv)
	rw.on_adjust_restored()
	await _look("saturation", [-50.0])
	panel.select_layer("terrain")
	await _frames(3)
	var cards := _cards()
	_check(cards.size() == 2, "the panel lists 2 rows for [Look, Levels] -- the Look once (%d)" % cards.size())
	if cards.size() == 2:
		_check(bool(cards[0].get_meta("adj_look", false)) and not bool(cards[1].get_meta("adj_look", false)), "row 1 is tagged Look, row 2 is not")
		var down0 := _find(panel, "AdjDown_0") as Button
		var up1 := _find(panel, "AdjUp_1") as Button
		_check(down0 != null and down0.disabled and up1 != null and up1.disabled, "pinned: Look's Down and row 2's Up are disabled")
		# A panel edit of the Look entry moves the Look slider.
		var ps: HSlider = null
		for s in cards[0].find_children("*", "HSlider", true, false):
			if String(s.get_meta("adj_param", "")) == "saturation":
				ps = s
		_check(ps != null and is_equal_approx(ps.value, -0.5), "the panel shows the Look's saturation -0.5")
		if ps != null:
			ps.drag_started.emit()
			ps.value = -0.25
			await get_tree().create_timer(0.1).timeout
			ps.drag_ended.emit(true)
			await _frames(3)
			_check(_slider("saturation").value == -25.0, "a panel edit moves the Look slider to -25 (%.1f)" % _slider("saturation").value)
		var del := _find(panel, "AdjDel_0") as Button
		if del != null:
			del.pressed.emit()
			await _frames(4)
		_check(_slider("saturation").value == 0.0, "deleting the Look row in the panel returns the slider to 0")
		var te := _entries("terrain")
		_check(te.size() == 1 and (te[0]["kind"] as Dictionary).has("levels"), "... and leaves the user's Levels")
	bridge.adjust_set("terrain", "{\"entries\":[]}")
	rw.on_adjust_restored()
	await _pick_group("all")


# ===================================================================== G ====

func _leg_g() -> void:
	_log("--- G: locked while generating")
	bridge.generate({"seed": _seed + 1, "width_km": 1200.0, "grid_w": _grid.x, "grid_h": _grid.y,
		"archetype": "", "villages": true, "sea_level": 0.42})
	await _frames(2)
	_check(bridge.generating, "a generate is running")
	_check(not _slider("saturation").editable and rw._quick_pick.disabled, "sliders and picker locked")
	var ov0: String = OA.get_stack_json()
	rw._on_quick_slider("saturation", -30.0)
	rw._quick_end()
	_check(OA.get_stack_json() == ov0, "a forced write during the generate changes nothing (overlay)")
	var spins := 0
	while bridge.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await _frames(6)
	_check(_slider("saturation").editable and not rw._quick_pick.disabled, "unlocked after the generate")


# ===================================================================== H ====

func _leg_h() -> void:
	if not DccTheme.is_touch():
		_log("--- H: skipped (pointer density)")
		return
	_log("--- H: touch floor")
	var low := 0
	for k in rw._quick_rows:
		var h: Dictionary = rw._quick_rows[k]
		if (h["row"] as Control).size.y < 44.0 or (h["slider"] as Control).size.y < 44.0:
			low += 1
			_log("  %s row %.1f slider %.1f" % [k, (h["row"] as Control).size.y, (h["slider"] as Control).size.y])
	_check(low == 0, "every Look slider row and track is >= 44 px tall (touch)")
