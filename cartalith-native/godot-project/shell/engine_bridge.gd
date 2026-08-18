extends Node
class_name EngineBridge

## The one place the shell touches `WorldGen`.
##
## Every workspace, dock and menu reads world state through this node and never
## holds a `WorldGen` of its own. That is not tidiness for its own sake: the
## generate call runs on a worker thread, and a second caller reaching into the
## engine while that thread is mid-`generate_terrain` would read a half-built
## world. One owner, one thread, one set of signals.
##
## Ported from `main.gd`'s wiring rather than rewritten -- the archetype branch
## in `_worker` in particular was silently broken once (fixed in a265b2b) and
## its shape is deliberate.

signal generation_started()
signal generation_finished(ok: bool)
signal params_changed()            ## A dial moved; downstream is stale.
signal params_applied()            ## A generate landed; nothing is stale.
signal world_loaded()              ## A save or asset pack changed the world.

var world_gen: WorldGen = WorldGen.new()

var generating := false
var has_world := false
var last_summary := ""
var last_width_km := 0.0
var last_height_km := 0.0

## `generate_sized` and friends landed after the first GDExtension shipped, so
## the shell degrades to the square-only API rather than crashing against an
## older binary. `reference_grid_height` is the cheapest probe of the set.
var sized_api := false

var params_dirty := false
var _param_info: Dictionary = {}     ## key -> the info Dictionary from Rust
var _param_defaults: Dictionary = {}
var _params_available := false
var _thread: Thread

# -- Lifecycle ----------------------------------------------------------------

func _ready() -> void:
	sized_api = world_gen.has_method("generate_sized") \
		and world_gen.has_method("reference_grid_height") \
		and world_gen.has_method("get_map_height_km")
	_read_param_table()

func _exit_tree() -> void:
	if _thread != null and _thread.is_started():
		_thread.wait_to_finish()

# -- Parameters ---------------------------------------------------------------

func _read_param_table() -> void:
	if not world_gen.has_method("get_param_info"):
		return
	_param_info = world_gen.get_param_info()
	_param_defaults = world_gen.get_param_defaults()
	_params_available = not _param_info.is_empty()

func params_available() -> bool:
	return _params_available

func param_info(key: String) -> Dictionary:
	return _param_info.get(key, {})

func param_keys() -> Array:
	return _param_info.keys()

func param_groups() -> PackedStringArray:
	if world_gen.has_method("get_param_groups"):
		return world_gen.get_param_groups()
	return PackedStringArray()

func param_default(key: String):
	return _param_defaults.get(key)

func param_get(key: String):
	if not _params_available:
		return null
	var values: Dictionary = world_gen.get_params()
	return values.get(key)

## Write one parameter. The engine validates and returns the values it actually
## took, so a rejected write is visible here rather than silently ignored.
func param_set(key: String, value) -> bool:
	if not _params_available:
		return false
	var accepted: Dictionary = world_gen.set_params({key: value})
	var ok: bool = accepted.has(key)
	if ok:
		mark_dirty()
	return ok

func reset_params(keys: Array = []) -> void:
	if keys.is_empty():
		world_gen.reset_params()
	else:
		var restore := {}
		for k in keys:
			if _param_defaults.has(k):
				restore[k] = _param_defaults[k]
		world_gen.set_params(restore)
	mark_dirty()

func apply_archetype(name: String) -> bool:
	if not world_gen.has_method("apply_archetype"):
		return false
	var ok: bool = world_gen.apply_archetype(name)
	if ok:
		mark_dirty()
	return ok

func archetypes() -> PackedStringArray:
	if world_gen.has_method("get_archetypes"):
		return world_gen.get_archetypes()
	return PackedStringArray()

## Cartalith is a one-shot generator: a moved dial does not recompute a stage,
## it marks the world stale until the next full generate. §7's stale marks and
## the status bar both hang off this.
func mark_dirty() -> void:
	if params_dirty:
		return
	params_dirty = true
	params_changed.emit()

# -- Generation ---------------------------------------------------------------

## `request` carries everything the setup surface decided: seed, extent in km,
## grid dimensions, archetype (empty for Classic) and the five values that live
## outside the parameter table.
func generate(request: Dictionary) -> void:
	if generating:
		return
	generating = true
	generation_started.emit()

	if world_gen.has_method("set_experimental_flags"):
		world_gen.set_experimental_flags(
			request.get("dynamic_lithology", false),
			request.get("volcanic_provinces", false),
			request.get("wind_deflection", false),
			request.get("ocean_currents", false))
	world_gen.set_villages_enabled(request.get("villages", true))
	world_gen.set_sea_level(request.get("sea_level", 0.5))

	_thread = Thread.new()
	_thread.start(_worker.bind(
		int(request.get("seed", 0)),
		float(request.get("width_km", 1000.0)),
		int(request.get("grid_w", 2048)),
		int(request.get("grid_h", 2048)),
		String(request.get("archetype", ""))))

