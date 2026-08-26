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
signal dirty_changed(dirty: bool)  ## `world_dirty` flipped.
signal project_saved(path: String) ## A `.zip` was written.

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

## Same degrade-rather-than-crash probe for the heightmap-import pair
## (`WorldGen::import_heightmap` / `heightmap_grid_size`). The welcome
## screen and the Data manager both hide their import affordance outright
## when this is false, rather than drawing a button that cannot work.
var import_api := false

## `WorldGen::save_project` landed with FI-01; an older GDExtension has no
## writer, and every save affordance degrades to disabled rather than
## crashing -- the same probe shape `sized_api`/`import_api` established.
var save_api := false
## The caller-owned documents the last `load_save()` found in the archive,
## `{slot: json_text}` — `project_open`'s own `documents` key, verbatim, and
## empty when the archive carried none or the flat reader was used. Whoever
## owns a slot reads it from here after `world_loaded`; nothing parses it on
## the way through, because Godot's JSON floats every integer it touches.
var last_documents: Dictionary = {}

## Whether the world has changed since it was last saved or opened
## (`GUI_GAP_REGISTER.md` FI-01). Driven by the two signals this node owns:
## a finished generation, and `world_loaded` (which every world-changing
## wrapper here already emits -- the centring pass, the fjord carve, an
## applied asset pack). Cleared by `save_project()` and `load_save()`.
##
## **What it does not see**, stated rather than implied: a Milestone-F tool
## commit that mutates the world without emitting `world_loaded` leaves this
## `false`. That is why `DccApp.close_project()` prompts whenever a world
## exists rather than only when this is set -- a close is the one moment
## where under-reporting costs the user work. Autosave *does* gate on it,
## because re-writing an unchanged multi-hundred-megabyte world every few
## minutes is the worse failure there.
var world_dirty := false

var params_dirty := false
var _param_info: Dictionary = {}     ## key -> the info Dictionary from Rust
var _param_defaults: Dictionary = {}
var _params_available := false
var _thread: Thread

# -- Lifecycle ----------------------------------------------------------------

func _ready() -> void:
	sized_api = _has("generate_sized") \
		and _has("reference_grid_height") \
		and _has("get_map_height_km")
	import_api = _has("import_heightmap") \
		and _has("heightmap_grid_size")
	gpu_api = _has("gpu_enumerate_devices") \
		and _has("gpu_set_multi_mode")
	npr_api = _has("set_npr") \
		and _has("get_npr")
	appearance_api = _has("get_appearance") \
		and _has("set_appearance") \
		and _has("list_appearance_tunables") \
		and _has("reset_appearance")
	ramp_api = _has("get_color_ramp") \
		and _has("set_color_ramp") \
		and _has("list_ramp_presets") \
		and _has("load_ramp_preset")
	ramp_mode_api = _has("list_ramp_modes") \
		and _has("get_ramp_mode") \
		and _has("set_ramp_mode")
	look_api = _has("list_looks") \
		and _has("get_look") \
		and _has("set_look")
	preset_api = _has("save_appearance_preset") \
		and _has("load_appearance_preset") \
		and _has("peek_appearance_preset")
	measure_api = _has("measure_section") \
		and _has("measure_area") \
		and _has("measure_radius") \
		and _has("measure_vertical")
	## `project_save`, not `save_project`. The latter still exists and still
	## works, but since 2026-08-25 it writes the **flat interoperability
	## export** (`SAVEFILE_COMPAT.md` §1.1) -- seven root entries and no
	## project layer. Probing for it would leave every save affordance enabled
	## against a writer that silently drops the civ layer, which is worse than
	## a disabled button.
	save_api = _has("project_save")
	## Dirty tracking rides the two signals this node already emits rather
	## than being set by hand in each mutator -- see `world_dirty`.
	world_loaded.connect(func(): _set_dirty(true))
	generation_finished.connect(func(ok: bool): if ok: _set_dirty(true))
	_restore_gpu_prefs()
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

# -- The missing-binding probe ------------------------------------------------
#
# Every wrapper in this file is guarded on whether the loaded GDExtension
# actually exports the `#[func]` it is about to call, so a shell running
# against an older binary degrades to a safe default instead of crashing.
# That guard is right, and for two years it was also **silent** -- which is
# how a stale `libcartalith_godot.so` shipped to the phone on 2026-08-23 and
# ran for 21 commits with no error, no crash and a clean logcat while whole
# panels (NPR, Measure's area/radius/section, the faction roster, the city
# viewer, save/undo, the erosion passes, the debug views, GeoJSON export)
# quietly did nothing. The app looked healthy. It was 21 commits behind.
#
# So the guard now speaks. `_has()` is the single choke point every guard in
# this file goes through: it answers the same question `world_gen.has_method()`
# did, and the first time an answer is `false` it says so with
# `push_warning()`, which reaches the Godot console on desktop and `logcat`
# on Android (unlike `print()`, which this project has repeatedly found does
# not survive the Android log path -- `ANDROID_BUILD_SCOPE.md`, 2026-08-24).
#
# **Once per method name, not once per call.** Several of these wrappers are
# polled from `_process` or from a redraw, and a per-frame warning would bury
# the signal it exists to produce. `_missing_bindings` is the seen-set.
var _missing_bindings := {}

## True if the loaded GDExtension exports `method`. Warns once per name when
## it does not -- see the block comment above.
func _has(method: String) -> bool:
	if world_gen.has_method(method):
		return true
	if not _missing_bindings.has(method):
		_missing_bindings[method] = true
		push_warning(
			"Cartalith: the loaded GDExtension has no WorldGen.%s(). "
			% method
			+ "Whatever needed it is degraded to a safe default. This almost "
			+ "always means the native library is older than the shell "
			+ "(a stale libcartalith_godot.so) -- rebuild and re-export "
			+ "before treating the missing feature as a bug."
		)
	return false

## Every binding this session found missing, in the order it noticed. Empty on
## a matched shell/engine pair; anything in it is the staleness fingerprint.
func missing_bindings() -> PackedStringArray:
	return PackedStringArray(_missing_bindings.keys())

# -- Parameters ---------------------------------------------------------------

func _read_param_table() -> void:
	if not _has("get_param_info"):
		return
	_param_info = world_gen.get_param_info()
	_param_defaults = world_gen.get_param_defaults()
	_params_available = not _param_info.is_empty()
	if _params_available:
		_params_cache = world_gen.get_params()

func params_available() -> bool:
	return _params_available

func param_info(key: String) -> Dictionary:
	return _param_info.get(key, {})

func param_keys() -> Array:
	return _param_info.keys()

func param_groups() -> PackedStringArray:
	if _has("get_param_groups"):
		return world_gen.get_param_groups()
	return PackedStringArray()

func param_default(key: String):
	return _param_defaults.get(key)

## Last `get_params()` answer, so a read during a generation is served without
## reaching into an engine object the worker thread owns -- see the multi-GPU
## block's `_gpu_read` note for what happens when one does. Exact rather than
## approximate: `param_set` is the only writer and it is refused for the same
## window, so nothing can change the table while this stands in for it.
var _params_cache: Dictionary = {}

func param_get(key: String):
	if not _params_available:
		return null
	if generating:
		return _params_cache.get(key)
	var values: Dictionary = world_gen.get_params()
	_params_cache = values
	return values.get(key)

## Write one parameter. The engine validates and returns the values it actually
## took, so a rejected write is visible here rather than silently ignored.
##
## Refused outright while a generation is in flight: `generate_terrain` reads
## the table once at the start of the run, so a write landing mid-run could
## never have affected the world being built -- and reaching the `#[func]` at
## all while the worker holds the object is the `Gd<T>::bind()` failure the
## multi-GPU block documents.
func param_set(key: String, value) -> bool:
	if not _params_available or generating:
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
	if not _has("apply_archetype"):
		return false
	var ok: bool = world_gen.apply_archetype(name)
	if ok:
		mark_dirty()
	return ok

func archetypes() -> PackedStringArray:
	if _has("get_archetypes"):
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
	## PR-05, `Fallback when VRAM full ▸ Fail with error`. The check belongs
	## here rather than in Rust: `generate_terrain` returns a world, not a
	## `Result`, and "refuse and say why" is a UI act. The other two settings
	## need nothing here -- `cpu_tile_pass` is the engine silently taking its
	## existing CPU route, which is correct-by-construction.
	var vram := gpu_vram_estimate(int(request.get("grid_w", 0)), int(request.get("grid_h", 0)))
	if String(vram.get("action", "gpu")) == "fail":
		last_summary = "Refused: the %dx%d grid needs about %d MB of GPU buffers, over the %d MB VRAM budget, and Preferences ▸ Fallback when VRAM full is set to Fail with error." % [
			int(vram.get("gw", 0)), int(vram.get("gh", 0)),
			int(vram.get("estimate_mb", 0)), int(vram.get("budget_mb", 0))]
		push_warning(last_summary)
		generation_finished.emit(false)
		return
	## Snapshot the parameter table before the worker takes the engine: from
	## here until `_finish`, `param_get` answers from this and nothing reaches
	## a `#[func]` on the borrowed object.
	if _params_available:
		_params_cache = world_gen.get_params()
	generating = true
	_gen_start_msec = Time.get_ticks_msec()
	generation_started.emit()

	if _has("set_experimental_flags"):
		world_gen.set_experimental_flags(
			request.get("dynamic_lithology", false),
			request.get("volcanic_provinces", false),
			request.get("wind_deflection", false),
			request.get("ocean_currents", false))
	world_gen.set_villages_enabled(request.get("villages", true))
	_apply_civ_options(request)
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
## The two opt-in civ passes the reference gates behind its own auto-populate
## controls (`civMetropolisChk`, reference line 1409; `civRecoveryPhase`,
## line 1424). Both default OFF/Stable exactly as the reference's do, and
## both are `has_method`-guarded so a shell running against an older
## extension build degrades to the reference's own defaults rather than
## erroring -- the same shape `set_experimental_flags` above already uses.
func _apply_civ_options(request: Dictionary) -> void:
	if _has("set_metropolis_enabled"):
		world_gen.set_metropolis_enabled(bool(request.get("metropolis", false)))
	if _has("set_recovery_phase"):
		world_gen.set_recovery_phase(int(request.get("recovery_phase", 0)))
	## `civBiomeKChk` (reference line 1406 / `_biomeK` line 6441), the third
	## one -- default off, same guard, added 2026-08-23 (`PARITY_AUDIT.md`
	## §5 item 12: the engine parameter always existed, nothing could set it).
	if _has("set_biome_k_enabled"):
		world_gen.set_biome_k_enabled(bool(request.get("biome_k", false)))


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

## The third way into a world, alongside `generate()` and `load_save()`:
## bring in a PNG heightmap and let the engine infer a tectonic substrate
## under it (`WorldGen::import_heightmap`, the reference's own `Import ▸ Load
## heightmap…` + `Infer tectonics from heightmap` pair).
##
## Threaded and signalled exactly like `generate()`, because it *is* a
## generate-scale call -- the inversion is followed by the full climate and
## flow pipeline. Reusing `generation_started`/`generation_finished` means
## the status bar, the busy state and the `generating` guard all work with
## no extra wiring, and the shell can never start an import while a
## generation is in flight.
##
## `grid_h` is deliberately absent from `request`: the engine derives it from
## the image's own aspect ratio (see `import_heightmap`'s Rust doc), and
## `grid_size()` reports what was actually used.
func import_heightmap(path: String, request: Dictionary) -> void:
	if generating or not import_api:
		return
	## Snapshot the parameter table before the worker takes the engine: from
	## here until `_finish`, `param_get` answers from this and nothing reaches
	## a `#[func]` on the borrowed object.
	if _params_available:
		_params_cache = world_gen.get_params()
	generating = true
	_gen_start_msec = Time.get_ticks_msec()
	generation_started.emit()

	world_gen.set_villages_enabled(request.get("villages", true))
	_apply_civ_options(request)
	world_gen.set_sea_level(request.get("sea_level", 0.5))

	_thread = Thread.new()
	_thread.start(_import_worker.bind(
		path,
		int(request.get("seed", 0)),
		float(request.get("width_km", 1000.0)),
		int(request.get("grid_w", 2048))))

## Runs off the main thread. Touches only `world_gen`, never a node -- same
## contract `_worker` above holds to.
func _import_worker(path: String, seed_value: int, width_km: float, grid_w: int) -> void:
	var ok: bool = world_gen.import_heightmap(path, seed_value, width_km, grid_w)
	_finish_import.call_deferred(path, seed_value, ok)

func _finish_import(path: String, seed_value: int, ok: bool) -> void:
	_thread.wait_to_finish()
	_thread = null
	generating = false
	last_generate_ms = Time.get_ticks_msec() - _gen_start_msec

	if not ok or world_gen.get_width() <= 0:
		last_summary = "heightmap import failed -- see console"
		generation_finished.emit(false)
		return

	last_width_km = world_gen.get_map_width_km() if sized_api else 0.0
	last_height_km = world_gen.get_map_height_km() if sized_api else 0.0
	has_world = true
	params_dirty = false
	## Names the source file, because "which heightmap is this?" is the first
	## question an imported world raises and nothing else on screen answers
	## it.
	last_summary = "%s -- %d x %d cells, %.0f x %.0f km, inferred tectonics" % [
		path.get_file(),
		world_gen.get_width(), world_gen.get_height(),
		last_width_km, last_height_km]
	generation_finished.emit(true)
	params_applied.emit()
	world_loaded.emit()

