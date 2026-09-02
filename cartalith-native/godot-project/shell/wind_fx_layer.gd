extends Control
class_name WindFxLayer

## Animated particle streaks over the Wind and Ocean-currents field views --
## the reference's own `#windFxCanvas` overlay (`_windFx*`, reference HTML
## lines 2113-2209), which this port had not carried across: both views
## rendered as correct but perfectly static rasters, and the owner reported
## exactly that ("the ocean current layer isnt animated as the HTML version
## is. (same for wind)", 2026-08-23).
##
## **The static rasters are untouched.** `sample_bridge.rs`'s hue-by-bearing
## Wind view and SST-anomaly Ocean view still draw underneath; this is the
## second, independent overlay the reference also keeps separate ("independent
## of the normal render pipeline (renderNow/drawLODView never touch
## #windFxCanvas)", line 2114).
##
## ## What was ported, and what deliberately differs
##
## The particle model is the reference's, constant for constant: 260 wind /
## 200 ocean particles ([`N_WIND`]/[`N_CUR`]), lifetimes `50+rand*50` and
## `60+rand*60` ticks, advection `p += vec * 0.315` per tick (the reference's
## own v1.82 "slow down the arrows ... by about 65%" figure, 0.9 * 0.35),
## respawn on leaving the map, ageing out, or -- ocean only -- drifting onto
## land, and the ocean spawner's 30-try rejection loop for a water cell.
## Colours and the `1px` stroke are the reference's too.
##
## The **trail** is where the technique differs, and it is the one place it
## does. The reference fades a persistent canvas by compositing
## `destination-out rgba(0,0,0,0.14)` every frame, so a streak is the
## accumulated history of one-segment strokes. Godot's canvas is cleared
## every frame, so reproducing that literally would mean a never-cleared
## `SubViewport` -- a retained GPU render target running whether or not
## anyone is looking at it, which is precisely the resource-lifecycle hazard
## class this project already has one live bug from (`ENUMERATION_BACKENDS`,
## `cartalith-gpu/src/multi.rs`). Instead each particle carries its last
## [`TRAIL`] positions and redraws them with the *same* geometric decay the
## fade produces (`FADE ** k`), which is visually the same streak with no
## retained target at all: `_draw()` is a handful of `draw_multiline` calls
## over plain arrays, and the whole thing costs literally nothing when off.
##
## ## The camera transform, and what it cost (2026-08-25)
##
## The owner: *"from the ocean and windcurrent visualisation it doesn't scale
## with zoom so. It doesn't show finer patterns. And also can we make the tip
## be an arrow head instead of a square pixel."* Three separate faults, all of
## them downstream of one fact this file originally treated as free -- that
## `ViewportHost._camera` magnifies everything `_draw()` emits, and nothing
## here compensated:
##
## 1. **Density.** Particles were seeded across the whole grid and the count
##    was fixed, so on a 512x384 world at eight times zoom the layer put
##    **4 of 260** streaks on screen, and at sixteen, one. Seeding into the
##    *visible* slice instead ([method _update_view]) holds it at 195-208.
## 2. **Width.** `maxf(1.0, sx)` is the reference's one-cell hairline, and one
##    cell magnified eight times is a ribbon. Divided by the camera scale now
##    (`DECISIONS.md` §7d), which is the reference's intent rather than its
##    arithmetic -- see `_draw()`.
## 3. **The tip.** `draw_multiline`'s butt caps left a flat stub, which at deep
##    zoom is the "square pixel": the ocean view at eight times zoom drew two
##    fat squares on an otherwise empty map. Arrowheads now, all of them in one
##    batched triangle array -- see the call at the end of `_draw()`.
##
## Field *sampling* was never part of it, though it is the obvious suspect:
## `_sample()` has been bilinear since this layer landed, so there was no
## stair-stepping to remove. What was missing was particles to sample with.
##
## ## Nothing runs while the layer is off
##
## `_process()` is one `debug_view()` read; if it is not `wind`/`ocean` the
## node holds no field, no particles and is `visible = false`, so `_draw()`
## is never called either. Deriving the flow field is likewise deferred to
## the first frame a view is actually up -- the same "derive when picked,
## keep nothing after" rule `sample_bridge.rs`'s own rasters follow. Polling
## rather than hooking `set_debug_layer` is the reference's own choice too
## (`_windFxStep` re-reads `state.debug` every tick and stops itself), and
## here it is strictly better: the port sets the debug view from four call
## sites, one of which is a world reload, and a missed hook would strand a
## running animation over the wrong map.

