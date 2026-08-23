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

## Same degrade-rather-than-crash probe for the heightmap-import pair
## (`WorldGen::import_heightmap` / `heightmap_grid_size`). The welcome
## screen and the Data manager both hide their import affordance outright
## when this is false, rather than drawing a button that cannot work.
var import_api := false

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
	import_api = world_gen.has_method("import_heightmap") \
		and world_gen.has_method("heightmap_grid_size")
	gpu_api = world_gen.has_method("gpu_enumerate_devices") \
		and world_gen.has_method("gpu_set_multi_mode")
	npr_api = world_gen.has_method("set_npr") \
		and world_gen.has_method("get_npr")
	measure_api = world_gen.has_method("measure_section") \
		and world_gen.has_method("measure_area") \
		and world_gen.has_method("measure_radius") \
		and world_gen.has_method("measure_vertical")
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

# -- Parameters ---------------------------------------------------------------

func _read_param_table() -> void:
	if not world_gen.has_method("get_param_info"):
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
	if world_gen.has_method("get_param_groups"):
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

	if world_gen.has_method("set_experimental_flags"):
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
	if world_gen.has_method("set_metropolis_enabled"):
		world_gen.set_metropolis_enabled(bool(request.get("metropolis", false)))
	if world_gen.has_method("set_recovery_phase"):
		world_gen.set_recovery_phase(int(request.get("recovery_phase", 0)))
	## `civBiomeKChk` (reference line 1406 / `_biomeK` line 6441), the third
	## one -- default off, same guard, added 2026-08-23 (`PARITY_AUDIT.md`
	## §5 item 12: the engine parameter always existed, nothing could set it).
	if world_gen.has_method("set_biome_k_enabled"):
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

## Town layouts for the given settlement indices (`urban_bridge.rs`,
## `URBAN_MORPHOLOGY_SCOPE.md` milestones 1-7). Shorter than `indices`
## whenever the engine refuses a settlement -- a pin in open water gets no
## town, which is the reference's own `_umModelFor` refusal. Each entry
## carries its own `index` back, and its `stages` array names which generator
## stages produced it; there is no `blocks`/`buildings`/`wall` key at all,
## because milestones 8-17 do not exist and an empty array would read as
## "this town has none". Empty against a binary built before this landed, the
## same `has_method` guard `lod_tile_cells()` above uses for its own milestone.
func urban_layouts(indices: PackedInt32Array) -> Array:
	if not world_gen.has_method("urban_layouts"):
		return []
	return world_gen.urban_layouts(indices)

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
	if not world_gen.has_method("center_landmasses"):
		return {"ok": false, "reason": "This build of the engine has no centring pass."}
	var r: Dictionary = world_gen.center_landmasses()
	if bool(r.get("ok", false)) and int(r.get("offset", 0)) != 0:
		world_loaded.emit()
	return r

## Overdeepen the glacially-carvable coastal valleys into fjords. Returns
## the engine's summary dictionary: `ok`, `cells_masked`, `cells_carved`,
## and `reason` when `ok` is false.
func carve_fjords() -> Dictionary:
	if not world_gen.has_method("carve_fjords"):
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
	if not world_gen.has_method("can_undo"):
		return false
	return world_gen.can_undo()

## The operation `undo_last()` would revert ("Sculpt commit", "Carve fjords"),
## or "" when there is nothing to revert.
func undo_label() -> String:
	if not world_gen.has_method("undo_label"):
		return ""
	return String(world_gen.undo_label())

## Reverts the height field one step. Returns the reverted operation's label,
## or "" if nothing happened. The caller repaints -- the engine deliberately
## does not re-run flow/climate here (see `undo.rs`).
func undo_last() -> String:
	if not world_gen.has_method("undo_last"):
		return ""
	return String(world_gen.undo_last())

## `depth`, `max_steps`, `bytes`, `budget_bytes`, `step_bytes`, `label` --
## the reference's `#undoMem` readout as data. Empty dictionary on an older
## cdylib.
func undo_stats() -> Dictionary:
	if not world_gen.has_method("undo_stats"):
		return {}
	return world_gen.undo_stats()