## What `import_heightmap` would resample a given image onto, for a dialog
## that wants to show the working grid before committing. Returns
## `Vector2i.ZERO` when the extension predates the import API.
func heightmap_grid_size(grid_w: int, image_size: Vector2i) -> Vector2i:
	if not import_api:
		return Vector2i.ZERO
	return world_gen.heightmap_grid_size(grid_w, image_size.x, image_size.y)

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
func lod_level_for_zoom(px_per_cell: float) -> int:
	if not _has("lod_level_for_zoom"):
		return 0
	return world_gen.lod_level_for_zoom(px_per_cell)

## Tiles per axis at pyramid level `z` (`2^z`). `0` against a binary built
## before this milestone, which `ViewportHost._update_lod()` reads as "stay
## Z1-only" -- the same degrade-cleanly contract the retired
## `lod_tile_cells()` had.
func lod_tiles_per_axis(z: int) -> int:
	if not _has("lod_tiles_per_axis"):
		return 0
	return world_gen.lod_tiles_per_axis(z)

## One synthesized deep-zoom tile, addressed as the reference's own pyramid
## chunk `(z, col, row)` -- the same address the bake stores under. `null`
## for an out-of-range chunk, before any world, or against a binary without
## this milestone's `#[func]`s.
func lod_synthesize_tile(z: int, col: int, row: int) -> Texture2D:
	if not _has("lod_synthesize_tile"):
		return null
	return world_gen.lod_synthesize_tile(z, col, row)

func territory_texture() -> Texture2D:
	return world_gen.build_territory_texture()

## `state.viz.territoryOpacity` -- how heavily the territory wash is laid over
## the map (`GUI_GAP_REGISTER.md` CA-17). Takes effect on the next
## `territory_texture()`; the caller repaints.
func set_territory_opacity(a: float) -> void:
	if not _has("set_territory_opacity"):
		return
	world_gen.set_territory_opacity(a)

func territory_opacity() -> float:
	if not _has("territory_opacity"):
		return 82.0 / 255.0
	return world_gen.territory_opacity()

func territory_opacity_default() -> float:
	if not _has("territory_opacity_default"):
		return 82.0 / 255.0
	return world_gen.territory_opacity_default()

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

## Town layouts for the given settlement indices (`urban_bridge.rs`,
## `URBAN_MORPHOLOGY_SCOPE.md` milestones 1-7 and 12). Shorter than `indices`
## whenever the engine refuses a settlement -- a pin in open water gets no
## town, which is the reference's own `_umModelFor` refusal. Each entry
## carries its own `index` back, and its `stages` array names which generator
## stages produced it. Milestone 12 added `blocks`, `parcels`, `parcel_tone`
## and `parcel_cls`; there is still no `buildings` or `wall` key at all,
## because milestones 13 and 10 do not exist and an empty array would read as
## "this town has none". Empty against a binary built before this landed, the
## same `has_method` guard `lod_level_for_zoom()` above uses for its own
## milestone.
func urban_layouts(indices: PackedInt32Array) -> Array:
	if not _has("urban_layouts"):
		return []
	return world_gen.urban_layouts(indices)

func explain_settlement(index: int) -> Dictionary:
	return world_gen.explain_settlement(index)

func border_inset_frac() -> float:
	return world_gen.get_border_inset_frac()

func gpu_stages_used() -> PackedStringArray:
	if _has("get_gpu_stages_used"):
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

# -- The named look (2026-08-24) ----------------------------------------------
#
# A fourth capability flag rather than a widening of `appearance_api`, for the
# reason `ramp_api` and `ramp_mode_api` are third and second: this landed after
# those binaries shipped, and a shell running against an older `.dll` should
# lose the picker rather than crash on a missing method.
#
# The look is the appearance **base** -- colour, chroma, light shaping and
# grade, layered over the quality tier -- so it is a separate authority from
# both the tier and the user's own slider overrides, and all three survive each
# other.
var look_api := false

## `["Quality tier", "Natural Vibrant", "Antique Parchment"]` -- the engine's
## own named looks. `Array`, not `PackedStringArray`, for the reason
## `ramp_presets()` already converts: the panel feeds it straight to
## `DccWidgets.choice`, which takes an `Array`.
func looks() -> Array:
	if not look_api:
		return []
	return Array(world_gen.list_looks())

func look() -> String:
	if not look_api:
		return ""
	return world_gen.get_look()

func set_look(name: String) -> bool:
	if not look_api:
		return false
	return world_gen.set_look(name)

# -- NPR / "Painter" styles (`GUI_GAP_REGISTER.md` RN-01) ---------------------
#
# `has_method` guards for the same reason every wrapper above has one: these
# landed after earlier GDExtension binaries shipped, and the RENDER panel
# should disable its rows against an older `.dll` rather than crash on a
# missing method. `npr_api` is what the panel reads to decide that.
var npr_api := false

func npr_settings() -> Dictionary:
	if not npr_api:
		return {}
	return world_gen.get_npr()

## Send one or more changed keys; the rest keep their current value. Returns
## how many keys the engine recognised, so a typo reads as 0 rather than as a
## silent no-op.
func set_npr(values: Dictionary) -> int:
	if not npr_api:
		return 0
	return world_gen.set_npr(values)


# -- Terrain appearance (`GUI_GAP_REGISTER.md` CA-01/PR-09) --------------------
#
# The colour/relief half of the reference's Cartography ▸ Map view and
# Rendering-advanced blocks. Same `has_method` degrade and the same
# every-key-optional contract as the NPR pair above; `appearance_api` is what
# the CARTO panel reads to decide whether to draw the rows at all.
var appearance_api := false

## The values the engine is **actually rendering with** -- the quality tier's
## own, with any overrides already merged in. Not the override map.
func appearance() -> Dictionary:
	if not appearance_api:
		return {}
	return world_gen.get_appearance()

## `[[key, min, max, label], ...]` -- the engine's own ranges, so a panel never
## builds a slider that can send a value the engine will clamp.
func appearance_tunables() -> Array:
	if not appearance_api:
		return []
	return world_gen.list_appearance_tunables()

## Send one or more changed keys; the rest keep their current value. Returns
## how many the engine recognised (0 = a typo, not a silent no-op).
func set_appearance(values: Dictionary) -> int:
	if not appearance_api:
		return 0
	return world_gen.set_appearance(values)

## Hand every appearance value back to the active quality tier. Returns how
## many overrides were dropped.
func reset_appearance() -> int:
	if not appearance_api:
		return 0
	return world_gen.reset_appearance()


# -- Colour ramp (`GUI_GAP_REGISTER.md` CA-02) ---------------------------------
#
# A second, later capability flag rather than a widening of `appearance_api`:
# the tunable surface shipped one commit earlier, so an in-between binary has
# the sliders and not the ramp, and the panel should lose the ramp block rather
# than fail to draw the sliders.
var ramp_api := false

## `["Earth", "Elevation", ...]` -- the engine's own named ramps.
func ramp_presets() -> Array:
	if not ramp_api:
		return []
	return Array(world_gen.list_ramp_presets())

## `[[position, Color], ...]`, sorted by position. Position is relative land
## elevation: 0 at the shoreline, 1 at the world's highest point. The `Color`'s
## **alpha is the stop's own opacity** (2026-08-24) -- the row shape did not
## change when per-stop alpha landed, which is why `ramp_api` still covers both.
func color_ramp() -> Array:
	if not ramp_api:
		return []
	return world_gen.get_color_ramp()

## Replace the whole ramp. Add, delete and reorder are all this one call --
## the engine sorts by position, so dragging a stop past its neighbour *is*
## the reorder. Returns the number of stops accepted (0 = nothing changed).
func set_color_ramp(stops: Array) -> int:
	if not ramp_api:
		return 0
	return world_gen.set_color_ramp(stops)

func load_ramp_preset(name: String) -> bool:
	if not ramp_api:
		return false
	return world_gen.load_ramp_preset(name)

## `["Linear", "Ease", "Step"]` -- the engine's own interpolation modes.
##
## A third flag, for the same reason `ramp_api` is a second one: Ease/Step and
## per-stop alpha shipped a commit after the ramp itself, so an in-between
## binary has the stop editor and no mode picker, and the panel should lose the
## picker rather than fail to draw the stops.
var ramp_mode_api := false

func ramp_modes() -> Array:
	if not ramp_mode_api:
		return []
	return Array(world_gen.list_ramp_modes())

func ramp_mode() -> String:
	if not ramp_mode_api:
		return ""
	return String(world_gen.get_ramp_mode())

## `false` for a name this build does not have. The mode is a property of the
## ramp, not of a stop, and survives `set_color_ramp`.
func set_ramp_mode(name: String) -> bool:
	if not ramp_mode_api:
		return false
	return world_gen.set_ramp_mode(name)


# -- Appearance presets (`GUI_GAP_REGISTER.md` CA-08) --------------------------
#
# A named look, saved beside the project rather than inside it: a look is
# reusable across worlds, which is the whole reason to save one. See
# `WorldGen::save_appearance_preset` for why it is not a block in the `.zip`.

## Where named looks live. `user://` so it survives an export and is writable
## on Android, and one folder so the picker is a directory listing.
const PRESET_DIR := "user://appearance_presets"

var preset_api := false

## The engine takes native OS paths (`save_project`'s own convention), so every
## call here globalizes first.
func _preset_path(name: String) -> String:
	DirAccess.make_dir_recursive_absolute(PRESET_DIR)
	return ProjectSettings.globalize_path("%s/%s.json" % [PRESET_DIR, _preset_slug(name)])

## A filename that cannot escape the preset folder or collide with a shell
## quoting rule, while the *display* name inside the file stays whatever the
## user typed.
##
## Idempotent by construction (`a-z0-9_` maps to itself), which is what lets
## `save_appearance_preset` take a display name and `load_appearance_preset`
## take either that or the slug `appearance_presets()` handed back.
func _preset_slug(name: String) -> String:
	var out := ""
	for c in name.strip_edges().to_lower():
		out += c if (c >= "a" and c <= "z") or (c >= "0" and c <= "9") else "_"
	return out.strip_edges() if out != "" else "preset"

func save_appearance_preset(name: String) -> bool:
	if not preset_api:
		return false
	return world_gen.save_appearance_preset(_preset_path(name), name)

func load_appearance_preset(name: String) -> bool:
	if not preset_api:
		return false
	return world_gen.load_appearance_preset(_preset_path(name))

## `[[display name, slug], ...]` for every preset file in `PRESET_DIR`, read
## through the engine's own `peek_appearance_preset` so a stray JSON file that
## is not a Cartalith preset is skipped rather than offered.
func appearance_presets() -> Array:
	if not preset_api:
		return []
	var out: Array = []
	var dir := DirAccess.open(PRESET_DIR)
	if dir == null:
		return out
	for f in dir.get_files():
		if not f.ends_with(".json"):
			continue
		var slug := f.trim_suffix(".json")
		var display: String = world_gen.peek_appearance_preset(
			ProjectSettings.globalize_path("%s/%s" % [PRESET_DIR, f]))
		if display == "":
			continue
		out.append([display, slug])
	out.sort_custom(func(a, b): return String(a[0]).naturalnocasecmp_to(String(b[0])) < 0)
	return out


# -- Multi-GPU (`DCC_SHELL_SPEC.md` §2.5, `GUI_GAP_REGISTER.md` PR-01/02/04/05)

## Thin pass-throughs, same `has_method` degrade the sized/import APIs use:
## these landed after earlier GDExtension binaries shipped, and Preferences
## should disable its rows against an older `.dll` rather than crash on a
## missing method.
var gpu_api := false

## Why every row below is guarded on `generating` (2026-08-23 owner crash
## report, "a crash when you get higher than 2k and start changing settings
## for resources such as GPU/CPU").
##
## `generate()` runs `generate_terrain` on a `Thread`, and gdext holds the
## **whole `WorldGen`** mutably borrowed for that call's duration. Any
## `#[func]` reached from the main thread meanwhile fails its own
## `Gd<T>::bind()`: a Rust panic per call, a garbage default returned to
## GDScript, and -- because this build does not enable gdext's
## `experimental-threads`, so the borrow state is a plain non-atomic
## `Cell` -- two threads read-modify-writing the same counters, which is
## undefined behaviour rather than a mere error. Measured on a real
## non-headless run: opening Preferences ▸ Performance during one 4096x2624
## generation produced 360 `Gd<T>::bind() failed, already bound;
## T = cartalith_godot::WorldGen` panics and left the Devices submenu latched
## on an empty list ("No GPU detected") for the rest of the session. Small
## grids hide it only because they finish before anyone can reach the menu.
##
## Serving the readers from a cache is exact rather than approximate: these
## four settings live in a process-global `RwLock<GpuPreferences>` in
## `cartalith-gpu`, and this file is the only thing in the shell that writes
## them. The setters refuse outright and say so -- they take effect on the
## next generate anyway, so deferring them costs nothing, and `menus.gd`
## disables the rows for the same reason rather than letting a click no-op
## silently.
var _gpu_cache := {}

## Cached read-through. `busy_value` is what to answer while the worker owns
## the engine and nothing has been cached yet.
func _gpu_read(key: String, busy_value: Variant, fetch: Callable) -> Variant:
	if not gpu_api:
		return busy_value
	if generating:
		return _gpu_cache.get(key, busy_value)
	var v: Variant = fetch.call()
	_gpu_cache[key] = v
	return v