## The reference's `WINDFX_N_WIND` / `WINDFX_N_CUR` (line 2131).
const N_WIND := 260
const N_CUR := 200

## The reference's per-tick advection scale (line 2199).
const ADVECT := 0.315

## Per-frame trail decay, and how many segments stay worth drawing at it.
## `0.86` is `1 - 0.14`, the reference's own `destination-out` alpha (line
## 2190); `0.86 ** 12 == 0.16`, at which point a streak's tail has faded into
## the map and further segments buy nothing.
const FADE := 0.86
const TRAIL := 12

## `ctx.strokeStyle` for each kind (line 2193), alpha included.
const WIND_COLOR := Color(238.0 / 255.0, 244.0 / 255.0, 250.0 / 255.0, 0.55)
const CUR_COLOR := Color(127.0 / 255.0, 232.0 / 255.0, 255.0 / 255.0, 0.8)

## Must match `FLOWFX_SCALE` in `sample_bridge.rs` -- the half-range its
## 12-bit packing encodes each flow component against.
const FLOWFX_SCALE := 8.0

## Spawn margin around the visible slice, as a fraction of its larger side, so
## streaks drift in and out of frame instead of popping at the window edge.
## The margin scales with the slice, so on-screen density is unaffected by it.
const VIS_MARGIN := 0.08

## Arrowhead at the head of every live trail, in *screen* pixels -- divided
## back out by the camera scale below so it is the same visual weight at every
## zoom, the same treatment the streak width gets.
const HEAD_LEN := 7.0
const HEAD_WIDE := 4.5

var _bridge: EngineBridge
var _host: ViewportHost

var _kind := ""                  ## "wind", "ocean", or "" when idle.
var _refused := ""               ## A kind whose field this world could not answer; retried only after a regenerate.
var _data: PackedByteArray       ## The packed flow field, `flowfx:` raster, 4 bytes per grid cell.
var _fw := 0
var _fh := 0

## Particle state, parallel arrays rather than an array of dictionaries: this
## is touched 260 times a frame and a `Dictionary` per particle would allocate
## on every field read.
var _px := PackedFloat32Array()
var _py := PackedFloat32Array()
var _age := PackedInt32Array()
var _life := PackedInt32Array()
var _hist := PackedVector2Array()   ## `TRAIL + 1` slots per particle, used as a ring.
var _hlen := PackedInt32Array()     ## Valid trailing segments, reset to 0 on respawn so a streak never jumps the map.
var _slot := 0                      ## Ring head, shared: every particle advances in lockstep.

## Screen geometry, refreshed once per frame by [method _update_view] and read
## by both `_step()` (which seeds and retires against `_vis`) and `_draw()`.
var _rect := Rect2()   ## The map's displayed rect, in this control's own unscaled coordinates.
var _zoom := 1.0       ## `ViewportHost._zoom`, read off the accumulated canvas transform.
var _vis := Rect2()    ## The slice of the grid actually on screen, in cells, plus [constant VIS_MARGIN].

var _rng := RandomNumberGenerator.new()

func setup(b: EngineBridge, h: ViewportHost) -> void:
	_bridge = b
	_host = h
	set_anchors_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	visible = false
	## A regenerate or a load invalidates the field this holds (and may change
	## the grid the particles live on), so drop everything and let the next
	## frame re-derive if a flow view is still up. This is the reference's own
	## `_windFxFieldGen !== _climGen` check (line 2186), reached by signal
	## instead of a generation counter.
	b.generation_finished.connect(func(_ok: bool): _reset())
	b.world_loaded.connect(_reset)

func _reset() -> void:
	_stop()
	_refused = ""

func _process(_delta: float) -> void:
	var view: String = _host.debug_view()
	var kind := view if (view == "wind" or view == "ocean") else ""
	if kind == "":
		if _kind != "":
			_stop()
		return
	if kind != _kind:
		if kind == _refused:
			return
		if not _start(kind):
			_refused = kind
			_stop()
			return
	if not _update_view():
		return
	_step()
	queue_redraw()

