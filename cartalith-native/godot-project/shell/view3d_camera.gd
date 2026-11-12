extends RefCounted
## No `class_name`, deliberately: `view3d.gd` preloads this file, so it works in
## a tree whose `.godot` class cache predates 3D-2 (3D-1's reason, `view3d.gd`).

## **The 3D view's camera** (`3D_VIEWPORT_SCOPE.md` §3.9 and §3.11, milestone
## 3D-2): an orbit camera about a ground pivot and an orthographic top-down
## camera at the 2D view's scale, with the clamps §3.9 names. Pure state and
## arithmetic, no nodes, so its rules can be read and probed without a world.
##
## Units and axes: world metres, +X east (map x), +Z south (map y, so yaw 0 is
## north-up like the 2D map), +Y up. Every coordinate is held as a GDScript
## `float`, which is 64-bit: §3.11's camera-relative rule needs the subtraction
## `node_origin - camera_position` done before anything is cast to the 32-bit
## `Vector3` the renderer takes, so this file never stores a position in a
## `Vector2`/`Vector3`.
##
## What it must never do:
## - put the camera below the displayed ground (bar B4): `clamp_to_ground()`
##   is called with the drawn surface after every change;
## - let the pivot leave the world rectangle, or the pitch leave 5°..90° (§3.9);
## - write anything into the world, its save or `WorldState`: it is view state,
##   kept for the session only.

## Vertical field of view in degrees for the perspective (orbit) camera. A
## labelled judgement: Godot's own `Camera3D.fov` default of 75° makes the
## near field of an oblique terrain view tiny against its horizon; 50° is the
## figure 3D-0's prototype used (`td0_probe.gd.txt`) and its frames read well.
const FOV_DEG := 50.0
## §3.9's Orbit pitch range, in degrees: 5° is the floor (a grazing view still
## shows ground), 90° straight down.
const PITCH_MIN_DEG := 5.0
const PITCH_MAX_DEG := 90.0
## The pitch the entry tilt ends at and `frame_world()` uses (labelled
## judgement: steep enough that the pivot's surroundings stay legible, shallow
## enough that relief reads as relief).
const ORBIT_PITCH_DEG := 55.0
## The entry tilt's length in seconds (§3.9: "starts top-down at the current 2D
## view, then tilts"). Labelled judgement: long enough to be read as a motion,
## short enough not to be waited for.
const TILT_SECONDS := 0.6
## The closest view: a ground span no narrower than the 2D map's own closest
## span, `ViewportHost.ZOOM_TARGET_SPAN_KM` (5 km, the reference's
## `lodMaxZoom()`), so 3D's zoom range ends where 2D's does. Read from that
## constant by `view3d.gd`, never restated here.
var min_span_m := 5000.0
## The farthest view: the camera this many world diagonals from the pivot.
## Labelled judgement: 2.5 diagonals leaves the whole world in frame at any
## pitch with a margin, and more is empty background.
const MAX_DIST_DIAGONALS := 2.5
## §3.9's ground clearance: the camera at least `max(2 cells × 0.1,
## near-clearance)` above the drawn surface under it, a labelled judgement of
## the scope's. The near-clearance is `NEAR_CLEARANCE_NEARS` near planes, so
## the ground never enters the near plane (labelled judgement: 2).
const CLEARANCE_CELLS := 0.2
const NEAR_CLEARANCE_NEARS := 2.0
## §3.11's dynamic near plane, `clamp(0.05 · height_above_ground, 1 m, 1 km)`,
## a labelled judgement of the scope's, to be calibrated at 3D-3.
const NEAR_FRACTION := 0.05
const NEAR_MIN_M := 1.0
const NEAR_MAX_M := 1000.0
## The far plane: the farthest world corner from the camera, times this
## (labelled judgement: a 5 % margin so a corner never clips). §3.11's "the
## horizon distance, clamped to the world diagonal": a flat map has no
## horizon, so the world's own extent is the bound.
const FAR_MARGIN := 1.05
## How a pitch-only clamp gives up: raise the pitch in this many steps of
## `CLAMP_PITCH_STEP_DEG`, then back the camera off by `CLAMP_DIST_STEP` per
## step until clear. Labelled judgements, sized so a clamp always terminates
## inside one frame (40 steps of 2° reach 90° from 5°).
const CLAMP_STEPS := 40
const CLAMP_PITCH_STEP_DEG := 2.5
const CLAMP_DIST_STEP := 1.15

## The world, in metres (0 until `set_world()`).
var world_w_m := 0.0
var world_h_m := 0.0
var cell_m := 0.0

