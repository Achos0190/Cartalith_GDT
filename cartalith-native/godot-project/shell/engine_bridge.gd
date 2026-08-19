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
	## `WorldParams::default()` is `false` in Rust, and stays that way: it is
	## the golden-parity reference path (`GPU_LAYER_INTEGRATION_SCOPE.md`
	## milestone 9), and the CPU-path tests pin it exactly. This is the
	## shell's own default instead -- turning on hardware the owner has
	## repeatedly asked this port to actually use, without touching the
	## engine's own notion of "default". `Preferences ▸ GPU acceleration`
	## can turn it back off before the first generate.
	if _params_available:
		param_set("use_gpu", true)
		params_dirty = false

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
	## `set_params` (`cartalith-godot/src/lib.rs`) returns `{"rejected": [...],
	## "clamped": [...]}` -- there is no "accepted" list, so a key only ever
	## appears here when something went wrong with it. `.has(key)` on that dict
	## was checking for the key as a top-level entry of `{rejected, clamped}`,
	## which is never true for any real parameter name -- every call site of
	## `param_set` has been silently getting `false` back, always, regardless
	## of whether the write succeeded (it does; `set_params` applies the value
	## to `self.params` unconditionally on `Applied`/`Clamped`, so the engine
	## state was correct the whole time -- only the GDScript-side signal that
	## depends on this return value, `mark_dirty()`, was never firing from it).
	## Found while wiring the GPU toggle's checkbox feedback, which needed the
	## real answer to `did this write land`.
	var result: Dictionary = world_gen.set_params({key: value})
	var ok: bool = not (result.get("rejected", []) as Array).has(key)
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
var last_generate_ms := 0
var _gen_start_msec := 0

func generate(request: Dictionary) -> void:
	if generating:
		return
	generating = true
	_gen_start_msec = Time.get_ticks_msec()
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
	last_generate_ms = Time.get_ticks_msec() - _gen_start_msec

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

## `LOD_TILING_INTEGRATION_SCOPE.md` milestone M1. `has_method` guards match
## `sized_api`'s own reasoning above: a binary built before this milestone
## landed simply has no `lod_synthesize_tile`, and `ViewportHost`'s deep-zoom
## compositor degrades to "off" rather than erroring against it (`0`/`null`
## are both values that method already treats as "nothing to show").
func lod_tile_cells() -> int:
	if not world_gen.has_method("lod_tile_cells"):
		return 0
	return world_gen.lod_tile_cells()

## One synthesized, coloured deep-zoom tile (`tile_x`/`tile_y` are tile-grid
## indices at `lod_tile_cells()` coarse cells each, not pixels) -- `null`
## for an out-of-range tile, before any world, or against a binary without
## this milestone's `#[func]`s.
func lod_synthesize_tile(tile_x: int, tile_y: int, detail_level: int) -> Texture2D:
	if not world_gen.has_method("lod_synthesize_tile"):
		return null
	return world_gen.lod_synthesize_tile(tile_x, tile_y, detail_level)

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

# -- Milestone F tool bindings ------------------------------------------------
#
# One thin wrapper per bound-but-unwired #[func], added together so no domain
# workspace needs to touch this file at all -- every wrapper follows sized_api's
# own established shape (`has_method` guard, safe default on an older binary),
# so a workspace built against these never crashes against a GDExtension that
# predates one specific tool's binding.

# sculpt_bridge.rs
func get_sculpt_features() -> Array:
	if not world_gen.has_method("get_sculpt_features"):
		return []
	return world_gen.get_sculpt_features()

func get_sculpt_presets() -> Array:
	if not world_gen.has_method("get_sculpt_presets"):
		return []
	return world_gen.get_sculpt_presets()

func get_sculpt_globals_info() -> Array:
	if not world_gen.has_method("get_sculpt_globals_info"):
		return []
	return world_gen.get_sculpt_globals_info()

func get_sculpt_freehand_modes() -> PackedStringArray:
	if not world_gen.has_method("get_sculpt_freehand_modes"):
		return PackedStringArray()
	return world_gen.get_sculpt_freehand_modes()

func sculpt_get_globals() -> Dictionary:
	if not world_gen.has_method("sculpt_get_globals"):
		return {}
	return world_gen.sculpt_get_globals()

func sculpt_set_globals(values: Dictionary) -> Dictionary:
	if not world_gen.has_method("sculpt_set_globals"):
		return {}
	return world_gen.sculpt_set_globals(values)

