extends Node
## VERIFIER probe for owner ruling 14's visible half. Independent of
## `_iconmerge_probe.gd`: a REAL world, a REAL `landmark_run()` and a REAL
## `icon_generate({"family":"POI"})`, not a hand-written fixture.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _vfy_iconmerge_probe.tscn
##   Godot_v4.7.1-stable_win64_console.exe --path . _vfy_iconmerge_probe.tscn -- --head
##
## `--head` loads `res://_vfy_overlay_head.gd` instead of the working tree's
## `map_overlay.gd`, so the two runs are the BEFORE and AFTER of one change
## over identical engine data. That file is NOT kept in the tree -- a stale
## 3 400-line copy of the overlay beside the real one is a hazard, and it is
## one command to make:
##
##   git show <ref>:cartalith-native/godot-project/map_overlay.gd \
##     > cartalith-native/godot-project/_vfy_overlay_head.gd
##
## Without it, `--head` aborts rather than silently measuring the same build
## twice.
##
## Windowed, deliberately: every number below is a framebuffer census, and
## `RenderingServer.frame_post_draw` never fires under the dummy driver.

const W := 1600
const H := 1000
const GW := 128
const GH := 80
const SEED := 483920
const BG := Color(0.03, 0.03, 0.04)

## Half-width of the census box, framebuffer px. The largest mark this can be
## asked to contain is a continental ring at importance 1.0:
## `LANDMARK_CLASS_RADIUS["continental"]` 9.0 * (0.75 + 0.5) = 11.25 px plus
## half of its 2.4 px outline = 12.45. A POI diamond is `ICON_BASE_RADIUS` 5.5.
const HALF := 14
## No OTHER mark's centre may come within this, or its pixels enter the box:
## HALF + the same 12.45 px worst-case mark, rounded up.
const CLEAR_PX := 27.0

var _fails := 0
var _vp: SubViewport
var _blank_vp: SubViewport
var _ov: Control
var _blank: PackedByteArray = PackedByteArray()
var _cellpx := 0.0


func _p(s: String) -> void:
	print("VFYMERGE  %s" % s)


