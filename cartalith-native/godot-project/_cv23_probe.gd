extends Node
## Committed probe for GUI_GAP_REGISTER.md CV-23.
##
## Drives, windowed, against a real multi-faction world:
##   * `civ_territory_influence()` returns real per-faction and per-border
##     numbers, and the borders it names are pairs of factions that really
##     share ground.
##   * The Layers popover offers Civilization ▸ Contested borders, it is
##     available, it has a legend, and picking it really sets the view.
##   * The drawn raster reads as contested AT borders and not in interiors --
##     measured by recovering the contested scalar back out of the pixels
##     (the ramp is `owner_colour * (0.26 + 0.74 t²)`, so the ratio of the
##     Contested raster to the Political-control raster IS the lift).
##   * CIVIL ▸ Territories ▸ Borders & influence renders the same numbers.
##   * Memory: the process working set does not grow across 25 calls, which
##     is the actual CV-23 claim (nothing is retained).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _cv23_probe.tscn
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

var _app: Node
var _bridge
var _fail := 0

func _p(s: String) -> void:
	print("CV23  %s" % s)

func _bad(s: String) -> void:
	_fail += 1
	print("CV23  FAIL  %s" % s)

func _ok(s: String) -> void:
	print("CV23  ok    %s" % s)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)

func _all(root: Node) -> Array:
	var out: Array = []
	_walk(root, out)
	return out

func _texts(root: Node) -> String:
	var parts: Array[String] = []
	for n in _all(root):
		if n is Label:
			parts.append((n as Label).text)
		elif n is Button:
			parts.append((n as Button).text)
		elif n is RichTextLabel:
			parts.append((n as RichTextLabel).get_parsed_text())
	return "\n".join(parts)

func _button(root: Node, needle: String) -> Button:
	for n in _all(root):
		if n is Button and needle in (n as Button).text:
			return n
	return null

## Real process RSS, in MB -- Godot's own memory monitors only see Godot's
## allocator, and every byte this probe is about is allocated by Rust.
func _ws_mb() -> float:
	var out: Array = []
	OS.execute("powershell", ["-NoProfile", "-Command",
		"(Get-Process -Id %d).WorkingSet64" % OS.get_process_id()], out, false)
	if out.is_empty():
		return -1.0
	return float(String(out[0]).strip_edges()) / 1048576.0

## Windows' own high-water mark for this process, which is the only way to
## see the peak of a call that blocks the main thread throughout.
func _peak_mb() -> float:
	var out: Array = []
	OS.execute("powershell", ["-NoProfile", "-Command",
		"(Get-Process -Id %d).PeakWorkingSet64" % OS.get_process_id()], out, false)
	if out.is_empty():
		return -1.0
	return float(String(out[0]).strip_edges()) / 1048576.0


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 600.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	_bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)
	_p("world: %d settlements, %d factions, %d ways" % [
		_bridge.settlements().size(), _bridge.civ_faction_count(),
		_bridge.roads().size()])

	await _quantity()
	await _layer()
	await _pixels()
	await _dock()
	await _memory()
	await _memory_large()

	_check_bindings()
	_p("=== %s ===" % ("PASS" if _fail == 0 else "%d FAILURES" % _fail))
	get_tree().quit(0 if _fail == 0 else 1)


# ------------------------------------------------- the quantity itself
var _inf: Dictionary = {}