func sculpt_get_feature() -> String:
	if not world_gen.has_method("sculpt_get_feature"):
		return ""
	return world_gen.sculpt_get_feature()

func sculpt_set_feature(feature_key: String) -> bool:
	if not world_gen.has_method("sculpt_set_feature"):
		return false
	return world_gen.sculpt_set_feature(feature_key)

func sculpt_get_feature_params() -> Dictionary:
	if not world_gen.has_method("sculpt_get_feature_params"):
		return {}
	return world_gen.sculpt_get_feature_params()

func sculpt_set_feature_params(values: Dictionary) -> Dictionary:
	if not world_gen.has_method("sculpt_set_feature_params"):
		return {}
	return world_gen.sculpt_set_feature_params(values)

func sculpt_apply_preset(index: int) -> bool:
	if not world_gen.has_method("sculpt_apply_preset"):
		return false
	return world_gen.sculpt_apply_preset(index)

func sculpt_get_freehand_mode() -> String:
	if not world_gen.has_method("sculpt_get_freehand_mode"):
		return ""
	return world_gen.sculpt_get_freehand_mode()

func sculpt_set_freehand_mode(mode_key: String) -> bool:
	if not world_gen.has_method("sculpt_set_freehand_mode"):
		return false
	return world_gen.sculpt_set_freehand_mode(mode_key)

func sculpt_get_seed() -> int:
	if not world_gen.has_method("sculpt_get_seed"):
		return -1
	return world_gen.sculpt_get_seed()

func sculpt_set_seed(seed: int) -> void:
	if not world_gen.has_method("sculpt_set_seed"):
		return
	world_gen.sculpt_set_seed(seed)

func sculpt_begin_stroke() -> bool:
	if not world_gen.has_method("sculpt_begin_stroke"):
		return false
	return world_gen.sculpt_begin_stroke()

func sculpt_add_point(x: float, y: float) -> int:
	if not world_gen.has_method("sculpt_add_point"):
		return -1
	return world_gen.sculpt_add_point(x, y)

func sculpt_stroke_point_count() -> int:
	if not world_gen.has_method("sculpt_stroke_point_count"):
		return -1
	return world_gen.sculpt_stroke_point_count()

func sculpt_cancel_stroke() -> void:
	if not world_gen.has_method("sculpt_cancel_stroke"):
		return
	world_gen.sculpt_cancel_stroke()

func sculpt_end_stroke() -> int:
	if not world_gen.has_method("sculpt_end_stroke"):
		return -1
	return world_gen.sculpt_end_stroke()

func sculpt_stamp_count() -> int:
	if not world_gen.has_method("sculpt_stamp_count"):
		return -1
	return world_gen.sculpt_stamp_count()

func sculpt_list_stamps() -> Array:
	if not world_gen.has_method("sculpt_list_stamps"):
		return []
	return world_gen.sculpt_list_stamps()

func sculpt_get_selected_stamp() -> int:
	if not world_gen.has_method("sculpt_get_selected_stamp"):
		return -1
	return world_gen.sculpt_get_selected_stamp()

func sculpt_select_stamp(index: int) -> bool:
	if not world_gen.has_method("sculpt_select_stamp"):
		return false
	return world_gen.sculpt_select_stamp(index)

func sculpt_set_stamp_hidden(index: int, hidden: bool) -> bool:
	if not world_gen.has_method("sculpt_set_stamp_hidden"):
		return false
	return world_gen.sculpt_set_stamp_hidden(index, hidden)

func sculpt_move_stamp_up(index: int) -> bool:
	if not world_gen.has_method("sculpt_move_stamp_up"):
		return false
	return world_gen.sculpt_move_stamp_up(index)

func sculpt_move_stamp_down(index: int) -> bool:
	if not world_gen.has_method("sculpt_move_stamp_down"):
		return false
	return world_gen.sculpt_move_stamp_down(index)

func sculpt_delete_stamp(index: int) -> bool:
	if not world_gen.has_method("sculpt_delete_stamp"):
		return false
	return world_gen.sculpt_delete_stamp(index)

func sculpt_can_undo() -> bool:
	if not world_gen.has_method("sculpt_can_undo"):
		return false
	return world_gen.sculpt_can_undo()

func sculpt_can_redo() -> bool:
	if not world_gen.has_method("sculpt_can_redo"):
		return false
	return world_gen.sculpt_can_redo()