func _bad(s: String) -> void:
	_fails += 1
	_p("FAIL  %s" % s)


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		_p("WATCHDOG -- probe did not finish")
		get_tree().quit(2))
	wd.start()

	if DisplayServer.get_name() == "headless":
		_p("ABORT: pixels; frame_post_draw never fires headless.")
		get_tree().quit(2)
		return

	var use_head := false
	for a in OS.get_cmdline_user_args():
		if a == "--head":
			use_head = true
	var script_path := "res://_vfy_overlay_head.gd" if use_head else "res://map_overlay.gd"
	if use_head and not ResourceLoader.exists(script_path):
		_p("ABORT: --head needs %s; make it with the git command in the header."
			% script_path)
		get_tree().quit(2)
		return
	_p("overlay script: %s" % script_path)

	var g := WorldGen.new()
	g.set_sea_level(0.45)
	g.set_villages_enabled(true)
	g.generate_sized(SEED, 2400.0, GW, GH)
	if not g.landmark_run():
		_bad("landmark_run() returned false")
		get_tree().quit(1)
		return
	var lms: Array = g.landmarks()
	_p("landmarks=%d" % lms.size())
	if lms.size() < 10:
		_bad("too few landmarks to select isolated cells from")
		get_tree().quit(1)
		return

	var rep: Dictionary = g.icon_generate({"family": "POI"})
	_p("icon_generate(POI) -> %s" % rep)
	var icons: Array = g.icon_list()
	_p("icon_list() rows=%d" % icons.size())
	if icons.is_empty():
		_bad("the POI pass placed nothing; every census would be vacuous")
		get_tree().quit(1)
		return

	## --- attack item 3: the `origin` key, read on the GDScript side ---
	var missing := 0
	var gen := 0
	var other := []
	for r: Dictionary in icons:
		if not r.has("origin"):
			missing += 1
			continue
		match String(r["origin"]):
			"generated": gen += 1
			"manual": pass
			_: other.append(r["origin"])
	if missing > 0:
		_bad("%d of %d icon_list() rows have NO `origin` key" % [missing, icons.size()])
	if not other.is_empty():
		_bad("unknown origin values: %s" % [other])
	if gen != icons.size():
		_bad("every row came from the generated pass; %d of %d say so" % [gen, icons.size()])
	_p("origin census: generated=%d missing=%d  sample=%s" % [gen, missing, icons[0]])

	## --- select census cells from ENGINE data only, so BEFORE and AFTER agree ---
	var lm_cell := {}          ## Vector2i -> class string
	for lm: Dictionary in lms:
		lm_cell[Vector2i(int(lm["x"]), int(lm["y"]))] = String(lm.get("class", "local"))
	var gen_cell := {}
	for r: Dictionary in icons:
		if String(r.get("family", "")) == "poi":
			gen_cell[Vector2i(roundi(float(r["x"])), roundi(float(r["y"])))] = true

	var rect0 := Rect2(Vector2.ZERO, Vector2(W, H))
	_cellpx = float(W) / float(GW)
	var clear_cells := int(ceil(CLEAR_PX / _cellpx)) + 1
	_p("cell = %.2f px; isolation radius = %d cells" % [_cellpx, clear_cells])

	var occupied := {}
	for k in lm_cell:
		occupied[k] = true
	for k in gen_cell:
		occupied[k] = true

	var dup_cells := _pick(lm_cell, gen_cell, occupied, clear_cells, true, 3)
	var ring_cells := _pick(lm_cell, gen_cell, occupied, clear_cells, false, 2)
	_p("DUP cells   (landmark + generated POI icon): %s" % [dup_cells])
	_p("RING cells  (landmark, POI pass culled it):  %s" % [ring_cells])
	if dup_cells.is_empty():
		_bad("no isolated landmark cell also carries a generated POI icon -- "
			+ "the duplicate this batch removes is not in view")
		get_tree().quit(1)
		return
	if ring_cells.is_empty():
		_bad("no isolated ring-only landmark cell; the 'the layer still draws' "
			+ "control would be missing")
		get_tree().quit(1)
		return

	## Three authored rows, appended to the engine's own list exactly as
	## `viewport_host.gd` hands it over. These are the capability-loss check:
	## a hand-placed icon must survive whatever rule the merge chose.
	var free_cells := _free(occupied, clear_cells, 2)
	if free_cells.size() < 2:
		_bad("could not find two empty isolated cells for the authored icons")
		get_tree().quit(1)
		return
	var manual_poi: Vector2i = free_cells[0]      ## hand-placed POI, nothing else there
	var legacy_poi: Vector2i = free_cells[1]      ## hand-placed POI with NO origin key
	var manual_on_ring: Vector2i = ring_cells[0]  ## hand-placed POI ON a landmark's cell
	icons.append({"x": float(manual_poi.x) + 0.5, "y": float(manual_poi.y) + 0.5,
		"family": "poi", "slot": "cave", "set": "", "scale": 1.0, "origin": "manual"})
	icons.append({"x": float(legacy_poi.x) + 0.5, "y": float(legacy_poi.y) + 0.5,
		"family": "poi", "slot": "cave", "set": "", "scale": 1.0})
	icons.append({"x": float(manual_on_ring.x) + 0.5, "y": float(manual_on_ring.y) + 0.5,
		"family": "poi", "slot": "cave", "set": "", "scale": 1.0, "origin": "manual"})
	_p("authored POI icons at %s (manual), %s (no origin key), %s (on a ring cell)"
		% [manual_poi, legacy_poi, manual_on_ring])

	_vp = _make_vp()
	_ov = Control.new()
	_ov.set_script(load(script_path))
	_ov.size = Vector2(W, H)
	_vp.add_child(_ov)
	_ov.set_civ_data([], [], [], GW, GH, 0.0)
	_ov.set_landmarks(lms)
	_ov.set_manual_icons(icons)
	_blank_vp = _make_vp()

	for f in 3:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	_blank = _blank_vp.get_texture().get_image().get_data()
	if _blank.is_empty():
		_bad("blank control captured nothing")

	var cells: Array[Vector2i] = []
	for c in dup_cells:
		cells.append(c)
	for c in ring_cells:
		cells.append(c)
	cells.append(manual_poi)
	cells.append(legacy_poi)

	for on in [true, false]:
		var name := "ON " if on else "OFF"
		_ov.set_landmarks_visible(on)
		_ov.queue_redraw()
		for f in 3:
			await get_tree().process_frame
		await RenderingServer.frame_post_draw
		var img: Image = _vp.get_texture().get_image()
		if img.get_data() == _blank:
			_bad("%s: frame byte-identical to the blank control" % name)
			continue
		var seen := {}
		for c in cells:
			var t := _census(img, c)
			seen[c] = t
			_p("%s  cell %-10s ring=%-5d poi=%-5d" % [name, "(%d,%d)" % [c.x, c.y],
				t["ring"], t["poi"]])
		var mr := _census(img, manual_on_ring)
		seen[manual_on_ring] = mr
		_p("%s  cell %-10s ring=%-5d poi=%-5d   (authored icon ON a landmark cell)"
			% [name, "(%d,%d)" % [manual_on_ring.x, manual_on_ring.y], mr["ring"], mr["poi"]])

		## Positive controls FIRST: without these every "gone" reading is vacuous.
		for c in [manual_poi, legacy_poi, manual_on_ring]:
			if int(seen[c]["poi"]) <= 0:
				_bad("%s: authored icon at (%d,%d) DREW NOTHING -- capability loss"
					% [name, c.x, c.y])
		for c in dup_cells:
			var t: Dictionary = seen[c]
			if on:
				if int(t["ring"]) <= 0:
					_bad("%s: (%d,%d) lost its landmark ring" % [name, c.x, c.y])
				if int(t["poi"]) != 0:
					_bad("%s: (%d,%d) draws a POI glyph beside its ring (poi=%d) "
						% [name, c.x, c.y, t["poi"]] + "-- drawn twice")
			else:
				if int(t["ring"]) != 0 or int(t["poi"]) != 0:
					_bad("%s: (%d,%d) still draws with Landmarks off (ring=%d poi=%d)"
						% [name, c.x, c.y, t["ring"], t["poi"]])
		for c in ring_cells:
			var t2: Dictionary = seen[c]
			if on and int(t2["ring"]) <= 0:
				_bad("%s: ring-only cell (%d,%d) lost its ring" % [name, c.x, c.y])
			if not on and int(t2["ring"]) != 0:
				_bad("%s: ring-only cell (%d,%d) ignores the flag" % [name, c.x, c.y])

	_p("---- %s ----" % ("PASS" if _fails == 0 else "%d FAILED" % _fails))
	get_tree().quit(1 if _fails > 0 else 0)