func _quantity() -> void:
	_p("=== the quantity ===")
	if not _bridge._has("civ_territory_influence"):
		_bad("civ_territory_influence missing from the loaded extension -- stale .dll?")
		return
	var t0 := Time.get_ticks_msec()
	_inf = _bridge.civ_territory_influence()
	var ms := Time.get_ticks_msec() - t0
	if _inf.is_empty():
		_bad("empty influence record on a generated multi-faction world")
		return
	_p("built in %d ms; transient %.2f MB (%d bytes)" % [ms,
		float(_inf.get("transient_bytes", 0)) / 1048576.0, int(_inf.get("transient_bytes", 0))])
	var owned := int(_inf.get("owned_cells", 0))
	var frontier := int(_inf.get("contested_cells", 0))
	_p("owned=%d frontier=%d (%.2f%%) mean_contested=%.4f mean_influence=%.2f thr=%.2f" % [
		owned, frontier, 100.0 * float(frontier) / maxf(1.0, float(owned)),
		float(_inf.get("mean_contested", 0.0)), float(_inf.get("mean_influence", 0.0)),
		float(_inf.get("frontier_threshold", 0.0))])
	if owned <= 0:
		_bad("no owned cells")
	if frontier <= 0:
		_bad("no contested cells at all on a multi-faction world")
	if frontier >= owned:
		_bad("every owned cell is a frontier -- the field is not discriminating")
	var mc := float(_inf.get("mean_contested", 0.0))
	if mc <= 0.0 or mc >= 1.0:
		_bad("mean contest %.4f is outside (0,1)" % mc)

	var factions: Array = _inf.get("factions", [])
	_p("factions: %d rows" % factions.size())
	var sum_cells := 0
	for row in factions:
		var r: Dictionary = row
		_p("  #%d %-14s cells=%-7d frontier=%-6d reach=%.1f contest=%.4f" % [
			int(r.get("id", 0)), String(r.get("name", "?")), int(r.get("cells", 0)),
			int(r.get("frontier_cells", 0)), float(r.get("mean_influence", 0.0)),
			float(r.get("mean_contested", 0.0))])
		sum_cells += int(r.get("cells", 0))
		if float(r.get("mean_influence", 0.0)) <= 0.0:
			_bad("faction %d has zero mean influence" % int(r.get("id", 0)))
	if sum_cells != owned:
		_bad("per-faction cells %d != owned %d" % [sum_cells, owned])
	if factions.size() < 2:
		_bad("fewer than two factions hold ground -- the fixture cannot test contest")

	var borders: Array = _inf.get("borders", [])
	_p("borders: %d faction pairs" % borders.size())
	var border_cells := 0
	for row in borders:
		var r: Dictionary = row
		_p("  %s <-> %s  cells=%d contest=%.4f" % [String(r.get("a_name", "?")),
			String(r.get("b_name", "?")), int(r.get("cells", 0)),
			float(r.get("mean_contested", 0.0))])
		border_cells += int(r.get("cells", 0))
		if int(r.get("a", 0)) == int(r.get("b", 0)):
			_bad("a faction is listed as contesting itself")
		if float(r.get("mean_contested", 0.0)) < float(_inf.get("frontier_threshold", 0.88)):
			_bad("a border's mean contest is below the frontier threshold it was counted at")
	if borders.is_empty():
		_bad("no faction pair contests anything")
	if border_cells > frontier:
		_bad("pair cells %d exceed frontier cells %d" % [border_cells, frontier])

	## Determinism: the same world must give the same answer twice, or the
	## on-demand rebuild is not reproducing the resident territory grid.
	var again: Dictionary = _bridge.civ_territory_influence()
	if int(again.get("owned_cells", -1)) != owned or int(again.get("contested_cells", -1)) != frontier:
		_bad("two calls on one world disagree (%d/%d vs %d/%d)" % [
			owned, frontier, int(again.get("owned_cells", -1)), int(again.get("contested_cells", -1))])
	else:
		_ok("two on-demand rebuilds agree exactly")