## True while a settings write must be refused. Public so `menus.gd` can
## disable the rows rather than draw a control that silently does nothing.
func gpu_settings_locked() -> bool:
	return gpu_api and generating

func gpu_devices() -> Array:
	return _gpu_read("devices", [], func(): return world_gen.gpu_enumerate_devices())

func gpu_selected_devices() -> PackedStringArray:
	return _gpu_read("selected", PackedStringArray(), func(): return world_gen.gpu_selected_devices())

func gpu_set_selected_devices(keys: PackedStringArray) -> void:
	if gpu_api and not generating:
		world_gen.gpu_set_selected_devices(keys)
		_gpu_cache["selected"] = keys
		DccSettings.set_gpu_devices(keys)

func gpu_multi_mode() -> String:
	return _gpu_read("mode", "single_device", func(): return String(world_gen.gpu_multi_mode()))

func gpu_set_multi_mode(mode: String) -> bool:
	if not gpu_api or generating:
		return false
	var ok: bool = world_gen.gpu_set_multi_mode(mode)
	if ok:
		_gpu_cache["mode"] = mode
		DccSettings.set_gpu_mode(mode)
	return ok

func gpu_vram_budget_gb() -> float:
	return _gpu_read("budget", 0.0, func(): return float(world_gen.gpu_vram_budget_gb()))

func gpu_set_vram_budget_gb(gb: float) -> void:
	if gpu_api and not generating:
		world_gen.gpu_set_vram_budget_gb(gb)
		_gpu_cache["budget"] = gb
		DccSettings.set_gpu_vram_budget_gb(gb)

func gpu_vram_fallback() -> String:
	return _gpu_read("fallback", "cpu_tile_pass", func(): return String(world_gen.gpu_vram_fallback()))

func gpu_set_vram_fallback(name: String) -> bool:
	if not gpu_api or generating:
		return false
	var ok: bool = world_gen.gpu_set_vram_fallback(name)
	if ok:
		_gpu_cache["fallback"] = name
		DccSettings.set_gpu_fallback(name)
	return ok

## `gw`/`gh` are the grid the **next** generate will use. Both `0` asks about
## the last generated grid instead -- which is `0x0` before the first
## generate, so callers that know the pending size should pass it.
##
## Not cached across sizes: the answer depends on the arguments, so while the
## worker owns the engine this returns `{}` and callers already handle that
## (`generate()` below defaults its action to `"gpu"`, `menus.gd` skips the
## estimate row on an empty Dictionary).
func gpu_vram_estimate(gw: int = 0, gh: int = 0) -> Dictionary:
	if not gpu_api or generating:
		return {}
	return world_gen.gpu_vram_estimate(gw, gh)

func gpu_last_device_usage() -> Array:
	return _gpu_read("usage", [], func(): return world_gen.gpu_last_device_usage())

## Push the persisted §2.5 Performance settings into the engine at startup.
##
## Order matters: devices before mode, because `split_tiles` only actually
## splits once at least two devices are selected. A device key that no longer
## resolves (a GPU removed, a settings file copied between machines) is not
## an error -- the engine degrades that selection to automatic on its own.
func _restore_gpu_prefs() -> void:
	if not gpu_api:
		return
	var keys := DccSettings.gpu_devices()
	if not keys.is_empty():
		world_gen.gpu_set_selected_devices(keys)
	var mode := DccSettings.gpu_mode()
	if mode != "":
		world_gen.gpu_set_multi_mode(mode)
	var gb := DccSettings.gpu_vram_budget_gb()
	if gb > 0.0:
		world_gen.gpu_set_vram_budget_gb(gb)
	var fb := DccSettings.gpu_fallback()
	if fb != "":
		world_gen.gpu_set_vram_fallback(fb)

# -- Files --------------------------------------------------------------------

## `MVP_SCOPE.md` criterion 7: opens a real HTML-app `.zip` and renders that
## save's terrain. `load_save` reads the save's own stored fields directly --
## no `generate` call is involved, so nothing here goes through the worker.
func load_save(path: String) -> bool:
	## `project_open`, not `world_gen.load_save`. They are two different
	## readers and the shell was calling the wrong one: `project_save` (File ▸
	## Save) writes the TREE, and `world_gen.load_save` is the FLAT HTML-format
	## reader. Measured 2026-08-26 by `_openpath_probe.gd`: save a world with 8
	## settlements, reopen it through this function, get **0** back. The terrain
	## returned and everything else -- the civilisation layer, appearance, and
	## every document -- was silently dropped, with `ok == true` and no warning.
	##
	## `project_open` reads BOTH layouts (verified in the same probe: it opens a
	## flat export and reports `layout == "flat"`), which is exactly the owner's
	## rule -- read the old format and the new one, write only the new one -- so
	## this is one call rather than a format sniff. `world_gen.load_save` stays
	## only as the fallback for a binary too old to have `project_open`.
	var documents: Dictionary = {}
	var ok := false
	if _has("project_open"):
		var r: Dictionary = world_gen.project_open(path)
		ok = bool(r.get("ok", false))
		if ok:
			documents = r.get("documents", {})
		else:
			push_warning("Cartalith: project_open could not read %s (%s) -- falling back to the flat reader"
				% [path, String(r.get("error", "unknown"))])
	if not ok:
		ok = world_gen.load_save(path)
	last_documents = documents
	if ok:
		has_world = true
		params_dirty = false
		last_width_km = world_gen.get_map_width_km() if sized_api else 0.0
		last_height_km = world_gen.get_map_height_km() if sized_api else 0.0
		last_summary = "%s -- %d x %d cells" % [
			path.get_file(), world_gen.get_width(), world_gen.get_height()]
		## A freshly opened world is by definition identical to what is on
		## disk. The clear comes *after* the emit because Godot delivers
		## signals synchronously, and the handler above sets the flag.
		world_loaded.emit()
		_set_dirty(false)
		## The dials moved to whatever the save carried, so anything reading
		## `param_get` has to re-read them.
		_read_param_table()
	return ok

## `File ▸ Save project` / `Save as…` (`GUI_GAP_REGISTER.md` FI-01) — writes
## the current world to `path` as a `.zip` in the format
## `SAVEFILE_COMPAT.md` documents. Returns `false` (leaving any existing file
## untouched) when the engine has no writer, when there is no world, or when
## the write itself fails; the engine logs the reason.
func save_project(path: String, extra_documents: Dictionary = {}) -> bool:
	if not save_api or not has_world:
		return false
	## `project_save` writes the documented tree (`SAVEFILE_COMPAT.md`) and
	## returns `{ok, error, bytes, entries}` rather than a bool, so the reason
	## a save failed can be reported instead of a bare false. The engine's own
	## `save_project` is deliberately not called here any more: it is now the
	## flat export, and a Save that quietly dropped settlements, factions,
	## ways, the timeline and the vault links would be the worst kind of
	## regression -- one the user only discovers on reopening.
	## `project_save_with_documents` when the caller has state of its own to
	## store, `project_save` otherwise -- the former is what the latter calls
	## anyway, so this is one branch for one guard rather than two paths.
	## `extra_documents` maps a registered slot to that document's JSON TEXT;
	## a Dictionary would go through Godot's JSON, which floats every integer,
	## and that is precisely how KV-04 discarded every knowledge link.
	var r: Dictionary
	if extra_documents.is_empty() or not _has("project_save_with_documents"):
		r = world_gen.project_save(path)
	else:
		r = world_gen.project_save_with_documents(path, extra_documents)
	var ok: bool = bool(r.get("ok", false))
	if ok:
		_set_dirty(false)
		project_saved.emit(path)
	else:
		push_warning("Cartalith: could not write %s (%s)" % [path, String(r.get("error", "unknown"))])
	return ok

## `File ▸ Close project` — drops the world and returns this node to the
## state it had before the first generate.
##
## The engine has no `unload`: `WorldGen` holds exactly one world for its
## whole lifetime, and every accessor on it already answers honestly *before*
## a world exists. So closing is replacing the handle, which is also the only
## way to release the field memory. The two caches that were read off the old
## instance -- the parameter table and the GPU preferences -- are re-read
## against the new one here, because a stale `_params_cache` pointing at a
## freed world is exactly the kind of bug that surfaces three dialogs later.
func close_world() -> void:
	if generating:
		return
	world_gen = WorldGen.new()
	has_world = false
	last_summary = ""
	last_width_km = 0.0
	last_height_km = 0.0
	params_dirty = false
	_restore_gpu_prefs()
	_read_param_table()
	world_loaded.emit()
	## After the emit, for the same reason `load_save` clears it there: the
	## handler above sets the flag, and an empty shell has nothing to save.
	_set_dirty(false)

func _set_dirty(value: bool) -> void:
	if world_dirty == value:
		return
	world_dirty = value
	dirty_changed.emit(value)

func load_asset_pack(path: String) -> bool:
	var ok: bool = world_gen.load_asset_pack(path)
	if ok:
		world_loaded.emit()
	return ok

func has_asset_pack() -> bool:
	return world_gen.has_asset_pack()

# -- Post-generation field operations -----------------------------------------
#
# Two opt-in passes the reference runs from a button, never during generate:
# `#centerBtn` and the Glacial panel's `#fjordBtn`. Both rewrite the height
# field in place, so both emit `world_loaded` -- the viewport, layers popover
# and right dock all already listen for it, which is exactly the "no
# regeneration needed, the render path reads the field fresh" contract the
# engine's own doc comments state. Same `has_method` guard as every other
# wrapper here, so an older GDExtension degrades instead of crashing.

## Rotate the wrapped world so the landmasses sit away from the x-seam
## (`GUI_GAP_REGISTER.md` MS-01). Returns the engine's summary dictionary:
## `ok`, `offset`, `seam_column`, and `reason` when `ok` is false.
func center_landmasses() -> Dictionary:
	if not _has("center_landmasses"):
		return {"ok": false, "reason": "This build of the engine has no centring pass."}
	var r: Dictionary = world_gen.center_landmasses()
	if bool(r.get("ok", false)) and int(r.get("offset", 0)) != 0:
		world_loaded.emit()
	return r

## Overdeepen the glacially-carvable coastal valleys into fjords. Returns
## the engine's summary dictionary: `ok`, `cells_masked`, `cells_carved`,
## and `reason` when `ok` is false.
func carve_fjords() -> Dictionary:
	if not _has("carve_fjords"):
		return {"ok": false, "reason": "This build of the engine has no fjord pass."}
	var r: Dictionary = world_gen.carve_fjords()
	if bool(r.get("ok", false)) and int(r.get("cells_carved", 0)) > 0:
		world_loaded.emit()
	return r

# -- Milestone F tool bindings ------------------------------------------------
#
# One thin wrapper per bound-but-unwired #[func], added together so no domain
# workspace needs to touch this file at all -- every wrapper follows sized_api's
# own established shape (`has_method` guard, safe default on an older binary),
# so a workspace built against these never crashes against a GDExtension that
# predates one specific tool's binding.

# sculpt_bridge.rs
func get_sculpt_features() -> Array:
	if not _has("get_sculpt_features"):
		return []
	return world_gen.get_sculpt_features()

func get_sculpt_presets() -> Array:
	if not _has("get_sculpt_presets"):
		return []
	return world_gen.get_sculpt_presets()

func get_sculpt_globals_info() -> Array:
	if not _has("get_sculpt_globals_info"):
		return []
	return world_gen.get_sculpt_globals_info()

func get_sculpt_freehand_modes() -> PackedStringArray:
	if not _has("get_sculpt_freehand_modes"):
		return PackedStringArray()
	return world_gen.get_sculpt_freehand_modes()

func sculpt_get_globals() -> Dictionary:
	if not _has("sculpt_get_globals"):
		return {}
	return world_gen.sculpt_get_globals()

func sculpt_set_globals(values: Dictionary) -> Dictionary:
	if not _has("sculpt_set_globals"):
		return {}
	return world_gen.sculpt_set_globals(values)

func sculpt_get_feature() -> String:
	if not _has("sculpt_get_feature"):
		return ""
	return world_gen.sculpt_get_feature()

func sculpt_set_feature(feature_key: String) -> bool:
	if not _has("sculpt_set_feature"):
		return false
	return world_gen.sculpt_set_feature(feature_key)

func sculpt_get_feature_params() -> Dictionary:
	if not _has("sculpt_get_feature_params"):
		return {}
	return world_gen.sculpt_get_feature_params()

func sculpt_set_feature_params(values: Dictionary) -> Dictionary:
	if not _has("sculpt_set_feature_params"):
		return {}
	return world_gen.sculpt_set_feature_params(values)

func sculpt_apply_preset(index: int) -> bool:
	if not _has("sculpt_apply_preset"):
		return false
	return world_gen.sculpt_apply_preset(index)

func sculpt_get_freehand_mode() -> String:
	if not _has("sculpt_get_freehand_mode"):
		return ""
	return world_gen.sculpt_get_freehand_mode()

func sculpt_set_freehand_mode(mode_key: String) -> bool:
	if not _has("sculpt_set_freehand_mode"):
		return false
	return world_gen.sculpt_set_freehand_mode(mode_key)

func sculpt_get_seed() -> int:
	if not _has("sculpt_get_seed"):
		return -1
	return world_gen.sculpt_get_seed()

func sculpt_set_seed(seed: int) -> void:
	if not _has("sculpt_set_seed"):
		return
	world_gen.sculpt_set_seed(seed)