func sculpt_undo() -> bool:
	if not world_gen.has_method("sculpt_undo"):
		return false
	return world_gen.sculpt_undo()

func sculpt_redo() -> bool:
	if not world_gen.has_method("sculpt_redo"):
		return false
	return world_gen.sculpt_redo()

func build_sculpt_preview_texture() -> Texture2D:
	if not world_gen.has_method("build_sculpt_preview_texture"):
		return null
	return world_gen.build_sculpt_preview_texture()

func sculpt_commit(reason: String) -> Dictionary:
	if not world_gen.has_method("sculpt_commit"):
		return {}
	return world_gen.sculpt_commit(reason)

func sculpt_discard() -> int:
	if not world_gen.has_method("sculpt_discard"):
		return -1
	return world_gen.sculpt_discard()


# icon_bridge.rs
func icon_arm(family: String, variant: int, scale: float, rotation: float, jitter: float) -> bool:
	if not world_gen.has_method("icon_arm"):
		return false
	return world_gen.icon_arm(family, variant, scale, rotation, jitter)

func icon_armed() -> Dictionary:
	if not world_gen.has_method("icon_armed"):
		return {}
	return world_gen.icon_armed()

func icon_disarm() -> void:
	if not world_gen.has_method("icon_disarm"):
		return
	world_gen.icon_disarm()

func icon_place(gx: float, gy: float) -> int:
	if not world_gen.has_method("icon_place"):
		return -1
	return world_gen.icon_place(gx, gy)

func icon_hit_test(gx: float, gy: float) -> int:
	if not world_gen.has_method("icon_hit_test"):
		return -1
	return world_gen.icon_hit_test(gx, gy)

func icon_resize(index: int, cx: float, cy: float, gx: float, gy: float, start_dist: float) -> bool:
	if not world_gen.has_method("icon_resize"):
		return false
	return world_gen.icon_resize(index, cx, cy, gx, gy, start_dist)

func icon_get(index: int) -> Dictionary:
	if not world_gen.has_method("icon_get"):
		return {}
	return world_gen.icon_get(index)

func icon_delete(index: int) -> bool:
	if not world_gen.has_method("icon_delete"):
		return false
	return world_gen.icon_delete(index)

func icon_list() -> Array:
	if not world_gen.has_method("icon_list"):
		return []
	return world_gen.icon_list()

func icon_clear_all() -> void:
	if not world_gen.has_method("icon_clear_all"):
		return
	world_gen.icon_clear_all()


# civ_bridge.rs
func civ_pick_place_at(gx: float, gy: float) -> int:
	if not world_gen.has_method("civ_pick_place_at"):
		return -1
	return world_gen.civ_pick_place_at(gx, gy)

func civ_drop_settlement(gx: float, gy: float, kind: String, faction: int, name: String, snap_to_water: bool) -> int:
	if not world_gen.has_method("civ_drop_settlement"):
		return -1
	return world_gen.civ_drop_settlement(gx, gy, kind, faction, name, snap_to_water)

func civ_territory_paint_at(gx: float, gy: float, faction: int, radius: float, subtract: bool) -> void:
	if not world_gen.has_method("civ_territory_paint_at"):
		return
	world_gen.civ_territory_paint_at(gx, gy, faction, radius, subtract)

func civ_territory_commit() -> void:
	if not world_gen.has_method("civ_territory_commit"):
		return
	world_gen.civ_territory_commit()

func civ_territory_discard() -> void:
	if not world_gen.has_method("civ_territory_discard"):
		return
	world_gen.civ_territory_discard()

func civ_faction_territory_stats(faction: int) -> Dictionary:
	if not world_gen.has_method("civ_faction_territory_stats"):
		return {}
	return world_gen.civ_faction_territory_stats(faction)

func get_factions() -> Array:
	if not world_gen.has_method("get_factions"):
		return []
	return world_gen.get_factions()


# paint_bridge.rs
func get_paint_layers() -> PackedStringArray:
	if not world_gen.has_method("get_paint_layers"):
		return PackedStringArray()
	return world_gen.get_paint_layers()

func get_paint_palette(layer: String) -> Array:
	if not world_gen.has_method("get_paint_palette"):
		return []
	return world_gen.get_paint_palette(layer)

func paint_set_layer(layer: String) -> bool:
	if not world_gen.has_method("paint_set_layer"):
		return false
	return world_gen.paint_set_layer(layer)

