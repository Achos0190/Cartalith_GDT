extends Node
## **Ruling BR: the paint "Original" overlay shows the armed layer's
## as-generated raster at a slider opacity, and is a viewing aid only.**
##
## Driven through the real shell (`app.tscn`) and the real Paint tool bar, in a
## SubViewport so `--vp 1080x2340 --force-touch` is a real phone composition.
##
## Protects, in order:
##  1. the slider exists only while the Paint bar is armed (and not on Splat,
##     where a one-line reason stands in), and the layer is hidden and holds no
##     texture when disarmed;
##  2. setting the slider to 0, 0.5 and 1 changes the pixels actually drawn over
##     the map, monotonically, and returning to 0 restores them EXACTLY
##     (negative control: idle capture-to-capture difference is 0);
##  3. none of it touches world data: a fingerprint of the height, biome and
##     terrain rasters, the paint counts, the draft count and `world_dirty` is
##     identical before and after every slider move;
##  4. the original really EXCLUDES hand paint: after a committed dab the map
##     pixel under it moves (positive control) while the original texture stays
##     byte-identical to the engine's own `bclass` field;
##  5. switching to Terrain swaps the original to `cterrain`;
##  6. no per-frame cost: over an idle window the layer never redraws, its
##     texture is not rebuilt, and draw calls and process time do not rise
##     against the same window with the aid at 0.
##
## Pixel readback, so WINDOWED -- never `--headless` (`ImageTexture` readback is
## a no-op there and every check would fail identically):
##   godot --path . _paintorig_probe.tscn -- --tag desktop --shotdir DIR
##   godot --path . _paintorig_probe.tscn -- --force-touch --vp 1080x2340 --tag phone --shotdir DIR
##
## No palette is assumed: every pixel assertion compares rasters against each
## other or against themselves, never against a colour written for one theme.

var app: Node
var bridge: EngineBridge
var _vp: SubViewport
var _tag := "porig"
var _shotdir := ""
var _fail := 0

const SEED := 4242
const GRID_W := 256
const GRID_H := 164
const SETTLE := 8        ## frames to let the shell lay out and redraw after a change
const IDLE_FRAMES := 120 ## the "idle" window for the cost measurement

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])