func sculpt_begin_stroke() -> bool:
	if not _has("sculpt_begin_stroke"):
		return false
	return world_gen.sculpt_begin_stroke()

func sculpt_add_point(x: float, y: float) -> int:
	if not _has("sculpt_add_point"):
		return -1
	return world_gen.sculpt_add_point(x, y)

func sculpt_stroke_point_count() -> int:
	if not _has("sculpt_stroke_point_count"):
		return -1
	return world_gen.sculpt_stroke_point_count()

func sculpt_cancel_stroke() -> void:
	if not _has("sculpt_cancel_stroke"):
		return
	world_gen.sculpt_cancel_stroke()

func sculpt_end_stroke() -> int:
	if not _has("sculpt_end_stroke"):
		return -1
	return world_gen.sculpt_end_stroke()

func sculpt_stamp_count() -> int:
	if not _has("sculpt_stamp_count"):
		return -1
	return world_gen.sculpt_stamp_count()

func sculpt_list_stamps() -> Array:
	if not _has("sculpt_list_stamps"):
		return []
	return world_gen.sculpt_list_stamps()

func sculpt_get_selected_stamp() -> int:
	if not _has("sculpt_get_selected_stamp"):
		return -1
	return world_gen.sculpt_get_selected_stamp()

func sculpt_select_stamp(index: int) -> bool:
	if not _has("sculpt_select_stamp"):
		return false
	return world_gen.sculpt_select_stamp(index)

func sculpt_set_stamp_hidden(index: int, hidden: bool) -> bool:
	if not _has("sculpt_set_stamp_hidden"):
		return false
	return world_gen.sculpt_set_stamp_hidden(index, hidden)

func sculpt_move_stamp_up(index: int) -> bool:
	if not _has("sculpt_move_stamp_up"):
		return false
	return world_gen.sculpt_move_stamp_up(index)

func sculpt_move_stamp_down(index: int) -> bool:
	if not _has("sculpt_move_stamp_down"):
		return false
	return world_gen.sculpt_move_stamp_down(index)

func sculpt_delete_stamp(index: int) -> bool:
	if not _has("sculpt_delete_stamp"):
		return false
	return world_gen.sculpt_delete_stamp(index)

func sculpt_can_undo() -> bool:
	if not _has("sculpt_can_undo"):
		return false
	return world_gen.sculpt_can_undo()

func sculpt_can_redo() -> bool:
	if not _has("sculpt_can_redo"):
		return false
	return world_gen.sculpt_can_redo()

func sculpt_undo() -> bool:
	if not _has("sculpt_undo"):
		return false
	return world_gen.sculpt_undo()

func sculpt_redo() -> bool:
	if not _has("sculpt_redo"):
		return false
	return world_gen.sculpt_redo()

func build_sculpt_preview_texture() -> Texture2D:
	if not _has("build_sculpt_preview_texture"):
		return null
	return world_gen.build_sculpt_preview_texture()

func sculpt_commit(reason: String) -> Dictionary:
	if not _has("sculpt_commit"):
		return {}
	return world_gen.sculpt_commit(reason)

func sculpt_discard() -> int:
	if not _has("sculpt_discard"):
		return -1
	return world_gen.sculpt_discard()

# -- Global heightmap undo (Edit ▸ Undo, Ctrl+Z) -------------------------------
#
# The reference's `pushUndo`/`undoLast`/`updateUndoUI`, register ED-01/PR-11.
# Deliberately NOT the same thing as `sculpt_undo`/`sculpt_redo` above: those
# pop a stamp off an uncommitted draft, these pop a whole committed height
# field. The reference keeps the same two apart under the same names.
#
# Same degrade-rather-than-crash `has_method` guard every wrapper here uses --
# an older cdylib without these five `#[func]`s reports "nothing to undo"
# rather than erroring.

func can_undo() -> bool:
	if not _has("can_undo"):
		return false
	return world_gen.can_undo()

## The operation `undo_last()` would revert ("Sculpt commit", "Carve fjords"),
## or "" when there is nothing to revert.
func undo_label() -> String:
	if not _has("undo_label"):
		return ""
	return String(world_gen.undo_label())

## Reverts the height field one step. Returns the reverted operation's label,
## or "" if nothing happened. The caller repaints -- the engine deliberately
## does not re-run flow/climate here (see `undo.rs`).
func undo_last() -> String:
	if not _has("undo_last"):
		return ""
	return String(world_gen.undo_last())

## `depth`, `max_steps`, `bytes`, `budget_bytes`, `step_bytes`, `label` --
## the reference's `#undoMem` readout as data. Empty dictionary on an older
## cdylib.
func undo_stats() -> Dictionary:
	if not _has("undo_stats"):
		return {}
	return world_gen.undo_stats()

func set_undo_budget_mb(mb: int) -> void:
	if _has("set_undo_budget_mb"):
		world_gen.set_undo_budget_mb(mb)

func clear_undo() -> void:
	if _has("clear_undo"):
		world_gen.clear_undo()


# -- bake / tile pyramid / persistent atlas / finalize (bake_bridge.rs) --------
#
# `GUI_GAP_REGISTER.md` WW-01 (Finalize · bake & freeze), PR-10/S4 (atlas
# cache), PR-12 (Clear caches), S5, SH-07 (the status bar's `atlas` slot).
#
# The atlas root is the one piece the engine cannot work out for itself:
# `AtlasStore` wants a real OS directory and Godot's `user://` is not one.
# Resolved once here, at first use rather than in `_ready()`, so a session that
# never bakes never creates the directory.

var atlas_api := false
var _atlas_root_set := false

## True once the engine has somewhere to put baked chunks. Idempotent.
func atlas_ready() -> bool:
	if _atlas_root_set:
		return true
	if not _has("atlas_set_root"):
		return false
	## `DccSettings`' own `atlas_cache` root, not a hardcoded `user://atlas`:
	## it is already the right shape, already user-settable from
	## File ▸ Storage locations, and `GUI_GAP_REGISTER.md` §7.7 item 3 already
	## called it the root the cache should use when the cache shipped.
	var dir := DccSettings.storage_root("atlas_cache")
	_atlas_root_set = world_gen.atlas_set_root(dir)
	if not _atlas_root_set:
		push_warning("Cartalith: could not create the atlas cache directory at %s" % dir)
	return _atlas_root_set

## `chunks`, `bytes`, `bytes_text`, `deepest_level`, `text`, `finalized`,
## `tile_size`, `world_key`, `root`. Never empty on a current cdylib -- an
## unconfigured or empty atlas is reported in `text`, not by absence.
func atlas_status() -> Dictionary:
	if not _has("atlas_status"):
		return {}
	atlas_ready()
	return world_gen.atlas_status()

## What a bake to `max_z` would cost: `tiles`, `already_baked`, `remaining`,
## `seconds`. Shown *before* the user commits, because depth 5 is 1365 tiles.
func bake_estimate(max_z: int) -> Dictionary:
	if not _has("bake_estimate"):
		return {}
	atlas_ready()
	return world_gen.bake_estimate(max_z)

## Bake every tile of every level 0..max_z. Synchronous and slow -- the caller
## must show a busy state. Returns `ok`, `baked`, `skipped`, `failed`, `total`,
## `seconds`, `error`.
func bake_all(max_z: int) -> Dictionary:
	if not _has("bake_all"):
		return {"ok": false, "error": "this build has no bake_all()"}
	if not atlas_ready():
		return {"ok": false, "error": "no atlas cache directory"}
	return world_gen.bake_all(max_z)

## Bake just the tiles a view rectangle (in coarse grid cells) touches.
func bake_visible(z: int, x0: float, y0: float, x1: float, y1: float) -> Dictionary:
	if not _has("bake_visible"):
		return {"ok": false, "error": "this build has no bake_visible()"}
	if not atlas_ready():
		return {"ok": false, "error": "no atlas cache directory"}
	return world_gen.bake_visible(z, x0, y0, x1, y1)

## Throw away this world's baked chunks; returns how many went. Clears the
## finalize lock too -- a lock protecting nothing would strand the user.
func atlas_clear() -> int:
	if not _has("atlas_clear"):
		return 0
	atlas_ready()
	return int(world_gen.atlas_clear())

func atlas_export_zip(gzip: bool = true) -> PackedByteArray:
	if not _has("atlas_export_zip"):
		return PackedByteArray()
	atlas_ready()
	return world_gen.atlas_export_zip(gzip)

func atlas_import_zip(bytes: PackedByteArray) -> Dictionary:
	if not _has("atlas_import_zip"):
		return {"ok": false, "error": "this build has no atlas_import_zip()"}
	if not atlas_ready():
		return {"ok": false, "error": "no atlas cache directory"}
	return world_gen.atlas_import_zip(bytes)

## One baked chunk's stored visual as PNG bytes, or empty when that chunk was
## never baked -- in which case the caller falls through to live synthesis,
## exactly as the reference's `atlasLoadImg` does.
func atlas_tile_png(z: int, col: int, row: int) -> PackedByteArray:
	if not _has("atlas_tile_png"):
		return PackedByteArray()
	atlas_ready()
	return world_gen.atlas_tile_png(z, col, row)

func atlas_tile_size() -> int:
	if not _has("atlas_tile_size"):
		return 1024
	return int(world_gen.atlas_tile_size())

func set_atlas_tile_size(px: int) -> void:
	if _has("atlas_set_tile_size"):
		world_gen.atlas_set_tile_size(px)

func is_finalized() -> bool:
	return _has("is_finalized") and world_gen.is_finalized()

## Finalizing needs a non-empty atlas; un-finalizing always succeeds.
func set_finalized(on: bool) -> bool:
	if not _has("set_finalized"):
		return false
	var ok: bool = world_gen.set_finalized(on)
	if ok:
		finalize_changed.emit(on)
	return ok

## "" when the change may proceed, otherwise the sentence to show. `kind` is
## `generation`, `height_edit` or `presentation`.
func finalize_check(kind: String) -> String:
	if not _has("finalize_check"):
		return ""
	return String(world_gen.finalize_check(kind))

signal finalize_changed(on: bool)


# icon_bridge.rs
func icon_arm(family: String, variant: int, scale: float, rotation: float, jitter: float) -> bool:
	if not _has("icon_arm"):
		return false
	return world_gen.icon_arm(family, variant, scale, rotation, jitter)

func icon_armed() -> Dictionary:
	if not _has("icon_armed"):
		return {}
	return world_gen.icon_armed()

func icon_disarm() -> void:
	if not _has("icon_disarm"):
		return
	world_gen.icon_disarm()

func icon_place(gx: float, gy: float) -> int:
	if not _has("icon_place"):
		return -1
	return world_gen.icon_place(gx, gy)

## `GUI_GAP_REGISTER.md` CA-05: `label_get_selected`'s own icon counterpart.
func icon_get_selected() -> int:
	if not _has("icon_get_selected"):
		return -1
	return world_gen.icon_get_selected()

func icon_hit_test(gx: float, gy: float) -> int:
	if not _has("icon_hit_test"):
		return -1
	return world_gen.icon_hit_test(gx, gy)

## `GUI_GAP_REGISTER.md` CA-05: the selected icon's on-canvas resize-handle
## circle -- `label_handles()`'s own one-handle mirror.
func icon_handles(index: int, zoom: float) -> Dictionary:
	if not _has("icon_handles"):
		return {}
	return world_gen.icon_handles(index, zoom)

func icon_resize(index: int, cx: float, cy: float, gx: float, gy: float, start_dist: float) -> bool:
	if not _has("icon_resize"):
		return false
	return world_gen.icon_resize(index, cx, cy, gx, gy, start_dist)

func icon_get(index: int) -> Dictionary:
	if not _has("icon_get"):
		return {}
	return world_gen.icon_get(index)

func icon_delete(index: int) -> bool:
	if not _has("icon_delete"):
		return false
	return world_gen.icon_delete(index)

func icon_list() -> Array:
	if not _has("icon_list"):
		return []
	return world_gen.icon_list()

func icon_clear_all() -> void:
	if not _has("icon_clear_all"):
		return
	world_gen.icon_clear_all()


# civ_bridge.rs
func civ_pick_place_at(gx: float, gy: float) -> int:
	if not _has("civ_pick_place_at"):
		return -1
	return world_gen.civ_pick_place_at(gx, gy)

func civ_drop_settlement(gx: float, gy: float, kind: String, faction: int, name: String, snap_to_water: bool) -> int:
	if not _has("civ_drop_settlement"):
		return -1
	return world_gen.civ_drop_settlement(gx, gy, kind, faction, name, snap_to_water)

func civ_territory_paint_at(gx: float, gy: float, faction: int, radius: float, subtract: bool) -> void:
	if not _has("civ_territory_paint_at"):
		return
	world_gen.civ_territory_paint_at(gx, gy, faction, radius, subtract)

func civ_territory_commit() -> void:
	if not _has("civ_territory_commit"):
		return
	world_gen.civ_territory_commit()

func civ_territory_discard() -> void:
	if not _has("civ_territory_discard"):
		return
	world_gen.civ_territory_discard()

func civ_faction_territory_stats(faction: int) -> Dictionary:
	if not _has("civ_faction_territory_stats"):
		return {}
	return world_gen.civ_faction_territory_stats(faction)

func get_factions() -> Array:
	if not _has("get_factions"):
		return []
	return world_gen.get_factions()