# ------------------------------------------------- the Layers popover row
func _layer() -> void:
	_p("=== the Layers row ===")
	var offered := false
	for g in _bridge.debug_layers():
		var grp: Dictionary = g
		for it in grp.get("items", []):
			var item: Dictionary = it
			if String(item.get("id", "")) != "contested":
				continue
			offered = true
			_p("row: group=%s label=%s available=%s legend=%d" % [
				String(grp.get("group", "?")), String(item.get("label", "?")),
				bool(item.get("available", false)), (item.get("legend", []) as Array).size()])
			if String(grp.get("group", "")) != "Civilization":
				_bad("contested is not in the Civilization group")
			if not bool(item.get("available", false)):
				_bad("contested is offered but unavailable on a generated world")
			if (item.get("legend", []) as Array).is_empty():
				_bad("contested has no legend")
			if String(item.get("hint", "")).is_empty():
				_bad("contested has no hint")
	if not offered:
		_bad("contested is not in debug_layers()")

	## Open the real popover and click the real row.
	var pop = _app.layers_popover if "layers_popover" in _app else null
	if pop == null:
		_p("note: app has no layers_popover field; driving ViewportHost directly")
	else:
		pop.open()
		await _frames(6)
		var row := _button(pop, "Contested borders")
		if row == null:
			_bad("no 'Contested borders' row in the open popover")
		elif row.disabled:
			_bad("the 'Contested borders' row is disabled")
		else:
			row.pressed.emit()
			await _frames(6)
			_ok("clicked the real popover row")
		pop.hide()
		await _frames(4)
	_app.viewport.set_debug_layer("contested")
	await _frames(6)
	if _app.viewport.debug_view() != "contested":
		_bad("set_debug_layer('contested') did not stick (got %s)" % _app.viewport.debug_view())
	else:
		_ok("the map is drawing the Contested-borders view")


# ------------------------------------------------- the pixels themselves
const BG_WATER := Color8(18, 30, 48)
const BG_LAND := Color8(40, 42, 46)

func _is_bg(c: Color) -> bool:
	return (c.is_equal_approx(BG_WATER) or c.is_equal_approx(BG_LAND))

func _pixels() -> void:
	_p("=== border vs interior, measured off the raster ===")
	var ctl: Texture2D = _bridge.debug_texture("control")
	var con: Texture2D = _bridge.debug_texture("contested")
	if ctl == null or con == null:
		_bad("could not build both rasters")
		return
	var ci: Image = ctl.get_image()
	var xi: Image = con.get_image()
	if ci.get_width() != xi.get_width() or ci.get_height() != xi.get_height():
		_bad("rasters disagree about size")
		return
	var w: int = ci.get_width()
	var h: int = ci.get_height()

	## Owner id per pixel, taken from the Political-control raster's own
	## faction swatches -- the two views are drawn from one owner grid, so
	## this is the honest way to find a border without a per-cell binding.
	var owner := PackedInt32Array()
	owner.resize(w * h)
	var palette: Array[Color] = []
	for y in h:
		for x in w:
			var c: Color = ci.get_pixel(x, y)
			if _is_bg(c):
				owner[y * w + x] = 0
				continue
			var id := 0
			for i in palette.size():
				if palette[i].is_equal_approx(c):
					id = i + 1
					break
			if id == 0:
				palette.append(c)
				id = palette.size()
			owner[y * w + x] = id
	_p("distinct faction swatches on the control raster: %d" % palette.size())

	## `contested = owner_colour * (0.26 + 0.74 t^2)`, so the per-pixel ratio
	## of the two rasters recovers the lift and therefore t itself. Hatched
	## frontier pixels carry the RIVAL's colour instead, which shows up as a
	## ratio that is not a plain scalar -- those are excluded from the mean
	## by taking the per-channel ratio only where the control channel is
	## bright enough to be meaningful, and by using the median-ish max
	## channel rather than luminance.
	var border_t: Array[float] = []
	var interior_t: Array[float] = []
	var hatched := 0
	for y in range(1, h - 1):
		for x in range(1, w - 1):
			var i: int = y * w + x
			var o := owner[i]
			if o == 0:
				continue
			var is_border := false
			for d in [[1, 0], [-1, 0], [0, 1], [0, -1]]:
				var nb := owner[(y + d[1]) * w + (x + d[0])]
				if nb != 0 and nb != o:
					is_border = true
			var base: Color = ci.get_pixel(x, y)
			var got: Color = xi.get_pixel(x, y)
			var ch := maxf(base.r, maxf(base.g, base.b))
			if ch < 0.05:
				continue
			var top := maxf(got.r, maxf(got.g, got.b))
			var lift := top / ch
			## The hatch swaps in the rival's swatch, whose brightest channel
			## need not match the owner's -- a lift above 1 is that, not a
			## contest above 1.
			if lift > 1.02:
				hatched += 1
				lift = 1.0
			var t := sqrt(maxf(0.0, (lift - 0.26) / 0.74))
			if is_border:
				border_t.append(t)
			else:
				## Interior = no different owner within 6 cells in either axis.
				var clean := true
				for k in range(-6, 7):
					var xx: int = clampi(x + k, 0, w - 1)
					var yy: int = clampi(y + k, 0, h - 1)
					if owner[y * w + xx] != 0 and owner[y * w + xx] != o:
						clean = false
					if owner[yy * w + x] != 0 and owner[yy * w + x] != o:
						clean = false
				if clean:
					interior_t.append(t)

	var bm := _mean(border_t)
	var im := _mean(interior_t)
	_p("border cells n=%d mean t=%.3f | interior cells n=%d mean t=%.3f | hatched pixels=%d" % [
		border_t.size(), bm, interior_t.size(), im, hatched])
	if border_t.size() < 100:
		_bad("too few border cells found to measure (%d)" % border_t.size())
	if interior_t.size() < 100:
		_bad("too few interior cells found to measure (%d)" % interior_t.size())
	if bm <= im:
		_bad("border cells are not more contested than interior cells (%.3f vs %.3f)" % [bm, im])
	elif bm - im < 0.10:
		_bad("border/interior contest separation is only %.3f -- not a readable difference" % (bm - im))
	else:
		_ok("borders read as contested: %.3f at a border vs %.3f in the interior (+%.3f)" % [bm, im, bm - im])
	if hatched <= 0:
		_bad("no hatched frontier pixels -- the rival's colour never appears")
	else:
		_ok("%d frontier pixels carry the rival faction's own colour" % hatched)

	## The raster must not be one flat colour, and must be opaque.
	var first: Color = xi.get_pixel(0, 0)
	var varied := false
	for y in range(0, h, 3):
		for x in range(0, w, 3):
			if not xi.get_pixel(x, y).is_equal_approx(first):
				varied = true
	if not varied:
		_bad("the contested raster is a single flat colour")