func set_undo_budget_mb(mb: int) -> void:
	if world_gen.has_method("set_undo_budget_mb"):
		world_gen.set_undo_budget_mb(mb)

func clear_undo() -> void:
	if world_gen.has_method("clear_undo"):
		world_gen.clear_undo()


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


# civ_roster_bridge.rs -- the place editor, the faction roster and the two
# readouts `PARITY_AUDIT.md` §5 items 3/4/7/9/10/12 found unported.

func civ_settlement_details(index: int) -> Dictionary:
	if not world_gen.has_method("civ_settlement_details"):
		return {}
	return world_gen.civ_settlement_details(index)

## `fields` carries only the keys that changed -- see the Rust doc comment;
## an invalid value rejects the whole batch rather than half-applying it.
func civ_edit_settlement(index: int, fields: Dictionary) -> bool:
	if not world_gen.has_method("civ_edit_settlement"):
		return false
	return world_gen.civ_edit_settlement(index, fields)

func civ_settlement_toggle_trait(index: int, key: String) -> bool:
	if not world_gen.has_method("civ_settlement_toggle_trait"):
		return false
	return world_gen.civ_settlement_toggle_trait(index, key)

func civ_reroll_settlement_name(index: int) -> String:
	if not world_gen.has_method("civ_reroll_settlement_name"):
		return ""
	return world_gen.civ_reroll_settlement_name(index)

func civ_delete_settlement(index: int) -> bool:
	if not world_gen.has_method("civ_delete_settlement"):
		return false
	return world_gen.civ_delete_settlement(index)

func civ_faction_count() -> int:
	if not world_gen.has_method("civ_faction_count"):
		return 0
	return world_gen.civ_faction_count()

func civ_add_faction() -> int:
	if not world_gen.has_method("civ_add_faction"):
		return -1
	return world_gen.civ_add_faction()

func civ_remove_faction() -> bool:
	if not world_gen.has_method("civ_remove_faction"):
		return false
	return world_gen.civ_remove_faction()

func civ_set_faction_field(faction: int, key: String, value: String) -> bool:
	if not world_gen.has_method("civ_set_faction_field"):
		return false
	return world_gen.civ_set_faction_field(faction, key, value)

func civ_faction_terrain_fits() -> Array:
	if not world_gen.has_method("civ_faction_terrain_fits"):
		return []
	return world_gen.civ_faction_terrain_fits()

func civ_agrarian_regional_total() -> Dictionary:
	if not world_gen.has_method("civ_agrarian_regional_total"):
		return {}
	return world_gen.civ_agrarian_regional_total()

func civ_trait_vocabulary() -> Array:
	if not world_gen.has_method("civ_trait_vocabulary"):
		return []
	return world_gen.civ_trait_vocabulary()

func civ_specialisation_vocabulary() -> Array:
	if not world_gen.has_method("civ_specialisation_vocabulary"):
		return []
	return world_gen.civ_specialisation_vocabulary()

func civ_religion_vocabulary() -> Array:
	if not world_gen.has_method("civ_religion_vocabulary"):
		return []
	return world_gen.civ_religion_vocabulary()

func civ_government_vocabulary() -> Array:
	if not world_gen.has_method("civ_government_vocabulary"):
		return []
	return world_gen.civ_government_vocabulary()

func civ_ag_tech_vocabulary() -> Array:
	if not world_gen.has_method("civ_ag_tech_vocabulary"):
		return []
	return world_gen.civ_ag_tech_vocabulary()

func civ_culture_vocabulary() -> PackedStringArray:
	if not world_gen.has_method("civ_culture_vocabulary"):
		return PackedStringArray()
	return world_gen.civ_culture_vocabulary()

func set_biome_k_enabled(enabled: bool) -> void:
	if not world_gen.has_method("set_biome_k_enabled"):
		return
	world_gen.set_biome_k_enabled(enabled)