## Runs off the main thread. Touches only `world_gen` (plain Rust state), never
## a node.
##
## `generate_sized` and `generate_world_structure_sized` are both full,
## equally expensive `generate_terrain` calls mutating the same state -- this
## must be the ONE call site. The archetype branch is load-bearing: a non-empty
## archetype must reach `generate_world_structure_sized`, or the World shape
## choice never affects generation at all. Its bool return is the
## archetype-name check, surfaced as a real failure rather than swallowed.
func _worker(seed_value: int, width_km: float, grid_w: int, grid_h: int, archetype: String) -> void:
	var ok := true
	if archetype.is_empty():
		if sized_api:
			world_gen.generate_sized(seed_value, width_km, grid_w, grid_h)
		else:
			world_gen.generate(seed_value, width_km, grid_w)
	elif sized_api:
		ok = world_gen.generate_world_structure_sized(seed_value, width_km, grid_w, grid_h, archetype)
	else:
		ok = world_gen.generate_world_structure(seed_value, width_km, grid_w, archetype)
	_finish.call_deferred(seed_value, width_km, ok)

func _finish(seed_value: int, width_km: float, ok: bool) -> void:
	_thread.wait_to_finish()
	_thread = null
	generating = false

	if not ok or world_gen.get_width() <= 0:
		last_summary = "generate failed -- see console"
		generation_finished.emit(false)
		return

	## Read the real extent back from the engine rather than echoing what the
	## setup surface asked for: `get_map_height_km` is derived from the world
	## actually built (width_km * gh / gw), so a mismatch between this line and
	## the dialog's own readout is a real bug, and visible.
	last_width_km = world_gen.get_map_width_km() if sized_api else width_km
	last_height_km = world_gen.get_map_height_km() if sized_api else width_km
	has_world = true
	params_dirty = false
	last_summary = "%d x %d cells, %.0f x %.0f km, seed %d" % [
		world_gen.get_width(), world_gen.get_height(),
		last_width_km, last_height_km, seed_value]
	generation_finished.emit(true)
	params_applied.emit()

# -- World state readers ------------------------------------------------------

func grid_size() -> Vector2i:
	return Vector2i(world_gen.get_width(), world_gen.get_height())

func reference_grid_height(grid_w: int, world: bool) -> int:
	if sized_api:
		return world_gen.reference_grid_height(grid_w, world)
	return grid_w

func color_texture() -> Texture2D:
	return world_gen.build_color_texture()

func territory_texture() -> Texture2D:
	return world_gen.build_territory_texture()

func province_boundary_texture() -> Texture2D:
	return world_gen.build_province_boundary_texture()

func settlements() -> Array:
	return world_gen.get_settlements()

func roads() -> Array:
	return world_gen.get_roads()

func sea_routes() -> Array:
	return world_gen.get_sea_routes()

func provinces() -> Array:
	return world_gen.get_provinces()

func trade_balances() -> Array:
	return world_gen.get_trade_balances()

func explain_settlement(index: int) -> Dictionary:
	return world_gen.explain_settlement(index)

func border_inset_frac() -> float:
	return world_gen.get_border_inset_frac()

func gpu_stages_used() -> PackedStringArray:
	if world_gen.has_method("get_gpu_stages_used"):
		return world_gen.get_gpu_stages_used()
	return PackedStringArray()

func quality_tier() -> String:
	return world_gen.get_quality_tier()

func set_quality_tier(name: String) -> bool:
	return world_gen.set_quality_tier(name)

func quality_tiers() -> PackedStringArray:
	return world_gen.list_quality_tiers()

func recommended_quality_tier() -> String:
	return world_gen.get_recommended_quality_tier()

# -- Files --------------------------------------------------------------------

## `MVP_SCOPE.md` criterion 7: opens a real HTML-app `.zip` and renders that
## save's terrain. `load_save` reads the save's own stored fields directly --
## no `generate` call is involved, so nothing here goes through the worker.
func load_save(path: String) -> bool:
	var ok: bool = world_gen.load_save(path)
	if ok:
		has_world = true
		params_dirty = false
		last_width_km = world_gen.get_map_width_km() if sized_api else 0.0
		last_height_km = world_gen.get_map_height_km() if sized_api else 0.0
		last_summary = "%s -- %d x %d cells" % [
			path.get_file(), world_gen.get_width(), world_gen.get_height()]
		world_loaded.emit()
	return ok

func load_asset_pack(path: String) -> bool:
	var ok: bool = world_gen.load_asset_pack(path)
	if ok:
		world_loaded.emit()
	return ok

func has_asset_pack() -> bool:
	return world_gen.has_asset_pack()
