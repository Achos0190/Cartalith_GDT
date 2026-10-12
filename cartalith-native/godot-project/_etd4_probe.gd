extends Node
## **ET-D4 (GDScript lane): drag-to-draw roads, through the real map input path.**
##
## Protects (EDITING_TOOLS_SCOPE.md ET-D4, shell part). Every check names what it guards:
##  a. a drag with the Way tool commits ONE way whose polyline starts and ends at the drag's
##     ends, follows the drag within a bar, and is ROUTED round a lake the straight drag
##     crosses (it is not the straight raster, and none of its vertices is water);
##  b. with the seam `_way_drag_options = {mode: freehand, smooth: false, spacing: null}` the
##     committed way is exactly the Bresenham raster of the cells the shell fed the engine;
##  c. a freehand drag across water is refused, tells the user where, pushes no way, while the
##     same drag routed commits (positive control);
##  d. a plain click is unchanged: the waypoint exists the instant of the press (the old
##     shell's timing), a two-click way equals the same calls made straight on the bridge, and
##     a drag made while a click draft exists leaves that draft exactly as it was;
##  e. vertical symmetry (via the seam) commits TWO ways that are exact reflections; a plain
##     drag is one (positive control);
##  f. the engine drag is CLOSED after release, tool change, new press, focus loss, Escape and
##     a world change (authority: `way_stroke_sample` returning -1), each with an OPEN-drag
##     positive control;
##  g. while `bridge.generating` the engine is not called (an open engine drag is neither
##     fed, ended nor cancelled; the next press after generation cancels it);
##  h. without the stroke API the shell is the click path, unchanged; a refused seam option
##     leaves the press as a plain waypoint and says so;
##  i. every touched .gd and shell/app.gd parses (`can_instantiate`).
##
## Run WINDOWED (never headless):
##   godot --path . res://_etd4_probe.tscn -- [--force-touch --vp 1080x2340] [--tag T]

var app: Node
var bridge: EngineBridge
var _vp: SubViewport
var _tag := "etd4"
var _fail := 0
var _checks := 0
var _phone := false
var _dev := 0
var _last_at := Vector2.ZERO

const SEED := 4242
var _seed := SEED
const GRID_W := 160
const GRID_H := 100
const SETTLE := 6
const STEP := 2.5
const FREEHAND := {"mode": "freehand", "smooth": false, "spacing": null}