func _check(ok: bool, what: String) -> void:
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
	var known := ["--force-touch", "--vp", "--tag", "--shotdir"]
	var args := OS.get_cmdline_user_args()
	for i in args.size():
		var s := String(args[i])
		if s.begins_with("--") and not (s in known):
			_log("ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			return false
	return true

func _finish() -> void:
	_log("RESULT %s fail=%d" % [_tag, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

## True only when `ImageTexture` readback reaches the renderer (not `--headless`).
func _guard() -> bool:
	var img := Image.create_empty(2, 2, false, Image.FORMAT_RGBA8)
	img.fill(Color(0, 0, 0, 0))
	var t := ImageTexture.create_from_image(img)
	img.set_pixel(1, 1, Color(1, 0, 0, 1))
	t.update(img)
	return t.get_image().get_pixel(1, 1).a > 0.0

# -- Measurement helpers -------------------------------------------------------

## The viewport's drawn pixels over the map host, as a small RGBA image.
func _map_shot() -> Image:
	var full := _vp.get_texture().get_image()
	var r := Rect2i(app.viewport.get_global_rect())
	r = r.intersection(Rect2i(Vector2i.ZERO, full.get_size()))
	var img := full.get_region(r)
	img.convert(Image.FORMAT_RGBA8)
	return img

## Mean per-channel absolute difference of two images, after a common shrink to
## 96x96 so the loop stays cheap. 0.0 iff they are indistinguishable at that size.
func _mad(a: Image, b: Image) -> float:
	var x := a.duplicate() as Image
	var y := b.duplicate() as Image
	x.resize(96, 96, Image.INTERPOLATE_BILINEAR)
	y.resize(96, 96, Image.INTERPOLATE_BILINEAR)
	var da := x.get_data()
	var db := y.get_data()
	var tot := 0
	for i in da.size():
		tot += absi(int(da[i]) - int(db[i]))
	return float(tot) / float(da.size())

func _save_shot(name: String) -> void:
	if _shotdir == "":
		return
	var out := "%s/%s_%s.png" % [_shotdir, _tag, name]
	_vp.get_texture().get_image().save_png(out)
	_log("  shot -> %s" % out)

## A fingerprint of everything Ruling BR must not touch: the engine's own
## unpainted height, biome and terrain rasters, the paint counts, the pending
## draft, the grid and the dirty flag.
func _fingerprint() -> String:
	var parts: Array = []
	for v in ["elevation", "bclass", "cterrain"]:
		var t := bridge.debug_texture(v)
		parts.append(str(hash(t.get_image().get_data())) if t != null else "none")
	parts.append(str(bridge.paint_painted_counts()))
	parts.append(str(bridge.paint_draft_count()))
	parts.append(str(bridge.grid_size()))
	parts.append(str(bridge.world_dirty))
	return "|".join(parts)

## Every HSlider under `n`, depth first.
func _sliders(n: Node, out: Array) -> void:
	if n is HSlider and (n as Control).is_visible_in_tree():
		out.append(n)
	for c in n.get_children():
		_sliders(c, out)

## The visible slider whose row label reads `label`, or null.
func _slider_labelled(label: String) -> HSlider:
	var all: Array = []
	_sliders(app, all)
	for s in all:
		var row := (s as Node).get_parent()
		if row != null and row.get_child_count() > 0:
			var first := row.get_child(0)
			if first is Label and (first as Label).text == label:
				return s
	return null

## True when any Label (visible or not) reads exactly `text` -- a diagnostic.
func _find_label_anywhere(n: Node, text: String) -> bool:
	if n is Label and (n as Label).text == text:
		return true
	for c in n.get_children():
		if _find_label_anywhere(c, text):
			return true
	return false

## Any visible Label whose text starts with `prefix`.
func _label_starting(n: Node, prefix: String) -> Label:
	if n is Label and (n as Label).is_visible_in_tree() and (n as Label).text.begins_with(prefix):
		return n
	for c in n.get_children():
		var hit := _label_starting(c, prefix)
		if hit != null:
			return hit
	return null

func _orig_tex_bytes() -> PackedByteArray:
	var t: Texture2D = app.viewport.paint_original_node().texture
	return t.get_image().get_data() if t != null else PackedByteArray()

func _debug_bytes(view: String) -> PackedByteArray:
	var t := bridge.debug_texture(view)
	return t.get_image().get_data() if t != null else PackedByteArray()

func _set_pct(s: HSlider, pct: float) -> void:
	s.value = pct   ## fires value_changed exactly as a drag would
	await _frames(SETTLE)

# -- Main ----------------------------------------------------------------------

func _ready() -> void:
	_tag = _arg("--tag", "porig")
	_shotdir = _arg("--shotdir", "")
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
	if not _guard():
		_log("ABORT run WINDOWED -- ImageTexture readback is a no-op under --headless")
		get_tree().quit(2)
		return
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
	if parts.size() != 2:
		_log("ABORT --vp wants WxH")
		get_tree().quit(2)
		return
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
	var phone: bool = app.is_phone()
	_log("viewport %dx%d  phone=%s" % [_vp.size.x, _vp.size.y, phone])
	_log("DLL %s" % ProjectSettings.globalize_path("res://").path_join("../target/debug/cartalith_godot.dll"))

	bridge.generate({"seed": SEED, "width_km": 800.0, "grid_w": GRID_W, "grid_h": GRID_H,
		"sea_level": 0.5, "villages": true})
	await bridge.generation_finished
	await _frames(20)
	_check(bridge.has_world, "a world landed")
	var po = app.paint_original
	_check(po != null, "the shell holds a paint-original helper")
	if not bridge.has_world or po == null:
		_finish()
		return

	# 1. Disarmed: no control, no layer, no texture --------------------------------
	_log("1. disarmed")
	_check(app.armed_tool == "inspect", "starts on Inspect")
	_check(_slider_labelled("Original") == null, "no Original slider while no paint tool is armed")
	var st0: Dictionary = app.viewport.paint_original_state()
	_check(not st0["visible"] and not st0["has_texture"], "the layer is hidden and holds no texture")

	# 2. Arm Paint on Biome ---------------------------------------------------------
	## The phone draws the tool options only inside the WORLD domain's tool sheet.
	app.select_domain("world")
	await _frames(SETTLE)
	if phone:
		## The phone's default GENERATE tab swaps the tool-options row for the
		## generation column (`_refresh_phone_gen_panel`), so the Paint bar is
		## reachable only from another tab; MAP is the one that shows it.
		app._pick_phone_tab("map")
		await _frames(SETTLE)
	app.tool_bar._select_mode("paint")
	app.tool_bar._on_paint_layer("biome")
	await _frames(SETTLE)
	if phone and app._phone_tab == "gen":
		## Arming Paint selects the WORLD domain, which lights GENERATE and swaps
		## the options row for the generation column, so THIS bar's own row is
		## hidden on a phone. A thumb now reaches Paint's options through the
		## GENERATE sheet's PAINT segment instead, and `_phonepaint_probe.gd` proves
		## that with real taps. What this leg still measures is the BAR'S OWN row
		## (its Original slider's touch size, the shared slider state), so it
		## REVEALS the already-built, hidden row by writing the tab and refreshing
		## the sheet -- a test-only path, not the user's route.
		app._phone_tab = "map"
		app._refresh_phone_gen_panel()
		await _frames(SETTLE)
		_log("FORCED REVEAL of the bar row (the user route is _phonepaint_probe.gd): tab=%s armed=%s bar mode=%s" % [app._phone_tab, app.armed_tool, app.tool_bar.mode])
	_check(app.armed_tool == "paint", "Paint is armed")
	var sl := _slider_labelled("Original")
	if sl == null:
		## Diagnostic for a miss: where is the options row, and is the control in it at all?
		var tor: Control = app.tool_options_row
		var chain := ""
		var n: Node = tor
		while n != null and n != app:
			chain += "%s(vis=%s) < " % [n.name, (n as CanvasItem).visible if n is CanvasItem else "-"]
			n = n.get_parent()
		_log("  DIAG phone tab = %s  active domain = %s" % [app._phone_tab, app.get("active_domain") if app.get("active_domain") != null else "?"])
		_log("  DIAG tool_options_row chain: %s  size=%s  children=%d" % [chain, tor.size, tor.get_child_count()])
		var found_any := _find_label_anywhere(app, "Original")
		_log("  DIAG a label reading 'Original' exists anywhere in the tree: %s" % found_any)
	_check(sl != null, "the Original slider is drawn on the Paint bar (Biome)")
	if sl == null:
		_finish()
		return
	_check(is_equal_approx(sl.value, 0.0), "it starts at 0 (off)")
	var size_sl := _slider_labelled("Size")
	_check(size_sl != null, "the Size slider beside it is still drawn (control)")
	var fp0 := _fingerprint()
	var base_dirty: bool = bridge.world_dirty

	# 3. Pixels at 0 / 0.5 / 1 ------------------------------------------------------
	_log("2. slider 0 / 0.5 / 1")
	_save_shot("a_slider0")
	var s0 := _map_shot()
	var s0b := _map_shot()
	_check(_mad(s0, s0b) == 0.0, "negative control: two idle captures at 0 are identical")
	await _set_pct(sl, 50.0)
	_check(is_equal_approx(app.viewport.paint_original_state()["alpha"], 0.5), "alpha is 0.5 at 50 %")
	_check(app.viewport.paint_original_state()["visible"], "the layer is visible at 50 %")
	_save_shot("b_slider50")
	var s50 := _map_shot()
	await _set_pct(sl, 100.0)
	_check(is_equal_approx(app.viewport.paint_original_state()["alpha"], 1.0), "alpha is 1.0 at 100 %")
	_save_shot("c_slider100")
	var s100 := _map_shot()
	var d050 := _mad(s0, s50)
	var d5010 := _mad(s50, s100)
	var d0100 := _mad(s0, s100)
	_log("mean abs diff: 0->50 = %.3f   50->100 = %.3f   0->100 = %.3f" % [d050, d5010, d0100])
	_check(d050 > 0.0, "the drawn map changes between 0 and 0.5")
	_check(d5010 > 0.0, "the drawn map changes between 0.5 and 1")
	_check(d0100 > d050, "0->1 differs more than 0->0.5 (opacity is monotonic)")
	_check(_fingerprint() == fp0, "world fingerprint unchanged after moving the slider")
	_check(bridge.world_dirty == base_dirty, "world_dirty unchanged (nothing to save)")
	await _set_pct(sl, 0.0)
	_check(not app.viewport.paint_original_state()["visible"], "the layer hides again at 0")
	_check(not app.viewport.paint_original_state()["has_texture"], "and releases its texture at 0")
	_check(_mad(s0, _map_shot()) == 0.0, "returning to 0 restores the original pixels exactly")

	# 4. The original excludes hand paint -------------------------------------------
	_log("3. original excludes paint")
	await _set_pct(sl, 100.0)
	var frozen := _orig_tex_bytes()
	_check(frozen == _debug_bytes("bclass") and frozen.size() > 0,
		"at 1.0 the layer's texture is the engine's own bclass raster")
	var g: Vector2i = bridge.grid_size()
	## A LAND cell, found by its class in the frozen bclass raster (an input read
	## before any edit): outward from the centre, the first cell whose colour is
	## one of the 13 paintable land classes' own. Ocean cells would not show the
	## 0.60 paint blend, so the positive control needs land.
	var orig_img := bridge.debug_texture("bclass").get_image()
	var cell := Vector2i(-1, -1)
	var land_cols: Array = []
	for k in range(1, 14):
		land_cols.append(bridge.biome_color(k).to_html(false))
	for r in range(0, mini(g.x, g.y) / 2):
		for dy in range(-r, r + 1):
			for dx in range(-r, r + 1):
				if cell.x >= 0:
					break
				var p := Vector2i(g.x / 2 + dx, g.y / 2 + dy)
				if p.x < 4 or p.y < 4 or p.x >= g.x - 4 or p.y >= g.y - 4:
					continue
				if orig_img.get_pixelv(p).to_html(false) in land_cols:
					cell = p
		if cell.x >= 0:
			break
	_check(cell.x >= 0, "found a land cell in the frozen bclass raster")
	var m0 := bridge.color_texture().get_image().get_pixelv(cell)
	bridge.paint_set_layer("biome")
	## A class the cell is not already: pick by reading it, never by assuming.
	var cc := orig_img.get_pixelv(cell)
	var pal := bridge.get_paint_palette("biome")
	## A class whose own colour differs from the one already under the cell, so
	## the 0.60 blend must move the pixel: chosen by reading, never by assuming.
	var chosen := -1
	for pd in pal:
		var idx := int((pd as Dictionary).get("index", -1))
		if chosen < 0 and idx > 0 and bridge.biome_color(idx).to_html(false) != cc.to_html(false):
			chosen = idx
	_check(chosen > 0, "found a paintable class that differs from the cell's own")
	bridge.paint_set_brush(chosen, 3.0, 1.0, 0.0, false, false)
	bridge.paint_stroke_at(float(cell.x), float(cell.y))
	app.tool_bar._on_paint_commit()
	await _frames(SETTLE)
	var m1 := bridge.color_texture().get_image().get_pixelv(cell)
	_check(m0 != m1, "positive control: the committed dab moved the map pixel under it")
	_check(_orig_tex_bytes() == frozen,
		"the original texture is byte-identical to before the dab (it excludes hand paint)")
	_check(_orig_tex_bytes() == _debug_bytes("bclass"), "and still equals the engine's bclass raster")
	_check(_slider_labelled("Original") != null, "the slider survived the commit's bar rebuild")
	var sl2 := _slider_labelled("Original")
	_check(sl2 != null and is_equal_approx(sl2.value, 100.0), "and kept its value (session state)")

	# 5. Terrain layer ----------------------------------------------------------------
	_log("4. terrain layer")
	app.tool_bar._on_paint_layer("terrain")
	await _frames(SETTLE)
	var bt := _orig_tex_bytes()
	_check(bt == _debug_bytes("cterrain") and bt.size() > 0, "Terrain swaps the original to cterrain")
	_check(bt != frozen, "and it is a different raster than Biome's")
	_save_shot("d_terrain100")

	# 6. Splat: no original --------------------------------------------------------------
	_log("5. splat layer")
	app.tool_bar._on_paint_layer("splat")
	await _frames(SETTLE)
	var st_s: Dictionary = app.viewport.paint_original_state()
	_check(_slider_labelled("Original") == null, "no Original slider on Splat")
	_check(_label_starting(app, "no original") != null, "a one-line reason stands in")
	_check(not st_s["visible"] and not st_s["has_texture"], "the layer is hidden and holds no texture on Splat")
	var hint := _label_starting(app, "no original")
	_log("hint reads: %s" % (hint.text if hint != null else "(none)"))
	_check(hint != null and hint.size.x >= 100.0,
		"the hint is actually drawn with room to read (width %.0f px)" % (hint.size.x if hint != null else -1.0))
	_save_shot("e_splat_hint")

	# 7. Back to Biome, idle cost with the aid ON vs OFF --------------------------------
	## Alternating rounds (@50, @0, @50, @0 ...) after a settle, so a slow drift in
	## the machine, or the transient right after a texture change, is not blamed on
	## the aid. Each window also counts redraws of the layer and whether its
	## texture object changed.
	app.tool_bar._on_paint_layer("biome")
	await _frames(SETTLE)
	var sl3 := _slider_labelled("Original")
	await _frames(60)
	var on: Array = []
	var off: Array = []
	for round in 3:
		await _set_pct(sl3, 50.0)
		on.append(await _idle_cost())
		await _set_pct(sl3, 0.0)
		off.append(await _idle_cost())
	await _set_pct(sl3, 50.0)
	var on_ms: Array = on.map(func(d): return snappedf(d["proc_ms"], 0.001))
	var off_ms: Array = off.map(func(d): return snappedf(d["proc_ms"], 0.001))
	_log("median process ms/frame, 3 alternating rounds: @50 = %s   @0 = %s" % [str(on_ms), str(off_ms)])
	_log("draw calls: @50 = %s   @0 = %s" % [str(on.map(func(d): return d["draw_calls"])), str(off.map(func(d): return d["draw_calls"]))])
	var redraws_total := 0
	var tex_changed := false
	for d in on:
		redraws_total += int(d["redraws"])
		tex_changed = tex_changed or bool(d["tex_changed"])
	_log("layer redraws over %d idle frames x3 windows = %d" % [IDLE_FRAMES, redraws_total])
	_check(redraws_total == 0, "no per-frame redraw of the layer while idle")
	_check(not tex_changed, "the texture is not rebuilt while idle")
	var dc_on: int = on[on.size() - 1]["draw_calls"]
	var dc_off: int = off[off.size() - 1]["draw_calls"]
	_check(dc_on <= dc_off + 2, "draw calls %d at 50 vs %d at 0 (the aid costs at most the one extra layer)" % [dc_on, dc_off])
	var med_on: float = on_ms[1]
	var med_off: float = off_ms[1]
	_check(med_on <= med_off * 1.25 + 0.5, "median frame time with the aid on (%.2f ms) is within 25 %% of off (%.2f ms)" % [med_on, med_off])

	# 8. Layout: width and touch sizing ---------------------------------------------------------
	_log("6. layout")
	var sl4 := _slider_labelled("Original")
	var row: Control = sl4.get_parent()
	var win_scale := _vp.size.x / maxf(1.0, float(app.size.x))
	_log("Original row min=%s  slider size=%s" % [row.get_combined_minimum_size(), sl4.size])
	var size_s := _slider_labelled("Size")
	if size_s != null:
		_log("Size slider size=%s  Original slider size=%s" % [size_s.size, sl4.size])
		_check(sl4.size.y >= size_s.size.y - 0.01, "Original is at least as tall as the Size slider (same fit pass)")
	var bar: Control = app.tool_options_row
	var min_with: float = bar.get_combined_minimum_size().x
	row.visible = false
	await _frames(3)
	var min_without: float = bar.get_combined_minimum_size().x
	row.visible = true
	await _frames(3)
	_log("tool options row: width now = %.0f   min with Original = %.0f   without = %.0f   (delta %.0f)" % [
		bar.size.x, min_with, min_without, min_with - min_without])
	var bar_col := bar.get_child(0) as Control
	if bar_col != null and bar_col.get_child_count() > 0:
		for ch in bar_col.get_children():
			if ch is HBoxContainer:
				_log("  bar row '%s' min width = %.0f (visible=%s)" % [ch.get_child(0).get_class() if ch.get_child_count() > 0 else "-", (ch as Control).get_combined_minimum_size().x, ch.visible])
	_log("  clips? with Original: %s   without: %s" % [min_with > bar.size.x + 0.5, min_without > bar.size.x + 0.5])
	_check(phone or min_with <= bar.size.x + 0.5 or min_without > bar.size.x + 0.5,
		"desktop: the control does not turn a fitting bar into a clipping one")
	if phone:
		_check(sl4.size.y * win_scale >= 1.0, "phone: the slider has a drawn height (%.1f px)" % (sl4.size.y * win_scale))
	_save_shot("f_biome_armed")

	# 9. Disarm ------------------------------------------------------------------------------------
	_log("7. disarm")
	app.arm_tool("inspect")
	await _frames(SETTLE)
	_check(_slider_labelled("Original") == null, "the slider is gone after disarm")
	var st_d: Dictionary = app.viewport.paint_original_state()
	_check(not st_d["visible"] and not st_d["has_texture"], "the layer is hidden and holds no texture after disarm")
	_check(is_equal_approx(po.opacity, 0.5), "the slider value is remembered for the next arm (session state)")
	_check(_fingerprint().find("none") == -1, "(sanity) the fingerprint inputs all answered")

	_finish()

## Idle cost over `IDLE_FRAMES` frames: the MEDIAN per-frame process time, the
## last frame's draw calls, how many times the paint-original layer redrew, and
## whether its texture object changed. Nothing is changed during the window.
## Median, not mean: this shell's debug build has frame-time outliers of tens of
## ms right after a texture change, which a mean pins on whatever changed last.
func _idle_cost() -> Dictionary:
	await _frames(10)
	var node: TextureRect = app.viewport.paint_original_node()
	var tex_before: Texture2D = node.texture
	var redraws := [0]
	var cb := func(): redraws[0] += 1
	node.draw.connect(cb)
	var ts: Array = []
	var dc := 0
	for i in IDLE_FRAMES:
		await get_tree().process_frame
		ts.append(Performance.get_monitor(Performance.TIME_PROCESS))
		dc = int(Performance.get_monitor(Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME))
	node.draw.disconnect(cb)
	ts.sort()
	return {"proc_ms": float(ts[ts.size() / 2]) * 1000.0, "draw_calls": dc,
		"redraws": redraws[0], "tex_changed": node.texture != tex_before}