# civ_roster_bridge.rs -- the place editor, the faction roster and the two
# readouts `PARITY_AUDIT.md` §5 items 3/4/7/9/10/12 found unported.

func civ_settlement_details(index: int) -> Dictionary:
	if not _has("civ_settlement_details"):
		return {}
	return world_gen.civ_settlement_details(index)

## `fields` carries only the keys that changed -- see the Rust doc comment;
## an invalid value rejects the whole batch rather than half-applying it.
func civ_edit_settlement(index: int, fields: Dictionary) -> bool:
	if not _has("civ_edit_settlement"):
		return false
	return world_gen.civ_edit_settlement(index, fields)

func civ_settlement_toggle_trait(index: int, key: String) -> bool:
	if not _has("civ_settlement_toggle_trait"):
		return false
	return world_gen.civ_settlement_toggle_trait(index, key)

func civ_reroll_settlement_name(index: int) -> String:
	if not _has("civ_reroll_settlement_name"):
		return ""
	return world_gen.civ_reroll_settlement_name(index)

func civ_delete_settlement(index: int) -> bool:
	if not _has("civ_delete_settlement"):
		return false
	return world_gen.civ_delete_settlement(index)

## `GUI_GAP_REGISTER.md` SG-02 / ED-03d's "Recompute now" for the civ layer.
## Re-derives everything downstream of the settlement list -- roads, sea
## lanes, territory, provinces, trade balances, suitability explanations --
## against the current (edited) terrain, holding the settlements themselves,
## the faction roster, the place-edit side table and any hand-painted
## territory fixed. Placement is NOT re-rolled; that is what Generate does.
##
## Synchronous and not cheap (seconds at a large grid, `recompute_
## civilisation`'s own doc has the measured figures), so every caller shows a
## wait cursor and reports the `ms` the engine hands back rather than
## pretending it was instant.
func civ_recompute() -> Dictionary:
	if not _has("recompute_civilisation"):
		return {"ok": false, "reason": "This build's extension has no recompute_civilisation."}
	return world_gen.recompute_civilisation()

## `GUI_GAP_REGISTER.md` SG-01: which pipeline stages are stale right now, and
## why -- read straight off the engine's own `StageGraph`, so this is the real
## dirty state and not a flag this script keeps in parallel.
##
## `{stage_name: {origin: String, reason: String, tiles: int}}`, one entry per
## stale stage; `{}` means nothing is stale, which is the healthy state and
## also what a world-less session reports. `tiles == 0` means the entry is not
## tile-scoped (only `civ`'s "a settlement was edited" source is).
##
## A pure read on the engine side -- every `StageGraph` query takes `&self`,
## so asking can never trigger a recompute -- but still refused mid-generation
## for the same `Gd<T>::bind()` reason `param_set` is: the worker thread owns
## the object while a generation is in flight.
func stale_stages() -> Dictionary:
	if generating or not _has("stale_stages"):
		return {}
	return world_gen.stale_stages()

func civ_faction_count() -> int:
	if not _has("civ_faction_count"):
		return 0
	return world_gen.civ_faction_count()

func civ_add_faction() -> int:
	if not _has("civ_add_faction"):
		return -1
	return world_gen.civ_add_faction()

func civ_remove_faction() -> bool:
	if not _has("civ_remove_faction"):
		return false
	return world_gen.civ_remove_faction()

func civ_set_faction_field(faction: int, key: String, value: String) -> bool:
	if not _has("civ_set_faction_field"):
		return false
	return world_gen.civ_set_faction_field(faction, key, value)

## A faction's **identity colour** (`GUI_GAP_REGISTER.md` CV-21). Read back
## as `get_factions()`' `color_r/g/b`, with `color_custom` saying whether it
## is this or the palette rule. Every faction renderer -- the territory wash,
## the Political-control analysis field, the roster banner -- draws from it.
##
## Refused for faction 0 (Unclaimed, which nothing renders). The caller
## refreshes the map; nothing here invalidates a texture, the same contract
## every other roster edit has.
func civ_set_faction_color(faction: int, c: Color) -> bool:
	if not _has("civ_set_faction_color"):
		return false
	return world_gen.civ_set_faction_color(faction,
		int(round(c.r * 255.0)), int(round(c.g * 255.0)), int(round(c.b * 255.0)))

## Back to the palette rule for this faction. See `civ_set_faction_color`.
func civ_clear_faction_color(faction: int) -> bool:
	if not _has("civ_clear_faction_color"):
		return false
	return world_gen.civ_clear_faction_color(faction)

## Whether any faction carries a user identity colour.
func civ_has_faction_colors() -> bool:
	if not _has("civ_has_faction_colors"):
		return false
	return world_gen.civ_has_faction_colors()

func civ_faction_terrain_fits() -> Array:
	if not _has("civ_faction_terrain_fits"):
		return []
	return world_gen.civ_faction_terrain_fits()

## CIVIL ▸ Military (`GUI_GAP_REGISTER.md` CV-25). `{"factions": [...],
## "settlements": [...]}` -- per-faction military power and fortification
## counts, and every settlement's wall spec and defensive strength. One
## on-demand aggregate pass, so a modal-open call, not a per-frame one.
##
## Each faction row also carries a nested `"manpower"` dictionary
## (`MILITARY_MANPOWER_SCOPE.md`): the four headcounts (standing army,
## sustainable field army, emergency mobilization, and a four-rung
## force/duration ladder), the populations behind them, the five variables
## that drove them, and the derived era band with its verdict. Nested rather
## than flattened because `military` beside it is a *relative* 0-100
## heuristic and these are absolute headcounts -- reading one for the other
## is the mistake worth making structurally hard.
func civ_military_summary() -> Dictionary:
	if not _has("civ_military_summary"):
		return {}
	return world_gen.civ_military_summary()

## CIVIL ▸ Relationships (`GUI_GAP_REGISTER.md` CV-26). One row per
## unordered faction pair, with the four terms beside the verdict. Derived
## and recomputed on every call; nothing here is stored or saved.
func civ_faction_relations() -> Array:
	if not _has("civ_faction_relations"):
		return []
	return world_gen.civ_faction_relations()

func civ_agrarian_regional_total() -> Dictionary:
	if not _has("civ_agrarian_regional_total"):
		return {}
	return world_gen.civ_agrarian_regional_total()

func civ_trait_vocabulary() -> Array:
	if not _has("civ_trait_vocabulary"):
		return []
	return world_gen.civ_trait_vocabulary()

func civ_specialisation_vocabulary() -> Array:
	if not _has("civ_specialisation_vocabulary"):
		return []
	return world_gen.civ_specialisation_vocabulary()

func civ_religion_vocabulary() -> Array:
	if not _has("civ_religion_vocabulary"):
		return []
	return world_gen.civ_religion_vocabulary()

func civ_government_vocabulary() -> Array:
	if not _has("civ_government_vocabulary"):
		return []
	return world_gen.civ_government_vocabulary()

func civ_ag_tech_vocabulary() -> Array:
	if not _has("civ_ag_tech_vocabulary"):
		return []
	return world_gen.civ_ag_tech_vocabulary()

func civ_culture_vocabulary() -> PackedStringArray:
	if not _has("civ_culture_vocabulary"):
		return PackedStringArray()
	return world_gen.civ_culture_vocabulary()

func set_biome_k_enabled(enabled: bool) -> void:
	if not _has("set_biome_k_enabled"):
		return
	world_gen.set_biome_k_enabled(enabled)

func get_biome_k_enabled() -> bool:
	if not _has("get_biome_k_enabled"):
		return false
	return world_gen.get_biome_k_enabled()


# paint_bridge.rs
func get_paint_layers() -> PackedStringArray:
	if not _has("get_paint_layers"):
		return PackedStringArray()
	return world_gen.get_paint_layers()

func get_paint_palette(layer: String) -> Array:
	if not _has("get_paint_palette"):
		return []
	return world_gen.get_paint_palette(layer)

func paint_set_layer(layer: String) -> bool:
	if not _has("paint_set_layer"):
		return false
	return world_gen.paint_set_layer(layer)

func paint_set_brush(value: int, radius: float, hardness: float, softness: float, erase: bool, land_only: bool) -> Dictionary:
	if not _has("paint_set_brush"):
		return {}
	return world_gen.paint_set_brush(value, radius, hardness, softness, erase, land_only)

func paint_stroke_at(gx: float, gy: float) -> void:
	if not _has("paint_stroke_at"):
		return
	world_gen.paint_stroke_at(gx, gy)

func build_paint_preview_texture() -> Texture2D:
	if not _has("build_paint_preview_texture"):
		return null
	return world_gen.build_paint_preview_texture()

func paint_painted_counts() -> Dictionary:
	if not _has("paint_painted_counts"):
		return {}
	return world_gen.paint_painted_counts()

## `GUI_GAP_REGISTER.md` WW-13: the pending-draft dab count, which is what
## Commit / Discard act on. `paint_painted_counts()["total"]` above is the
## committed-plus-pending composite and is a legend's number, not a button's.
func paint_draft_count() -> int:
	if not _has("paint_draft_count"):
		return 0
	return world_gen.paint_draft_count()

func paint_commit() -> Dictionary:
	if not _has("paint_commit"):
		return {}
	return world_gen.paint_commit()

func paint_discard() -> int:
	if not _has("paint_discard"):
		return -1
	return world_gen.paint_discard()


# way_bridge.rs
func way_begin(way_type: String) -> bool:
	if not _has("way_begin"):
		return false
	return world_gen.way_begin(way_type)

func way_append_point(gx: float, gy: float) -> bool:
	if not _has("way_append_point"):
		return false
	return world_gen.way_append_point(gx, gy)

func way_commit() -> int:
	if not _has("way_commit"):
		return -1
	return world_gen.way_commit()

func way_discard() -> void:
	if not _has("way_discard"):
		return
	world_gen.way_discard()


# route_bridge.rs
func route_begin(mode: String) -> bool:
	if not _has("route_begin"):
		return false
	return world_gen.route_begin(mode)

func route_append_stop(gx: float, gy: float) -> bool:
	if not _has("route_append_stop"):
		return false
	return world_gen.route_append_stop(gx, gy)

func route_commit() -> int:
	if not _has("route_commit"):
		return -1
	return world_gen.route_commit()

func route_discard() -> void:
	if not _has("route_discard"):
		return
	world_gen.route_discard()

func route_count() -> int:
	if not _has("route_count"):
		return 0
	return world_gen.route_count()

func route_get(index: int) -> Dictionary:
	if not _has("route_get"):
		return {}
	return world_gen.route_get(index)

## `GUI_GAP_REGISTER.md` IN-09's second half. Deleting renumbers: every later
## route's index drops by one, so any caller holding one (`jp_compute`'s
## `route` key, `jp_reroute`, a list row) must re-read `route_count()` after
## this returns true. That is the engine's deliberate contract, not an
## accident -- see `route_delete`'s own doc comment in `lib.rs`.
func route_delete(index: int) -> bool:
	if not _has("route_delete"):
		return false
	return world_gen.route_delete(index)

func route_set_name(index: int, name: String) -> bool:
	if not _has("route_set_name"):
		return false
	return world_gen.route_set_name(index, name)


# measure_bridge.rs
func measure_begin() -> void:
	if not _has("measure_begin"):
		return
	world_gen.measure_begin()

func measure_add_point(gx: float, gy: float) -> void:
	if not _has("measure_add_point"):
		return
	world_gen.measure_add_point(gx, gy)

func measure_result() -> Dictionary:
	if not _has("measure_result"):
		return {}
	return world_gen.measure_result()

func measure_clear() -> void:
	if not _has("measure_clear"):
		return
	world_gen.measure_clear()

## The measurement toolbar's world-reading half (`measure_bridge.rs`,
## `design/Cartalith Measurement Toolbar.dc.html`). All four are stateless
## queries -- the caller owns the points -- so unlike the chain above there is
## no begin/clear pair to mirror here.
##
## `measure_api` gates the whole group at load, the same shape `sized_api` /
## `npr_api` already use, so a shell running against an older cdylib draws the
## toolbar's Measure modes disabled rather than erroring per click.
var measure_api := false

func measure_section(ax: float, ay: float, bx: float, by: float, samples: int) -> Dictionary:
	if not measure_api:
		return {}
	return world_gen.measure_section(ax, ay, bx, by, samples)

func measure_area(points: PackedVector2Array) -> Dictionary:
	if not measure_api:
		return {}
	return world_gen.measure_area(points)

func measure_radius(cx: float, cy: float, px: float, py: float) -> Dictionary:
	if not measure_api:
		return {}
	return world_gen.measure_radius(cx, cy, px, py)

func measure_vertical(ax: float, ay: float, bx: float, by: float) -> Dictionary:
	if not measure_api:
		return {}
	return world_gen.measure_vertical(ax, ay, bx, by)


# region_bridge.rs
func region_set(gx: float, gy: float, gw: float, gh: float) -> void:
	if not _has("region_set"):
		return
	world_gen.region_set(gx, gy, gw, gh)

func region_get() -> Dictionary:
	if not _has("region_get"):
		return {}
	return world_gen.region_get()

func region_clear() -> void:
	if not _has("region_clear"):
		return
	world_gen.region_clear()

func region_export_tiles(opts: Dictionary) -> PackedByteArray:
	if not _has("region_export_tiles"):
		return PackedByteArray()
	return world_gen.region_export_tiles(opts)