func _mean(a: Array[float]) -> float:
	if a.is_empty():
		return 0.0
	var s := 0.0
	for v in a:
		s += v
	return s / a.size()


# ------------------------------------------------- the dock readout
func _dock() -> void:
	_p("=== CIVIL ▸ Territories ▸ Borders & influence ===")
	_app.select_domain_category("civilization", "Territories")
	await _frames(8)
	var ws: Node = _app
	var body := _texts(ws)
	if "Borders & influence" in body or "BORDERS & INFLUENCE" in body:
		_ok("the section is on the rail")
	else:
		_bad("no 'Borders & influence' section in CIVIL ▸ Territories")
	if "there is no contested-claim value" in body:
		_bad("the stale CV-23 denial is still on screen")
	var b := _button(ws, "Analyse contested borders")
	if b == null:
		_bad("no 'Analyse contested borders' button")
		return
	if b.disabled:
		_bad("the analyse button is disabled on a generated world")
		return
	b.pressed.emit()
	await _frames(10)
	var after := _texts(ws)
	for needle in ["owned land cells", "Mean contest", "PER FACTION", "CONTESTED BORDERS",
			"Built on demand and dropped"]:
		if needle in after:
			_ok("readout renders: %s" % needle)
		else:
			_bad("readout is missing: %s" % needle)