func get_biome_k_enabled() -> bool:
	if not world_gen.has_method("get_biome_k_enabled"):
		return false
	return world_gen.get_biome_k_enabled()


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


# geojson_bridge.rs -- GUI_GAP_REGISTER.md DM-03. Empty before the first
# generate()/load, and on a build whose GDExtension predates the binding.
func export_geojson() -> String:
	if not world_gen.has_method("export_geojson"):
		return ""
	return world_gen.export_geojson()


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

## The wildlife ecoregion under a click, for the Wildlife view's roster
## popup (the reference's own `showWildInfo`). `{}` when the click missed
## every region marker, when the Wildlife view has no world to read, or on
## an engine build without the binding.
func wildlife_region_at(gx: float, gy: float) -> Dictionary:
	if not world_gen.has_method("wildlife_region_at"):
		return {}
	return world_gen.wildlife_region_at(gx, gy)


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

## Decodes a sprite sheet and holds it on the session for slicing (AS-09).
## `{"ok": true, "w", "h", "name"}` or `{"ok": false, "error": ...}`. PNG only.
func as_load_sheet(sheet_name: String, bytes: PackedByteArray) -> Dictionary:
	if not world_gen.has_method("as_load_sheet"):
		return {"ok": false, "error": "as_load_sheet not available on this binary", "w": 0, "h": 0}
	return world_gen.as_load_sheet(sheet_name, bytes)

## Drops the loaded sheet (the slicer modal closing).
func as_clear_sheet() -> bool:
	if not world_gen.has_method("as_clear_sheet"):
		return false
	return world_gen.as_clear_sheet()

## The real `N cells detected · M non-empty` pass plus the overlay's grid lines
## (AS-09). `{"ok", "total", "non_empty", "usable", "col_x0"/"col_x1"/"row_y0"/
## "row_y1"}` -- the four span arrays are in sheet pixels, engine-computed, so
## the overlay draws exactly the cells `as_slice_apply` will cut.
func as_slice_preview(opts: Dictionary) -> Dictionary:
	if not world_gen.has_method("as_slice_preview"):
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
	if not world_gen.has_method("as_slice_apply"):
		return {"ok": false, "error": "as_slice_apply not available on this binary",
			"added": 0, "skipped_blank": 0, "unplaced": 0, "uids": PackedStringArray()}
	return world_gen.as_slice_apply(opts)

## AS-07: writes one item's scale/pan directly. `false` for an unknown
## uid/index or an older binary.
func as_set_item_transform(uid: String, index: int, scale: float, pan_x: float, pan_y: float) -> bool:
	if not world_gen.has_method("as_set_item_transform"):
		return false
	return world_gen.as_set_item_transform(uid, index, scale, pan_x, pan_y)

## AS-07: resets one item's transform to identity, re-fitting to the slot's
## family when `fit` is true. `{"ok", "scale", "pan_x", "pan_y"}` or
## `{"ok": false}` for an unknown uid/index or an older binary.
func as_reset_item_transform(uid: String, index: int, fit: bool) -> Dictionary:
	if not world_gen.has_method("as_reset_item_transform"):
		return {"ok": false}
	return world_gen.as_reset_item_transform(uid, index, fit)

## AS-17: moves interior line `index` of `lines` to `frac`, clamped strictly
## between its neighbours -- `lines` unchanged on an older binary.
func as_slicer_move_line(lines: PackedFloat64Array, index: int, frac: float) -> PackedFloat64Array:
	if not world_gen.has_method("as_slicer_move_line"):
		return lines
	return world_gen.as_slicer_move_line(lines, index, frac)

## AS-17: the uniform `n+1`-line array a fresh grid (or a cols/rows edit)
## falls back to. Empty on an older binary.
func as_uniform_lines(n: int) -> PackedFloat64Array:
	if not world_gen.has_method("as_uniform_lines"):
		return PackedFloat64Array()
	return world_gen.as_uniform_lines(n)