# geojson_bridge.rs -- GUI_GAP_REGISTER.md DM-03. Empty before the first
# generate()/load, and on a build whose GDExtension predates the binding.
func export_geojson() -> String:
	if not _has("export_geojson"):
		return ""
	return world_gen.export_geojson()


# label_bridge.rs
func label_create(gx: float, gy: float, text: String) -> int:
	if not _has("label_create"):
		return -1
	return world_gen.label_create(gx, gy, text)

func label_move(index: int, gx: float, gy: float) -> bool:
	if not _has("label_move"):
		return false
	return world_gen.label_move(index, gx, gy)

func label_select(index: int) -> bool:
	if not _has("label_select"):
		return false
	return world_gen.label_select(index)

func label_get_selected() -> int:
	if not _has("label_get_selected"):
		return -1
	return world_gen.label_get_selected()

func label_confirm_edit() -> void:
	if not _has("label_confirm_edit"):
		return
	world_gen.label_confirm_edit()

func label_cancel_edit() -> bool:
	if not _has("label_cancel_edit"):
		return false
	return world_gen.label_cancel_edit()

func label_get(index: int) -> Dictionary:
	if not _has("label_get"):
		return {}
	return world_gen.label_get(index)

func label_list() -> Array:
	if not _has("label_list"):
		return []
	return world_gen.label_list()

func label_set(index: int, values: Dictionary) -> Dictionary:
	if not _has("label_set"):
		return {}
	return world_gen.label_set(index, values)

func label_delete(index: int) -> bool:
	if not _has("label_delete"):
		return false
	return world_gen.label_delete(index)

func label_clear_all() -> void:
	if not _has("label_clear_all"):
		return
	world_gen.label_clear_all()

func label_hit_test(gx: float, gy: float) -> int:
	if not _has("label_hit_test"):
		return -1
	return world_gen.label_hit_test(gx, gy)

func label_handles(index: int, zoom: float) -> Dictionary:
	if not _has("label_handles"):
		return {}
	return world_gen.label_handles(index, zoom)

func label_glyph_layout(index: int, zoom: float, char_widths: PackedFloat64Array, total_w: float) -> Array:
	if not _has("label_glyph_layout"):
		return []
	return world_gen.label_glyph_layout(index, zoom, char_widths, total_w)

func label_resize_size(start_size: float, cx: float, cy: float, gx: float, gy: float, start_dist: float) -> float:
	if not _has("label_resize_size"):
		return 0.0
	return world_gen.label_resize_size(start_size, cx, cy, gx, gy, start_dist)

func label_rotate_deg(cx: float, cy: float, gx: float, gy: float) -> float:
	if not _has("label_rotate_deg"):
		return 0.0
	return world_gen.label_rotate_deg(cx, cy, gx, gy)

func label_arc_value(cx: float, cy: float, grab_angle_deg: float, side: float, gx: float, gy: float) -> float:
	if not _has("label_arc_value"):
		return 0.0
	return world_gen.label_arc_value(cx, cy, grab_angle_deg, side, gx, gy)


# journey_bridge.rs
func jp_options() -> Dictionary:
	if not _has("jp_options"):
		return {}
	return world_gen.jp_options()

func jp_default_plan() -> Dictionary:
	if not _has("jp_default_plan"):
		return {}
	return world_gen.jp_default_plan()

## `_jpEnsurePlan(jn)` in full for one committed route: the route-aware
## defaults `jp_default_plan()` is only the route-blind half of. A route the
## `mixed` cost grid took mostly across open water opens on Sea Faring, and the
## vessel guess is corrected from the route's own derived stages
## (`jpAutoPickVessel`). Empty Dictionary when the binary predates it or the
## index has no route -- the caller falls back to `jp_default_plan()`.
func jp_plan_for_route(route_index: int) -> Dictionary:
	if not _has("jp_plan_for_route"):
		return {}
	return world_gen.jp_plan_for_route(route_index)

func jp_compute(request: Dictionary) -> Dictionary:
	if not _has("jp_compute"):
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
	if not _has("sample_cell"):
		return {}
	return world_gen.sample_cell(gx, gy)

## The Layers popover's grouped menu, in the reference's own `LAYER_GROUPS`
## order. Each item carries `available`, which is false for a view this
## particular world has no input for.
func debug_layers() -> Array:
	if not _has("debug_layers"):
		return []
	return world_gen.debug_layers()

## One debug view as a grid-sized `Texture2D`. `null` for "off", an unknown
## id, or a view this world has no input for.
func debug_texture(view: String) -> Texture2D:
	if not _has("build_debug_texture"):
		return null
	return world_gen.build_debug_texture(view)

## The wildlife ecoregion under a click, for the Wildlife view's roster
## popup (the reference's own `showWildInfo`). `{}` when the click missed
## every region marker, when the Wildlife view has no world to read, or on
## an engine build without the binding.
func wildlife_region_at(gx: float, gy: float) -> Dictionary:
	if not _has("wildlife_region_at"):
		return {}
	return world_gen.wildlife_region_at(gx, gy)

## Borders, claims and influence as three separate quantities
## (`GUI_GAP_REGISTER.md` **CV-23**): per-faction cell counts, mean influence
## and mean contest, plus one row per pair of factions that actually meet.
##
## **Computed on demand and held nowhere** -- the per-cell influence field
## behind these numbers is built, read and dropped inside the call (the same
## shape `wildlife_regions()` uses). The returned `transient_bytes` is what
## that cost for this world. `{}` before any generate, on a loaded save, and
## on a world with no capital.
func civ_territory_influence() -> Dictionary:
	if not _has("civ_territory_influence"):
		return {}
	return world_gen.civ_territory_influence()

## Trade **flows** -- who supplies whom, over what water, along which way
## (`GUI_GAP_REGISTER.md` **IN-13**).
##
## **Computed on demand and held nowhere**, the same contract
## `civ_territory_influence()` above ships on: `cartalith_civ::trade` matches
## every surplus against every deficit it can actually reach, routes what
## lands on a road, and drops the whole thing before returning. `CivData`
## gains no field and nothing is saved.
##
## Returns the *entire* answer in one dictionary -- world totals, per-good
## rows, every flow, unmet needs, per-settlement navigability and per-way
## load -- because three surfaces read it (the Trade category, the place
## editor's ledger, the map's way-load overlay) and re-running a quarter-
## second match per place-editor open would be the cost this design exists to
## avoid. `TradeStore` is what keeps the result on the shell side, where it
## can be dropped.
##
## `{}` before any generate, on a loaded save (no civilisation layer), and on
## a world with no settlements.
func civ_trade_flows() -> Dictionary:
	if not _has("civ_trade_flows"):
		return {}
	return world_gen.civ_trade_flows()

## The coordinate frame this world's fields and its GeoJSON export are in
## (`GUI_GAP_REGISTER.md` WW-15). `{}` before any generate.
func world_crs() -> Dictionary:
	if not _has("world_crs"):
		return {}
	return world_gen.world_crs()

## The world's ecology in one record -- `GUI_GAP_REGISTER.md` **WW-14**:
## land-only mean/max net primary productivity (Miami model, g/m²/yr) plus
## the wildlife ecoregion segmentation's own count, species total and its
## eight largest regions. `{}` before any generate.
##
## `regions` is empty (and `region_count` 0) with the NPP figures still real
## on a loaded save: productivity needs climate only, ecoregions need the
## civilisation layer's water bodies.
func ecology_summary() -> Dictionary:
	if not _has("ecology_summary"):
		return {}
	return world_gen.ecology_summary()


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
	if not _has("civ_add_year"):
		return
	world_gen.civ_add_year(year)

## `civGotoYear`: moves the active-year cursor and restores `territory` from
## that year's recorded snapshot. Never touches settlements/ways. A no-op
## before any generate.
func civ_goto_year(year: int) -> void:
	if not _has("civ_goto_year"):
		return
	world_gen.civ_goto_year(year)

## `civRemoveYear`: deletes a recorded year. If it was the active year, falls
## back to the earliest remaining one (or year 0 if none remain). A no-op
## before any generate or for a year that was never recorded.
func civ_remove_year(year: int) -> void:
	if not _has("civ_remove_year"):
		return
	world_gen.civ_remove_year(year)

## The active timeline cursor (reference `civYear`). `0` before any
## generate/`civ_add_year` call.
func get_civ_year() -> int:
	if not _has("get_civ_year"):
		return 0
	return world_gen.get_civ_year()

## Every recorded timeline year, ascending -- the pill list's own data source.
func get_civ_timeline_years() -> PackedInt64Array:
	if not _has("get_civ_timeline_years"):
		return PackedInt64Array()
	return world_gen.get_civ_timeline_years()

## `_civYearDiff`: `{"present": PackedInt64Array, "removed": PackedInt64Array,
## "added": PackedInt64Array}` of settlement/way tids, diffing `year` against
## the chronologically-previous recorded year -- the ghost/highlight/
## exist-only overlay's own data source. Empty sets (not an error) before any
## generate or for an unrecorded year.
func civ_year_diff(year: int) -> Dictionary:
	if not _has("civ_year_diff"):
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
	if not _has("civ_run_collapse_simulation"):
		return {"ok": false, "error": "civ_run_collapse_simulation not available on this binary"}
	return world_gen.civ_run_collapse_simulation(request)


# travel_bridge.rs / lib.rs's Travel Library #[func] block (TRAVEL_LIBRARY_SPEC.md,
# GUI_GAP_REGISTER.md DM-15/O1). `kind` is one of "animal"/"vehicle"/"vessel"/"preset"
# throughout. `has_method` guards match every wrapper above: a binary built before this
# landed simply has no `tl_*` methods, and `travel_library_window.gd` falls back to an
# empty library rather than erroring.

## `{kind: {"total": int, "custom": int, "stock": int}}` for all four definition types.
func tl_counts() -> Dictionary:
	if not _has("tl_counts"):
		return {}
	return world_gen.tl_counts()

## Every entry of one definition type, stock-then-custom order, each row carrying
## `id`/`name`/`origin`/`editable`/`subtitle`/`species_key`/`validation_state`/
## `validation_missing`/`validation_conflicts`/`usage_presets`/`usage_journeys`.
func tl_list(kind: String) -> Array:
	if not _has("tl_list"):
		return []
	return world_gen.tl_list(kind)

## One entry's full detail -- `tl_list`'s own per-row keys plus every field
## `TRAVEL_LIBRARY_SPEC.md` §3 lists for `kind`. An unset optional field is simply
## absent from the returned Dictionary -- test `has()`, don't assume a default.
func tl_get(kind: String, id: String) -> Dictionary:
	if not _has("tl_get"):
		return {"ok": false}
	return world_gen.tl_get(kind, id)

## Clones `id` (stock or custom) into a new editable custom entry.
## `{"ok": true, "id": new_id}` or `{"ok": false, "error": ...}`.
func tl_duplicate(kind: String, id: String) -> Dictionary:
	if not _has("tl_duplicate"):
		return {"ok": false, "error": "tl_duplicate not available on this binary"}
	return world_gen.tl_duplicate(kind, id)

## A brand-new custom entry with every field unset. `{"ok": true, "id": new_id}`.
func tl_add_blank(kind: String, name: String) -> Dictionary:
	if not _has("tl_add_blank"):
		return {"ok": false, "error": "tl_add_blank not available on this binary"}
	return world_gen.tl_add_blank(kind, name)

## Deletes a custom entry. No-op on an unknown id or a stock one.
func tl_delete(kind: String, id: String) -> Dictionary:
	if not _has("tl_delete"):
		return {"ok": false}
	return world_gen.tl_delete(kind, id)

## Discards every custom entry of one kind, restoring the stock-only bootstrap.
func tl_reset_to_stock(kind: String) -> Dictionary:
	if not _has("tl_reset_to_stock"):
		return {"ok": false}
	return world_gen.tl_reset_to_stock(kind)

## Applies a partial `fields` Dictionary onto an existing custom entry (stock entries
## are read-only -- duplicate first). Returns `{"ok", "error", "rejected",
## "validation_state", "validation_missing", "validation_conflicts"}`.
func tl_edit(kind: String, id: String, fields: Dictionary) -> Dictionary:
	if not _has("tl_edit"):
		return {"ok": false, "error": "tl_edit not available on this binary", "rejected": []}
	return world_gen.tl_edit(kind, id, fields)

## "Capture party from planner": a new custom party preset from `plan`, in
## `jp_default_plan()`/`jp_compute`'s own `plan` key vocabulary.
func tl_capture_preset_from_plan(name: String, plan: Dictionary) -> Dictionary:
	if not _has("tl_capture_preset_from_plan"):
		return {"ok": false, "error": "tl_capture_preset_from_plan not available on this binary"}
	return world_gen.tl_capture_preset_from_plan(name, plan)


# `asset_bridge.rs` / `lib.rs`'s Asset Library #[func] block (`GUI_GAP_REGISTER.md`
# AS-01..AS-08/AS-13, DM-05). `has_method` guards match every wrapper above: a binary
# built before this landed simply has no `as_*` methods, and `asset_library_window.gd`
# falls back to an empty/disabled state rather than erroring.

## Decode `bytes` as a PNG and add it as a new item on `uid`. `{"ok": true}` or
## `{"ok": false, "error": ...}`.
func as_import_item(uid: String, item_name: String, bytes: PackedByteArray) -> Dictionary:
	if not _has("as_import_item"):
		return {"ok": false, "error": "as_import_item not available on this binary"}
	return world_gen.as_import_item(uid, item_name, bytes)