func paint_set_brush(value: int, radius: float, hardness: float, softness: float, erase: bool, land_only: bool) -> Dictionary:
	if not world_gen.has_method("paint_set_brush"):
		return {}
	return world_gen.paint_set_brush(value, radius, hardness, softness, erase, land_only)

func paint_stroke_at(gx: float, gy: float) -> void:
	if not world_gen.has_method("paint_stroke_at"):
		return
	world_gen.paint_stroke_at(gx, gy)

func build_paint_preview_texture() -> Texture2D:
	if not world_gen.has_method("build_paint_preview_texture"):
		return null
	return world_gen.build_paint_preview_texture()

func paint_painted_counts() -> Dictionary:
	if not world_gen.has_method("paint_painted_counts"):
		return {}
	return world_gen.paint_painted_counts()

func paint_commit() -> Dictionary:
	if not world_gen.has_method("paint_commit"):
		return {}
	return world_gen.paint_commit()

func paint_discard() -> int:
	if not world_gen.has_method("paint_discard"):
		return -1
	return world_gen.paint_discard()


# way_bridge.rs
func way_begin(way_type: String) -> bool:
	if not world_gen.has_method("way_begin"):
		return false
	return world_gen.way_begin(way_type)

func way_append_point(gx: float, gy: float) -> bool:
	if not world_gen.has_method("way_append_point"):
		return false
	return world_gen.way_append_point(gx, gy)

func way_commit() -> int:
	if not world_gen.has_method("way_commit"):
		return -1
	return world_gen.way_commit()

func way_discard() -> void:
	if not world_gen.has_method("way_discard"):
		return
	world_gen.way_discard()


# route_bridge.rs
func route_begin(mode: String) -> bool:
	if not world_gen.has_method("route_begin"):
		return false
	return world_gen.route_begin(mode)

func route_append_stop(gx: float, gy: float) -> bool:
	if not world_gen.has_method("route_append_stop"):
		return false
	return world_gen.route_append_stop(gx, gy)

func route_commit() -> int:
	if not world_gen.has_method("route_commit"):
		return -1
	return world_gen.route_commit()

func route_discard() -> void:
	if not world_gen.has_method("route_discard"):
		return
	world_gen.route_discard()

func route_count() -> int:
	if not world_gen.has_method("route_count"):
		return 0
	return world_gen.route_count()

func route_get(index: int) -> Dictionary:
	if not world_gen.has_method("route_get"):
		return {}
	return world_gen.route_get(index)


# measure_bridge.rs
func measure_begin() -> void:
	if not world_gen.has_method("measure_begin"):
		return
	world_gen.measure_begin()

func measure_add_point(gx: float, gy: float) -> void:
	if not world_gen.has_method("measure_add_point"):
		return
	world_gen.measure_add_point(gx, gy)

func measure_result() -> Dictionary:
	if not world_gen.has_method("measure_result"):
		return {}
	return world_gen.measure_result()

func measure_clear() -> void:
	if not world_gen.has_method("measure_clear"):
		return
	world_gen.measure_clear()


# region_bridge.rs
func region_set(gx: float, gy: float, gw: float, gh: float) -> void:
	if not world_gen.has_method("region_set"):
		return
	world_gen.region_set(gx, gy, gw, gh)

func region_get() -> Dictionary:
	if not world_gen.has_method("region_get"):
		return {}
	return world_gen.region_get()

func region_clear() -> void:
	if not world_gen.has_method("region_clear"):
		return
	world_gen.region_clear()

func region_export_tiles(opts: Dictionary) -> PackedByteArray:
	if not world_gen.has_method("region_export_tiles"):
		return PackedByteArray()
	return world_gen.region_export_tiles(opts)


# label_bridge.rs
func label_create(gx: float, gy: float, text: String) -> int:
	if not world_gen.has_method("label_create"):
		return -1
	return world_gen.label_create(gx, gy, text)

func label_move(index: int, gx: float, gy: float) -> bool:
	if not world_gen.has_method("label_move"):
		return false
	return world_gen.label_move(index, gx, gy)

func label_select(index: int) -> bool:
	if not world_gen.has_method("label_select"):
		return false
	return world_gen.label_select(index)

func label_get_selected() -> int:
	if not world_gen.has_method("label_get_selected"):
		return -1
	return world_gen.label_get_selected()