var ws: Node          ## the InfrastructureWorkspace (Way/Route handlers)
var _wat := PackedByteArray()   ## 1 = not plain land, per cell

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

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
	var known := ["--force-touch", "--vp", "--tag"]
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in known):
			_log("ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			return false
	return true

func _finish() -> void:
	_log("RESULT %s checks=%d fail=%d" % [_tag, _checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

## Windowed-ness guard: `ImageTexture.update()` is a no-op headless.
func _guard() -> bool:
	var img := Image.create_empty(2, 2, false, Image.FORMAT_RGBA8)
	img.fill(Color(0, 0, 0, 0))
	var t := ImageTexture.create_from_image(img)
	img.set_pixel(1, 1, Color(1, 0, 0, 1))
	t.update(img)
	return t.get_image().get_pixel(1, 1).a > 0.0

# -- Real input, pushed into the SubViewport the app lives in ------------------------------

func _screen_of(gx: float, gy: float) -> Vector2:
	var ov: Control = app.viewport.overlay
	var rect: Rect2 = ov._displayed_rect()
	var local := rect.position + Vector2(gx / float(GRID_W), gy / float(GRID_H)) * rect.size
	return ov.get_global_transform() * local

func _button(at: Vector2, pressed: bool) -> void:
	var e := InputEventMouseButton.new()
	e.button_index = MOUSE_BUTTON_LEFT
	e.pressed = pressed
	e.position = at
	e.global_position = at
	e.device = _dev
	_vp.push_input(e, true)

func _motion(at: Vector2, prev: Vector2) -> void:
	var m := InputEventMouseMotion.new()
	m.position = at
	m.global_position = at
	m.relative = at - prev
	m.button_mask = MOUSE_BUTTON_MASK_LEFT
	m.pressure = 0.0 if _phone else 1.0
	m.device = _dev
	_vp.push_input(m, true)

## Press at `vertices[0]` and move along the polyline in <= `step`-cell steps, one motion per
## frame. `release == false` leaves the button down (the caller ends the gesture).
func _drag_path(vertices: Array, step: float, release: bool = true) -> void:
	var v0: Vector2 = vertices[0]
	var prev := _screen_of(v0.x, v0.y)
	_button(prev, true)
	await _frames(1)
	for i in range(1, vertices.size()):
		var a: Vector2 = vertices[i - 1]
		var b: Vector2 = vertices[i]
		var n := maxi(1, int(ceil(a.distance_to(b) / step)))
		for k in range(1, n + 1):
			var p := a.lerp(b, float(k) / float(n))
			var at := _screen_of(p.x, p.y)
			_motion(at, prev)
			prev = at
			await _frames(1)
	_last_at = prev
	if release:
		_button(prev, false)
		await _frames(SETTLE)

func _release_now() -> void:
	_button(_last_at, false)
	await _frames(SETTLE)

func _tap(at: Vector2) -> void:
	_button(at, true)
	await _frames(1)
	_button(at, false)
	await _frames(SETTLE)

## A click at grid cell (x, y) through the real overlay path. The phone withholds the press
## until it is released, so the waypoint exists after the release there; the desktop one at
## the press.
func _click_cell(x: float, y: float) -> void:
	await _tap(_screen_of(x, y))

# -- World, fixtures --------------------------------------------------------------------------

func _generate(seed_: int) -> void:
	bridge.generate({"seed": seed_, "width_km": 800.0, "grid_w": GRID_W, "grid_h": GRID_H,
		"sea_level": 0.5, "villages": true})
	await bridge.generation_finished
	await _frames(12)

func _read_water() -> void:
	_wat = PackedByteArray()
	_wat.resize(GRID_W * GRID_H)
	for y in GRID_H:
		for x in GRID_W:
			var d: Dictionary = bridge.world_gen.sample_cell(x, y)
			_wat[y * GRID_W + x] = 0 if String(d.get("water", "")) == "land" else 1

func _is_water(x: int, y: int) -> bool:
	if x < 0 or y < 0 or x >= GRID_W or y >= GRID_H:
		return true
	return _wat[y * GRID_W + x] != 0

static func _rnd(v: float) -> int:
	return int(floor(v + 0.5))

## A fresh deterministic world with Way armed (a DISCLOSED stand-in for the real route, which
## `_reach_way` proves once at the start) and the seam cleared.
func _fresh(options: Dictionary = {}) -> void:
	await _generate(_seed)
	## Re-arm every time: `way_begin` runs only on arming, and a world change drops the engine's
	## click draft, so an already-armed tool would click into nothing (pre-existing; noted).
	app.arm_tool("inspect")
	await _frames(2)
	app.arm_tool("way")
	await _frames(SETTLE)
	ws.set("_way_drag_options", options.duplicate())
	app.set_status("hint", "")

## Waits until the busy overlay stops swallowing input. Flipping `bridge.generating` by hand
## (checks g and the arm-while-generating tail) makes `busy_overlay.gd` acquire an
## unsignalled engine source; clearing the flag does not release it until its watchdog sees
## `WATCHDOG_FRAMES` frames of `generating == false`, and until then its `_input` marks EVERY
## event handled, so a tap issued straight after is swallowed and never reaches the map. (The
## (g) "next press cancels the stale drag" check failed for exactly this, from the overlay's
## arrival in 6717e45f's neighbours until fixed here 2026-10-12.) Bounded, and a miss is a
## check failure rather than a hang.
func _busy_released() -> void:
	var guard := 0
	while app.busy_overlay != null and app.busy_overlay.is_blocking() and guard < 120:
		await get_tree().process_frame
		guard += 1
	_check(app.busy_overlay == null or not app.busy_overlay.is_blocking(), "(g) the busy overlay released after the hand-set generating flag cleared (%d frames)" % guard)

func _hint() -> String:
	return String((app._status_labels["hint"] as Label).text)

func _manual_count() -> int:
	var n := 0
	while not bridge.world_gen.way_get("manual", n).is_empty():
		n += 1
	return n

func _way(i: int) -> Dictionary:
	return bridge.world_gen.way_get("manual", i)

## `way_stroke_sample`'s answer at the grid corner; a count >= 0 means a drag is OPEN (the
## sample itself is a throw-away waypoint, so call this only as the last thing before a close
## or as a closed test). -1 = closed. Calls the engine directly, not through the bridge's
## `generating` gate.
func _engine_open() -> int:
	return bridge.world_gen.way_stroke_sample(1.0, 1.0, 0)

func _ws_get(name: String, dflt: Variant) -> Variant:
	var v = ws.get(name)
	return dflt if v == null else v

func _visible_cells() -> Array:
	var host: Rect2 = app.viewport.get_global_rect().grow(-24.0)
	if _phone:
		host = Rect2(host.position + Vector2(0.0, host.size.y * 0.3), Vector2(host.size.x, host.size.y * 0.4))
	var ov: Control = app.viewport.overlay
	var rect: Rect2 = ov._displayed_rect()
	var inv: Transform2D = ov.get_global_transform().affine_inverse()
	var a: Vector2 = (inv * host.position - rect.position) / rect.size * Vector2(GRID_W, GRID_H)
	var b: Vector2 = (inv * host.end - rect.position) / rect.size * Vector2(GRID_W, GRID_H)
	return [maxf(0.0, minf(a.x, b.x)), minf(float(GRID_W - 1), maxf(a.x, b.x)), maxf(0.0, minf(a.y, b.y)), minf(float(GRID_H - 1), maxf(a.y, b.y))]

## Lake fixtures: rows where a run of 5..14 water cells has 3 land cells on each side, all on
## screen, nearest the visible middle first. Probe-fixture judgement: the width keeps the
## detour short enough for the guided corridor to relax, not fail.
func _lake_candidates() -> Array:
	var vis := _visible_cells()
	var out: Array = []
	var cy: float = (vis[2] + vis[3]) * 0.5
	var cx: float = (vis[0] + vis[1]) * 0.5
	for y in range(maxi(int(ceil(vis[2])), 4), mini(int(floor(vis[3])), GRID_H - 5) + 1):
		var x := maxi(int(ceil(vis[0])), 4)
		var xmax := mini(int(floor(vis[1])), GRID_W - 5)
		while x <= xmax:
			if _is_water(x, y) and not _is_water(x - 1, y):
				var e := x
				while _is_water(e + 1, y) and e + 1 <= xmax:
					e += 1
				var len := e - x + 1
				var land_l := not _is_water(x - 1, y) and not _is_water(x - 2, y) and not _is_water(x - 3, y)
				var land_r := not _is_water(e + 1, y) and not _is_water(e + 2, y) and not _is_water(e + 3, y)
				if len >= 5 and len <= 14 and land_l and land_r and x - 6 >= vis[0] and e + 6 <= vis[1]:
					out.append({"y": y, "x0": x - 5, "x1": e + 5, "score": absf(float(y) - cy) + absf(float(x + e) * 0.5 - cx) * 0.3})
				x = e + 1
			else:
				x += 1
	out.sort_custom(func(a, b): return a["score"] < b["score"])
	return out

## The L-shaped land path used by (b) and (e): 14 cells right, then 8 cells diagonally down.
## Its cells and their vertical mirror (x -> GRID_W-1-x) must all be land with a one-cell
## margin, on screen, and source and mirror must not overlap. Nearest the visible middle wins.
func _land_path() -> Array:
	var vis := _visible_cells()
	var best := 1e9
	var found: Array = []
	for y0 in range(maxi(int(ceil(vis[2])), 3), mini(int(floor(vis[3])), GRID_H - 12) + 1, 1):
		for x0 in range(maxi(int(ceil(vis[0])), 3), mini(int(floor(vis[1])), GRID_W - 28) + 1, 1):
			var vs := [Vector2(x0, y0), Vector2(x0 + 14, y0), Vector2(x0 + 22, y0 + 8)]
			if not ((x0 + 22 < GRID_W - 1 - (x0 + 22)) or (x0 > GRID_W - 1 - x0)):
				continue
			var ok := true
			for c in _cells_of(vs):
				for dx in range(-1, 2):
					for dy in range(-1, 2):
						if _is_water(c.x + dx, c.y + dy) or _is_water(GRID_W - 1 - (c.x + dx), c.y + dy):
							ok = false
							break
					if not ok:
						break
				if not ok:
					break
			if ok:
				var s := absf(float(y0 + 4) - (vis[2] + vis[3]) * 0.5) + absf(float(x0 + 11) - (vis[0] + vis[1]) * 0.5)
				if s < best:
					best = s
					found = vs
	return found

# -- A re-implementation of the engine's cell rule, for the freehand oracle ----------------
# `infra_tools_bridge.rs::way_cell`/`bresenham`/`rasterise_cells`, written again here from
# the Rust doc comments' description (js_round clamped to the grid; the all-octant integer
# Bresenham with the `>=`/`<=` tie pair; consecutive duplicate cells dropped).

func _bresenham(a: Vector2i, b: Vector2i, out: Array) -> void:
	var x := a.x
	var y := a.y
	var dx := absi(b.x - a.x)
	var dy := -absi(b.y - a.y)
	var sx := 1 if a.x < b.x else -1
	var sy := 1 if a.y < b.y else -1
	var err := dx + dy
	while true:
		if out.is_empty() or out[out.size() - 1] != Vector2i(x, y):
			out.append(Vector2i(x, y))
		if x == b.x and y == b.y:
			break
		var e2 := 2 * err
		if e2 >= dy:
			err += dy
			x += sx
		if e2 <= dx:
			err += dx
			y += sy

## True when two polylines have the same vertex count and every vertex pair is within `tol` cells.
## Exists for check (d)'s f32-vs-full-precision replay; it must stay a tolerance, never a hidden
## `==` (a different vertex count or a vertex a cell away still fails).
func _polys_close(a: PackedVector2Array, b: PackedVector2Array, tol: float) -> bool:
	if a.size() != b.size() or a.is_empty():
		return false
	for i in a.size():
		if a[i].distance_to(b[i]) > tol:
			return false
	return true

func _cells_of(pts) -> Array:
	var cells: Array = []
	var prev = null
	for p in pts:
		var c := Vector2i(clampi(_rnd(p.x), 0, GRID_W - 1), clampi(_rnd(p.y), 0, GRID_H - 1))
		if prev == null:
			cells.append(c)
		else:
			_bresenham(prev, c, cells)
		prev = c
	return cells

func _as_cells(points: PackedVector2Array) -> Array:
	var out: Array = []
	for p in points:
		out.append(Vector2i(_rnd(p.x), _rnd(p.y)))
	return out

func _dist_to_polyline(p: Vector2, poly: Array) -> float:
	var best := 1e9
	for i in range(1, poly.size()):
		var a: Vector2 = poly[i - 1]
		var b: Vector2 = poly[i]
		var cp := Geometry2D.get_closest_point_to_segment(p, a, b)
		best = minf(best, p.distance_to(cp))
	return best

# -- Reaching the Way tool through the shell's own route -------------------------------------

func _walk(root: Node, out: Array) -> void:
	for c in root.get_children():
		if c is Control and not (c as Control).is_visible_in_tree():
			continue
		if c is Control:
			out.append(c)
		_walk(c, out)

func _visible(root: Node) -> Array:
	var out: Array = []
	_walk(root, out)
	return out

func _all(n: Node, out: Array) -> void:
	for c in n.get_children():
		out.append(c)
		_all(c, out)

func _texts_of(n: Node) -> Array:
	var out: Array = []
	var all: Array = []
	_all(n, all)
	for c in all:
		if c is Label:
			out.append(String((c as Label).text))
	return out

## Desktop: the CIVIL domain's tool strip, the drawn "Way" button pressed by a real click.
## Phone: the phone menu's CIVIL screen, its drawn "Way" row tapped the way a finger does
## (the `_phonemore_act_probe` route).
func _reach_way() -> bool:
	app.arm_tool("inspect")
	await _frames(SETTLE)
	if not _phone:
		app.select_domain("civilization")
		await _frames(SETTLE)
		var btn: Button = null
		for c in _visible(app):
			if c is Button and String((c as Button).get_meta(DccWidgets.TOOL_ID_META, "")) == "way":
				btn = c
				break
		if btn == null:
			_log("DIAG _reach_way: no visible Way tool button")
			return false
		await _tap(btn.get_global_rect().get_center())
		return app.armed_tool == "way"
	var pm = app.get("_phone_menu")
	if pm == null:
		_log("DIAG _reach_way: no phone menu (run with -- --force-touch)")
		return false
	pm.open()
	pm._push_screen("civ")
	await _frames(2)
	var rows: Array = []
	_all(pm._screen_body, rows)
	var row: PanelContainer = null
	for c in rows:
		if c is PanelContainer:
			var t := _texts_of(c)
			if not t.is_empty() and String(t[0]).strip_edges() == "Way":
				row = c
				break
	if row == null:
		_log("DIAG _reach_way: no Way row on the phone civ screen")
		return false
	var down := InputEventMouseButton.new()
	down.button_index = MOUSE_BUTTON_LEFT
	down.pressed = true
	down.position = row.size * 0.5
	row.gui_input.emit(down)
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = row.size * 0.5
	row.gui_input.emit(up)
	await _frames(4)
	return app.armed_tool == "way"

# -- Main ----------------------------------------------------------------------------------------

func _ready() -> void:
	_tag = _arg("--tag", "etd4")
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
	if not _guard():
		_log("ABORT run WINDOWED")
		get_tree().quit(2)
		return
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
	_phone = app.is_phone()
	_dev = -1 if _phone else 0
	_log("viewport %dx%d phone=%s" % [_vp.size.x, _vp.size.y, _phone])
	_log("DLL mtime (unix) %d" % FileAccess.get_modified_time(ProjectSettings.globalize_path("res://").path_join("../target/debug/cartalith_godot.dll")))

	for path in ["res://map_overlay.gd", "res://shell/viewport_host.gd", "res://shell/engine_bridge.gd",
			"res://shell/workspaces/infrastructure_workspace.gd", "res://shell/workspaces/civilization_workspace.gd",
			"res://shell/workspaces/world_workspace.gd", "res://shell/app.gd"]:
		var sc = ResourceLoader.load(path, "GDScript", ResourceLoader.CACHE_MODE_IGNORE)
		_check(sc != null and sc.can_instantiate(), "(i) parses: %s" % path)

	await _generate(_seed)
	_check(bridge.has_world, "a world landed")
	if not bridge.has_world:
		_finish()
		return
	ws = (app._workspace_panels["civilization"] as Node)._infra
	var api: bool = bridge.has_method("has_way_stroke_api") and bridge.has_way_stroke_api()
	_log("ET-D4 stroke API present in shell+DLL: %s" % api)
	_check(await _reach_way(), "Way armed by the real route (%s)" % ("phone menu CIVIL row" if _phone else "tool strip button click"))
	if app.armed_tool != "way":
		_finish()
		return
	await _run(api)
	_finish()

func _run(api: bool) -> void:
	await _fresh()
	_read_water()
	_check(typeof(ws.get("_way_drag_options")) == TYPE_DICTIONARY and (ws.get("_way_drag_options") as Dictionary).is_empty(), "(seam) _way_drag_options exists and defaults to {}")

	var lakes := _lake_candidates()
	var land := _land_path()
	var tries := 0
	## Probe-fixture search (logged): the phone shows only a band of the map, so walk seeds
	## upward until the visible part holds both a lake to cross and a mirrored land path.
	while (lakes.is_empty() or land.is_empty()) and tries < 25:
		tries += 1
		_seed += 1
		_log("DIAG no fixture pair on screen; trying seed %d" % _seed)
		await _fresh()
		_read_water()
		lakes = _lake_candidates()
		land = _land_path()
	_log("DIAG %d lake candidates on screen (seed %d)" % [lakes.size(), _seed])
	_check(not lakes.is_empty(), "a lake fixture exists on screen")
	_log("DIAG land path %s" % str(land))
	_check(not land.is_empty(), "a land path (and its vertical mirror) exists on screen")
	if lakes.is_empty() or land.is_empty():
		return

	# a. routed drag round a lake ---------------------------------------------------------------
	var routed_ok := false
	var lake: Dictionary = {}
	var tried := 0
	for cand in lakes:
		if tried >= 8:
			break
		tried += 1
		lake = cand
		var y: float = float(cand["y"])
		var vs := [Vector2(cand["x0"], y), Vector2(cand["x1"], y)]
		var m0 := _manual_count()
		app.set_status("hint", "")
		await _drag_path(vs, STEP, false)
		var fed_open: PackedVector2Array = _ws_get("_way_drag_points", PackedVector2Array())
		if tried == 1:
			## Shell flag only: probing the engine here would feed it a stray sample.
			_check(bool(_ws_get("_way_drag_open", false)) and fed_open.size() >= 3, "(a) positive control: mid-drag the shell's drag is open (%d points fed)" % fed_open.size())
		await _release_now()
		var m1 := _manual_count()
		if m1 != m0 + 1:
			_log("DIAG candidate %d row %d: no way committed (hint: %s)" % [tried, int(y), _hint()])
			continue
		var w := _way(m0)
		var pts: PackedVector2Array = w["points"]
		var straight := _cells_of([vs[0], vs[1]])
		var any_water := false
		var n_wvert := 0
		var n_swater := 0
		for c in straight:
			if _is_water(c.x, c.y):
				n_swater += 1
		var maxdev := 0.0
		var maxdist := 0.0
		for p in pts:
			if _is_water(_rnd(p.x), _rnd(p.y)):
				any_water = true
				n_wvert += 1
			maxdev = maxf(maxdev, absf(p.y - y))
			maxdist = maxf(maxdist, _dist_to_polyline(p, [vs[0], vs[1]]))
		if n_wvert >= n_swater or maxdev < 1.5:
			_log("DIAG candidate %d row %d: %d water vertices vs %d straight water cells, maxdev %.2f: next; undoing" % [tried, int(y), n_wvert, n_swater, maxdev])
			bridge.way_stroke_undo()
			continue
		routed_ok = true
		_check(m1 == m0 + 1, "(a) ONE way committed (%d -> %d)" % [m0, m1])
		_check(_hint().begins_with("Way #%d committed by drag" % m0), "(a) the status line names the new way: '%s'" % _hint().left(60))
		_check(String(w.get("way_type", "")) == "road", "(a) the way has the shell's type (road)")
		## The ends may legitimately move by the engine's point snap-to-place: `civ_snap_radius(gw)`
		## = max(5, gw/70) = 5 cells at this 160-column fixture (`cartalith-civ/src/tools.rs`),
		## the reference's `_civSnapRadius` base, so a drag that ends near a settlement lands ON
		## it, up to 5.00 cells away. The bar is that radius plus half a cell of slack (a labelled
		## judgement), NOT the old 3.5, which only held while no place sat within 5 cells of an
		## end: it failed at the default 1600x900 viewport (a tool-bar row moved the fixture next to
		## a place) and passed at 1600x1000. Ends still have to be near the drag's own ends.
		const END_BAR := 5.5
		var ends_ok: bool = (pts[0] as Vector2).distance_to(vs[0]) <= END_BAR and (pts[pts.size() - 1] as Vector2).distance_to(vs[1]) <= END_BAR
		_check(ends_ok, "(a) it starts and ends at the drag's ends (%.2f, %.2f cells off)" % [(pts[0] as Vector2).distance_to(vs[0]), (pts[pts.size() - 1] as Vector2).distance_to(vs[1])])
		_check(maxdist <= 12.0, "(a) it follows the drag: farthest vertex %.2f cells from the drag line (bar 12, a probe judgement)" % maxdist)
		_check(maxdev >= 1.5 and n_wvert < n_swater, "(a) it is ROUTED round the lake: leaves the straight row by %.2f cells and has %d vertices in water against the straight drag's %d water cells (the road type may still ford a narrow stretch)" % [maxdev, n_wvert, n_swater])
		_check(_as_cells(pts) != straight, "(a) and it is not the straight raster of the drag")
		_check(fed_open.size() >= 3 and (fed_open[0] as Vector2).distance_to(vs[0]) < 0.6, "(a) the shell fed the engine the press then the samples (%d points, first at the press)" % fed_open.size())
		_check(bridge.way_stroke_undo() == 1 and _manual_count() == m0, "(a) the drag-level undo removes exactly that way")
		break
	_check(routed_ok, "(a) a routed drag round a lake committed within %d candidates" % tried)
	if not routed_ok:
		return

	# c. freehand across the same lake -------------------------------------------------------------
	var y2: float = float(lake["y"])
	await _fresh(FREEHAND)
	var mc0 := _manual_count()
	await _drag_path([Vector2(lake["x0"], y2), Vector2(lake["x1"], y2)], STEP)
	_check(_manual_count() == mc0, "(c) freehand across water pushes no way (%d)" % _manual_count())
	var hint := _hint()
	_check(hint.contains("crosses water") and hint.contains("nothing was added") and hint.contains("("), "(c) the user is told, in words, where: '%s'" % hint.left(120))
	_check(_engine_open() == -1, "(c) the refused drag is consumed: the engine drag is closed")
	await _fresh({"mode": "routed", "smooth": false, "spacing": null})
	var mc1 := _manual_count()
	await _drag_path([Vector2(lake["x0"], y2), Vector2(lake["x1"], y2)], STEP)
	_check(_manual_count() == mc1 + 1, "(c) control: the same drag routed commits a way")

	# b. freehand raster ----------------------------------------------------------------------------
	await _fresh(FREEHAND)
	var mb0 := _manual_count()
	await _drag_path(land, STEP, false)
	var fed: PackedVector2Array = _ws_get("_way_drag_points", PackedVector2Array())
	await _release_now()
	var wb := _way(mb0)
	_check(_manual_count() == mb0 + 1 and not wb.is_empty(), "(b) one freehand way committed (hint '%s')" % _hint().left(80))
	var expect := _cells_of(fed)
	var got := _as_cells(wb.get("points", PackedVector2Array()))
	_check(fed.size() > 8 and not got.is_empty() and got == expect, "(b) the committed way IS the raster of the %d fed points (%d cells expected, %d got)" % [fed.size(), expect.size(), got.size()])
	_check((wb.get("brks", PackedInt32Array()) as PackedInt32Array).is_empty(), "(b) a freehand way has no breaks")

	# e. symmetry ------------------------------------------------------------------------------------------
	await _fresh({"mode": "freehand", "smooth": false, "spacing": null, "symmetry": "vertical"})
	var me0 := _manual_count()
	await _drag_path(land, STEP)
	var me1 := _manual_count()
	_check(me1 == me0 + 2, "(e) vertical symmetry commits TWO ways (%d -> %d, hint '%s')" % [me0, me1, _hint().left(70)])
	if me1 == me0 + 2:
		var w0: PackedVector2Array = _way(me0)["points"]
		var w1: PackedVector2Array = _way(me0 + 1)["points"]
		var refl := PackedVector2Array()
		for p in w0:
			refl.append(Vector2(float(GRID_W - 1) - p.x, p.y))
		_check(w0.size() > 8 and w1 == refl, "(e) the second way is the exact reflection of the first (%d vertices each)" % w0.size())
		_check(w0 != w1, "(e) control: the two ways differ (the mirror is elsewhere)")
		_check(_hint().contains("2 ways"), "(e) the status line says two ways: '%s'" % _hint().left(60))
		_check(bridge.way_stroke_undo() == 2 and _manual_count() == me0, "(e) one drag undo removes both")
	await _fresh(FREEHAND)
	var me2 := _manual_count()
	await _drag_path(land, STEP)
	_check(_manual_count() == me2 + 1, "(e) control: without symmetry the same drag is ONE way")

	# d. clicks ----------------------------------------------------------------------------------------------
	await _fresh()
	var pa: Vector2 = land[0]
	var pb: Vector2 = land[1]
	var press_at := _screen_of(pa.x, pa.y)
	_button(press_at, true)
	await _frames(1)
	var wp_at_press := (_ws_get("_way_points", PackedVector2Array()) as PackedVector2Array).size()
	if _phone:
		_check(wp_at_press == 0, "(d) phone: the press is withheld until release, as before (waypoints at press: %d)" % wp_at_press)
	else:
		_check(wp_at_press == 1, "(d) desktop: the waypoint exists the instant of the press, as before (got %d)" % wp_at_press)
	_button(press_at, false)
	await _frames(SETTLE)
	_check((ws.get("_way_points") as PackedVector2Array).size() == 1 and _engine_open() == -1, "(d) a click is one waypoint and opens no engine drag")
	await _click_cell(pb.x, pb.y)
	_check((ws.get("_way_points") as PackedVector2Array).size() == 2, "(d) two clicks, two waypoints")
	var md0 := _manual_count()
	## The RAW clicked coordinates (pixel-rounded, so not exactly `pa`/`pb`): the reference
	## below must append what the shell appended.
	var raw_clicks: PackedVector2Array = (ws.get("_way_points") as PackedVector2Array).duplicate()
	ws.call("_commit_way")
	await _frames(2)
	var w_click := _way(md0)
	_check(_manual_count() == md0 + 1 and not w_click.is_empty(), "(d) the click way commits")
	await _fresh()
	bridge.way_begin("road")
	for rc in raw_clicks:
		bridge.way_append_point(rc.x, rc.y)
	var idx_ref := bridge.way_commit()
	var w_ref := _way(idx_ref)
	_check(idx_ref == 0 and w_ref.get("points") == w_click.get("points") and w_ref.get("km") == w_click.get("km"), "(d) the shell's two-click way equals the same calls made straight on the bridge (points and km)")

	## A drag made while a click draft exists leaves that draft as it was (the press's
	## waypoint is retracted), and the draft later commits to the way it would have made.
	var pc: Vector2 = land[2]
	await _fresh()
	await _click_cell(pa.x, pa.y)
	await _click_cell(pb.x, pb.y)
	var drag_vs := [Vector2(pc.x - 6.0, pc.y), Vector2(pc.x, pc.y)]
	var mx0 := _manual_count()
	await _drag_path(drag_vs, STEP, false)
	var fed2: PackedVector2Array = _ws_get("_way_drag_points", PackedVector2Array())
	var raw2: PackedVector2Array = (ws.get("_way_points") as PackedVector2Array).duplicate()
	var draft_during: int = raw2.size()
	await _release_now()
	_check(draft_during == 2, "(d) during the drag the click draft still holds exactly its 2 waypoints (got %d: the press's waypoint was retracted)" % draft_during)
	_check((ws.get("_way_points") as PackedVector2Array).size() == 2 and _manual_count() == mx0 + 1, "(d) after the drag: draft intact (2), one drag way committed")
	var w_drag_shell := _way(mx0)
	ws.call("_commit_way")
	await _frames(2)
	var w_draft_shell := _way(mx0 + 1)
	await _fresh()
	bridge.way_begin("road")
	for rc in raw2:
		bridge.way_append_point(rc.x, rc.y)
	var oo := {"way_type": "road", "spacing": 2.0}
	bridge.way_stroke_begin(oo)
	for p in fed2:
		bridge.way_stroke_sample(p.x, p.y, 0)
	bridge.way_stroke_end()
	bridge.way_commit()
	## Not `==`: the shell feeds the engine the unproject's FULL-precision cell coordinates, while
	## `_way_drag_points` (the replay source here) stores them rounded to f32. Where the pixel->cell
	## maths does not land on an f32-exact value (measured: 1920x1080 only; 1600x900 and 1600x1000
	## are exact) the replayed way differs by one f32 ulp of a coordinate near 100 (~1e-5 cells; the
	## failing run showed 95.50001 vs 95.5, 102.8,81.40001 vs 102.8,81.4) -- a fixture artefact, not
	## a shell/engine disagreement. The bar is 1e-3 cells (100x the ulp, far below a cell; a labelled
	## judgement), with the vertex COUNT and the km still exact-or-near.
	var w_ref_drag := _way(0)
	_check(not w_drag_shell.is_empty() and _polys_close(w_drag_shell.get("points", PackedVector2Array()), w_ref_drag.get("points", PackedVector2Array()), 1.0e-3) and absf(float(w_drag_shell.get("km", -1.0)) - float(w_ref_drag.get("km", -2.0))) < 1.0e-3, "(d) the drag's way equals the engine's own begin/sample/end over the same fed points (same vertex count, each within 1e-3 cells, km within 1e-3)")
	_check(not w_draft_shell.is_empty() and w_draft_shell.get("points") == _way(1).get("points"), "(d) the click draft committed afterwards equals the straight-bridge two-click way (nothing of the drag leaked in)")

	# f. the engine drag is closed -------------------------------------------------------------------------------
	var mid := [Vector2(land[0].x, land[0].y), Vector2(land[0].x + 8.0, land[0].y)]
	## release
	await _fresh()
	await _drag_path(mid, STEP)
	_check(_engine_open() == -1 and not bool(_ws_get("_way_drag_open", true)), "(f) closed after release (shell flag and engine)")
	## open positive control + tool change
	await _fresh()
	await _drag_path(mid, STEP, false)
	_check(bool(_ws_get("_way_drag_open", false)) and _engine_open() >= 0, "(f) control: mid-gesture the drag is open (shell flag and engine)")
	var m_f0 := _manual_count()
	app.arm_tool("inspect")
	await _frames(SETTLE)
	_check(_engine_open() == -1 and not bool(_ws_get("_way_drag_open", true)) and _manual_count() == m_f0, "(f) closed (cancelled, nothing committed) after a tool change")
	await _release_now()
	_check(_manual_count() == m_f0 and _engine_open() == -1, "(f) the stale release after a tool change adds nothing")
	app.arm_tool("way")
	await _frames(SETTLE)
	## new press
	await _drag_path(mid, STEP, false)
	_check(_engine_open() >= 0, "(f) control: open again before the new press")
	var other := _screen_of(land[1].x, land[1].y)
	_button(other, true)
	await _frames(2)
	if _phone:
		_motion(other + Vector2(2.0, 0.0), other)
	_button(other, false)
	await _frames(SETTLE)
	_check(_engine_open() == -1 and not bool(_ws_get("_way_drag_open", true)) and _manual_count() == m_f0, "(f) closed (cancelled, nothing committed) by a new press")
	## focus loss
	await _fresh()
	await _drag_path(mid, STEP, false)
	_check(_engine_open() >= 0, "(f) control: open again before the focus loss")
	get_tree().notification(Node.NOTIFICATION_APPLICATION_FOCUS_OUT)
	await _frames(2)
	var m_f1 := _manual_count()
	_check(_engine_open() == -1 and not bool(_ws_get("_way_drag_open", true)) and m_f1 == 0, "(f) closed (cancelled, nothing committed) after focus loss")
	var more := _screen_of(land[0].x + 12.0, land[0].y)
	_motion(more, _last_at)
	await _frames(2)
	_check(_engine_open() == -1, "(f) after focus loss the rest of that gesture opens nothing")
	_button(more, false)
	await _frames(SETTLE)
	_check(_manual_count() == 0 and _engine_open() == -1, "(f) and its release commits nothing")
	## Escape
	await _fresh()
	await _drag_path(mid, STEP, false)
	_check(_engine_open() >= 0, "(f) control: open again before Escape")
	app._escape_action()
	await _frames(2)
	_check(_engine_open() == -1 and not bool(_ws_get("_way_drag_open", true)) and _manual_count() == 0 and app.armed_tool == "way", "(f) closed by Escape: nothing committed, Way stays armed")
	_button(_last_at, false)
	await _frames(SETTLE)
	_check(_manual_count() == 0, "(f) the release after Escape commits nothing")
	## world change
	await _fresh()
	await _drag_path(mid, STEP, false)
	_check(_engine_open() >= 0 and bool(_ws_get("_way_drag_open", false)), "(f) control: open before the world change")
	bridge.generate({"seed": _seed + 1, "width_km": 800.0, "grid_w": GRID_W, "grid_h": GRID_H, "sea_level": 0.5, "villages": true})
	await _frames(2)
	var flag_during := bool(_ws_get("_way_drag_open", true))
	await bridge.generation_finished
	await _frames(12)
	_check(not flag_during and not bool(_ws_get("_way_drag_open", true)), "(f) the shell flag cleared at generation start (during: %s)" % flag_during)
	var eng_after_gen := _engine_open()
	_log("DIAG engine drag state right after the world change: %d (-1 = replaced with the world)" % eng_after_gen)
	_button(_last_at, false)
	await _frames(SETTLE)
	_check(_engine_open() == -1 and _manual_count() == 0, "(f) closed across a world change, and the stale release commits nothing")

	# g. generating -------------------------------------------------------------------------------------------------
	await _fresh()
	await _drag_path(mid, STEP, false)
	var p_last: Vector2 = mid[1]
	var n_open: int = bridge.world_gen.way_stroke_sample(p_last.x, p_last.y, 0)
	_check(n_open >= 0, "(g) control: an engine drag is open (%d waypoints)" % n_open)
	var pts_before: int = (_ws_get("_way_drag_points", PackedVector2Array()) as PackedVector2Array).size()
	bridge.generating = true
	ws.call("_way_drag", land[0].x + 10.0, land[0].y + 1.0)
	ws.call("_way_drag", land[0].x + 11.0, land[0].y + 3.0)
	_check((_ws_get("_way_drag_points", PackedVector2Array()) as PackedVector2Array).size() == pts_before, "(g) the shell itself ignores motion while generating (its own gate, not only the bridge's): %d points kept" % pts_before)
	ws.call("_way_release", 0.0, 0.0, true)
	await _frames(2)
	_check(bridge.way_stroke_sample(5.0, 5.0, 0) == -1 and bridge.way_stroke_end().is_empty(), "(g) the bridge wrappers refuse to touch the engine while generating")
	bridge.generating = false
	await _busy_released()
	var n_after: int = bridge.world_gen.way_stroke_sample(p_last.x, p_last.y, 0)
	_check(n_after >= 0 and n_after <= n_open + 1 and _manual_count() == 0, "(g) nothing reached the open drag while generating (count %d -> %d measured at the last drag point; a fed sample 10 cells away would add several; no way pushed)" % [n_open, n_after])
	_check(bool(_ws_get("_way_drag_stale", false)), "(g) the shell remembers it could not close the engine drag")
	var press3 := _screen_of(land[1].x, land[1].y)
	await _tap(press3)
	_check(_engine_open() == -1 and not bool(_ws_get("_way_drag_stale", true)), "(g) the next press (not generating) cancels the stale engine drag")
	await _fresh()
	await _drag_path(mid, STEP, false)
	app.arm_tool("inspect")
	await _frames(SETTLE)
	app.arm_tool("way")
	await _frames(SETTLE)
	bridge.generating = true
	app.arm_tool("inspect")
	bridge.generating = false
	await _busy_released()
	await _frames(SETTLE)

	# h. fallback and refusal -------------------------------------------------------------------------------------------
	if api:
		await _fresh()
		var NoApi = load("res://_etd4_noapi_bridge.gd")
		var nb = NoApi.new()
		nb.world_gen = bridge.world_gen
		var real_bridge = ws.bridge
		ws.bridge = nb
		var mh0 := _manual_count()
		await _drag_path(mid, STEP)
		ws.bridge = real_bridge
		nb.free()
		_check(_engine_open() == -1 and _manual_count() == mh0 and (ws.get("_way_points") as PackedVector2Array).size() == 1, "(h) fallback: no stroke API -> the old click path (1 waypoint, no drag, no way)")
		await _fresh({"mode": "bogus"})
		await _drag_path(mid, STEP)
		_check(_engine_open() == -1 and _manual_count() == 0 and (ws.get("_way_points") as PackedVector2Array).size() == 1 and _hint().contains("not started"), "(h) a refused seam option leaves the press a plain waypoint and says so: '%s'" % _hint().left(60))
	for r in ["water", "too_few_waypoints", "out_of_grid", "symmetry", "degenerate", "no_world", "something_new"]:
		var txt: String = ws.call("_way_drag_refusal_text", {"committed": false, "refusal": r, "first_x": 3, "first_y": 4, "blocked_cells": 2})
		_check(txt != "" and (r == "no_world" or txt.contains("nothing was added")), "(h) refusal '%s' has plain words: '%s'" % [r, txt.left(70)])