## Where the map is on screen, and how much of the grid that leaves visible.
##
## `displayed_rect()` is a *letterbox fit* and never changes with zoom: pan and
## zoom are `ViewportHost._camera`'s `position`/`scale`, and this node is a
## grandchild of that camera, so the whole of `_draw()` is magnified by a
## transform it never sees. That is the entire mechanism behind the owner's
## "it doesn't scale with zoom ... doesn't show finer patterns" (2026-08-25):
## the particle set is fixed and seeded across the *whole* grid, so zooming in
## magnifies a thinner and thinner scattering of the same streaks -- fewer per
## screen, sampling the field more coarsely, which is the opposite of what
## zooming in is for. (The field read itself was never the problem; `_sample()`
## has been bilinear since the layer landed.)
##
## The fix is to make the *seeding region* the visible slice rather than the
## whole grid, so the particle count is a screen-space density instead of a
## grid-space one. At the fit view the slice is the whole grid and nothing
## changes at all -- the reference's own behaviour there, constant for
## constant. Zoomed in, the same 260 streaks resolve a smaller region, which is
## the finer pattern; it also stops advecting particles nobody can see, which
## is what pays for holding the density up.
##
## Reading the camera scale off `get_global_transform()` rather than asking
## `ViewportHost` for it keeps this node's only dependency on its parent the
## one public `displayed_rect()` it already had.
func _update_view() -> bool:
	var parent := get_parent()
	if parent == null or not parent.has_method("displayed_rect"):
		return false
	_rect = parent.displayed_rect()
	if _rect.size.x <= 0.0 or _rect.size.y <= 0.0:
		return false
	var xf := get_global_transform()
	_zoom = maxf(xf.get_scale().x, 0.0001)
	var grid := Rect2(0.0, 0.0, float(_fw), float(_fh))
	## The map area, not the whole window -- the docks own the rest of it.
	var screen: Rect2 = (_host as Control).get_global_rect() if _host is Control else get_viewport_rect()
	var local := xf.affine_inverse() * screen
	var cells := Vector2(float(_fw), float(_fh))
	var v := Rect2((local.position - _rect.position) / _rect.size * cells,
		local.size / _rect.size * cells)
	v = v.grow(maxf(v.size.x, v.size.y) * VIS_MARGIN)
	_vis = v.intersection(grid)
	## Panned clean off the map: keep the whole grid rather than seed into a
	## degenerate rect.
	if _vis.size.x <= 0.0 or _vis.size.y <= 0.0:
		_vis = grid
	return true

func _stop() -> void:
	_kind = ""
	_data = PackedByteArray()
	_px = PackedFloat32Array()
	_py = PackedFloat32Array()
	_hist = PackedVector2Array()
	visible = false

## Fetches the packed flow field and seeds the particle pool. `false` if this
## world cannot answer the field (a loaded save carries none of the substrate
## `current_wind_field` reads), which is the same condition that makes the
## static view itself unavailable.
func _start(kind: String) -> bool:
	var tex: Texture2D = _bridge.debug_texture("flowfx:" + kind)
	if tex == null:
		return false
	var img: Image = tex.get_image()
	if img == null or img.get_width() <= 0:
		return false
	_data = img.get_data()
	_fw = img.get_width()
	_fh = img.get_height()
	if _data.size() < _fw * _fh * 4:
		return false

	_kind = kind
	_refused = ""
	visible = true
	var n := N_WIND if kind == "wind" else N_CUR
	_px.resize(n)
	_py.resize(n)
	_age.resize(n)
	_life.resize(n)
	_hlen.resize(n)
	_hist.resize(n * (TRAIL + 1))
	_slot = 0
	## Before the first `_spawn`, not after: seeding reads `_vis`.
	_vis = Rect2(0.0, 0.0, float(_fw), float(_fh))
	_update_view()
	for i in range(n):
		_spawn(i, _slot)
	return true

## `_windFxSpawnWind` / `_windFxSpawnCur` (lines 2145-2154). The ocean
## spawner's 30-try rejection loop is the reference's verbatim -- a world
## that is nearly all land simply places the particle anyway on the 30th try,
## and the next `_step()` respawns it.
##
## The one departure is the *region*: `_vis` rather than the whole grid, which
## is what holds the on-screen density constant under zoom. See
## [method _update_view]. At the fit view the two are identical.
##
## ponytail: zoomed hard into an all-land bay, the ocean loop burns its 30 wet
## tests per particle every frame and never succeeds. That is 7 800 four-byte
## reads a frame, measurably nothing, and the streaks are correctly absent --
## worth a real fallback only if a profile ever says so.
func _spawn(i: int, slot: int) -> void:
	var wet_only := _kind == "ocean"
	var x := 0.0
	var y := 0.0
	for _try in range(30):
		x = _vis.position.x + _rng.randf() * _vis.size.x
		y = _vis.position.y + _rng.randf() * _vis.size.y
		if not wet_only or _wet_at(x, y):
			break
	_px[i] = x
	_py[i] = y
	_age[i] = 0
	_life[i] = int((60.0 + _rng.randf() * 60.0) if wet_only else (50.0 + _rng.randf() * 50.0))
	_hlen[i] = 0
	_hist[i * (TRAIL + 1) + slot] = Vector2(x, y)