func label_confirm_edit() -> void:
	if not world_gen.has_method("label_confirm_edit"):
		return
	world_gen.label_confirm_edit()

func label_cancel_edit() -> bool:
	if not world_gen.has_method("label_cancel_edit"):
		return false
	return world_gen.label_cancel_edit()

func label_get(index: int) -> Dictionary:
	if not world_gen.has_method("label_get"):
		return {}
	return world_gen.label_get(index)

func label_list() -> Array:
	if not world_gen.has_method("label_list"):
		return []
	return world_gen.label_list()

func label_set(index: int, values: Dictionary) -> Dictionary:
	if not world_gen.has_method("label_set"):
		return {}
	return world_gen.label_set(index, values)

func label_delete(index: int) -> bool:
	if not world_gen.has_method("label_delete"):
		return false
	return world_gen.label_delete(index)

func label_clear_all() -> void:
	if not world_gen.has_method("label_clear_all"):
		return
	world_gen.label_clear_all()

func label_hit_test(gx: float, gy: float) -> int:
	if not world_gen.has_method("label_hit_test"):
		return -1
	return world_gen.label_hit_test(gx, gy)

func label_handles(index: int, zoom: float) -> Dictionary:
	if not world_gen.has_method("label_handles"):
		return {}
	return world_gen.label_handles(index, zoom)

func label_glyph_layout(index: int, zoom: float, char_widths: PackedFloat64Array, total_w: float) -> Array:
	if not world_gen.has_method("label_glyph_layout"):
		return []
	return world_gen.label_glyph_layout(index, zoom, char_widths, total_w)

func label_resize_size(start_size: float, cx: float, cy: float, gx: float, gy: float, start_dist: float) -> float:
	if not world_gen.has_method("label_resize_size"):
		return 0.0
	return world_gen.label_resize_size(start_size, cx, cy, gx, gy, start_dist)

func label_rotate_deg(cx: float, cy: float, gx: float, gy: float) -> float:
	if not world_gen.has_method("label_rotate_deg"):
		return 0.0
	return world_gen.label_rotate_deg(cx, cy, gx, gy)

func label_arc_value(cx: float, cy: float, grab_angle_deg: float, side: float, gx: float, gy: float) -> float:
	if not world_gen.has_method("label_arc_value"):
		return 0.0
	return world_gen.label_arc_value(cx, cy, grab_angle_deg, side, gx, gy)


# journey_bridge.rs
func jp_options() -> Dictionary:
	if not world_gen.has_method("jp_options"):
		return {}
	return world_gen.jp_options()

func jp_default_plan() -> Dictionary:
	if not world_gen.has_method("jp_default_plan"):
		return {}
	return world_gen.jp_default_plan()

func jp_compute(request: Dictionary) -> Dictionary:
	if not world_gen.has_method("jp_compute"):
		return {}
	return world_gen.jp_compute(request)


# sample_bridge.rs
#
# `DCC_SHELL_SPEC.md` §6's Sample context and the canvas Layers popover.
# `has_method` guards match every wrapper above: a binary built before this
# landed simply has no `sample_cell`, and the dock falls back to reading "—"
# rather than erroring against it.

## Every §6 Sample field for one grid cell, in one call. `{}` before any
## generate, for an out-of-grid cell, or on an older binary. Keys whose
## backing data genuinely is not there are **omitted**, never zero-filled --
## callers must use `has()`/`get(key, null)`, not `get(key, 0.0)`.
func sample_cell(gx: int, gy: int) -> Dictionary:
	if not world_gen.has_method("sample_cell"):
		return {}
	return world_gen.sample_cell(gx, gy)

## The Layers popover's grouped menu, in the reference's own `LAYER_GROUPS`
## order. Each item carries `available`, which is false for a view this
## particular world has no input for.
func debug_layers() -> Array:
	if not world_gen.has_method("debug_layers"):
		return []
	return world_gen.debug_layers()

## One debug view as a grid-sized `Texture2D`. `null` for "off", an unknown
## id, or a view this world has no input for.
func debug_texture(view: String) -> Texture2D:
	if not world_gen.has_method("build_debug_texture"):
		return null
	return world_gen.build_debug_texture(view)