# ------------------------------------------------- memory
func _memory() -> void:
	_p("=== memory ===")
	var per_call := int(_inf.get("transient_bytes", 0))
	_p("per-call transient field: %d bytes (%.2f MB) at %dx%d" % [
		per_call, float(per_call) / 1048576.0, _bridge.grid_size().x, _bridge.grid_size().y])
	var base := _ws_mb()
	if base < 0.0:
		_bad("could not read the process working set")
		return
	await _frames(4)
	var peak := base
	for i in 25:
		var d: Dictionary = _bridge.civ_territory_influence()
		if d.is_empty():
			_bad("call %d came back empty" % i)
		var now := _ws_mb()
		peak = maxf(peak, now)
		if i % 5 == 0:
			await _frames(2)
	await _frames(30)
	var after := _ws_mb()
	_p("working set: base %.1f MB -> peak %.1f MB -> after 25 calls %.1f MB (delta %+.1f MB)" % [
		base, peak, after, after - base])
	## 25 calls that each retained their field would be 25x the per-call
	## figure. Anything under two calls' worth is allocator noise, not
	## retention.
	var budget := maxf(12.0, 2.0 * float(per_call) / 1048576.0)
	if after - base > budget:
		_bad("the working set grew %.1f MB over 25 calls (budget %.1f) -- something is retained"
			% [after - base, budget])
	else:
		_ok("nothing retained: %+.1f MB across 25 rebuilds (budget %.1f MB)" % [after - base, budget])

## The same measurement at a grid big enough for the transient field to be
## visible against allocator noise -- 384x288 is only 2.1 MB, which proves
## nothing is retained but says little about what a *big* world costs.
func _memory_large() -> void:
	_p("=== memory, large grid ===")
	_bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": 1024, "grid_h": 768,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(20)
	var gs: Vector2i = _bridge.grid_size()
	_p("regenerated at %dx%d" % [gs.x, gs.y])
	var base := _ws_mb()
	var peak0 := _peak_mb()
	var t0 := Time.get_ticks_msec()
	var d: Dictionary = _bridge.civ_territory_influence()
	var ms := Time.get_ticks_msec() - t0
	if d.is_empty():
		_bad("empty influence record on the large world")
		return
	var per_call := int(d.get("transient_bytes", 0))
	var peak1 := _peak_mb()
	await _frames(40)
	var after := _ws_mb()
	_p("owned=%d frontier=%d in %d ms; transient %.1f MB" % [
		int(d.get("owned_cells", 0)), int(d.get("contested_cells", 0)), ms,
		float(per_call) / 1048576.0])
	_p("working set %.1f -> %.1f MB (delta %+.1f); process peak %.1f -> %.1f MB (delta %+.1f)" % [
		base, after, after - base, peak0, peak1, peak1 - peak0])
	if after - base > 2.0 * float(per_call) / 1048576.0 + 12.0:
		_bad("the large-grid field did not come back: %+.1f MB resident" % (after - base))
	else:
		_ok("large-grid field released: %+.1f MB resident after a %.1f MB build"
			% [after - base, float(per_call) / 1048576.0])
	## Straight arithmetic, stated rather than measured: the same four grids
	## at the port's own 8192x8192 ceiling.
	_p("extrapolated at 8192x8192: %.0f MB transient, 0 bytes resident"
		% (float(8192 * 8192 * 53) / 1048576.0))


## The staleness fingerprint, read off the shell instead of guessed at.
##
## `EngineBridge._has()` (`shell/engine_bridge.gd`) is the one choke point
## every binding guard in the shell goes through, and it records the name of
## each method the shell asked for that this build does not export;
## `EngineBridge.missing_bindings()` hands back the set. Nothing in this probe
## suite read it -- and a stale `target/debug/cartalith_godot.dll` has twice
## sent every `_has()` guard in a run down its degraded-fallback branch, which
## turns a whole sweep into a clean report over code that was never exercised.
## That is the failure mode this suite is least able to notice on its own, and
## the shell was already carrying the answer.
##
## Called last, after every surface this run drives has been driven: the set
## only fills as guards are reached, so an early read reports an empty one.
func _check_bindings() -> void:
	var mb: PackedStringArray = _bridge.missing_bindings()
	if mb.is_empty():
		return
	_bad("stale extension -- the shell asked for %d binding(s) this build "
		% mb.size()
		+ "does not export (%s). " % ", ".join(mb)
		+ "Every result above was measured against a degraded shell; rebuild "
		+ "the crates and re-run before believing any of it.")
