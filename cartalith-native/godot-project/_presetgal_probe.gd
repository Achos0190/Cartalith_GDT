extends Node
## Committed probe for the Map-style **preset gallery**
## (`shell/workspaces/render_workspace.gd`'s `_build_map_style` /
## `_preset_tile` / `_set_tile_on`).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _presetgal_probe.tscn -- --vp 1920x1080
##   Godot_v4.7.1-stable_win64_console.exe --path . _presetgal_probe.tscn -- --vp 2560x1600 --force-touch
##   Godot_v4.7.1-stable_win64_console.exe --path . _presetgal_probe.tscn -- --vp 1920x1080 --cost
##
## **Run WINDOWED.** Nothing here reads a framebuffer, but §4 measures drawn
## control sizes against the touch floor and a dummy driver lays out a window
## that was never shown.
##
## **Two flags, both real, both read below.** `--vp WxH` is required rather
## than defaulted, per `MISTAKES.md`'s "name the density beside every
## measurement" row: without it the same box gets reported as two densities.
## `--force-touch` is `DccShell`'s own override, not this probe's, and is what
## puts a 2560x1600 window on the tablet column. `--cost` adds §1, which
## generates three worlds and is slow (~3 min); it is off by default so the
## behavioural sections can be re-run cheaply. Anything else is refused rather
## than ignored.
##
## What it proves, in order:
##   0. driver, viewport, density -- printed beside every number below
##   1. `--cost` only. **The deciding measurement behind the "no thumbnail"
##      decision**: the median wall time of one `build_color_texture()` on a
##      real world, and of a whole apply-render-restore cycle, at three grid
##      sizes. Route "cache a render per preset" is that figure times the
##      preset count, re-paid on every world change.
##   2. the gallery draws one tile per `STYLE_PRESETS` entry, each carrying the
##      preset's own name and a bundle line derived from its own overrides --
##      and the chip row it replaced is gone rather than doubled
##   3. a tile applies its preset: look, Painter block, viewport readout, the
##      Custom note and the lit tile all follow; a manual edit afterwards
##      unlights every tile; and the next preset is lossless
##   4. every tile clears the touch floor, covers its own card, and the gallery
##      fits the dock -- at the density this run measured

var app: Node
var bridge
var rw: Node
var _fail := 0

const KNOWN_FLAGS := ["--vp", "--force-touch", "--cost"]


func _p(s: String) -> void:
	print("PRESETGAL  %s" % s)