## Add (or return the existing) custom slot. `{"ok": true, "uid": ...}`.
func as_add_custom_slot(slot_name: String, set_name: String) -> Dictionary:
	if not _has("as_add_custom_slot"):
		return {"ok": false, "error": "as_add_custom_slot not available on this binary"}
	return world_gen.as_add_custom_slot(slot_name, set_name)

## Every slot in `family_key`'s registry with real fill state -- each row carries
## `uid`/`id`/`name`/`item_count`/`filled`/`has_dupe`. Empty on an older binary.
func as_family_slots(family_key: String) -> Array:
	if not _has("as_family_slots"):
		return []
	return world_gen.as_family_slots(family_key)

## One slot's inspector detail: id/name/family/set, tags, collections, meta fields.
func as_slot_summary(uid: String) -> Dictionary:
	if not _has("as_slot_summary"):
		return {"ok": false}
	return world_gen.as_slot_summary(uid)

## One item's inspector detail: name, scale/pan_x/pan_y, decoded w/h, hash.
func as_item_summary(uid: String, index: int) -> Dictionary:
	if not _has("as_item_summary"):
		return {"ok": false}
	return world_gen.as_item_summary(uid, index)

## A real, baked PNG thumbnail for one stored item. Empty on a miss or an older binary.
func as_thumbnail_png(uid: String, index: int, size: int) -> PackedByteArray:
	if not _has("as_thumbnail_png"):
		return PackedByteArray()
	return world_gen.as_thumbnail_png(uid, index, size)

## Pack-level metadata and totals: name/author/license/total_items.
func as_pack_info() -> Dictionary:
	if not _has("as_pack_info"):
		return {"name": "", "author": "", "license": "", "total_items": 0}
	return world_gen.as_pack_info()

## Sets the pack's name/author/license fields directly.
func as_set_pack_info(pack_name: String, author: String, license: String) -> bool:
	if not _has("as_set_pack_info"):
		return false
	return world_gen.as_set_pack_info(pack_name, author, license)

## Removes one item from a slot.
func as_remove_item(uid: String, index: int) -> bool:
	if not _has("as_remove_item"):
		return false
	return world_gen.as_remove_item(uid, index)

## Resets the whole session to a fresh, empty library.
func as_clear_library() -> bool:
	if not _has("as_clear_library"):
		return false
	return world_gen.as_clear_library()

## `AssetValidator.run()`'s real, ordered warning strings.
func as_validate() -> PackedStringArray:
	if not _has("as_validate"):
		return PackedStringArray()
	return world_gen.as_validate()

## Bakes every stored item and writes the pack `.zip` bytes.
## `{"ok": true, "name": ..., "bytes": PackedByteArray}` or `{"ok": false, "error": ...}`.
func as_export_pack_bytes() -> Dictionary:
	if not _has("as_export_pack_bytes"):
		return {"ok": false, "error": "as_export_pack_bytes not available on this binary"}
	return world_gen.as_export_pack_bytes()

## Compiles the current session into a pack and loads it straight into the renderer
## -- the reference's own `applyToMap()`, same bake `as_export_pack_bytes` does.
func as_apply_to_map() -> Dictionary:
	if not _has("as_apply_to_map"):
		return {"ok": false, "error": "as_apply_to_map not available on this binary"}
	var result: Dictionary = world_gen.as_apply_to_map()
	if bool(result.get("ok", false)):
		world_loaded.emit()
	return result

## Comma-separated `tags_csv` onto every uid in `uids`.
func as_batch_tag(uids: PackedStringArray, tags_csv: String) -> Dictionary:
	if not _has("as_batch_tag"):
		return {"ok": false}
	return world_gen.as_batch_tag(uids, tags_csv)

## Adds every uid in `uids` to collection `coll_name`.
func as_batch_collect(uids: PackedStringArray, coll_name: String) -> Dictionary:
	if not _has("as_batch_collect"):
		return {"ok": false}
	return world_gen.as_batch_collect(uids, coll_name)

## `{base}_01`, `{base}_02`, ... over `uids` in order. `remap` carries
## `old_uid -> new_uid` for every custom slot whose uid changed.
func as_batch_rename(uids: PackedStringArray, base: String) -> Dictionary:
	if not _has("as_batch_rename"):
		return {"ok": false, "renamed": 0, "remap": {}}
	return world_gen.as_batch_rename(uids, base)

## Clones every slot in `uids` carrying at least one item into a new custom slot.
func as_batch_duplicate(uids: PackedStringArray) -> Dictionary:
	if not _has("as_batch_duplicate"):
		return {"ok": false, "made": 0}
	return world_gen.as_batch_duplicate(uids)

## Custom slots in `uids` are removed entirely; frozen slots have their items cleared.
func as_batch_delete(uids: PackedStringArray) -> Dictionary:
	if not _has("as_batch_delete"):
		return {"ok": false, "deleted": 0}
	return world_gen.as_batch_delete(uids)

## Decodes a sprite sheet and holds it on the session for slicing (AS-09).
## `{"ok": true, "w", "h", "name"}` or `{"ok": false, "error": ...}`. PNG only.
func as_load_sheet(sheet_name: String, bytes: PackedByteArray) -> Dictionary:
	if not _has("as_load_sheet"):
		return {"ok": false, "error": "as_load_sheet not available on this binary", "w": 0, "h": 0}
	return world_gen.as_load_sheet(sheet_name, bytes)

## Drops the loaded sheet (the slicer modal closing).
func as_clear_sheet() -> bool:
	if not _has("as_clear_sheet"):
		return false
	return world_gen.as_clear_sheet()

## The real `N cells detected · M non-empty` pass plus the overlay's grid lines
## (AS-09). `{"ok", "total", "non_empty", "usable", "col_x0"/"col_x1"/"row_y0"/
## "row_y1"}` -- the four span arrays are in sheet pixels, engine-computed, so
## the overlay draws exactly the cells `as_slice_apply` will cut.
func as_slice_preview(opts: Dictionary) -> Dictionary:
	if not _has("as_slice_preview"):
		return {"ok": false, "error": "as_slice_preview not available on this binary",
			"total": 0, "non_empty": 0, "usable": false, "blank": PackedInt32Array(),
			"col_x0": PackedFloat64Array(), "col_x1": PackedFloat64Array(),
			"row_y0": PackedFloat64Array(), "row_y1": PackedFloat64Array(),
			"col_lines_px": PackedFloat64Array(), "row_lines_px": PackedFloat64Array()}
	return world_gen.as_slice_preview(opts)

## Slices the loaded sheet into the library (AS-09/AS-10/AS-11). Non-destructive:
## the sheet stays loaded. `{"ok", "added", "skipped_blank", "unplaced", "uids"}`
## or `{"ok": false, "error": ...}`.
func as_slice_apply(opts: Dictionary) -> Dictionary:
	if not _has("as_slice_apply"):
		return {"ok": false, "error": "as_slice_apply not available on this binary",
			"added": 0, "skipped_blank": 0, "unplaced": 0, "uids": PackedStringArray()}
	return world_gen.as_slice_apply(opts)

## AS-07: writes one item's scale/pan directly. `false` for an unknown
## uid/index or an older binary.
func as_set_item_transform(uid: String, index: int, scale: float, pan_x: float, pan_y: float) -> bool:
	if not _has("as_set_item_transform"):
		return false
	return world_gen.as_set_item_transform(uid, index, scale, pan_x, pan_y)

## AS-07: resets one item's transform to identity, re-fitting to the slot's
## family when `fit` is true. `{"ok", "scale", "pan_x", "pan_y"}` or
## `{"ok": false}` for an unknown uid/index or an older binary.
func as_reset_item_transform(uid: String, index: int, fit: bool) -> Dictionary:
	if not _has("as_reset_item_transform"):
		return {"ok": false}
	return world_gen.as_reset_item_transform(uid, index, fit)

## AS-17: moves interior line `index` of `lines` to `frac`, clamped strictly
## between its neighbours -- `lines` unchanged on an older binary.
func as_slicer_move_line(lines: PackedFloat64Array, index: int, frac: float) -> PackedFloat64Array:
	if not _has("as_slicer_move_line"):
		return lines
	return world_gen.as_slicer_move_line(lines, index, frac)

## AS-17: the uniform `n+1`-line array a fresh grid (or a cols/rows edit)
## falls back to. Empty on an older binary.
func as_uniform_lines(n: int) -> PackedFloat64Array:
	if not _has("as_uniform_lines"):
		return PackedFloat64Array()
	return world_gen.as_uniform_lines(n)


# -- Markdown Vault (`MARKDOWN_VAULT_SCOPE.md` milestones 0-1) ---------------
#
# Same degrade-rather-than-crash `_has()` guard every wrapper above uses: a
# binary built before this milestone answers `false` once, with a warning, and
# the vault panels degrade to "not built into this engine" rather than
# erroring per rebuild.
#
# Reads (`vault_list_files`, `vault_links_for`, `vault_entity_summary`) return
# a safe empty value; writes return `{"ok": false, "error": ...}`, which is the
# same failure shape the engine itself returns, so no caller needs two
# branches.

const _VAULT_UNAVAILABLE := {
	"ok": false,
	"error": "This engine build has no Markdown Vault support — rebuild the GDExtension.",
}

## Addressable landmasses, largest first (`get_continents()`).
## `id` is a **rank by area**, not a persistent identity — see the engine's
## own doc comment on that binding before storing one anywhere.
func continents() -> Array:
	if not _has("get_continents"):
		return []
	return world_gen.get_continents()

func vault_connect(path: String, display_name: String = "") -> Dictionary:
	if not _has("vault_connect"):
		return _VAULT_UNAVAILABLE
	return world_gen.vault_connect(path, display_name)

func vault_disconnect() -> void:
	if _has("vault_disconnect"):
		world_gen.vault_disconnect()

func vault_info() -> Dictionary:
	if not _has("vault_info"):
		return {"bound": false, "root": "", "display_name": "", "vault_id": "", "link_count": 0}
	return world_gen.vault_info()

func vault_list_files(limit: int = 2000) -> PackedStringArray:
	if not _has("vault_list_files"):
		return PackedStringArray()
	return world_gen.vault_list_files(limit)

## The templates in the connected vault (`GUI_GAP_REGISTER.md` VA-02), each
## `{rel, label}`. A `.md` with "template" in its path -- Cartalith ships
## none of its own.
func vault_templates() -> Array:
	if not _has("vault_templates"):
		return []
	return world_gen.vault_templates()

## Where a new note for this entity goes -- v3's `Settlements/{name}.md`
## convention, generalised to every kind. A suggestion, not a rule.
func vault_suggested_path(kind: String, name: String) -> String:
	if not _has("vault_suggested_path"):
		return ""
	return world_gen.vault_suggested_path(kind, name)

## Creates a note from a template, substituting `name` for the template's own
## name placeholder and nothing else. Refuses an existing path; never
## overwrites. `{ok, path, text}` or `{ok: false, error}`.
func vault_create_from_template(template_rel: String, rel: String, name: String) -> Dictionary:
	if not _has("vault_create_from_template"):
		return {"ok": false, "error": "this engine build has no note creator"}
	return world_gen.vault_create_from_template(template_rel, rel, name)

func vault_file_headings(rel: String) -> Array:
	if not _has("vault_file_headings"):
		return []
	return world_gen.vault_file_headings(rel)

func vault_read_file(rel: String) -> String:
	if not _has("vault_read_file"):
		return ""
	return world_gen.vault_read_file(rel)

func vault_attach(kind: String, entity_id: int, label: String, rel: String, heading: String) -> Dictionary:
	if not _has("vault_attach"):
		return _VAULT_UNAVAILABLE
	return world_gen.vault_attach(kind, entity_id, label, rel, heading)

func vault_detach(link_id: String) -> bool:
	if not _has("vault_detach"):
		return false
	return world_gen.vault_detach(link_id)

func vault_links_for(kind: String, entity_id: int) -> Array:
	if not _has("vault_links_for"):
		return []
	return world_gen.vault_links_for(kind, entity_id)

func vault_all_links() -> Array:
	if not _has("vault_all_links"):
		return []
	return world_gen.vault_all_links()

func vault_entity_summary(kind: String, entity_id: int) -> Dictionary:
	if not _has("vault_entity_summary"):
		return {"link_count": 0, "status": ""}
	return world_gen.vault_entity_summary(kind, entity_id)

func vault_link_text(link_id: String) -> String:
	if not _has("vault_link_text"):
		return ""
	return world_gen.vault_link_text(link_id)

func vault_set_link_text(link_id: String, text: String) -> bool:
	if not _has("vault_set_link_text"):
		return false
	return world_gen.vault_set_link_text(link_id, text)

func vault_reload_link(link_id: String) -> Dictionary:
	if not _has("vault_reload_link"):
		return _VAULT_UNAVAILABLE
	return world_gen.vault_reload_link(link_id)

func vault_preview_section_write(link_id: String) -> Dictionary:
	if not _has("vault_preview_section_write"):
		return _VAULT_UNAVAILABLE
	return world_gen.vault_preview_section_write(link_id)

func vault_write_section(link_id: String, expect_hash: String) -> Dictionary:
	if not _has("vault_write_section"):
		return _VAULT_UNAVAILABLE
	return world_gen.vault_write_section(link_id, expect_hash)