# timeline_bridge.rs
#
# `TIMELINE_SCOPE.md` §5 milestone 5's Godot-facing surface: manual timeline
# authoring, the ghost/highlight/exist-only overlay's own diff source, and the
# mechanistic collapse/recovery simulator. `has_method` guards match every
# wrapper above -- a binary built before this landed simply has no timeline
# methods at all. Milestone 6 (UI playback controls) is what will actually
# call these from a dock; nothing in this shell does yet.

## `civAddYear`: snapshots the currently-active year (never losing its live
## edits), then creates -- or jumps to, if already recorded -- an entry for
## `year`, carrying territory/settlements/ways forward from the nearest
## earlier recorded year. A no-op before any generate.
func civ_add_year(year: int) -> void:
	if not world_gen.has_method("civ_add_year"):
		return
	world_gen.civ_add_year(year)

## `civGotoYear`: moves the active-year cursor and restores `territory` from
## that year's recorded snapshot. Never touches settlements/ways. A no-op
## before any generate.
func civ_goto_year(year: int) -> void:
	if not world_gen.has_method("civ_goto_year"):
		return
	world_gen.civ_goto_year(year)

## `civRemoveYear`: deletes a recorded year. If it was the active year, falls
## back to the earliest remaining one (or year 0 if none remain). A no-op
## before any generate or for a year that was never recorded.
func civ_remove_year(year: int) -> void:
	if not world_gen.has_method("civ_remove_year"):
		return
	world_gen.civ_remove_year(year)

## The active timeline cursor (reference `civYear`). `0` before any
## generate/`civ_add_year` call.
func get_civ_year() -> int:
	if not world_gen.has_method("get_civ_year"):
		return 0
	return world_gen.get_civ_year()

## Every recorded timeline year, ascending -- the pill list's own data source.
func get_civ_timeline_years() -> PackedInt64Array:
	if not world_gen.has_method("get_civ_timeline_years"):
		return PackedInt64Array()
	return world_gen.get_civ_timeline_years()

## `_civYearDiff`: `{"present": PackedInt64Array, "removed": PackedInt64Array,
## "added": PackedInt64Array}` of settlement/way tids, diffing `year` against
## the chronologically-previous recorded year -- the ghost/highlight/
## exist-only overlay's own data source. Empty sets (not an error) before any
## generate or for an unrecorded year.
func civ_year_diff(year: int) -> Dictionary:
	if not world_gen.has_method("civ_year_diff"):
		return {}
	return world_gen.civ_year_diff(year)

## `_civRunCollapseSimulation`: runs the mechanistic collapse/recovery
## timeline simulator over the live settlements and writes one timeline entry
## per step. `request` keys (all optional): `mode` ("collapse"/"recovery"),
## `character` ("mixed"/"trade"/"disease"/"conflict", collapse-mode only),
## `severity` (float 0-1, collapse-mode only), `rate` (float, fraction/year,
## recovery-mode only), `start_year`/`duration`/`step_years` (int),
## `confirm_overwrite` (bool). If the run would overwrite already-recorded
## years, the first call (without `confirm_overwrite`) returns
## `{"ok": false, "needs_confirm": true, "clobber_years": [...]}` instead of
## running -- re-send the same request with `confirm_overwrite: true` to
## proceed. On success, the timeline cursor is left at the run's `end_year`.
func civ_run_collapse_simulation(request: Dictionary) -> Dictionary:
	if not world_gen.has_method("civ_run_collapse_simulation"):
		return {"ok": false, "error": "civ_run_collapse_simulation not available on this binary"}
	return world_gen.civ_run_collapse_simulation(request)


# travel_bridge.rs / lib.rs's Travel Library #[func] block (TRAVEL_LIBRARY_SPEC.md,
# GUI_GAP_REGISTER.md DM-15/O1). `kind` is one of "animal"/"vehicle"/"vessel"/"preset"
# throughout. `has_method` guards match every wrapper above: a binary built before this
# landed simply has no `tl_*` methods, and `travel_library_window.gd` falls back to an
# empty library rather than erroring.

## `{kind: {"total": int, "custom": int, "stock": int}}` for all four definition types.
func tl_counts() -> Dictionary:
	if not world_gen.has_method("tl_counts"):
		return {}
	return world_gen.tl_counts()

## Every entry of one definition type, stock-then-custom order, each row carrying
## `id`/`name`/`origin`/`editable`/`subtitle`/`species_key`/`validation_state`/
## `validation_missing`/`validation_conflicts`/`usage_presets`/`usage_journeys`.
func tl_list(kind: String) -> Array:
	if not world_gen.has_method("tl_list"):
		return []
	return world_gen.tl_list(kind)