## The pivot on the ground (x east, z south) and the ground height under it.
var pivot_x := 0.0
var pivot_z := 0.0
var pivot_y := 0.0
## Orbit: yaw about +Y (0 = north up), pitch above the horizon, distance.
var yaw := 0.0
var pitch := deg_to_rad(PITCH_MAX_DEG)
var dist := 1000.0
## Top-down: orthographic, `ortho_size` metres of ground from the bottom of the
## view to the top (Godot's `Camera3D.size` with `KEEP_HEIGHT`).
var ortho := false
var ortho_size := 1000.0
## The entry tilt in progress: seconds left. 0 = none.
var tilt_left := 0.0
## **Probe seam only** (`_view3d_terrain_probe.gd`): true skips the ground
## clamp, so the probe's fly path can show it does reach below the ground
## without it (the B4 leg's positive control). Nothing in the shell sets it.
var debug_no_clamp := false

## Sets the world's size. `gw` is the coarse grid width and `cell_m` the
## engine's `map_width_km * 1000 / gw` (`WorldGen.view3d_tile()`'s `cell_m`,
## the one km-to-cell quotient), so the camera and the tiles share one scale.
func set_world(gw: int, gh: int, cell_metres: float) -> void:
	cell_m = cell_metres
	world_w_m = float(gw) * cell_m
	world_h_m = float(gh) * cell_m

func has_world() -> bool:
	return world_w_m > 0.0 and world_h_m > 0.0

func world_diagonal() -> float:
	return sqrt(world_w_m * world_w_m + world_h_m * world_h_m)

## The perspective camera's distance limits.
func min_dist() -> float:
	return min_span_m * 0.5 / tan(deg_to_rad(FOV_DEG) * 0.5)

func max_dist() -> float:
	return maxf(world_diagonal() * MAX_DIST_DIAGONALS, min_dist())

## The orthographic view's height limits (the same spans as the perspective
## camera's at its pivot).
func min_ortho() -> float:
	return min_span_m

func max_ortho() -> float:
	return maxf(world_diagonal() * MAX_DIST_DIAGONALS * 2.0 * tan(deg_to_rad(FOV_DEG) * 0.5), min_span_m)

## Ground metres from the bottom of the view to its top, at the pivot.
func view_span_m() -> float:
	if ortho:
		return ortho_size
	return 2.0 * dist * tan(deg_to_rad(FOV_DEG) * 0.5)

## Metres of ground per screen pixel at the pivot, for a view `view_h_px`
## physical pixels tall. The level the tiles are drawn at follows from it.
func metres_per_px(view_h_px: float) -> float:
	return view_span_m() / maxf(view_h_px, 1.0)

## **Entering 3D** (§3.9, Q9's answer): top-down at the 2D view's own centre and
## scale, then tilt. `centre_cell` is the cell under the 2D view's centre and
## `px_per_cell` its physical pixels per cell, both from
## `ViewportHost.view_for_3d()`; `view_h_px` this view's physical height.
func enter_from_2d(centre_cell: Vector2, px_per_cell: float, view_h_px: float, tilt: bool) -> void:
	pivot_x = float(centre_cell.x) * cell_m
	pivot_z = float(centre_cell.y) * cell_m
	_clamp_pivot()
	yaw = 0.0
	pitch = deg_to_rad(PITCH_MAX_DEG)
	ortho = true
	ortho_size = clampf(view_h_px / maxf(px_per_cell, 1e-9) * cell_m, min_ortho(), max_ortho())
	dist = clampf(ortho_size * 0.5 / tan(deg_to_rad(FOV_DEG) * 0.5), min_dist(), max_dist())
	tilt_left = TILT_SECONDS if tilt else 0.0
	if tilt:
		## A perspective camera at 90° shows the pivot's plane at the same
		## scale as the orthographic one, so the switch itself does not jump.
		ortho = false

## **⌖ in 3D** (Ruling EB item 4): the whole world in frame, orbiting at the
## default pitch, north up. `aspect` is the view's width / height.
func frame_world(aspect: float) -> void:
	if not has_world():
		return
	pivot_x = world_w_m * 0.5
	pivot_z = world_h_m * 0.5
	yaw = 0.0
	tilt_left = 0.0
	var span := maxf(world_h_m, world_w_m / maxf(aspect, 1e-3))
	if ortho:
		pitch = deg_to_rad(PITCH_MAX_DEG)
		ortho_size = clampf(span, min_ortho(), max_ortho())
		return
	pitch = deg_to_rad(ORBIT_PITCH_DEG)
	## The span at the pivot plus room for the tilt's foreshortening: a
	## labelled 1.1 margin over the straight-down figure.
	dist = clampf(span * 1.1 * 0.5 / tan(deg_to_rad(FOV_DEG) * 0.5), min_dist(), max_dist())

## **− and + in 3D** (Ruling EB item 4): the camera moves in by `factor` (> 1)
## or out by its inverse, about the pivot, as `ViewportHost.zoom_step()` zooms
## 2D about the view's centre. The step is the caller's (`ZOOM_BUTTON_STEP`).
func zoom(factor: float) -> void:
	if factor <= 0.0:
		return
	if ortho:
		ortho_size = clampf(ortho_size / factor, min_ortho(), max_ortho())
	else:
		dist = clampf(dist / factor, min_dist(), max_dist())