## `_windFxStep` (lines 2182-2208), minus the canvas fade -- see this file's
## own header for why the trail is kept per-particle instead.
func _step() -> void:
	var n := _px.size()
	var next := (_slot + 1) % (TRAIL + 1)
	var wet_only := _kind == "ocean"
	for i in range(n):
		var x := _px[i] + _sample(_px[i], _py[i], 0) * ADVECT
		var y := _py[i] + _sample(_px[i], _py[i], 1) * ADVECT
		_age[i] += 1
		## Leaving the *visible* slice, not the grid -- the counterpart of
		## `_spawn`'s region, and what keeps a particle that drifts off screen
		## from being animated forever where nobody can see it.
		var gone := not _vis.has_point(Vector2(x, y)) \
			or _age[i] > _life[i] or (wet_only and not _wet_at(x, y))
		if gone:
			_spawn(i, next)
			continue
		_px[i] = x
		_py[i] = y
		_hist[i * (TRAIL + 1) + next] = Vector2(x, y)
		_hlen[i] = mini(_hlen[i] + 1, TRAIL)
	_slot = next

## Bilinear read of component `c` (0 = u, 1 = v) out of the packed raster.
## The 12/12/8 layout is `flow_fx_raster`'s (`sample_bridge.rs`); the two
## sides are round-tripped against each other by
## `flowfx_channel_round_trips_the_flow_vectors` there, which is what keeps
## this decode from silently drifting off the encode.
func _sample(x: float, y: float, c: int) -> float:
	var fx := clampf(x, 0.0, float(_fw - 1))
	var fy := clampf(y, 0.0, float(_fh - 1))
	var x0 := int(fx)
	var y0 := int(fy)
	var x1 := mini(x0 + 1, _fw - 1)
	var y1 := mini(y0 + 1, _fh - 1)
	var tx := fx - float(x0)
	var ty := fy - float(y0)
	var a := _decode(x0, y0, c)
	var b := _decode(x1, y0, c)
	var d := _decode(x0, y1, c)
	var e := _decode(x1, y1, c)
	return (a * (1.0 - tx) + b * tx) * (1.0 - ty) + (d * (1.0 - tx) + e * tx) * ty

func _decode(x: int, y: int, c: int) -> float:
	var o := (y * _fw + x) * 4
	var raw := 0
	if c == 0:
		raw = (_data[o] << 4) | (_data[o + 1] >> 4)
	else:
		raw = ((_data[o + 1] & 0xF) << 8) | _data[o + 2]
	return (float(raw) / 4095.0 * 2.0 - 1.0) * FLOWFX_SCALE

## `_windFxOceanAt` (line 2141): the alpha byte is the ocean mask, and the
## reference tests it bilinearly at `>= 0.5` rather than per-cell so a
## particle half a cell off a coast gets one consistent answer.
func _wet_at(x: float, y: float) -> bool:
	var fx := clampf(x, 0.0, float(_fw - 1))
	var fy := clampf(y, 0.0, float(_fh - 1))
	var x0 := int(fx)
	var y0 := int(fy)
	var x1 := mini(x0 + 1, _fw - 1)
	var y1 := mini(y0 + 1, _fh - 1)
	var tx := fx - float(x0)
	var ty := fy - float(y0)
	var a := float(_data[(y0 * _fw + x0) * 4 + 3])
	var b := float(_data[(y0 * _fw + x1) * 4 + 3])
	var d := float(_data[(y1 * _fw + x0) * 4 + 3])
	var e := float(_data[(y1 * _fw + x1) * 4 + 3])
	return (a * (1.0 - tx) + b * tx) * (1.0 - ty) + (d * (1.0 - tx) + e * tx) * ty >= 127.5