## Landmark cells of a PHYSICAL class (a cultural landmark's accent orange and
## the POI yellow differ only in the green channel, and a classifier needing a
## threshold between 0.639 and 0.894 can be wrong) that either do or do not
## also carry a generated POI icon, and that nothing else comes near.
func _pick(lm_cell: Dictionary, gen_cell: Dictionary, occupied: Dictionary,
		clear: int, want_icon: bool, n: int) -> Array[Vector2i]:
	var out: Array[Vector2i] = []
	var keys := lm_cell.keys()
	keys.sort_custom(func(a: Vector2i, b: Vector2i) -> bool:
		return (a.y * 100000 + a.x) < (b.y * 100000 + b.x))
	for k: Vector2i in keys:
		if String(lm_cell[k]) == "cultural":
			continue
		if gen_cell.has(k) != want_icon:
			continue
		if not _isolated(k, occupied, clear):
			continue
		var too_close := false
		for c in out:
			if absi(c.x - k.x) <= clear * 2 and absi(c.y - k.y) <= clear * 2:
				too_close = true
		if too_close:
			continue
		out.append(k)
		if out.size() >= n:
			break
	return out


func _free(occupied: Dictionary, clear: int, n: int) -> Array[Vector2i]:
	var out: Array[Vector2i] = []
	for y in range(clear + 2, GH - clear - 2):
		for x in range(clear + 2, GW - clear - 2):
			var k := Vector2i(x, y)
			if occupied.has(k):
				continue
			if not _isolated(k, occupied, clear):
				continue
			var too_close := false
			for c in out:
				if absi(c.x - k.x) <= clear * 3 and absi(c.y - k.y) <= clear * 3:
					too_close = true
			if too_close:
				continue
			out.append(k)
			if out.size() >= n:
				return out
	return out


func _isolated(k: Vector2i, occupied: Dictionary, clear: int) -> bool:
	for dy in range(-clear, clear + 1):
		for dx in range(-clear, clear + 1):
			if dx == 0 and dy == 0:
				continue
			if occupied.has(Vector2i(k.x + dx, k.y + dy)):
				return false
	return true


func _make_vp() -> SubViewport:
	var vp := SubViewport.new()
	vp.size = Vector2i(W, H)
	vp.transparent_bg = false
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var bg := ColorRect.new()
	bg.size = Vector2(W, H)
	bg.color = BG
	vp.add_child(bg)
	return vp


## Pixels around a cell, classified by hue against a near-black ground.
## The overlay's own `_cell_to_screen` is CALLED, not re-derived.
func _census(img: Image, cell: Vector2i) -> Dictionary:
	var rect: Rect2 = _ov.displayed_rect()
	var c: Vector2 = _ov._cell_to_screen(Vector2(cell.x, cell.y), rect)
	var out := {"ring": 0, "poi": 0}
	var x0 := maxi(0, int(c.x) - HALF)
	var x1 := mini(W - 1, int(c.x) + HALF)
	var y0 := maxi(0, int(c.y) - HALF)
	var y1 := mini(H - 1, int(c.y) + HALF)
	for y in range(y0, y1 + 1):
		for x in range(x0, x1 + 1):
			var p := img.get_pixel(x, y)
			if maxf(p.r, maxf(p.g, p.b)) < 0.30:
				continue
			if p.b > p.r and p.b > p.g:
				out["ring"] += 1
			elif p.r > 0.45 and p.g > 0.45 and p.b < 0.40:
				out["poi"] += 1
	return out