## Moves the pivot by a screen drag of `d` physical pixels, so the ground under
## the pointer follows it (at the pivot's depth).
func pan_px(d: Vector2, view_h_px: float) -> void:
	var m := metres_per_px(view_h_px)
	var c := cos(yaw)
	var s := sin(yaw)
	## Screen right on the ground is (cos yaw, -sin yaw); screen up is
	## (-sin yaw, -cos yaw). Dragging right moves the pivot left; dragging down
	## moves it toward screen-up.
	pivot_x += (-d.x * c - d.y * s) * m
	pivot_z += (d.x * s - d.y * c) * m
	_clamp_pivot()

## Orbits by `dyaw` and tilts by `dpitch` radians. Tilting leaves top-down for
## Orbit, since an orthographic camera has no pitch (§3.9's two modes).
func orbit(dyaw: float, dpitch: float) -> void:
	yaw = wrapf(yaw + dyaw, -PI, PI)
	if dpitch != 0.0 and ortho:
		set_ortho(false)
	pitch = clampf(pitch + dpitch, deg_to_rad(PITCH_MIN_DEG), deg_to_rad(PITCH_MAX_DEG))
	tilt_left = 0.0

## Switches between top-down (orthographic, straight down) and Orbit, keeping
## the ground scale at the pivot.
func set_ortho(on: bool) -> void:
	if on == ortho:
		return
	if on:
		ortho_size = clampf(view_span_m(), min_ortho(), max_ortho())
		pitch = deg_to_rad(PITCH_MAX_DEG)
	else:
		dist = clampf(ortho_size * 0.5 / tan(deg_to_rad(FOV_DEG) * 0.5), min_dist(), max_dist())
	ortho = on
	tilt_left = 0.0

## Advances the entry tilt by `delta` seconds; true while it is running.
func step(delta: float) -> bool:
	if tilt_left <= 0.0:
		return false
	tilt_left = maxf(tilt_left - delta, 0.0)
	var t := 1.0 - tilt_left / TILT_SECONDS
	t = t * t * (3.0 - 2.0 * t)
	pitch = lerpf(deg_to_rad(PITCH_MAX_DEG), deg_to_rad(ORBIT_PITCH_DEG), t)
	return true

## The camera's position in world metres, `[x, y, z]` as 64-bit floats. The
## orthographic camera sits straight above the pivot at `dist`, which only sets
## its depth range.
func position() -> Array:
	var cp := cos(pitch)
	return [pivot_x + dist * sin(yaw) * cp, pivot_y + dist * sin(pitch), pivot_z + dist * cos(yaw) * cp]

## The camera's rotation for `Camera3D.rotation` (Godot's YXZ order): looking
## at the pivot.
func rotation() -> Vector3:
	return Vector3(-pitch, yaw, 0.0)

## §3.9's clearance above the drawn ground, in metres, for a near plane `near`.
func clearance(near: float) -> float:
	return maxf(CLEARANCE_CELLS * cell_m, NEAR_CLEARANCE_NEARS * near)

## §3.11's near plane for the camera's height above the ground.
static func near_for(height_above_ground: float) -> float:
	return clampf(NEAR_FRACTION * height_above_ground, NEAR_MIN_M, NEAR_MAX_M)

## The far plane for a camera at `pos` (`position()`'s array): the farthest
## world corner, with `FAR_MARGIN`.
func far_for(pos: Array) -> float:
	var far := 0.0
	for c in [[0.0, 0.0], [world_w_m, 0.0], [0.0, world_h_m], [world_w_m, world_h_m]]:
		var dx: float = pos[0] - c[0]
		var dz: float = pos[2] - c[1]
		far = maxf(far, sqrt(dx * dx + dz * dz + pos[1] * pos[1]))
	return far * FAR_MARGIN

## **Bar B4: the camera never below the drawn ground.** `surface` is a
## `Callable(x_m: float, z_m: float) -> float` answering an upper bound of the
## displayed surface's height near that point (`view3d_terrain.gd`'s
## `surface_bound()`), and `pivot_ground` the same for the pivot. Raises the
## pitch first (the orbit stays about its pivot), then backs the camera off.
## Returns how many corrective steps it took (0 = already clear).
func clamp_to_ground(surface: Callable) -> int:
	pivot_y = float(surface.call(pivot_x, pivot_z))
	if debug_no_clamp:
		return 0
	var steps := 0
	while steps < CLAMP_STEPS:
		var p := position()
		var ground: float = surface.call(p[0], p[2])
		var hag: float = p[1] - ground
		if hag >= clearance(near_for(maxf(hag, 0.0))):
			return steps
		steps += 1
		if pitch < deg_to_rad(PITCH_MAX_DEG) - 1e-6:
			pitch = minf(pitch + deg_to_rad(CLAMP_PITCH_STEP_DEG), deg_to_rad(PITCH_MAX_DEG))
		else:
			dist = minf(dist * CLAMP_DIST_STEP, max_dist())
	return steps

func _clamp_pivot() -> void:
	pivot_x = clampf(pivot_x, 0.0, world_w_m)
	pivot_z = clampf(pivot_z, 0.0, world_h_m)