## One `draw_multiline` per trail depth rather than one `draw_multiline_colors`
## over everything: the fade only varies *with* depth, so grouping by it turns
## a per-segment `PackedColorArray` (another 3000 writes a frame) into twelve
## uniform colours.
##
## `_windFxProject` (line 2133) is the whole coordinate story: particles live
## in plain grid coordinates and land on screen by ratio against the map's own
## displayed rect. Taking that rect from `map_overlay.gd`'s public
## `displayed_rect()` -- this node's own parent -- rather than recomputing the
## letterbox fit is the same "let the shared thing carry it" idiom the
## reference notes at line 2123. Cached into `_rect` by [method _update_view]
## once a frame rather than fetched here, because that function needs the same
## rect to work out which slice of the grid is on screen.
##
## Pan and zoom still need no *placement* code here -- `ViewportHost._camera`'s
## transform does that -- but they are no longer free: see [method _update_view]
## for what the camera scale costs a fixed particle set, and `width` below for
## what it costs a hairline.
func _draw() -> void:
	if _kind == "" or _fw <= 0 or _rect.size.x <= 0.0 or _rect.size.y <= 0.0:
		return
	var sx := _rect.size.x / float(_fw)
	var sy := _rect.size.y / float(_fh)
	var origin := _rect.position
	## The reference strokes `lineWidth = 1` into a `GW x GH` backing canvas
	## that CSS then stretches over the map, so its hairline is one *cell* wide
	## on screen, not one pixel -- matched here rather than hardcoding 1.
	##
	## Dividing by the camera scale is a deliberate departure from that literal
	## formula, under `DECISIONS.md` §7d: the reference has no camera transform
	## at all, and here everything `_draw()` emits is magnified by one. Left
	## alone, the reference's *intent* -- a hairline -- became a ribbon eight
	## cells thick at eight times zoom, which is half of what the owner was
	## looking at when he reported the layer not scaling. The width is now
	## pinned to whatever it renders as at the fit view, at every zoom, which
	## is the intent rather than the arithmetic.
	var width := maxf(1.0, sx) / _zoom
	var base := WIND_COLOR if _kind == "wind" else CUR_COLOR
	var n := _px.size()
	var stride := TRAIL + 1
	var pts := PackedVector2Array()
	pts.resize(n * 2)
	## Arrowheads, accumulated across the `k == 0` pass and issued as one
	## triangle array below -- see that call for why it is not one polygon per
	## particle.
	var tri := PackedVector2Array()
	tri.resize(n * 3)
	var t := 0
	for k in range(TRAIL):
		var head := (_slot - k + stride) % stride
		var tail := (head - 1 + stride) % stride
		var w := 0
		for i in range(n):
			if _hlen[i] <= k:
				continue
			var a: Vector2 = _hist[i * stride + tail]
			var b: Vector2 = _hist[i * stride + head]
			var pa := origin + Vector2(a.x * sx, a.y * sy)
			var pb := origin + Vector2(b.x * sx, b.y * sy)
			pts[w] = pa
			pts[w + 1] = pb
			w += 2
			if k == 0:
				var d := pb - pa
				var dl := d.length()
				## A particle that has not moved this tick has no bearing to
				## point an arrow along; it gets one next frame.
				if dl > 0.0001:
					d /= dl
					var side := Vector2(-d.y, d.x) * (HEAD_WIDE * 0.5 / _zoom)
					var back := pb - d * (HEAD_LEN / _zoom)
					tri[t] = pb
					tri[t + 1] = back + side
					tri[t + 2] = back - side
					t += 3
		if w == 0:
			continue
		var seg := pts.slice(0, w)
		var c := base
		c.a = base.a * pow(FADE, float(k))
		draw_multiline(seg, c, width)
	## The owner's second request (2026-08-25): "can we make the tip be an arrow
	## head instead of a square pixel" -- `draw_multiline`'s segments have butt
	## caps, so a trail ended in a flat stub.
	##
	## **One call for every head in the field, not one per head.** A
	## `draw_colored_polygon` per particle would be 260 more canvas commands a
	## frame, which is precisely the mistake `map_overlay.gd`'s
	## `_draw_dashed_polyline` made and is being unwound from as this lands
	## (311 237 objects in a frame, 290 -> 501 MB of GPU buffers, because
	## buffer memory tracks object count). `canvas_item_add_triangle_array`
	## takes the whole batch as loose triangles -- an empty index array is
	## legal as long as the vertex count divides by three -- so the arrowheads
	## cost exactly **one** object, the same as one of the twelve
	## `draw_multiline` calls above. A single-entry colour array is applied to
	## every vertex, so there is no per-vertex `PackedColorArray` either.
	##
	## Issued after the loop so the heads sit over the trails, and only for
	## `k == 0`, since that is the only depth that *is* a head.
	if t > 0:
		RenderingServer.canvas_item_add_triangle_array(
			get_canvas_item(), PackedInt32Array(), tri.slice(0, t),
			PackedColorArray([base]))