## One entry's full detail -- `tl_list`'s own per-row keys plus every field
## `TRAVEL_LIBRARY_SPEC.md` §3 lists for `kind`. An unset optional field is simply
## absent from the returned Dictionary -- test `has()`, don't assume a default.
func tl_get(kind: String, id: String) -> Dictionary:
	if not world_gen.has_method("tl_get"):
		return {"ok": false}
	return world_gen.tl_get(kind, id)

## Clones `id` (stock or custom) into a new editable custom entry.
## `{"ok": true, "id": new_id}` or `{"ok": false, "error": ...}`.
func tl_duplicate(kind: String, id: String) -> Dictionary:
	if not world_gen.has_method("tl_duplicate"):
		return {"ok": false, "error": "tl_duplicate not available on this binary"}
	return world_gen.tl_duplicate(kind, id)

## A brand-new custom entry with every field unset. `{"ok": true, "id": new_id}`.
func tl_add_blank(kind: String, name: String) -> Dictionary:
	if not world_gen.has_method("tl_add_blank"):
		return {"ok": false, "error": "tl_add_blank not available on this binary"}
	return world_gen.tl_add_blank(kind, name)

## Deletes a custom entry. No-op on an unknown id or a stock one.
func tl_delete(kind: String, id: String) -> Dictionary:
	if not world_gen.has_method("tl_delete"):
		return {"ok": false}
	return world_gen.tl_delete(kind, id)

## Discards every custom entry of one kind, restoring the stock-only bootstrap.
func tl_reset_to_stock(kind: String) -> Dictionary:
	if not world_gen.has_method("tl_reset_to_stock"):
		return {"ok": false}
	return world_gen.tl_reset_to_stock(kind)

## Applies a partial `fields` Dictionary onto an existing custom entry (stock entries
## are read-only -- duplicate first). Returns `{"ok", "error", "rejected",
## "validation_state", "validation_missing", "validation_conflicts"}`.
func tl_edit(kind: String, id: String, fields: Dictionary) -> Dictionary:
	if not world_gen.has_method("tl_edit"):
		return {"ok": false, "error": "tl_edit not available on this binary", "rejected": []}
	return world_gen.tl_edit(kind, id, fields)

## "Capture party from planner": a new custom party preset from `plan`, in
## `jp_default_plan()`/`jp_compute`'s own `plan` key vocabulary.
func tl_capture_preset_from_plan(name: String, plan: Dictionary) -> Dictionary:
	if not world_gen.has_method("tl_capture_preset_from_plan"):
		return {"ok": false, "error": "tl_capture_preset_from_plan not available on this binary"}
	return world_gen.tl_capture_preset_from_plan(name, plan)


# `asset_bridge.rs` / `lib.rs`'s Asset Library #[func] block (`GUI_GAP_REGISTER.md`
# AS-01..AS-08/AS-13, DM-05). `has_method` guards match every wrapper above: a binary
# built before this landed simply has no `as_*` methods, and `asset_library_window.gd`
# falls back to an empty/disabled state rather than erroring.

## Decode `bytes` as a PNG and add it as a new item on `uid`. `{"ok": true}` or
## `{"ok": false, "error": ...}`.
func as_import_item(uid: String, item_name: String, bytes: PackedByteArray) -> Dictionary:
	if not world_gen.has_method("as_import_item"):
		return {"ok": false, "error": "as_import_item not available on this binary"}
	return world_gen.as_import_item(uid, item_name, bytes)

## Add (or return the existing) custom slot. `{"ok": true, "uid": ...}`.
func as_add_custom_slot(slot_name: String, set_name: String) -> Dictionary:
	if not world_gen.has_method("as_add_custom_slot"):
		return {"ok": false, "error": "as_add_custom_slot not available on this binary"}
	return world_gen.as_add_custom_slot(slot_name, set_name)

## Every slot in `family_key`'s registry with real fill state -- each row carries
## `uid`/`id`/`name`/`item_count`/`filled`/`has_dupe`. Empty on an older binary.
func as_family_slots(family_key: String) -> Array:
	if not world_gen.has_method("as_family_slots"):
		return []
	return world_gen.as_family_slots(family_key)