func _ok(cond: bool, s: String) -> void:
	if cond:
		print("PRESETGAL  ok    %s" % s)
	else:
		_fail += 1
		print("PRESETGAL  FAIL  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


## Depth-first search for the node whose script is `script_file`.
func _find(n: Node, script_file: String) -> Node:
	if n.get_script() != null and String(n.get_script().resource_path).ends_with(script_file):
		return n
	for c in n.get_children(true):
		var r := _find(c, script_file)
		if r != null:
			return r
	return null


## `-- --vp WxH`, one token or two. Refuses rather than defaults -- see header.
func _parse_vp() -> Vector2i:
	var args := OS.get_cmdline_user_args()
	## Loud on an argument this probe does not read, which is the failure
	## `MISTAKES.md`'s probe-header row describes: a flag that is documented and
	## silently ignored measures the same box twice and calls it two runs.
	for i in args.size():
		var a := String(args[i])
		if not a.begins_with("--"):
			continue
		var known := false
		for f in KNOWN_FLAGS:
			if a == f or a.begins_with(f + "="):
				known = true
		if not known:
			print("PRESETGAL  UNKNOWN ARGUMENT %s -- known: %s" % [a, str(KNOWN_FLAGS)])
			return Vector2i.ZERO
	for i in args.size():
		if not String(args[i]).begins_with("--vp"):
			continue
		var v := String(args[i]).substr(4).lstrip("= ")
		if v == "" and i + 1 < args.size():
			v = String(args[i + 1])
		var wh: PackedStringArray = v.split("x")
		if wh.size() == 2 and wh[0].is_valid_int() and wh[1].is_valid_int():
			return Vector2i(int(wh[0]), int(wh[1]))
		print("PRESETGAL  BAD --vp argument: %s (want WxH)" % v)
		return Vector2i.ZERO
	print("PRESETGAL  NO --vp WxH given; this probe will not guess a density.")
	return Vector2i.ZERO


func _generate(seed_v: int, gw: int, gh: int, km: float) -> void:
	bridge.generate({
		"seed": seed_v, "width_km": km, "grid_w": gw, "grid_h": gh,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout


## Median with min..max, the shape `MISTAKES.md` requires of any quoted timing.
func _stat(ms: Array) -> String:
	var s := ms.duplicate()
	s.sort()
	return "%.1f ms (%.1f..%.1f, n=%d)" % [
		float(s[s.size() / 2]), float(s[0]), float(s[s.size() - 1]), s.size()]


## One `build_color_texture()`, timed. Nothing is drawn with it -- the cost
## under test is the renderer's, not a texture upload's.
func _time_render() -> float:
	var t0 := Time.get_ticks_usec()
	var tex: Texture2D = bridge.color_texture()
	var t1 := Time.get_ticks_usec()
	if tex == null:
		_p("  !! color_texture() returned null -- the world is not generated")
	return float(t1 - t0) / 1000.0


## The whole route-1 cycle for ONE tile: select the preset's look, push its
## Painter bundle, render, then put both back. This is what a cached per-preset
## thumbnail actually costs, measured as the cycle rather than as the render
## alone, because a gallery cannot leave the last preset applied.
func _time_cycle(entry: Array, look_back: String, npr_back: Dictionary) -> float:
	var values: Dictionary = RenderWorkspace.STYLE_MANAGED.duplicate()
	for k in Dictionary(entry[2]):
		values[k] = Dictionary(entry[2])[k]
	var t0 := Time.get_ticks_usec()
	bridge.set_look(String(entry[1]))
	bridge.set_npr(values)
	var _tex: Texture2D = bridge.color_texture()
	bridge.set_look(look_back)
	bridge.set_npr(npr_back)
	var t1 := Time.get_ticks_usec()
	return float(t1 - t0) / 1000.0


## The viewport chrome's style readout. `ViewportHost._style_readout` is a
## plain `String`, so the pushed value alone would prove only that a setter
## ran; `_drawn_readout()` beside it reads the label that is actually on the
## map, which is what "the readout names the preset" claims.
func _readout() -> String:
	return String(app.viewport.get("_style_readout"))


func _drawn_readout() -> String:
	var lbl = app.viewport.get("_readout_label")
	return String(lbl.text) if lbl != null else "<no label>"


## Which tiles are lit, by the one thing `_set_tile_on()` changes that cannot
## be confused with anything else: the name label's ink.
func _lit(tiles: Array) -> Array:
	var out: Array = []
	for i in tiles.size():
		if (tiles[i]["name"] as Label).get_theme_color("font_color") == DccTheme.c("accent"):
			out.append(i)
	return out


## How many tiles share the first row's Y -- the gallery's real column count at
## whatever width the dock gave it, rather than the count it was designed for.
func _columns(tiles: Array) -> int:
	if tiles.is_empty():
		return 0
	var y0: float = (tiles[0]["card"] as Control).position.y
	var n := 0
	for t in tiles:
		if absf((t["card"] as Control).position.y - y0) < 1.0:
			n += 1
	return n


func _cost_pass() -> void:
	_p("=== §1 what a per-preset thumbnail costs ===")
	_p("STYLE_PRESETS entries = %d" % RenderWorkspace.STYLE_PRESETS.size())
	for grid in [[384, 288, 2400.0], [1024, 656, 2400.0], [2048, 1311, 4000.0]]:
		await _generate(483920, int(grid[0]), int(grid[1]), float(grid[2]))
		await _frames(6)
		var look_back: String = bridge.look()
		var npr_back: Dictionary = {}
		for k in RenderWorkspace.STYLE_MANAGED:
			npr_back[k] = RenderWorkspace.STYLE_MANAGED[k]
		## Warm once -- the first call after a generate pays for caches the five
		## after it do not, and quoting that would be quoting the wrong number
		## for a gallery that renders six back to back.
		var warm := _time_render()
		var plain: Array = []
		for i in 5:
			plain.append(_time_render())
		var cycles: Array = []
		for e in RenderWorkspace.STYLE_PRESETS:
			cycles.append(_time_cycle(e, look_back, npr_back))
		var total := 0.0
		for c in cycles:
			total += float(c)
		_p("grid %dx%d (%d cells)" % [int(grid[0]), int(grid[1]), int(grid[0]) * int(grid[1])])
		_p("   one build_color_texture()      %s   (warm-up %.1f ms)" % [_stat(plain), warm])
		_p("   one apply+render+restore cycle %s" % _stat(cycles))
		_p("   WHOLE GALLERY (%d tiles)        %.1f ms" % [cycles.size(), total])


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 1200.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG TIMEOUT"); get_tree().quit(3))
	wd.start()

	var want := _parse_vp()
	if want == Vector2i.ZERO:
		get_tree().quit(2)
		return
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	bridge = app.bridge

	# ============================================================ §0 conditions
	var vp := get_viewport().get_visible_rect().size
	_p("=== §0 conditions ===")
	_p("driver=%s  viewport=%dx%d  is_tablet=%s  is_phone=%s  is_laptop=%s" % [
		DisplayServer.get_name(), int(vp.x), int(vp.y),
		str(DccTheme.is_tablet()), str(DccTheme.is_phone()), str(DccTheme.is_laptop())])
	if DisplayServer.get_name() == "headless":
		_ok(false, "run WINDOWED -- see this probe's header")

	# ================================================= §1 the thumbnail's cost
	if "--cost" in OS.get_cmdline_user_args():
		await _cost_pass()
	else:
		_p("=== §1 skipped (no --cost) ===")

	await _generate(483920, 384, 288, 2400.0)
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(8)

	# ================================================= §2 the gallery's content
	_p("=== §2 the gallery ===")
	app.select_domain_category("cartography", "Map style")
	await _frames(8)
	rw = _find(app, "render_workspace.gd")
	_ok(rw != null, "the RenderWorkspace node is reachable")
	var tiles: Array = rw.get("_preset_tiles")
	_p("tiles=%d  STYLE_PRESETS=%d" % [tiles.size(), RenderWorkspace.STYLE_PRESETS.size()])
	_ok(tiles.size() == RenderWorkspace.STYLE_PRESETS.size(),
		"one tile per STYLE_PRESETS entry")

	## The chip row the gallery replaced is gone, not doubled: two selectors for
	## one setting is the defect this replaces, so its absence is asserted
	## rather than assumed. A surviving `segment()` chip would be a Button whose
	## own `text` is a preset name; a tile's text lives in child Labels.
	var names: Array = []
	for e in RenderWorkspace.STYLE_PRESETS:
		names.append(String(e[0]))
	var stray := 0
	var found: Array = []
	_walk(rw, found)
	for n in found:
		if n is Button and not (n is CheckBox) and String((n as Button).text) in names:
			stray += 1
	_p("Buttons whose own text is a preset name: %d" % stray)
	_ok(stray == 0, "no leftover chip row -- the tiles are the only selector")

	for i in tiles.size():
		var t: Dictionary = tiles[i]
		var nm := String((t["name"] as Label).text)
		var lk := String((t["look"] as Label).text)
		var bd := String((t["bundle"] as Label).text)
		_p("  tile %d  name=%s | look=%s | bundle=%s" % [i, nm, lk, bd])
		_ok(nm == String(RenderWorkspace.STYLE_PRESETS[i][0]),
			"tile %d carries its own preset name" % i)
		_ok(bd.strip_edges() != "", "tile %d states its bundle" % i)
		## Derived, not authored: every managed key the preset actually turns on
		## has to appear in the line, so a tile and its bundle cannot drift.
		var over: Dictionary = RenderWorkspace.STYLE_PRESETS[i][2]
		for key in over:
			var wanted := ""
			for st in RenderWorkspace.STYLES:
				if String(st[0]) == String(key):
					wanted = String(st[1])
			if wanted == "" and RenderWorkspace.MANAGED_LABEL.has(key):
				wanted = String(RenderWorkspace.MANAGED_LABEL[key])
			var live := (typeof(over[key]) != TYPE_BOOL) or bool(over[key])
			if wanted != "" and live:
				_ok(bd.contains(wanted),
					"tile %d names its %s override" % [i, String(key)])
		if over.is_empty():
			_ok(bd.contains("No Painter styles"),
				"tile %d says what an empty bundle produces, rather than nothing" % i)

	## The look line is a claim about this cdylib, so it is checked against this
	## cdylib: a look the engine publishes must be drawn plain, and only a look
	## it does not have may carry the disclosure.
	var engine_looks: Array = bridge.looks()
	_p("engine looks (%d): %s" % [engine_looks.size(), str(engine_looks)])
	for i in tiles.size():
		var want_look := String(RenderWorkspace.STYLE_PRESETS[i][1])
		var lk := String((tiles[i]["look"] as Label).text)
		if engine_looks.has(want_look):
			_ok(lk == want_look, "tile %d names its base look plainly" % i)
		elif not engine_looks.is_empty():
			_ok(lk.contains("not in this build"),
				"tile %d discloses a look this build does not have" % i)

	## **Both branches of the disclosure, driven directly.** Every preset in the
	## table happens to name a look this cdylib has, so the walk above only ever
	## exercises the plain arm -- and a probe that walks only the live path
	## cannot see an inversion. `_look_line` is called here with all three
	## inputs it can receive.
	_p("_look_line(known)   = %s" % rw._look_line(String(engine_looks[0]), engine_looks))
	_p("_look_line(absent)  = %s" % rw._look_line("No Such Look", engine_looks))
	_p("_look_line(no API)  = %s" % rw._look_line("No Such Look", []))
	_ok(String(rw._look_line(String(engine_looks[0]), engine_looks)) == String(engine_looks[0]),
		"a look this build has is drawn plain")
	_ok(String(rw._look_line("No Such Look", engine_looks)).contains("not in this build"),
		"a look this build lacks is disclosed")
	_ok(String(rw._look_line("No Such Look", [])) == "No Such Look",
		"with no look list, nothing is claimed either way")

	# ============================================== §3 a tile applies its preset
	_p("=== §3 selection ===")
	var target := -1
	for i in RenderWorkspace.STYLE_PRESETS.size():
		if String(RenderWorkspace.STYLE_PRESETS[i][0]) == "Antique":
			target = i
	_ok(target >= 0, "the Antique preset is in the table")
	(tiles[target]["button"] as Button).pressed.emit()
	await _frames(6)
	_p("look after Antique   = %s" % bridge.look())
	_p("style readout        = %s" % _readout())
	_p("custom note visible  = %s" % str((rw.get("_custom_note") as Label).visible))
	_p("lit tiles            = %s" % str(_lit(tiles)))
	_ok(bridge.look() == String(RenderWorkspace.STYLE_PRESETS[target][1]),
		"the tile selected its own base look")
	_p("drawn readout label  = %s" % _drawn_readout().replace("\n", " / "))
	_ok(_readout() == "Antique", "the viewport readout names the preset")
	_ok(_drawn_readout().contains("Antique"),
		"and the label actually on the map carries it")
	_ok(not (rw.get("_custom_note") as Label).visible, "the Custom note is cleared")
	_ok(_lit(tiles) == [target], "exactly the pressed tile is lit")
	_ok(absf(float(bridge.npr_settings().get("sepia", 0.0)) - 0.35) < 0.001,
		"the Painter half of the bundle reached the engine (sepia 0.35)")

	## A manual edit afterwards: every tile must go unlit, the note must appear
	## and the readout must say Custom -- the half of `_mark_custom()` that used
	## to be `DccWidgets.set_segment_on(chip, false)` and is now `_set_tile_on`.
	bridge.set_npr({"ink": 0.42})
	rw._mark_custom()
	await _frames(4)
	_p("after a manual edit: lit=%s  note=%s  readout=%s" % [
		str(_lit(tiles)), str((rw.get("_custom_note") as Label).visible), _readout()])
	_ok(_lit(tiles).is_empty(), "no tile stays lit after a manual edit")
	_ok((rw.get("_custom_note") as Label).visible, "the Custom note appears")
	_ok(_readout() == "Custom", "the readout says Custom")

	## Back to a preset: lossless, which is the property the absolute bundles
	## exist for and the reason this does not stop at the first apply.
	(tiles[0]["button"] as Button).pressed.emit()
	await _frames(6)
	_ok(_lit(tiles) == [0], "re-selecting relights exactly one tile")
	_ok(absf(float(bridge.npr_settings().get("ink", -1.0))) < 0.001,
		"the manual ink edit was reset by the next preset")

	# ==================================== §4 geometry at the measured density
	## **The floor is a TOUCH requirement and this says so.** `role_px` resolves
	## `btn_min_h` to `[0, 44]` -- 44 on tablet, and **0 on pointer, where
	## `ROLE` states no floor at all**. Asserting `>= 0` on a pointer run would
	## pass for any control ever drawn, which is `MISTAKES.md`'s "an assertion
	## that passes because the bad state cannot exist yet". So the floor is
	## asserted only where there is one; the pointer run prints the number and
	## claims nothing from it.
	_p("=== §4 geometry at is_tablet=%s ===" % str(DccTheme.is_tablet()))
	var floor_h := float(DccTheme.role_px("btn_min_h"))
	if not DccTheme.is_tablet():
		_p("pointer density: ROLE states no btn_min_h floor (0). Heights below")
		_p("are reported, not asserted -- re-run with --force-touch for the floor.")
	var dock_w: float = float(app.get("_left_width"))
	var flow: Control = (tiles[0]["card"] as Control).get_parent()
	_p("left dock width=%.0f  gallery min width=%.1f  gallery drawn=%.1fx%.1f  columns=%d" % [
		dock_w, flow.get_combined_minimum_size().x, flow.size.x, flow.size.y,
		_columns(tiles)])
	_ok(flow.get_combined_minimum_size().x <= dock_w,
		"the gallery's minimum width fits the dock it is drawn in")
	## The gallery introduces the first hard minimum width this section has had,
	## so the question is not only "does it fit" but "is it now what holds the
	## dock open". Both numbers are printed; the assertion is that the dock's
	## own minimum did not become the gallery's, which is the shape
	## `MISTAKES.md`'s overflow row asks for.
	var dock_body: Control = app.get("left_dock_body")
	if dock_body != null:
		_p("left dock body min width=%.1f  gallery min=%.1f  dock=%.0f" % [
			dock_body.get_combined_minimum_size().x,
			flow.get_combined_minimum_size().x, dock_w])
		_ok(dock_body.get_combined_minimum_size().x <= dock_w,
			"the whole left dock still fits its own width with the gallery in it")
	## The column count is **measured, not assumed**. `TILE_W`/`TILE_W_T` were
	## picked to fit two in the dock at both densities; the first tablet figure
	## tried fitted one, and only this line caught it. A narrower band (the
	## 1366 LAPTOP dock) is allowed to reflow to one -- that is the flow doing
	## its job -- so the assertion is scoped to the two densities the constants
	## were measured for.
	if not DccTheme.is_laptop():
		_ok(_columns(tiles) == 2,
			"the gallery draws two columns at this density")
	for i in tiles.size():
		var b := tiles[i]["button"] as Button
		var c := tiles[i]["card"] as Control
		_p("  tile %d  card=%.1fx%.1f  hit=%.1fx%.1f  visible=%s" % [
			i, c.size.x, c.size.y, b.size.x, b.size.y, str(b.is_visible_in_tree())])
		_ok(b.is_visible_in_tree(), "tile %d is actually on screen" % i)
		if DccTheme.is_tablet():
			_ok(b.size.y >= floor_h,
				"tile %d hit target clears the %.0f px touch floor" % [i, floor_h])
			_ok(b.size.x >= floor_h,
				"tile %d hit target clears it on the other axis too" % i)
		## The hit target IS the card. A gallery whose clickable region is
		## smaller than the thing it draws is the defect the overlay exists to
		## avoid, and reasoning about anchors does not prove it.
		_ok(absf(b.size.x - c.size.x) < 0.5 and absf(b.size.y - c.size.y) < 0.5,
			"tile %d hit target covers its own card exactly" % i)
		## Nothing clipped: the card is a container, so its drawn height is the
		## content's own -- assert that rather than trusting it.
		var vb := (tiles[i]["name"] as Label).get_parent() as Control
		_ok(vb.size.y + 0.5 >= vb.get_combined_minimum_size().y,
			"tile %d draws its three lines without clipping" % i)
		for part in ["name", "look", "bundle"]:
			var l := tiles[i][part] as Label
			_ok(l.size.y + 0.5 >= l.get_combined_minimum_size().y,
				"tile %d's %s line is drawn at its full height" % [i, part])

	_p("=== done, %d failures ===" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