## Every entity kind this build can address in a vault. `faction` joined
## `settlement`/`province`/`continent` on 2026-08-25 (`GUI_GAP_REGISTER.md`
## CV-22).
func vault_entity_kinds() -> PackedStringArray:
	if not _has("vault_entity_kinds"):
		return PackedStringArray(["settlement", "province", "continent"])
	return world_gen.vault_entity_kinds()

func vault_export_fields(kind: String, entity_id: int) -> Array:
	if not _has("vault_export_fields"):
		return []
	return world_gen.vault_export_fields(kind, entity_id)

func vault_entity_values(kind: String, entity_id: int) -> Dictionary:
	if not _has("vault_entity_values"):
		return {}
	return world_gen.vault_entity_values(kind, entity_id)

func vault_block_body(kind: String, entity_id: int, selected: PackedStringArray) -> String:
	if not _has("vault_block_body"):
		return ""
	return world_gen.vault_block_body(kind, entity_id, selected)

func vault_preview_block(rel: String, kind: String, entity_id: int, body: String) -> Dictionary:
	if not _has("vault_preview_block"):
		return _VAULT_UNAVAILABLE
	return world_gen.vault_preview_block(rel, kind, entity_id, body)

func vault_write_block(rel: String, kind: String, entity_id: int, body: String, expect_hash: String) -> Dictionary:
	if not _has("vault_write_block"):
		return _VAULT_UNAVAILABLE
	return world_gen.vault_write_block(rel, kind, entity_id, body, expect_hash)

func vault_preview_field_fill(rel: String, kind: String, entity_id: int, overwrite: bool) -> Dictionary:
	if not _has("vault_preview_field_fill"):
		return _VAULT_UNAVAILABLE
	return world_gen.vault_preview_field_fill(rel, kind, entity_id, overwrite)

func vault_write_field_fill(rel: String, kind: String, entity_id: int, overwrite: bool, expect_hash: String) -> Dictionary:
	if not _has("vault_write_field_fill"):
		return _VAULT_UNAVAILABLE
	return world_gen.vault_write_field_fill(rel, kind, entity_id, overwrite, expect_hash)

## The link store as JSON. `VaultStore` (`vault_store.gd`) owns writing it to
## disk; this is only the engine's side of that.
# -- Backlinks (`GUI_GAP_REGISTER.md` VA-01) ---------------------------------
#
# The index is built only when a person asks. Every wrapper here is a read
# except `vault_refresh_backlinks`, which is the asking.

## Bring the backlink index up to date, reading only the notes whose
## `(modified, len)` moved. `{ok, seen, reread, dropped, unreadable, notes,
## links, entities, bytes, refreshed_at}`, or `{ok:false, error}`.
## Every committed operation this session, oldest first
## (`GUI_GAP_REGISTER.md` **ED-02**). One row per commit, not one per
## reversible commit: `kind` is `height`/`recorded`/`floor`, `reversible` says
## whether a snapshot is *still* held for it, `reason` says why not, and
## `steps` is how many undo steps reverting to it would take.
func undo_ledger() -> Array:
	if not _has("undo_ledger"):
		return []
	return world_gen.undo_ledger()

## Roll back to a ledger row, popping every height snapshot above it as well.
## Returns how many steps were reverted; `0` when the row is gone or was never
## reversible, in which case the caller should re-read `undo_ledger()`.
func undo_revert_to(seq: int) -> int:
	if not _has("undo_revert_to"):
		return 0
	return world_gen.undo_revert_to(seq)


func vault_refresh_backlinks(limit: int = 2000) -> Dictionary:
	if not _has("vault_refresh_backlinks"):
		return {"ok": false, "error": "this build predates the backlink index"}
	return world_gen.vault_refresh_backlinks(limit)

## Throw the index away so the next refresh re-reads everything.
func vault_rebuild_backlinks() -> void:
	if _has("vault_rebuild_backlinks"):
		world_gen.vault_rebuild_backlinks()

## `{built, notes, links, entities, broken, orphans, bytes, refreshed_at}`.
## `built` is the one every reader must branch on: "no notes" and "nothing
## indexed" are opposite statements on screen.
func vault_backlink_stats() -> Dictionary:
	if not _has("vault_backlink_stats"):
		return {"built": false}
	return world_gen.vault_backlink_stats()

## Every note that references this entity -- `{rel, form, count}`, `form` one
## of `wiki`, `markdown`, `block`. A `block` row is a note carrying this
## entity's own Cartalith block, which finds the entity even when it has no
## note of its own.
func vault_entity_backlinks(kind: String, entity_id: int) -> Array:
	if not _has("vault_entity_backlinks"):
		return []
	return world_gen.vault_entity_backlinks(kind, entity_id)

## Notes that name this entity in prose and do not link to it --
## `{rel, excerpt}`. A guess, and the panel draws it as one.
func vault_entity_mentions(kind: String, entity_id: int, name: String, max_rows: int = 12) -> Array:
	if not _has("vault_entity_mentions"):
		return []
	return world_gen.vault_entity_mentions(kind, entity_id, name, max_rows)

## `{built, broken: [{source, target}], orphans: [rel]}` -- both halves of
## Data ▸ Missing & orphan notes report…, from the one index.
func vault_backlink_report(limit: int = 200) -> Dictionary:
	if not _has("vault_backlink_report"):
		return {"built": false, "broken": [], "orphans": PackedStringArray()}
	return world_gen.vault_backlink_report(limit)

## The index as JSON. `VaultStore` writes it beside the link store -- and
## separately from it, because the store is portable project data and this is
## a cache of somebody's folder.
func vault_backlink_index_json() -> String:
	if not _has("vault_backlink_index_json"):
		return ""
	return world_gen.vault_backlink_index_json()

func vault_restore_backlink_index(json: String) -> bool:
	if not _has("vault_restore_backlink_index"):
		return false
	return world_gen.vault_restore_backlink_index(json)


func vault_state_json() -> String:
	if not _has("vault_state_json"):
		return ""
	return world_gen.vault_state_json()

func vault_restore_state(json: String) -> bool:
	if not _has("vault_restore_state"):
		return false
	return world_gen.vault_restore_state(json)


# =========================================================== PARITY_AUDIT §23 ==
#
# Wrappers added ahead of the five agents that close §23's findings, so that
# none of them has to edit this file and race the others in it. Every one is
# the same thin `_has()`-guarded forward the rest of this file uses; the guard
# matters because an older binary in `target/` is a normal state during a
# rebuild, and a missing method must disable a control rather than crash it.

# -- F9 · the vault: search, "confirm always", note data, undo a block --------

## `vault_search`: `{ok, error, indexed, scanned, truncated, hits: [{rel,
## in_name, excerpt}]}`. `limit`/`max_reads` of 0 take the engine's own
## defaults (50 / 40).
func vault_search(query: String, limit: int = 0, max_reads: int = 0) -> Dictionary:
	if not _has("vault_search"):
		return {"ok": false, "error": "vault_search not available on this binary", "hits": []}
	return world_gen.vault_search(query, limit, max_reads)

## One note's frontmatter and template fields, read from disk right now and
## stored nowhere -- what a search result or an attach dialog shows *before*
## the user commits to anything.
func vault_file_data(rel: String) -> Dictionary:
	if not _has("vault_file_data"):
		return {"ok": false, "error": "vault_file_data not available on this binary"}
	return world_gen.vault_file_data(rel)

## Every linked note's imported data for one entity, one Dictionary per link.
func vault_entity_data(kind: String, entity_id: int) -> Array:
	if not _has("vault_entity_data"):
		return []
	return world_gen.vault_entity_data(kind, entity_id)

## One link's imported data.
func vault_link_data(link_id: String) -> Dictionary:
	if not _has("vault_link_data"):
		return {"ok": false, "error": "vault_link_data not available on this binary"}
	return world_gen.vault_link_data(link_id)

## Which write confirmations the user has switched off: `{section, block,
## field_fill}`, all bool. These suppress the DIALOG, never the guard -- a
## caller with a preference set must still call the matching `vault_preview_*`
## (that is where `expect_hash` comes from) and simply not show it.
func vault_write_prefs() -> Dictionary:
	if not _has("vault_write_prefs"):
		return {}
	return world_gen.vault_write_prefs()

## Sets one of them. `path` is "section", "block" or "field_fill"; anything
## else returns false and changes nothing, so a typo cannot quietly disarm a
## confirmation.
func vault_set_write_pref(path: String, value: bool) -> bool:
	if not _has("vault_set_write_pref"):
		return false
	return world_gen.vault_set_write_pref(path, value)

## The preferences as JSON text, for the save file. Text, not a Dictionary --
## Godot's JSON floats every integer on the way through (GUI_GAP_REGISTER.md
## KV-04), so engine JSON travels as a string in both directions.
func vault_prefs_json() -> String:
	if not _has("vault_prefs_json"):
		return ""
	return world_gen.vault_prefs_json()

func vault_restore_prefs(json: String) -> bool:
	if not _has("vault_restore_prefs"):
		return false
	return world_gen.vault_restore_prefs(json)

## Removes a Cartalith-written block from a note. `expect_hash` comes from the
## matching preview and is what makes a note edited in between refuse instead
## of being overwritten.
func vault_remove_block(rel: String, kind: String, entity_id: int, expect_hash: String) -> Dictionary:
	if not _has("vault_remove_block"):
		return {"ok": false, "error": "vault_remove_block not available on this binary"}
	return world_gen.vault_remove_block(rel, kind, entity_id, expect_hash)

# -- F10 · project documents -------------------------------------------------

## `project_save` plus the caller's own documents: `{slot: json_text}` over the
## slots `project_document_slots()` lists. Text rather than a Dictionary for
## the same KV-04 reason as `vault_prefs_json` above.
func project_save_with_documents(path: String, extra_documents: Dictionary) -> Dictionary:
	if not _has("project_save_with_documents"):
		return {"ok": false, "error": "project_save_with_documents not available on this binary"}
	return world_gen.project_save_with_documents(path, extra_documents)

## Every slot the format defines.
func project_document_slots() -> PackedStringArray:
	if not _has("project_document_slots"):
		return PackedStringArray()
	return world_gen.project_document_slots()

## The subset the engine writes itself, which a caller may not supply.
func project_engine_owned_slots() -> PackedStringArray:
	if not _has("project_engine_owned_slots"):
		return PackedStringArray()
	return world_gen.project_engine_owned_slots()

func project_format_version() -> int:
	if not _has("project_format_version"):
		return 0
	return world_gen.project_format_version()

# -- F14 · read engine state back instead of trusting a shell copy -----------
#
# The setters were wired and these were not, so the shell kept its own copy of
# each toggle and nothing re-read the engine after a load. Same shape as the
# bug `route_get`'s `mode` carried until 2026-08-26.

func get_villages_enabled() -> bool:
	if not _has("get_villages_enabled"):
		return true
	return world_gen.get_villages_enabled()

func get_metropolis_enabled() -> bool:
	if not _has("get_metropolis_enabled"):
		return true
	return world_gen.get_metropolis_enabled()

func get_recovery_phase() -> int:
	if not _has("get_recovery_phase"):
		return 0
	return world_gen.get_recovery_phase()

## The engine's own ceiling for `lod_level_for_zoom`, so a caller clamps
## against the engine rather than against a number copied into GDScript.
func lod_max_level() -> int:
	if not _has("lod_max_level"):
		return 0
	return world_gen.lod_max_level()

# -- F13 · the two ops_bridge bindings the shell reaches for -----------------

## `_civRegionalPopulation` (reference line 23297): the modeled persons/km²
## field integrated over land, plus the painted-territory share. The
## reference's OTHER regional population figure -- distinct from
## `civ_agrarian_regional_total`'s settlement-sizing ceiling, and it never
## feeds back into it. `{total, land_km2, claimed}`, empty before a world.
##
## Recomputed fresh on every call, and comparable in cost to a real slice of
## Recompute civilisation -- call it from a button, never from a panel refresh.
func civ_regional_population() -> Dictionary:
	if not _has("civ_regional_population"):
		return {}
	return world_gen.civ_regional_population()

## Drops a whole asset collection by name. The grouping only -- the assets in
## it are untouched. A no-op, not an error, for a name that does not exist.
func as_drop_collection(name: String) -> Dictionary:
	if not _has("as_drop_collection"):
		return {"ok": false, "error": "as_drop_collection not available on this binary"}
	return world_gen.as_drop_collection(name)

# -- F13 · the two Journey Planner readouts ---------------------------------

## `_jpPackRange` (reference line 19518, v1.48): the wagon-equation ceiling,
## stated BEFORE the user configures their way past it. A pack animal carries
## its own fodder, so with no grazing there is a hard duration past which its
## whole capacity is its own food and it can carry nothing else. The number
## here IS the threshold `jp_auto_pick_transport`'s `fodder_infeasible` guard
## fires at -- the two read the same inputs, so the advisory can never disagree
## with the refusal. Pure: callable before `generate()`.
func jp_pack_range(plan: Dictionary, has_desert: bool) -> Dictionary:
	if not _has("jp_pack_range"):
		return {}
	return world_gen.jp_pack_range(plan, has_desert)

## `jpVesselMatrix` (reference line 17984): every vessel × every water type,
## plus which hull is fastest on each. "What is actually fast HERE", not the
## same vessel everywhere. `waters` comes back in the reference's own physical
## order (calm → rapids, sheltered → rough), not alphabetically.
func jp_vessel_matrix() -> Dictionary:
	if not _has("jp_vessel_matrix"):
		return {}
	return world_gen.jp_vessel_matrix()