## One slot's inspector detail: id/name/family/set, tags, collections, meta fields.
func as_slot_summary(uid: String) -> Dictionary:
	if not world_gen.has_method("as_slot_summary"):
		return {"ok": false}
	return world_gen.as_slot_summary(uid)

## One item's inspector detail: name, scale/pan_x/pan_y, decoded w/h, hash.
func as_item_summary(uid: String, index: int) -> Dictionary:
	if not world_gen.has_method("as_item_summary"):
		return {"ok": false}
	return world_gen.as_item_summary(uid, index)

## A real, baked PNG thumbnail for one stored item. Empty on a miss or an older binary.
func as_thumbnail_png(uid: String, index: int, size: int) -> PackedByteArray:
	if not world_gen.has_method("as_thumbnail_png"):
		return PackedByteArray()
	return world_gen.as_thumbnail_png(uid, index, size)

## Pack-level metadata and totals: name/author/license/total_items.
func as_pack_info() -> Dictionary:
	if not world_gen.has_method("as_pack_info"):
		return {"name": "", "author": "", "license": "", "total_items": 0}
	return world_gen.as_pack_info()

## Sets the pack's name/author/license fields directly.
func as_set_pack_info(pack_name: String, author: String, license: String) -> bool:
	if not world_gen.has_method("as_set_pack_info"):
		return false
	return world_gen.as_set_pack_info(pack_name, author, license)

## Removes one item from a slot.
func as_remove_item(uid: String, index: int) -> bool:
	if not world_gen.has_method("as_remove_item"):
		return false
	return world_gen.as_remove_item(uid, index)

## Resets the whole session to a fresh, empty library.
func as_clear_library() -> bool:
	if not world_gen.has_method("as_clear_library"):
		return false
	return world_gen.as_clear_library()

## `AssetValidator.run()`'s real, ordered warning strings.
func as_validate() -> PackedStringArray:
	if not world_gen.has_method("as_validate"):
		return PackedStringArray()
	return world_gen.as_validate()

## Bakes every stored item and writes the pack `.zip` bytes.
## `{"ok": true, "name": ..., "bytes": PackedByteArray}` or `{"ok": false, "error": ...}`.
func as_export_pack_bytes() -> Dictionary:
	if not world_gen.has_method("as_export_pack_bytes"):
		return {"ok": false, "error": "as_export_pack_bytes not available on this binary"}
	return world_gen.as_export_pack_bytes()

## Compiles the current session into a pack and loads it straight into the renderer
## -- the reference's own `applyToMap()`, same bake `as_export_pack_bytes` does.
func as_apply_to_map() -> Dictionary:
	if not world_gen.has_method("as_apply_to_map"):
		return {"ok": false, "error": "as_apply_to_map not available on this binary"}
	var result: Dictionary = world_gen.as_apply_to_map()
	if bool(result.get("ok", false)):
		world_loaded.emit()
	return result

## Comma-separated `tags_csv` onto every uid in `uids`.
func as_batch_tag(uids: PackedStringArray, tags_csv: String) -> Dictionary:
	if not world_gen.has_method("as_batch_tag"):
		return {"ok": false}
	return world_gen.as_batch_tag(uids, tags_csv)

## Adds every uid in `uids` to collection `coll_name`.
func as_batch_collect(uids: PackedStringArray, coll_name: String) -> Dictionary:
	if not world_gen.has_method("as_batch_collect"):
		return {"ok": false}
	return world_gen.as_batch_collect(uids, coll_name)

## `{base}_01`, `{base}_02`, ... over `uids` in order. `remap` carries
## `old_uid -> new_uid` for every custom slot whose uid changed.
func as_batch_rename(uids: PackedStringArray, base: String) -> Dictionary:
	if not world_gen.has_method("as_batch_rename"):
		return {"ok": false, "renamed": 0, "remap": {}}
	return world_gen.as_batch_rename(uids, base)

## Clones every slot in `uids` carrying at least one item into a new custom slot.
func as_batch_duplicate(uids: PackedStringArray) -> Dictionary:
	if not world_gen.has_method("as_batch_duplicate"):
		return {"ok": false, "made": 0}
	return world_gen.as_batch_duplicate(uids)

## Custom slots in `uids` are removed entirely; frozen slots have their items cleared.
func as_batch_delete(uids: PackedStringArray) -> Dictionary:
	if not world_gen.has_method("as_batch_delete"):
		return {"ok": false, "deleted": 0}
	return world_gen.as_batch_delete(uids)
