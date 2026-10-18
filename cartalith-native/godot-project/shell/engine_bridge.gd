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

## Emitted when a generate (`generate`) or a heightmap import (`import_heightmap`) starts its
## worker thread; connected by `busy_overlay.gd`, `app.gd` and `adjust_panel.gd`.
signal generation_started()
## Emitted with `ok` when a generate or import ends (and with `false` when `generate` refuses
## before starting); `_ready` also connects to it.
signal generation_finished(ok: bool)
## One emission per observed change of `GenerationProgress.snapshot()`'s
## `stage` while `generating` is true -- the Android spec's "per-stage
## progress + log" (`ANDROID_UI_SPEC.md` §1.3). `index` is `0..total-1`,
## `name` is the engine's own name for that stage (`cartalith-engine/src/
## progress.rs`'s `STAGE_NAMES`, which stays in the same order as this
## file's own `WorldWorkspace.STAGES`), `total` is the stage count (`10` on
## a current build). Never emitted at all against an older cdylib with no
## `GenerationProgress` class -- see `progress_api` below, the same
## degrade-rather-than-crash shape every `_has()` guard in this file uses.
signal generation_stage(index: int, name: String, total: int)
signal params_changed()            ## A dial moved; downstream is stale.
signal params_applied()            ## A generate landed; nothing is stale.
signal world_loaded()              ## A save or asset pack changed the world.
signal dirty_changed(dirty: bool)  ## `world_dirty` flipped.
## The sculpt draft stack changed shape (a stamp added, removed, reordered,
## undone, redone, committed or discarded). Added 2026-09-01 for
## `world_workspace.gd`'s WORLD - Terrain - Draft section, which reads
## `sculpt_stamp_count()` ONCE at build time and gates its Commit/Discard
## buttons on it, so a stroke drawn after that panel was built left both
## buttons greyed with a non-empty draft behind them. `_deadwire_probe` found
## it the first time it audited that surface.
##
## **Emitted after the engine call, and that is load-bearing.** Godot delivers
## signals synchronously. **Corrected 2026-09-05:** this said "both listeners
## re-read `sculpt_stamp_count()` inside the emit". Only one does —
## `right_dock.gd:569` connects `_sculpt_draft_backstop` with `CONNECT_DEFERRED`,
## so it runs a frame later and was never handed a pre-change count. The ordering
## below is still load-bearing, for the synchronous listener. Every emitter used
## to fire before its own
## `world_gen.sculpt_*()`, so both were handed the count from before the change
## (`_rdrefresh_probe.gd` T0, measured 2026-09-05: 0 stamps reported on the
## stroke that created the draft's first).
##
## **Two listeners, not one.** The line here used to add that the right dock
## "re-reads the stack every time `show_sculpt_stack()` rebuilds it, so it never
## needed one"; that is true of every shell gesture and false of the one that
## is not a gesture. A direct `bridge.sculpt_*` call rebuilds nothing, and the
## dock then draws a body built for the old count -- so `right_dock.gd` listens
## too, deferred and gated on the count having moved, as a backstop that costs
## nothing on the covered paths. See `_sculpt_draft_backstop()` there.
signal sculpt_draft_changed()
signal project_saved(path: String) ## A `.zip` was written.
## A landmark pass finished, carrying that run's own reply dictionary.
## Deliberately not `generation_finished` -- see `landmark_run()` far below
## for why a pass that changes no world state must not fire that one.
signal landmark_finished(result: Dictionary)
## A viewshed refine finished (Ruling AT), carrying `landmark_refine_status()`
## read on the main thread afterwards. Its own signal for `landmark_finished`'s
## reason: it changes no world state, so `generation_finished` is wrong too.
signal landmark_refine_finished(result: Dictionary)
## The raster layer stack was **accepted** -- visibility, opacity, blend mode or
## order (`GUI_GAP_REGISTER.md` CA-03/CA-04, RD-10). Two surfaces edit one stack
## (CARTO ▸ Layers ▸ Terrain raster, and the right dock's appended Layers
## section) and each is built once, so without this the one that did not make
## the change would keep drawing the arrangement from before it. The same defect
## `sculpt_draft_changed` above exists for, in the same two docks.
##
## Emitted only when `set_layer_stack()` returns 3. A refusal changes nothing,
## so there is nothing to re-read and no repaint to spend.
signal layer_stack_changed()
## `color_texture()` just rebuilt the base map texture and, with it, the river
## colour texture (`river_color_texture()`). Listened to by
## `map_overlay.gd::set_river_source`, whose base-view river stroke is textured
## with the river colour texture and must redraw against the new one. Added
## 2026-09-27 for the white-rivers regression from `2cf0143`: half the shell
## repaints with `map_view.texture = bridge.color_texture()` and never redraws
## the overlay, and the stroke's recorded texture was the one just replaced
## (freed), which the renderer draws as white. Emitted after the rebuild, so a
## listener reading `river_color_texture()` gets the new texture.
signal color_texture_rebuilt()

## What a click does to a selection set -- the wire codes
## `icon_hit_test_mode()`/`label_hit_test_mode()` take, matching
## `cartalith-godot/src/selection.rs`'s own `SelectMode::from_i64`.
##
## The shell already had exactly this convention and this is not a fourth one:
## `asset_library_window.gd:2174-2188`'s tile grid is plain-click-replaces,
## Ctrl/Cmd-click-toggles, Shift-click-extends-a-range. It is also the
## platform-conventional set on all three desktops.
const SEL_REPLACE := 0
## Selection mode: Ctrl/Cmd-click toggles; returned by `selection_mode_from_input`.
const SEL_TOGGLE := 1
## Selection mode: Shift-click extends a range; returned by `selection_mode_from_input`.
const SEL_EXTEND := 2

## The mode a real `InputEventMouseButton`-era click carried, read off the
## `Input` singleton at handler time. The tool click primitive
## (`map_overlay.gd`'s `map_clicked(gx, gy)` -> `app.gd::_on_map_clicked`)
## carries no modifier and is consumed by six workspaces, so widening its
## signature for two of them would be the larger change by far; `Input` is
## queried synchronously inside the same dispatch, so it reports the state the
## click was made with.
static func selection_mode_from_input() -> int:
	if Input.is_key_pressed(KEY_SHIFT):
		return SEL_EXTEND
	if Input.is_key_pressed(KEY_CTRL) or Input.is_key_pressed(KEY_META):
		return SEL_TOGGLE
	return SEL_REPLACE

## The reason text a mutator that answers `{"ok": false, "error": ...}` (or `reason`) gives while `generating`: the worker holds
## `WorldGen`, so the call is refused rather than made. Caller-gate audit 2026-10-12: every mutator and `civ_*` / `tl_*` /
## `conflict_*` wrapper gates itself, so a caller that forgot to test `generating` is refused with this, not a `bind() failed`
## panic. Never worded as a missing capability ("this build has no ..."): the build is fine, only the moment is wrong.
const _BUSY_TEXT := "The world is generating -- try again when it finishes."

## The engine object (`WorldGen`, from the GDExtension). The worker thread owns it while
## `generating`, so callers outside the worker must gate on `generating` (see `_BUSY_TEXT`).
## Replaced by a fresh instance when `close_world()` swaps it.
var world_gen: WorldGen = WorldGen.new()

## True from the moment `generate`/import/resample/landmarks/viewshed start a run until its finish
## handler clears it; the gate every `world_gen` wrapper tests. `busy_kind` says which run.
var generating := false
## WHICH run set `generating` last: `"generate"`, `"import"`, `"resample"`, `"landmarks"` or
## `"viewshed"`, written by each of the five entry points immediately before it sets `generating`
## and never cleared (read it only while `generating` is true). Read by `busy_overlay.gd` so its
## title and loading lines name the action really running -- the landmark pass and the viewshed
## refine emit no start signal and import/resample share `generation_started` with a generate, so
## nothing else tells them apart. A plain field: reading it never touches `world_gen`.
var busy_kind := ""
## True once a world exists: set by `_finish`, the import finish and `load_save`, cleared by
## `close_world`. A plain flag, so reading it never touches `world_gen`.
var has_world := false
## One-line description of the last generate, import or open ("N x M cells, ... seed S"), or the
## failure/refusal text; blanked by `close_world`.
var last_summary := ""
## Map width in km after the last generate or import, from `world_gen.get_map_width_km()` when the
## sized API exists (see `_finish`); read for the zoom limit and scale bar.
var last_width_km := 0.0
## Map height in km after the last generate; from `world_gen.get_map_height_km()` when the sized
## API exists, else `width_km` (square), as `_finish` does. Mirrors `last_width_km`.
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
## empty when the archive carried none or the flat reader was used. Nothing
## parses it on the way through, because Godot's JSON floats every integer it
## touches.
##
## **Read it from `DccApp._restore_project_documents()` and nowhere else.**
## It is a snapshot of the last *opened* archive, and it outlives that open:
## `world_loaded` also fires for a centring pass, a fjord carve, an applied
## asset pack and a close, and restoring from this dictionary on any of those
## replays the file's documents over whatever the user has authored since.
## That is how the journey list was being reset to its on-disk state every
## time someone pressed `Center landmasses`. Cleared by `close_world()`, for
## the same reason.
var last_documents: Dictionary = {}
## What the last successful `load_save()` read: `"tree"` or `"flat"`, and
## whatever the reader could not use. Empty before the first open. See
## `load_save()` for why discarding these mattered.
var last_open_layout := ""
## Whatever the last `load_save()` could not use; reset to empty at the start of `load_save` and in
## `close_world`.
var last_open_warnings := PackedStringArray()
## Which engine-owned payloads the last `project_open()` actually applied --
## `project_open`'s own `restored` key (`"civ"`, `"labels"`, `"icons"`,
## `"ways"`, `"region"`, `"appearance"`, `"vault"`, `"landmark settings"`,
## `"paint layers"`, `"sculpt draft"`). Empty before the first open, and after
## a flat-reader fallback, which restores nothing.
##
## Read alongside `last_documents`, not instead of it: the pair answers "the
## archive carried this, and it did / did not come back", which is the only
## way `_restore_project_documents()` can tell a draft that was refused for a
## grid mismatch from one the archive never held.
var last_open_restored := PackedStringArray()

## The staged generation readout (`ANDROID_UI_SPEC.md` §1.3). `GenerationProgress`
## is a second, stateless `RefCounted` class next to `WorldGen` -- deliberately
## not a `WorldGen` method, because the worker `Thread` `generate()` starts
## below holds `&mut WorldGen` for the run's whole duration and any `#[func]`
## on `world_gen` reached meanwhile fails its own `Gd<T>::bind()` (see the
## multi-GPU block's own note on exactly that failure, further down this
## file). Instantiated through `ClassDB` rather than a static
## `GenerationProgress.new()`: a static reference to an unregistered class
## name would fail to PARSE this whole file against an older cdylib, which is
## a harder break than every other degrade-on-missing-binding guard here
## produces -- `ClassDB.class_exists()`/`instantiate()` only fail the one
## call, at runtime, exactly like `_has()` already does for `world_gen`
## methods.
var progress_api := false
## The `GenerationProgress` instance made through `ClassDB.instantiate` in `_ready` (see the
## comment above for why not a static reference); null on a binary without it, and `progress_api`
## records that.
var _progress: Object = null
## The last `(run_token, stage)` this file has already turned into a
## `generation_stage` emission, so `_process` emits on change only -- see its
## own doc comment for why re-emitting every frame would bury the signal it
## exists to produce.
var _progress_seen_token := -1
## The last stage `_process` turned into a `generation_stage` emission; `-1` after a reset so the
## next stage emits. Pairs with `_progress_seen_token`.
var _progress_seen_stage := -1

## Whether the world has changed since it was last saved or opened
## (`GUI_GAP_REGISTER.md` FI-01). Cleared by `save_project()`, `load_save()`
## and `close_world()`. Three kinds of writer set it:
##
##   1. a finished generation (`generation_finished`);
##   2. `world_loaded` -- the centring pass, the fjord carve, an applied
##      asset pack, all of which emit it;
##   3. **every mutating wrapper in this file, through `mark_world_dirty()`**.
##
## (3) was missing until 2026-09-01, and its absence was the whole defect:
## the flag meant "a generate or an open happened", so a sculpt commit, an
## icon, a label, a civilisation edit or an undo made *after* a manual save
## left it `false`. Autosave gates on this flag and therefore stopped firing
## for the rest of the session, and the status bar said nothing. See
## `mark_world_dirty()` for why the marks are unconditional.
##
## `DccApp.close_project()` still prompts whenever a world exists rather than
## only when this is set: a close is the one moment where under-reporting
## costs the user work, and one belt-and-braces prompt is the cheap side of
## that trade. Autosave *does* gate on it, because re-writing an unchanged
## multi-hundred-megabyte world every few minutes is the worse failure there.
var world_dirty := false

## True when a parameter dial has moved since the last generate: set by `mark_dirty` (which emits
## `params_changed`) and cleared when a generate, import, open or close finishes. Distinct from
## `world_dirty`, which tracks unsaved work.
var params_dirty := false
var _param_info: Dictionary = {}     ## key -> the info Dictionary from Rust
## Default value per parameter key, read once from `world_gen.get_param_defaults()` in
## `_read_param_table`; served by `param_default`.
var _param_defaults: Dictionary = {}
## True when `_read_param_table` got a non-empty table from the engine; `_ready` tests it before
## using the parameters.
var _params_available := false
## The generation worker `Thread`, created in `generate`/import and nulled in the finish handler;
## `_exit_tree` joins it if still started.
var _thread: Thread

# -- Lifecycle ----------------------------------------------------------------

## Node entry. Restores the persisted CPU worker count first (before anything can run a Rayon
## `par_iter()`), probes the native `WorldGen` for each optional binding and records the result in the
## `*_api` flags the shell gates its controls on (an older cdylib simply leaves a flag false), connects
## the dirty-flag signal handlers, and restores GPU preferences, the parameter table and landmark kinds.
func _ready() -> void:
	## **First, before anything else in this function.** Rayon's global pool
	## builds exactly once per process and does it implicitly, so the persisted
	## worker count only means anything while nothing has run a `par_iter()`
	## yet. See `_restore_cpu_pref` for the full reasoning.
	_restore_cpu_pref()
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
	biome_colors_api = _has("set_biome_color") \
		and _has("reset_biome_color") \
		and _has("reset_biome_colors") \
		and _has("get_biome_color")
	colour_paint_api = _has("colour_paint_info") \
		and _has("colour_paint_document") \
		and _has("colour_paint_set_document") \
		and _has("colour_paint_add_mark") \
		and _has("colour_paint_set_visible")
	ramp_api = _has("get_color_ramp") \
		and _has("set_color_ramp") \
		and _has("list_ramp_presets") \
		and _has("load_ramp_preset")
	ramp_mode_api = _has("list_ramp_modes") \
		and _has("get_ramp_mode") \
		and _has("set_ramp_mode")
	color_space_api = _has("list_color_spaces") \
		and _has("get_color_space") \
		and _has("set_color_space")
	layer_stack_api = _has("get_layer_stack") \
		and _has("set_layer_stack") \
		and _has("list_blend_modes")
	look_api = _has("list_looks") \
		and _has("get_look") \
		and _has("set_look")
	preset_api = _has("save_appearance_preset") \
		and _has("load_appearance_preset") \
		and _has("peek_appearance_preset")
	adjust_api = _has("list_adjust_layers") \
		and _has("appearance_get_adjust") \
		and _has("appearance_set_adjust")
	measure_api = _has("measure_section") \
		and _has("measure_area") \
		and _has("measure_radius") \
		and _has("measure_vertical")
	if ClassDB.class_exists("GenerationProgress"):
		_progress = ClassDB.instantiate("GenerationProgress")
		progress_api = _progress != null
	## `project_save`, not `save_project`. The latter still exists and still
	## works, but since 2026-08-25 it writes the **flat interoperability
	## export** (`SAVEFILE_COMPAT.md` §1.1) -- seven root entries and no
	## project layer. Probing for it would leave every save affordance enabled
	## against a writer that silently drops the civ layer, which is worse than
	## a disabled button.
	save_api = _has("project_save")
	## The two signal-driven writers of `world_dirty`. The third -- the one
	## that catches a hand edit rather than a whole-world replacement -- is
	## `mark_world_dirty()`, called from inside each mutating wrapper below.
	world_loaded.connect(func(): _set_dirty(true))
	generation_finished.connect(func(ok: bool):
		if ok:
			_set_dirty(true)
		else:
			## The three `generation_finished.emit(false)` sites below (VRAM
			## refusal, generate failure, heightmap-import failure) all set
			## `last_summary` to the reason immediately before emitting --
			## signals are synchronous, so it is still that reason here.
			note_error(last_summary))
	_restore_gpu_prefs()
	_read_param_table()
	_read_landmark_kinds()
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

## Joins a still-running worker thread so quitting mid-generate never frees
## `world_gen` out from under it. Must not return with a started `Thread` unjoined.
func _exit_tree() -> void:
	if _thread != null and _thread.is_started():
		_thread.wait_to_finish()

## Polls `GenerationProgress.snapshot()` while a generate is in flight and
## turns a change into `generation_stage`. This is the ONLY thing in this
## file that touches `_progress` (a plain, separately-instantiated
## `RefCounted` -- it never reaches into `world_gen`, so calling it from the
## main thread while the worker owns `world_gen` is safe; see `progress_api`'s
## own doc comment above for why that distinction is the whole point of
## `GenerationProgress` existing as a second class).
##
## Emits on stage change only, never once per frame -- polled every `_process`
## tick (worst case ~1 frame of latency, cheap enough not to warrant a Timer)
## but the signal itself fires only when `stage` (or the run token) actually
## moved since the last tick.
##
## Also replays an undo budget chosen while a run held the engine (`set_undo_budget_mb`), on the
## first frame `generating` is false again -- every finish path (generate, import, resample,
## landmarks, refine) clears that flag, so one poll here covers them all.
func _process(_delta: float) -> void:
	if _pending_undo_budget_mb >= 0 and not generating:
		var mb := _pending_undo_budget_mb
		_pending_undo_budget_mb = -1
		set_undo_budget_mb(mb)
	if not progress_api or not generating:
		return
	var snap: Dictionary = _progress.snapshot()
	var token := int(snap.get("run_token", 0))
	if token != _progress_seen_token:
		## A new run started -- forget the previous run's last-seen stage so
		## this run's own first reading (even index 0 again) is not mistaken
		## for "nothing changed".
		_progress_seen_token = token
		_progress_seen_stage = -1
	var stage := int(snap.get("stage", 0))
	if stage == _progress_seen_stage:
		return
	_progress_seen_stage = stage
	generation_stage.emit(stage, String(snap.get("stage_name", "")), int(snap.get("stage_count", 10)))

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
# on Android.
#
# **Measured on the device 2026-09-07, and it corrects this comment.** The
# clause here used to add "unlike `print()`, which this project has
# repeatedly found does not survive the Android log path". On a release
# export to a OnePlus 6T (Android 15), `dcc_shell.gd`'s own
# `print("Cartalith shell build ", build_id())` arrives in the very first
# boot capture as `I/godot(27101): Cartalith shell build 8fd916035577`, so
# `print()` DOES survive on this template.
#
# `push_warning()` is still the right call here, for a better reason: it
# arrives at **priority E with an `at: push_warning` frame**, so a grep can
# tell an engine warning from ordinary output. A marker found in `logcat`
# without that frame proves nothing -- an unrelated Android subsystem
# printed the same unique string during the 2026-09-07 control run.
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

# -- Bindings with no forwarder, and why ---------------------------------------
#
# `_has()` above catches the *stale binary* case: the engine is older than the
# shell and a real binding is missing from it. This section is the other case,
# and the two get confused. A `#[func]` this file carries no wrapper for is
# invisible to the whole shell however healthy the `.dll` is, and the error a
# caller gets says nothing about the engine at all:
#
#   Nonexistent function 'X' in base 'Node (EngineBridge)'
#
# That has been read as a stale build twice in one week. It is not; it is this
# file being short a five-line wrapper, and the fix is here, not a rebuild.
#
# **Re-derived 2026-09-06.** All 468 `#[func]` attributes under
# `crates/cartalith-godot/src/`, matched against every `.gd` file in
# `godot-project/` -- probes included, 439 files when this was derived, a
# count that moves as probes land, so re-derive rather than trust it. Method:
# take every bare `#[func]` line and the `fn <name>` under it; cut every `.gd`
# line at its first unquoted `#`; report each name with no `name(` and no
# `"name"` left.
# Run with and without the comment strip -- the two answers agree here, which
# is what says no name is surviving only on a doc mention.
#
# Eight names come back, and they are NOT eight gaps. Two are reached by a
# route the grep cannot see, three are engine surface with the reason already
# written down beside them, three have no caller today. **A forwarder to a
# function no screen calls is a second dead end, not a fix**, so none was
# added. Per binding:
#
# REACHED, just not by name from GDScript -- adding a wrapper would duplicate
# a call the engine already makes for us:
#
#   * `asset_library_document_json` -- `project_bridge.rs:2840`,
#     `project_engine_built_documents()` composes it with the paint, sculpt and
#     travel document builders, and THAT is forwarded below and called by
#     `DccApp._project_documents()`. The assets slot reaches every save through
#     it. (Its three siblings are additionally called direct by
#     `_verifylanes_probe.gd`, which round-trips paint, sculpt and travel and
#     skips assets -- a probe coverage gap, not a binding gap.)
#   * `export_snapshot_png` -- `vault_bridge.rs:790`, inside `vault_snapshot`,
#     which is forwarded below and called by `vault_window.gd::
#     _generate_snapshot()`. Every §21 map snapshot goes through it.
#
# UNREACHED, with the reason already written down somewhere real -- open that
# before re-flagging any of these. Two are at the definition, one is on a
# shipping surface:
#
#   * `arc_label_line_width` (`ops_bridge.rs`) -- `map_overlay.gd::_draw_labels`
#     keeps its own halo width: that file holds no bridge reference and the
#     expression is inside a per-label draw loop, so a forwarder would add an
#     FFI round trip per label per frame to save one multiplication. **Its own
#     doc's grounds are stale and the conclusion is now stronger than they
#     say.** That doc calls the GDScript copy "one constant apart" and quotes it
#     as `maxi(1, int(font_px * 0.16))`; measured 2026-09-06, `map_overlay.gd`
#     reads `float(lb.get("halo_em", LABEL_HALO_EM_FALLBACK))` and then
#     `0 if halo_em <= 0.0 else int(maxf(1.0, font_px * halo_em))`. So 0.16 is
#     only the fallback for a row with no class spec, and there is a zero-halo
#     branch the engine function does not have (it would return 1 there, and the
#     design's halo slider starts at 0 meaning "no halo"). Routing this through
#     the binding would now CHANGE what is drawn, not just move the arithmetic.
#   * `project_read_document` (`project_bridge.rs`) -- carries a "No shell
#     caller, deliberately" section naming both candidate consumers and why
#     each was declined. It is the one capability `project_open` structurally
#     cannot offer (read one document, keep the current world).
#   * `geojson_inspect` (`geojson_bridge.rs`) -- its doc scopes it to
#     "validates and summarises rather than importing". The import itself
#     landed without it (`d79d776`): the Data manager's GeoJSON route calls
#     `apply_geojson_document` directly, with no preview step, so nothing
#     needs this summary yet. *(Corrected 2026-09-24: this said nothing could
#     ingest GeoJSON.)*
#
# UNREACHED, no caller today, and the missing piece is NOT the forwarder:
#
#   * `ping` (`WalkingSkeleton`, `lib.rs`) -- the Phase 0 load proof, on a class
#     that appears in no `.tscn` and no `.gd`. This node proves the library
#     loads by constructing `WorldGen`; `_has()` proves it is the right one.
#   * `vault_landmark_entity_id` (`vault_bridge.rs`) -- owner ruling 13
#     (2026-09-06) made `landmark` a vault entity kind and added this as the
#     only way to turn a `Landmark::key()` into an `entity_id`. There IS a
#     per-landmark panel and a landmark IS selected as one entity --
#     `right_dock.gd`'s `CTX_LANDMARK` context (`on_landmark_selected()`,
#     `_build_landmark()`), reached by clicking a landmark on the map. What
#     it lacks is the knowledge-link row `_knowledge_row()` gives
#     settlement/faction/province/continent/culture, which is what actually
#     calls `vault_window.open_for()`. The surface comes first; the wrapper
#     is four lines after the knowledge row exists on this one too.
#   * `labels_clear_generated` (`label_bridge/generate.rs`) -- drops the pass
#     output and leaves hand-placed labels alone. Its own doc says to call it
#     "when the world moves underneath it", and the three paths that move the
#     world drop or replace the whole `LabelBridge` instead, so none of them
#     needed it -- `absorb` assigns a fresh one, `release_world` and `load_save`
#     both set it to `None` (`lib.rs`). The path that DOES leave
#     a stale run is `sculpt_commit()` below, which emits only
#     `sculpt_draft_changed` -- neither `generation_finished` nor
#     `world_loaded` -- so `cartography_workspace.gd::_regenerate_labels()`
#     does not re-run and the map keeps drawing labels placed against the
#     pre-commit height field. **Clearing there is a UI decision, not a binding
#     decision**: with no re-run route on that path a clear trades a stale run
#     for a vanished one, which is the same shape as the `atlas_export_zip`
#     note further down. Registered rather than wired.
#
# `WorldGen::label_clear_all` is NOT the missing caller and was checked: it
# clears only the hand-placed collection, its shell button lives in the
# "Region labels" section and counts `label_list()`, and its Rust doc was
# corrected in the same pass for claiming it dropped every label.

# -- The last-error record ----------------------------------------------------
#
# `LARGE_ITEM_RULINGS.md`'s "Build" ruling on `Report an issue` names five
# readouts a diagnostic dump should write, and this is the one nothing in the
# codebase retained before it: `_report_failure()` (`app.gd`) -- the one place
# every user-facing failure already funnels through -- only ever wrote a
# status-bar label the next unrelated "hint" message overwrites, and
# `save_project()`/`load_save()` below already had the real failure reason in
# hand and only ever `push_warning()`ed it (console/logcat only, gone the
# moment the session ends or the log scrolls).
#
# Scoped deliberately to what THIS file can see and already has a real reason
# string for -- a failed generate/import/VRAM-refusal (`generation_finished`
# below), and a failed project save/load -- not every `_report_failure()` call
# in `app.gd`, several of which are purely local UI refusals ("this world has
# never been saved") with no engine-side reason to retain.
var _last_error: Dictionary = {}

## Records `text` (plus a wall-clock `at` stamp) as the session's last failure,
## replacing any earlier one. Callers pass a real engine reason, never a blank.
func note_error(text: String) -> void:
	_last_error = {"text": text, "at": Time.get_datetime_string_from_system()}

## `{}` when nothing has failed yet this session -- the honest answer, not a
## blank string standing in for one. `diagnostic_report.gd` dashes it with the
## reason above rather than printing an empty field.
func last_error() -> Dictionary:
	return _last_error

# -- Parameters ---------------------------------------------------------------

## Reads the engine's parameter metadata (`get_param_info`, `get_param_defaults`) and a fresh
## `get_params()` snapshot into the caches behind `param_info()`, `param_default()` and `params_available()`.
## Does nothing on a binary without `get_param_info`. Called from `_ready`, `load_save` and `close_world`,
## because loading or closing a world moves the dials.
func _read_param_table() -> void:
	if not _has("get_param_info"):
		return
	_param_info = world_gen.get_param_info()
	_param_defaults = world_gen.get_param_defaults()
	_params_available = not _param_info.is_empty()
	if _params_available:
		_params_cache = world_gen.get_params()

## True once `_read_param_table()` found a non-empty `get_param_info()` table.
## False against an older binary; parameter dialogs must then hide, not guess.
func params_available() -> bool:
	return _params_available

## The Rust-side metadata dictionary for dotted key `key` (`WorldGen.get_param_info`,
## read once at `_ready`). `{}` means "no such key / no table", not real metadata.
func param_info(key: String) -> Dictionary:
	return _param_info.get(key, {})

## Every dotted parameter key the engine published, in its own order. `[]` when the
## table is unavailable (`params_available()` false).
func param_keys() -> Array:
	return _param_info.keys()

## Forwards to `WorldGen.get_param_groups`: the dialog section names in display
## order. Empty `PackedStringArray` on a binary without it, not "no groups".
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers PackedStringArray().
func param_groups() -> PackedStringArray:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return PackedStringArray()
	if _has("get_param_groups"):
		return world_gen.get_param_groups()
	return PackedStringArray()

## The engine default for `key` from `WorldGen.get_param_defaults`, captured once
## at `_ready`. `null` for an unknown key or no table -- it is never a value.
func param_default(key: String):
	return _param_defaults.get(key)

## Last `get_params()` answer, so a read during a generation is served without
## reaching into an engine object the worker thread owns -- see the multi-GPU
## block's `_gpu_read` note for what happens when one does. Exact rather than
## approximate: `param_set` is the only writer and it is refused for the same
## window, so nothing can change the table while this stands in for it.
var _params_cache: Dictionary = {}

## Current value of dotted key `key`: `WorldGen.get_params()[key]`, or the snapshot
## taken before the worker started while `generating` (no `#[func]` is reached
## then). `null` for an unknown key or no table -- callers must test for it.
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

## Restores the whole table (`WorldGen.reset_params`) or just `keys` from the
## defaults captured at `_ready` (unknown keys are skipped), then `mark_dirty()`.
##
## **Refused outright while `generating`**, like `param_set` (found-bug sweep 2026-10-11, R7):
## the worker holds the `WorldGen` for the whole run, so reaching `reset_params` /
## `set_params` then is the `Gd<T>::bind() failed` panic the multi-GPU block documents,
## and a write could not have affected the world being built anyway. The two menu callers
## already test `generating`; this stops a third one that forgets. Nothing is marked dirty
## for a refused call, since nothing changed.
func reset_params(keys: Array = []) -> void:
	if generating:
		return
	if keys.is_empty():
		world_gen.reset_params()
	else:
		var restore := {}
		for k in keys:
			if _param_defaults.has(k):
				restore[k] = _param_defaults[k]
		world_gen.set_params(restore)
	mark_dirty()

## Forwards to `WorldGen.apply_archetype`, which writes the archetype's World
## Structure knobs into the persistent parameters. `false` for an unknown name or a
## binary without it; only a `true` marks the parameters dirty.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func apply_archetype(name: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("apply_archetype"):
		return false
	var ok: bool = world_gen.apply_archetype(name)
	if ok:
		mark_dirty()
	return ok

## Archetype names `apply_archetype` and the archetype branch of `generate` accept
## (`WorldGen.get_archetypes`). Empty on a binary without it.
func archetypes() -> PackedStringArray:
	if _has("get_archetypes"):
		return world_gen.get_archetypes()
	return PackedStringArray()

# -- New World seed preview (NWP-2 binding, NWP-3 consumer) ---------------------

## Whether the loaded extension exports the static `WorldGen.preview_land_mask`
## (`NEW_WORLD_PREVIEW_PROPOSAL.md` NWP-2). `false` on an older build, which the
## New World dialog dashes with its reason rather than drawing anything. Asks
## `has_method` on the object, which is a class-table lookup, not a `#[func]`
## call -- but the one caller still tests `generating` first.
func has_preview_binding() -> bool:
	return _has("preview_land_mask")

## A copy of the whole live parameter table, every key `get_params()` returns:
## the base of the preview binding's `params` argument (lane A's `BINDING.md`:
## every key is required, missing is a failure). Refreshed from the engine when
## no generation is running; the pre-generation snapshot while one is (no
## `#[func]` is reached then). `{}` on a build with no parameter table -- never
## a made-up table.
func params_snapshot() -> Dictionary:
	if not _params_available:
		return {}
	if not generating:
		_params_cache = world_gen.get_params()
	return _params_cache.duplicate()

## Forwards to `WorldGen.preview_land_mask(params, preview_w, target_w)`: the
## engine's own early tectonic stages at `preview_w` cells wide, rescaled to the
## `target_w` grid, as a row-major mask of `preview_w * preview_h` bytes
## (1 = land, 0 = ocean). **An empty array is the failure answer** -- a refused
## key, a stale binary, or a call while `generating`, which this refuses
## outright even though the binding is static and takes no `self`: the rule in
## this file is that no `world_gen` call is made while the worker owns it.
func preview_land_mask(params: Dictionary, preview_w: int, target_w: int) -> PackedByteArray:
	if generating or not _has("preview_land_mask"):
		return PackedByteArray()
	return world_gen.preview_land_mask(params, preview_w, target_w)

## Cartalith is a one-shot generator: a moved dial does not recompute a stage,
## it marks the world stale until the next full generate. §7's stale marks and
## the status bar both hang off this.
func mark_dirty() -> void:
	if params_dirty:
		return
	params_dirty = true
	params_changed.emit()

## The other half of `mark_dirty()`, and the one the SAVE path reads.
##
## `world_dirty` was driven entirely by the two signals `_ready()` connects --
## `world_loaded` and `generation_finished` -- which together mean "a generate
## or an open happened", not "the world changed". So every hand edit made
## after a manual save left the flag `false`: autosave gates on it and skipped
## those edits for the rest of the session, and `_refresh_save_status()`
## printed nothing where it should have said `unsaved changes`. A sculpt
## commit, an icon, a label, a civilisation edit and an undo are exactly the
## work an autosave exists to protect, and none of them were reaching it.
##
## Called from inside each mutating wrapper below rather than from its
## callers, so a new call site cannot forget it, and **unconditionally** --
## before the engine call, not gated on its return value. Marking an
## unchanged world dirty costs one redundant autosave beside the project;
## missing a real change costs the user's work, and only one of those two is
## recoverable.
##
## Guarded on `has_world` so the pre-first-generate shell, where every
## wrapper below is a no-op anyway, never reads as having unsaved changes.
##
## **Not** called from `sculpt_restore_document` / `asset_library_restore_document`
## / `travel_library_restore_document`: those run from
## `DccApp._restore_project_documents()` immediately after `load_save()` has
## cleared the flag, and a project that was just opened has nothing unsaved
## in it by definition.
func mark_world_dirty() -> void:
	if has_world:
		_set_dirty(true)

# -- Generation ---------------------------------------------------------------

## `request` carries everything the setup surface decided: seed, extent in km,
## grid dimensions, archetype (empty for Classic) and the five values that live
## outside the parameter table.
var last_generate_ms := 0
## `Time.get_ticks_msec()` at the start of a generate or import; `_finish` subtracts it to get
## `last_generate_ms`.
var _gen_start_msec := 0

## Starts an asynchronous generate on a worker thread and returns at once; the
## outcome arrives as `generation_finished(ok)` (and `params_applied` on success).
## Ignored while one is already running; may refuse on the VRAM policy. Never call
## a `world_gen` `#[func]` from the main thread until that signal.
func generate(request: Dictionary) -> void:
	if generating:
		return
	## PR-05, `Fallback when VRAM full ▸ Fail with error`. The check belongs
	## here rather than in Rust: `generate_terrain` returns a world, not a
	## `Result`, and "refuse and say why" is a UI act. The other two settings
	## need nothing here -- `cpu_tile_pass` is the engine silently taking its
	## existing CPU route.
	##
	## Only while GPU acceleration is on (ALIGNMENT_AUDIT Part 2 B11, fixed
	## 2026-09-24): `WorldGen::gpu_vram_estimate` judges the grid against the
	## budget without asking whether the GPU will be used at all, so with the
	## toggle off this refused a run that would never have touched the GPU.
	## `null` (no parameter table on this build) keeps the old always-check.
	var vram := {}
	var gpu_on = param_get("use_gpu")
	if gpu_on == null or gpu_on == true:
		vram = gpu_vram_estimate(int(request.get("grid_w", 0)), int(request.get("grid_h", 0)))
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
	busy_kind = "generate"
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
		String(request.get("archetype", "")),
		_warm_wanted()))

## Ruling BZ item 3 (2026-10-10): whether a worker should end its run with
## `WorldGen.warm_render_caches()` -- building the water flood, the river
## network, the valley shading and the river colour field there, so the first
## repaint (`ViewportHost.refresh()`, on the main thread) finds them cached
## instead of freezing the window to build them. Decided HERE, on the main
## thread before the worker starts: `_has()` reads the engine object, which
## the worker is about to own. `false` on an extension without the binding, and
## whenever `prewarm_render` is off.
func _warm_wanted() -> bool:
	return prewarm_render and _has("warm_render_caches")

## The control for `_genasync_probe.gd --mode identity`: `false` makes the
## next generate skip the warm, so the first repaint computes everything on the
## main thread as it did before Ruling BZ item 3 -- the synchronous path the
## warmed one must match byte for byte. Nothing in the shell writes it.
var prewarm_render := true

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
	## Through this file's own wrapper, not around it. The direct
	## `world_gen.set_biome_k_enabled(...)` that used to stand here is what
	## made that wrapper read as dead code -- and it also skipped the
	## `mark_world_dirty()` the wrapper performs, which is only harmless
	## because a generate follows immediately and marks the world itself.
	## `set_biome_k_enabled()` carries the identical `_has()` guard.
	set_biome_k_enabled(bool(request.get("biome_k", false)))


## Runs off the main thread. Touches only `world_gen` (plain Rust state), never
## a node.
##
## `generate_sized` and `generate_world_structure_sized` are both full,
## equally expensive `generate_terrain` calls mutating the same state -- this
## must be the ONE call site. The archetype branch is load-bearing: a non-empty
## archetype must reach `generate_world_structure_sized`, or the World shape
## choice never affects generation at all. Its bool return is the
## archetype-name check, surfaced as a real failure rather than swallowed.
##
## `warm`: end the run with `warm_render_caches()` (see `_warm_wanted()`), still
## on this thread and still before `_finish` hands the engine back -- so the
## warm cannot race an edit (every editing path refuses while `generating`),
## and a world that changed after it anyway is caught by the render's own cache
## keys, which drop the warmed result rather than draw it.
func _worker(seed_value: int, width_km: float, grid_w: int, grid_h: int, archetype: String, warm: bool = false) -> void:
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
	if ok and warm:
		world_gen.warm_render_caches()
	_finish.call_deferred(seed_value, width_km, ok)

## Main-thread completion of `_worker`: joins the thread, clears `generating`,
## reads the real extent back from the engine and emits `generation_finished`
## (`false` with `last_summary` as the reason on failure). Never run off-thread.
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
	busy_kind = "import"
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
		int(request.get("grid_w", 2048)),
		_warm_wanted()))

## Runs off the main thread. Touches only `world_gen`, never a node -- same
## contract `_worker` above holds to, the render warm included (`warm`).
func _import_worker(path: String, seed_value: int, width_km: float, grid_w: int, warm: bool = false) -> void:
	var ok: bool = world_gen.import_heightmap(path, seed_value, width_km, grid_w)
	if ok and warm:
		world_gen.warm_render_caches()
	_finish_import.call_deferred(path, seed_value, ok)

## Main-thread completion of `_import_worker`: same contract as `_finish`, but a
## success also emits `world_loaded` and the summary names the source file.
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

## The last grid size the engine reported, so `grid_size()` can answer while a
## worker thread holds `world_gen`. Reading through would panic
## `Gd<T>::bind() failed, already bound` across the gdext boundary, which
## `cartalith-rust-conventions` forbids -- a panic unwinding through a
## GDExtension callback can take the whole process down.
var _grid_size_cache := Vector2i.ZERO

## Working grid as `Vector2i(get_width, get_height)`. A zero width means no world
## yet. While `generating` it answers from `_grid_size_cache` (the last value read)
## rather than reaching the engine the worker thread owns.
func grid_size() -> Vector2i:
	## Guarded here, once, rather than at each of the ~30 callers: this is the
	## shared chokepoint they all route through. Became reachable when the
	## landmark pass moved off the main thread (2026-09-02) -- the freeze had
	## been making every sibling control unclickable, so responsiveness removed
	## an accidental guard. Demonstrated by `_poiverify_probe.tscn`.
	if generating:
		return _grid_size_cache
	_grid_size_cache = Vector2i(world_gen.get_width(), world_gen.get_height())
	return _grid_size_cache

## Forwards to `WorldGen.reference_grid_height`: the grid height the reference HTML
## app uses for `grid_w` (`world` true is 2:1). Returns `grid_w` (square) on a
## binary without the sized API.
func reference_grid_height(grid_w: int, world: bool) -> int:
	if sized_api:
		return world_gen.reference_grid_height(grid_w, world)
	return grid_w

## Builds the base map texture (`WorldGen::build_color_texture`), which also
## replaces the river colour texture (`river_color_texture()`), and then says
## so through `color_texture_rebuilt`. Every repaint in the shell comes through
## here -- `viewport_host.gd::refresh()` and the dozen direct
## `map_view.texture = bridge.color_texture()` lines in `app.gd`, `tool_bar.gd`,
## `menus.gd`, `right_dock.gd`, `world_workspace.gd`, `dcc_shell.gd` -- so this
## is the one place the base view's river stroke can learn its texture moved.
## Must never skip the emit: without it the overlay keeps drawing with the
## texture this call just replaced (white rivers, see the signal's comment).
func color_texture() -> Texture2D:
	## Mid-generation (row: unguarded bridge read wrappers): the worker holds `WorldGen`, so
	## serve the last texture this call built -- `null` before any -- and do NOT emit
	## `color_texture_rebuilt`: nothing was rebuilt, and the river overlay must not re-read.
	if generating:
		return _color_texture_cache
	var t: Texture2D = world_gen.build_color_texture()
	_color_texture_cache = t
	color_texture_rebuilt.emit()
	return t

## The last texture `color_texture()` built, served while `generating` (see there).
var _color_texture_cache: Texture2D = null

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

## `WorldGen.build_territory_texture`: the per-cell faction wash as a translucent
## RGBA overlay. May be `null` when the engine has no territory to draw; callers
## must test.
func territory_texture() -> Texture2D:
	return world_gen.build_territory_texture()

## `state.viz.territoryOpacity` -- how heavily the territory wash is laid over
## the map (`GUI_GAP_REGISTER.md` CA-17). Takes effect on the next
## `territory_texture()`; the caller repaints.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func set_territory_opacity(a: float) -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if not _has("set_territory_opacity"):
		return
	mark_world_dirty()
	world_gen.set_territory_opacity(a)

## Territory wash alpha, 0..1 (`WorldGen.territory_opacity`). The 82/255 stale-binary
## fallback is this port's own default (`TERRITORY_ALPHA_DEFAULT` in `lib.rs`).
func territory_opacity() -> float:
	if not _has("territory_opacity"):
		return 82.0 / 255.0
	return world_gen.territory_opacity()

## The default `territory_opacity` the Reset row returns to
## (`WorldGen.territory_opacity_default`); same 82/255 fallback on a stale binary.
func territory_opacity_default() -> float:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return 82.0 / 255.0
	if not _has("territory_opacity_default"):
		return 82.0 / 255.0
	return world_gen.territory_opacity_default()

## `WorldGen.build_province_boundary_texture`: province borders as a translucent RGBA
## line overlay, transparent elsewhere. May be `null` without a world.
func province_boundary_texture() -> Texture2D:
	return world_gen.build_province_boundary_texture()

## `WorldGen.get_settlements`: one dictionary per settlement (x, y, name,
## population, kind, ...). Indices into this array are the `index` other wrappers take.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func settlements() -> Array:
	if generating:
		return []
	return world_gen.get_settlements()

## `WorldGen.get_roads`: the generated (and committed manual) ways, one dictionary
## each with smoothed `points`; sea lanes are in `sea_routes()`.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func roads() -> Array:
	if generating:
		return []
	return world_gen.get_roads()

## `WorldGen.get_sea_routes`: sea-lane routes, same shape as `roads()` minus the way
## type.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func sea_routes() -> Array:
	if generating:
		return []
	return world_gen.get_sea_routes()

## `WorldGen.get_provinces`: one metadata dictionary per province. Cell counts live
## in `province_cell_counts()`.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func provinces() -> Array:
	if generating:
		return []
	return world_gen.get_provinces()

## `{id, cells}` per province from one pass over the province grid; `[]` when
## the grid is not known. Fetch once and cache, do not call per province.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func province_cell_counts() -> Array:
	if generating:
		return []
	return world_gen.get_province_cell_counts()

## `WorldGen.get_trade_balances`: per-settlement resource trade balance, same order
## and index as `settlements()`.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func trade_balances() -> Array:
	if generating:
		return []
	return world_gen.get_trade_balances()

## Every traced river run of Strahler order `min_order` and above, as the
## entity dictionaries `lib.rs::get_rivers` documents -- `order`, `km`,
## `discharge`, `catchment_km2`, `width_cells`, the two end cells, and the
## rest. `river_at()` (called direct off `world_gen` by `right_dock.gd`)
## returns one entry of this same shape for a click.
##
## **Pass `2` for anything that must agree with a written GeoJSON document.**
## `geojson_bridge.rs`'s `EXPORT_MIN_RIVER_ORDER` is `2`, and both sides build
## their set from the same `split_river_polylines(trace_river_polylines(order,
## recv, w, h, min_order), w, None)` pair -- `river_entities()` on this side,
## `export_geojson()` on that one -- so the two counts are equal by
## construction and not by coincidence. `1` traces every headwater trickle
## instead; `right_dock.gd`'s own note records 784 runs at order 1 against 128
## at order 2 for the world it measured.
##
## Empty for a loaded `.zip` save as well as before the first generate:
## `SAVEFILE_COMPAT.md` stores no channel topology, so `stream_order` and
## `channels` are `None` and there is nothing to trace. **That empty is an
## absence, not a measured zero** -- a caller putting the number in front of a
## user has to say which it is rather than print `0`, the way
## `data_manager_window.gd::_gis_count()` does.
##
## Refused mid-generation for the `Gd<T>::bind()` reason `stale_stages()` above
## is, and empty against a binary built before this landed.
func rivers(min_order: int) -> Array:
	if generating or not _has("get_rivers"):
		return []
	return world_gen.get_rivers(min_order)

## The Layers panel's Rivers switch: whether the base view draws its river
## stroke and the deep-zoom tiles rasterize theirs (`river_stroke.rs`). The
## caller redraws and rebuilds the tiles (`viewport_host.gd::set_layer_visible`). Refused mid-generation
## for the `Gd<T>::bind()` reason `rivers()` above is.
func set_rivers_in_map(shown: bool) -> void:
	if generating or not _has("set_rivers_in_map"):
		return
	world_gen.set_rivers_in_map(shown)

## The base view's river stroke (`WorldGen::river_view_mesh`): RV-2's
## triangle strip in screen pixels, with each vertex's UV into
## `river_color_texture()`. `{}` mid-generation (the `Gd<T>::bind()` reason
## `rivers()` above gives), and against a binary built before it existed.
func river_view_mesh(scale: Vector2, offset: Vector2, view: Rect2) -> Dictionary:
	if generating or not _has("river_view_mesh"):
		return {}
	return world_gen.river_view_mesh(scale, offset, view)

## One chunk of that stroke: the runs `first .. first + count` only, in draw
## order (`WorldGen::river_view_mesh_runs`). `map_overlay.gd`'s layer cache
## rebuilds the stroke a few chunks a frame after a zoom (2026-09-28, Ruling
## BP). `{}` mid-generation and against a binary without it -- the overlay then
## never asks, because `river_run_count()` below answers -1 there and it
## falls back to the whole call.
func river_view_mesh_runs(scale: Vector2, offset: Vector2, view: Rect2, first: int, count: int) -> Dictionary:
	if generating or not _has("river_view_mesh_runs"):
		return {}
	return world_gen.river_view_mesh_runs(scale, offset, view, first, count)

## How many runs `river_view_mesh_runs()` chunks over (`WorldGen::river_run_count`).
## `-1` mid-generation or against a binary without the ranged call, so the
## overlay knows to draw the stroke whole instead.
func river_run_count() -> int:
	if generating or not _has("river_run_count") or not _has("river_view_mesh_runs"):
		return -1
	return world_gen.river_run_count()

## The map with every river at full coverage in its styled colour, built by
## the last `color_texture()` (`WorldGen::river_color_texture`). `null`
## mid-generation, for a loaded save, and before any world.
func river_color_texture() -> Texture2D:
	if generating or not _has("river_color_texture"):
		return null
	return world_gen.river_color_texture()

## Which cells the map draws as water, one byte per cell (255 water), built by
## the last `color_texture()` beside the river colour texture
## (`WorldGen::river_water_mask`). The base view's river stroke discards itself
## on those cells, so water sits above the river (`map_overlay.gd::_draw_rivers`)
## -- when the map draws the cell coast; with RV-4's smooth shoreline it hides
## under `shore_field_texture()`'s water instead, and this stays the
## classification the probes read.
## `null` mid-generation, whenever `river_color_texture()` is, and against a
## binary built before it existed -- the stroke is then drawn unmasked, as it
## was.
func river_water_mask() -> Texture2D:
	if generating or not _has("river_water_mask"):
		return null
	return world_gen.river_water_mask()

## The smooth shoreline's field (RV-4), one float per cell, built by the last
## `color_texture()` beside the map texture (`WorldGen::shore_field_texture`,
## `render::shore_field`): positive on every cell the map draws as water,
## negative on every land cell, crossing zero on the sub-cell shoreline. Read
## by `map_shore.gdshader` (the base map) and `river_under_water.gdshader`
## (the stroke hiding under the water). `null` mid-generation, when the look
## draws the reference's cell coast (`TerrainAppearance::smooth_shores` off),
## and against a binary built before it existed -- both shaders then draw the
## cell coast they always did.
func shore_field_texture() -> Texture2D:
	if generating or not _has("shore_field_texture"):
		return null
	return world_gen.shore_field_texture()

## **RIM-1: the river as water in the base map's own pixels**
## (`WorldGen::river_field_texture`): a half-float RGBA distance field built by
## the last `color_texture()` -- per texel the distance to the nearest drawn
## river centreline, that river's half-width and its order-1 weight -- which
## `map_shore.gdshader` paints the river from. `null` mid-generation, when the
## look keeps the vector stroke (`TerrainAppearance::rivers_as_water` or
## `smooth_shores` off), for a grid over the field's texel budget, and against
## a binary built before it existed.
func river_field_texture() -> Texture2D:
	if generating or not _has("river_field_texture"):
		return null
	return world_gen.river_field_texture()

## Whether the base map paints the rivers itself (`WorldGen::rivers_painted`),
## in which case the overlay draws no stroke (RIM-6). `false` against a binary
## without it, so an older DLL keeps its stroke. Hit-testing never reads this.
func rivers_painted() -> bool:
	if generating or not _has("rivers_painted"):
		return false
	return bool(world_gen.rivers_painted())

## The shader's river parameters (`WorldGen::river_paint_params`): `{width,
## bank, bank_color}` -- the river width multiplier the field was built for,
## and (RIM-2) the preset's bank-outline opacity (0 = off) with its ink as a
## 0..1 Vector3. `{}` with no field.
func river_paint_params() -> Dictionary:
	if generating or not _has("river_paint_params"):
		return {}
	return world_gen.river_paint_params()

## Read-back for `set_rivers_in_map()`. `true` against a binary built before
## it existed, which draws its rivers whatever this says.
func rivers_in_map() -> bool:
	if not _has("rivers_in_map"):
		return true
	return bool(world_gen.rivers_in_map())

## Town layouts for the given settlement indices (`urban_bridge.rs`). Shorter
## than `indices` whenever the engine refuses a settlement -- a pin in open
## water gets no town, which is the reference's own `_umModelFor` refusal. Each
## entry carries its own `index` back, and its `stages` array reports what each
## generator stage actually produced.
##
## Since 2026-09-02 the adapter calls the reference's own `generate()` end to
## end, so an entry is a whole town: `buildings`/`building_ridge`/
## `building_tone`, `parcel_district`, `wall_ring`/`wall_gates`/`wall_style`,
## `markets` and `farmland` all arrived at once. `primaries` and `placed_len_m`
## went the other way -- `generate()` discards both, so `street_len_m`
## (`computeMetrics.totalLen`) replaces the second.
##
## The absent-vs-empty rule the keys are written to is unchanged: an **absent**
## key means the engine cannot produce the thing, an **empty** one means this
## town does not have it. So `wall_ring` is absent on a settlement the wall
## ladder never walled, and `buildings` is always present.
##
## Empty against a binary built before this landed, the same `has_method` guard
## `lod_level_for_zoom()` above uses for its own milestone.
func urban_layouts(indices: PackedInt32Array) -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("urban_layouts"):
		return []
	return world_gen.urban_layouts(indices)

## The Settlement diagnostics overlay's card data (`urban_bridge.rs`) --
## `um_site_profile`'s river classification plus `um_wall_spec`'s rung and
## `um_harbour_scale`, for every requested settlement in one call. Unlike
## `urban_layouts()` above this runs no town generation, so it is cheap
## enough to ask for every settlement on the map at once rather than one at
## a time. Empty against a binary built before this landed.
func settlement_diagnostics(indices: PackedInt32Array) -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("settlement_diagnostics"):
		return []
	return world_gen.settlement_diagnostics(indices)

## `WorldGen.explain_settlement`: the decomposed suitability score of settlement
## `index` (`score`, signed weighted `terms`, ...). `{}` for an out-of-range index
## or no world.
func explain_settlement(index: int) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	return world_gen.explain_settlement(index)

## Design canvas artboard `1h`'s "Generation rules" window, `lazy-riding-piglet.md`
## Batch F -- `urban_bridge.rs`'s flat dotted-key `Rules` surface. World-level,
## in-memory only: `get_active_urban_rules()` is `DEFAULT_RULES` until one of
## the writers below has run, matching `urban_layouts()`'s own unedited case.

## `DEFAULT_RULES`, as the dotted-key `Dictionary` every function below reads
## and writes (`"street.branch_angle_jitter"`, `"parcels.subdivision_cap"`, …).
## Empty against a binary built before this landed.
func get_default_urban_rules() -> Dictionary:
	if not _has("get_default_urban_rules"):
		return {}
	return world_gen.get_default_urban_rules()

## The rules `urban_layouts()` is actually generating against right now.
func get_active_urban_rules() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not _has("get_active_urban_rules"):
		return {}
	return world_gen.get_active_urban_rules()

## Applies a partial dotted-key `Dictionary` onto the active rules (the
## parameter tables' direct-edit path). Returns the resulting full rules
## `Dictionary` plus a `"rejected"` `PackedStringArray` of any keys that did
## not resolve.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func set_active_urban_rules(values: Dictionary) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("set_active_urban_rules"):
		return {}
	return world_gen.set_active_urban_rules(values)

## The "Wildness" slider -- runs the real `applyWildness(rules, w)` on the
## current active rules (via `cartalith_urban::apply_wildness`, the same
## formula the design canvas's own script documents) and returns the
## resulting full rules `Dictionary`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func apply_urban_wildness(w: float) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("apply_urban_wildness"):
		return {}
	return world_gen.apply_urban_wildness(w)

## The "Plot chaos" slider -- `cartalith_urban::apply_plot_chaos`'s real
## formula, the parcel-metrology counterpart to [apply_urban_wildness].
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func apply_urban_plot_chaos(c: float) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("apply_urban_plot_chaos"):
		return {}
	return world_gen.apply_urban_plot_chaos(c)

## Clears the active rules back to unedited -- `urban_layouts()` then
## generates every town at exactly `DEFAULT_RULES` again.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func reset_active_urban_rules() -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if _has("reset_active_urban_rules"):
		world_gen.reset_active_urban_rules()

## Replaces the active rules with a named Rust rule set
## (`cartalith_urban::rules_preset` -- `"market_town"`). Empty for an unknown
## id or an older binary, and the active rules are then left untouched.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func apply_urban_rules_preset(id: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("apply_urban_rules_preset"):
		return {}
	return world_gen.apply_urban_rules_preset(id)

## The vocabularies a per-settlement town plan can pick from (Ruling J):
## `cultures`/`rules_presets` as `[{id, name}]`, plus `default_culture`.
## Empty on an older binary.
func urban_town_plan_options() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not _has("urban_town_plan_options"):
		return {}
	return world_gen.urban_town_plan_options()

## One settlement's town-plan overrides -- `culture`, `rules_preset`,
## `variant`; see the Rust doc comment. All-or-nothing like
## `civ_edit_settlement`, and unlike it does not mark the civ layer stale: a
## town plan moves no territory, road or trade figure.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func civ_set_town_plan(index: int, fields: Dictionary) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("civ_set_town_plan"):
		return false
	mark_world_dirty()
	return world_gen.civ_set_town_plan(index, fields)

## `WorldGen.get_border_inset_frac`: the rendered plate's frame width as a fraction
## of the texture's width. `0.0` genuinely means "no frame" (or no world).
func border_inset_frac() -> float:
	return world_gen.get_border_inset_frac()

## Which GPU-eligible stages ran on the GPU in the last generate
## (`WorldGen.get_gpu_stages_used`). Empty on a binary without it.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func gpu_stages_used() -> PackedStringArray:
	if not generating and _has("get_gpu_stages_used"):
		return world_gen.get_gpu_stages_used()
	return PackedStringArray()

## Active appearance quality tier name (`WorldGen.get_quality_tier`; `"quality"` on
## a fresh session). Unguarded: needs a binary with the tier API.
func quality_tier() -> String:
	## While `generating` the last name read (`""` before any): `set_quality_tier` is refused
	## for the same window, so the cache cannot be stale.
	if generating:
		return _quality_tier_cache
	_quality_tier_cache = world_gen.get_quality_tier()
	return _quality_tier_cache

## The last name `quality_tier()` read, served while `generating`.
var _quality_tier_cache := ""

## Forwards to `WorldGen.set_quality_tier`; `false` and no change for an unknown
## tier name. Marks the world dirty before the call whatever the result.
func set_quality_tier(name: String) -> bool:
	## Refused while `generating` (the worker holds `WorldGen`), which is also why
	## `quality_tier()`'s mid-run cache cannot go stale.
	if generating:
		return false
	mark_world_dirty()
	return world_gen.set_quality_tier(name)

## Tier names, cheapest first (`WorldGen.list_quality_tiers`), so controls never
## duplicate the vocabulary.
func quality_tiers() -> PackedStringArray:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return PackedStringArray()
	return world_gen.list_quality_tiers()

## A suggested tier for this device (`WorldGen.get_recommended_quality_tier`).
## Nothing applies it; the default policy is an owner decision.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers "".
func recommended_quality_tier() -> String:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return ""
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
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not look_api:
		return []
	return Array(world_gen.list_looks())

## The named look in force (`WorldGen.get_look`). `""` when `look_api` is false --
## that is "no look API", not a look name.
func look() -> String:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return ""
	if not look_api:
		return ""
	return world_gen.get_look()

## Selects a named look (`WorldGen.set_look`). `false` for an unknown name or no
## look API, and nothing changes; marks the world dirty first.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func set_look(name: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not look_api:
		return false
	mark_world_dirty()
	return world_gen.set_look(name)

# -- NPR / "Painter" styles (`GUI_GAP_REGISTER.md` RN-01) ---------------------
#
# `has_method` guards for the same reason every wrapper above has one: these
# landed after earlier GDExtension binaries shipped, and the RENDER panel
# should disable its rows against an older `.dll` rather than crash on a
# missing method. `npr_api` is what the panel reads to decide that.
var npr_api := false

## The whole NPR block as a dictionary (`WorldGen.get_npr`), keyed as `set_npr` reads
## it. `{}` when `npr_api` is false.
func npr_settings() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not npr_api:
		return {}
	return world_gen.get_npr()

## Send one or more changed keys; the rest keep their current value. Returns
## how many keys the engine recognised, so a typo reads as 0 rather than as a
## silent no-op.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.
func set_npr(values: Dictionary) -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0
	if not npr_api:
		return 0
	mark_world_dirty()
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
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not appearance_api:
		return {}
	return world_gen.get_appearance()

## `[[key, min, max, label], ...]` -- the engine's own ranges, so a panel never
## builds a slider that can send a value the engine will clamp.
func appearance_tunables() -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not appearance_api:
		return []
	return world_gen.list_appearance_tunables()

## Send one or more changed keys; the rest keep their current value. Returns
## how many the engine recognised (0 = a typo, not a silent no-op).
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.
func set_appearance(values: Dictionary) -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0
	if not appearance_api:
		return 0
	mark_world_dirty()
	return world_gen.set_appearance(values)

## Stop overriding just these keys, so each returns to the tier/look value.
## Returns how many overrides were dropped; 0 on an older cdylib without it.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.
func drop_appearance_overrides(keys: PackedStringArray) -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0
	if not appearance_api or not world_gen.has_method("drop_appearance_overrides"):
		return 0
	mark_world_dirty()
	return world_gen.drop_appearance_overrides(keys)

## Hand every appearance value back to the active quality tier. Returns how
## many overrides were dropped.
func reset_appearance() -> int:
	if not appearance_api:
		return 0
	mark_world_dirty()
	return world_gen.reset_appearance()


# -- Biome colour table (`GUI_GAP_REGISTER.md` CA-19, Ruling P) ---------------
#
# Its own flag for the ramp's reason below: the four bindings shipped in
# `cc0f561`, after the tunables, so an older cdylib has the sliders and not
# these. Indices are 1-based, `CART_BIOME_COLS`' own convention.
#
# **Saved with the project** since 2026-09-24: `appearance.json`'s optional
# `biome_cols` member (`SAVEFILE_COMPAT.md` §13.2, `project_bridge.rs`'s
# `AppearanceDoc`). Until then an edit lasted the session and these setters
# deliberately did not call `mark_world_dirty()`, because marking a project
# unsaved for a change the save would not carry promised something it could
# not keep. Now the save carries it, so each setter marks the project dirty --
# **after** the engine call, and only when the engine says something changed
# (a refused index, a reset of a class that had no override, or a reset-all
# with nothing set changes nothing a save would write).
var biome_colors_api := false

## The colour biome class `index` (1..15) renders with right now -- the
## reference's own table unless it has been overridden.
## **Callers check `biome_colors_api` and `generating` first**: there is no
## honest colour to return without the binding, or while the worker holds the
## engine, and a stand-in black would read as a real table entry.
func biome_color(index: int) -> Color:
	return world_gen.get_biome_color(index)

## Override one class. `false` for an index outside 1..15 (nothing changes).
## The engine clamps each channel to 0..255; the alpha is not part of it.
func set_biome_color(index: int, c: Color) -> bool:
	if not biome_colors_api or generating:
		return false
	var ok: bool = world_gen.set_biome_color(index, c.r8, c.g8, c.b8)
	if ok:
		mark_world_dirty()
	return ok

## Drop one class's override. `true` only when one was actually set.
func reset_biome_color(index: int) -> bool:
	if not biome_colors_api or generating:
		return false
	var dropped: bool = world_gen.reset_biome_color(index)
	if dropped:
		mark_world_dirty()
	return dropped

## Drop every override. Returns how many were set.
func reset_biome_colors() -> int:
	if not biome_colors_api or generating:
		return 0
	var n: int = world_gen.reset_biome_colors()
	if n > 0:
		mark_world_dirty()
	return n


# -- Colour paint (Ruling CN, `COLOUR_PAINT_SCOPE.md` CP-2) ---------------------
#
# The engine holds the layer (`WorldGen::colour_paint`, marks in map
# kilometres) and composites it on the screen, the deep-zoom tiles and the
# export. Every call here is gated on `colour_paint_api` (the binding exists)
# and `generating` (the worker holds `WorldGen`; reaching a `#[func]` then is
# the `bind() failed` panic), and every one degrades to an honest "nothing"
# rather than a plausible value: `{}`, `""`, `false`.

## `true` when the loaded extension has the colour-paint binding (set in `_ready`).
var colour_paint_api := false

## `{marks, visible, emissive, revision}` plus `km_per_cell` when the world has a
## map width (absent otherwise: no scale, no paint). `{}` without the binding or
## while generating.
func colour_paint_info() -> Dictionary:
	if not colour_paint_api or generating:
		return {}
	var d = world_gen.callv("colour_paint_info", [])
	return d if d is Dictionary else {}

## The layer as its saved JSON document (`ColourPaintDoc`), `""` when there is no
## layer. The shell's undo keeps these strings.
func colour_paint_document() -> String:
	if not colour_paint_api or generating:
		return ""
	return String(world_gen.callv("colour_paint_document", []))

## Replaces the layer with `json` (`""` removes it). `{ok: true, marks}` or
## `{ok: false, error}`; a refused document changes nothing.
func colour_paint_set_document(json: String) -> Dictionary:
	if not colour_paint_api or generating:
		return {}
	var d = world_gen.callv("colour_paint_set_document", [json])
	if d is Dictionary and bool((d as Dictionary).get("ok", false)):
		mark_world_dirty()
	return d if d is Dictionary else {}

## Appends one mark (`ColourMark` JSON). Same reply shape as `colour_paint_set_document`.
func colour_paint_add_mark(json: String) -> Dictionary:
	if not colour_paint_api or generating:
		return {}
	var d = world_gen.callv("colour_paint_add_mark", [json])
	if d is Dictionary and bool((d as Dictionary).get("ok", false)):
		mark_world_dirty()
	return d if d is Dictionary else {}

## Shows or hides the layer. `true` when the flag changed.
func colour_paint_set_visible(v: bool) -> bool:
	if not colour_paint_api or generating:
		return false
	var changed := bool(world_gen.callv("colour_paint_set_visible", [v]))
	if changed:
		mark_world_dirty()
	return changed


# -- Colour ramp (`GUI_GAP_REGISTER.md` CA-02) ---------------------------------
#
# A second, later capability flag rather than a widening of `appearance_api`:
# the tunable surface shipped one commit earlier, so an in-between binary has
# the sliders and not the ramp, and the panel should lose the ramp block rather
# than fail to draw the sliders.
var ramp_api := false

## `["Earth", "Elevation", ...]` -- the engine's own named ramps.
func ramp_presets() -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not ramp_api:
		return []
	return Array(world_gen.list_ramp_presets())

## `[[position, Color], ...]`, sorted by position. Position is relative land
## elevation: 0 at the shoreline, 1 at the world's highest point. The `Color`'s
## **alpha is the stop's own opacity** (2026-08-24) -- the row shape did not
## change when per-stop alpha landed, which is why `ramp_api` still covers both.
func color_ramp() -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not ramp_api:
		return []
	return world_gen.get_color_ramp()

## Replace the whole ramp. Add, delete and reorder are all this one call --
## the engine sorts by position, so dragging a stop past its neighbour *is*
## the reorder. Returns the number of stops accepted (0 = nothing changed).
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.
func set_color_ramp(stops: Array) -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0
	if not ramp_api:
		return 0
	mark_world_dirty()
	return world_gen.set_color_ramp(stops)

## Loads a named colour ramp (`WorldGen.load_ramp_preset`). `false` for an unknown
## name or no ramp API; marks the world dirty first.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func load_ramp_preset(name: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not ramp_api:
		return false
	mark_world_dirty()
	return world_gen.load_ramp_preset(name)

## `["Linear", "Ease", "Step"]` -- the engine's own interpolation modes.
##
## A third flag, for the same reason `ramp_api` is a second one: Ease/Step and
## per-stop alpha shipped a commit after the ramp itself, so an in-between
## binary has the stop editor and no mode picker, and the panel should lose the
## picker rather than fail to draw the stops.
var ramp_mode_api := false

## Interpolation mode names the engine offers (`WorldGen.list_ramp_modes`, e.g.
## Linear/Ease/Step). `[]` when `ramp_mode_api` is false.
func ramp_modes() -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not ramp_mode_api:
		return []
	return Array(world_gen.list_ramp_modes())

## The ramp interpolation mode in force (`WorldGen.get_ramp_mode`). `""` when
## `ramp_mode_api` is false -- "unavailable", not a mode.
func ramp_mode() -> String:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return ""
	if not ramp_mode_api:
		return ""
	return String(world_gen.get_ramp_mode())

## `false` for a name this build does not have. The mode is a property of the
## ramp, not of a stop, and survives `set_color_ramp`.
func set_ramp_mode(name: String) -> bool:
	if not ramp_mode_api:
		return false
	mark_world_dirty()
	return world_gen.set_ramp_mode(name)


# -- The terrain and water adjustment stacks (ET-C2 bindings, ET-C4 panel) ------
#
# `EDITING_TOOLS_SCOPE.md` milestone C. A sixth capability flag beside the ramp's
# for the same reason those are separate: an in-between cdylib has the whole
# appearance surface and no adjustment stacks, and the panel should lose the two
# engine layers rather than fail to draw. The labels/overlays stack is NOT here:
# it is GDScript state (`shell/overlay_adjust.gd`) and never reaches the engine.
#
# **Every call here is gated on `generating`.** The worker thread owns
# `world_gen` for the whole of a generate, and `appearance_get_adjust` /
# `appearance_set_adjust` take the engine borrow, so a call from the main thread
# during a run is the `Gd<T>::bind()` panic `stale_stages()` refuses for the same
# reason. A read answers `{}` (absent, never an empty stack standing in for "I
# could not look"); a write answers `{ok: false, error}` and changes nothing.
var adjust_api := false

## The adjustable engine layers in the engine's own spelling (`["terrain",
## "water"]`), or `[]` without the binding or while generating.
func adjust_layers() -> Array:
	if not adjust_api or generating:
		return []
	return Array(world_gen.list_adjust_layers())

## `appearance_get_adjust(layer)`: `{ok, layer, stack (JSON text), entries,
## identity}`, `{ok: false, error}` for an unknown layer, or `{}` when the
## binding is absent or a generate is running (the caller must treat `{}` as
## "unknown", not as an empty stack).
func adjust_get(layer: String) -> Dictionary:
	if not adjust_api or generating:
		return {}
	return world_gen.appearance_get_adjust(layer)

## `appearance_set_adjust(layer, stack_json)`: replaces the layer's whole stack.
## `{ok: true, ...}` on success (and the project is marked unsaved, because the
## stacks ride in `appearance.json`); `{ok: false, error}` when the engine
## refuses (nothing changed) or when a generate is running (the engine was not
## called). `{"entries":[]}` clears the layer.
func adjust_set(layer: String, stack_json: String) -> Dictionary:
	if not adjust_api:
		return {"ok": false, "error": "this build has no adjustment stacks"}
	if generating:
		return {"ok": false, "error": "the world is generating; try again when it finishes"}
	var r: Dictionary = world_gen.appearance_set_adjust(layer, stack_json)
	if bool(r.get("ok", false)):
		mark_world_dirty()
	return r


# -- The output colour space (`LARGE_ITEM_RULINGS.md`, Colour management) ------
#
# A fourth flag beside `ramp_api`/`ramp_mode_api`, for the same reason those are
# second and third: this landed later than the appearance surface, so an
# in-between binary has every slider and no colour space, and the panel should
# lose the picker rather than fail to draw the sliders.
#
# **Deliberately not `mark_world_dirty()`, unlike every wrapper above.** The
# other appearance calls change what is saved -- the look, the ramp and the
# overrides all round-trip through `AppearanceDoc`. The display device does not:
# `WorldGen::color_space` is session state by design (it describes the monitor,
# not the map), so setting it changes no file and there is nothing to be dirty
# about. Marking it would put a false unsaved-changes star on the title bar.
var color_space_api := false

## `["sRGB", "Display P3"]` -- the display devices this binary can encode for.
func color_spaces() -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not color_space_api:
		return []
	return Array(world_gen.list_color_spaces())

## The ramp colour space in force (`WorldGen.get_color_space`). `""` when
## `color_space_api` is false -- "unavailable", not a space name.
func color_space() -> String:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return ""
	if not color_space_api:
		return ""
	return String(world_gen.get_color_space())

## `false` for a name this build does not have. Presentation only, and
## session-only: re-render to see it, and nothing is written to disk.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func set_color_space(name: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not color_space_api:
		return false
	return world_gen.set_color_space(name)

## ET-E3: the print (CMYK) gamut check for one picker colour
## (`WorldGen.color_print_gamut`, `EDITING_TOOLS_SCOPE.md` ET-E3). Returns
## `{delta_e, in_gamut, nearest, source}`, or **`{}`** when the loaded binary has
## no such binding or the engine has no CMYK profile -- "not checked", never
## "in gamut". `source` is `"own_icc"`: the engine's own synthetic coated-paper
## profile, not a press characterisation. `callv` because a GDScript direct call
## is arg-checked at parse time against the loaded binary, and this shell must
## still parse against one built before the binding existed. Stateless:
## presentation only, nothing marked dirty.
func color_print_gamut(c: Color) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if world_gen == null or not _has("color_print_gamut"):
		return {}
	var d = world_gen.callv("color_print_gamut", [c])
	return d if d is Dictionary else {}


# -- The raster layer stack (`GUI_GAP_REGISTER.md` CA-03/CA-04, RD-10) ---------
#
# A **fifth** flag beside `ramp_api`/`ramp_mode_api`/`color_space_api`, for the
# same reason those are second, third and fourth: `render::LayerStack` and its
# three bindings landed after every earlier cdylib shipped, so an in-between
# binary has the whole appearance surface and no separable categories, and the
# Layers panel should lose its rows rather than fail to draw the ramp.
#
# **Deliberately not `mark_world_dirty()`, and for a different reason than the
# colour space above.** That one is session state by design. This one is real
# `TerrainAppearance` state that a *saved look* carries (`save_appearance_preset`
# writes the merged `appearance()`, layers included) -- but the project `.zip`
# does not: `project_bridge.rs`'s `AppearanceDoc` round-trips quality, look,
# territory opacity, the override map, the ramp and the NPR block, and has **no
# layers field**. So a reordered stack survives a Save look and does not survive
# File ▸ Save + File ▸ Open. Marking the project dirty would put an unsaved-
# changes star on a title bar over a save that demonstrably does not carry the
# change -- a promise the format cannot keep. The panel says so in a note
# instead; the missing `AppearanceDoc` field is engine work, reported rather
# than papered over here.
var layer_stack_api := false

## The three separable raster categories, **top-first** -- the order a layer
## list draws them, which is the reverse of the order the renderer composites
## them in. Each row is a Dictionary carrying `id`, `label`, `visible`,
## `opacity` and `blend`.
func layer_stack() -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not layer_stack_api:
		return []
	return world_gen.get_layer_stack()

## Replace the whole stack, top-first. Visibility, opacity, blend mode **and
## order** are all this one call, on `set_color_ramp`'s own precedent: the panel
## sends the arrangement it wants and the engine takes it, so moving a row past
## its neighbour *is* the reorder.
##
## Returns 3 on success and 0 on any refusal, with nothing changed. All three
## ids exactly once or the whole call is refused; `visible`/`opacity`/`blend`
## are each optional **within** a row and an absent key means *unchanged*, never
## defaulted -- so a caller may send a visibility toggle without restating the
## opacities, and must not send a key it is not deliberately setting.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.
func set_layer_stack(rows: Array) -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0
	if not layer_stack_api:
		return 0
	var n: int = world_gen.set_layer_stack(rows)
	if n == 3:
		layer_stack_changed.emit()
	return n

## `["Normal", "Multiply", "Screen", "Overlay", "Add"]` -- the engine's own
## order. The picker is this list, not a second copy of it in GDScript.
func blend_modes() -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not layer_stack_api:
		return []
	return Array(world_gen.list_blend_modes())


# -- Appearance presets (`GUI_GAP_REGISTER.md` CA-08) --------------------------
#
# A named look, saved beside the project rather than inside it: a look is
# reusable across worlds, which is the whole reason to save one. See
# `WorldGen::save_appearance_preset` for why it is not a block in the `.zip`.

## Where named looks live. `user://` so it survives an export and is writable
## on Android, and one folder so the picker is a directory listing.
const PRESET_DIR := "user://appearance_presets"

## Whether the engine exposes `save_appearance_preset`, probed with `_has` in `_ready`;
## `save_appearance_preset` refuses without it.
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

## Writes the appearance in force as a JSON preset under `user://appearance_presets`
## (`WorldGen.save_appearance_preset(path, name)`). `false` on failure, no preset API, or while
## `generating` (the worker holds the engine then; reaching it is the `bind() failed` panic).
##
## Marks the world dirty first, unconditionally, like `load_ramp_preset` (found-bug sweep
## 2026-10-11, R7, as the coordinator specified). **Conservative, not load-bearing:** the engine
## writes a sidecar file under `user://appearance_presets` and does not touch the world, so this
## costs one redundant autosave and the unsaved-changes star after a save; it exists so the two
## preset wrappers and `load_ramp_preset` behave alike (`mark_world_dirty`'s own doc on why a
## spurious mark is the recoverable error).
##
## ET-F3: `preset` / `variant` name the style-preset variant the look is saved under; both
## non-empty writes the file's top-level `"style": {preset, variant}` through
## `WorldGen.save_appearance_preset_styled`. Either empty -- a base preset, Custom, every
## caller before ET-F3 -- takes the original call and writes the original bytes. A styled save
## on a cdylib without the styled call is REFUSED (`false`) rather than written without the
## variant: a look saved as "Antique (Dark)" that reopened as Custom would be a silent loss.
func save_appearance_preset(name: String, preset: String = "", variant: String = "") -> bool:
	if not preset_api or generating:
		return false
	if preset != "" and variant != "":
		if not world_gen.has_method("save_appearance_preset_styled"):
			return false
		mark_world_dirty()
		return bool(world_gen.callv("save_appearance_preset_styled", [_preset_path(name), name, preset, variant]))
	mark_world_dirty()
	return world_gen.save_appearance_preset(_preset_path(name), name)

## ET-F3: the `{preset, variant}` a saved look was saved under (`WorldGen.
## peek_appearance_preset_style`), read WITHOUT loading it; `{}` when the look has none, the
## file cannot be read, the cdylib predates the call, or a generate holds the engine (the
## worker owns it then). `name` is a display name or a slug, as `load_appearance_preset` takes.
func appearance_preset_style(name: String) -> Dictionary:
	if not preset_api or generating or not world_gen.has_method("peek_appearance_preset_style"):
		return {}
	return Dictionary(world_gen.callv("peek_appearance_preset_style", [_preset_path(name)]))

## Reads the preset file for `name` and makes it the look (`WorldGen.load_appearance_preset`).
## `false`, nothing changed, on any read/parse error, no preset API, or while `generating`.
## Marks the world dirty first, unconditionally, for the reason `save_appearance_preset` gives
## (R7): the look in force changed, and a failed read costs one redundant autosave, a missed
## change costs the user's work.
func load_appearance_preset(name: String) -> bool:
	if not preset_api or generating:
		return false
	mark_world_dirty()
	return world_gen.load_appearance_preset(_preset_path(name))

## The sidecar that carries a saved look's labels & overlays adjustment stack, beside the
## engine's `<slug>.json`. The stack is GDScript state (`overlay_adjust.gd`) the engine never
## sees, so the engine's preset file cannot hold it; this file does, so a look carries all
## three stacks (Terrain and Water inside `<slug>.json`, this one beside it). **Not
## `.json`**, so `appearance_presets()` never offers it as a look. Pure path arithmetic.
func preset_overlay_path(name: String) -> String:
	return "%s/%s.overlay_adjust" % [PRESET_DIR, _preset_slug(name)]

## Writes the overlay stack beside the look `name` (`json` is `OverlayAdjust.get_stack_json()`),
## or removes a stale sidecar when `json` is empty, so re-saving a look over an older one
## never leaves the older look's overlay stack behind. False only if a write failed.
func save_preset_overlay(name: String, json: String) -> bool:
	var path := preset_overlay_path(name)
	if json == "":
		if FileAccess.file_exists(path):
			return DirAccess.remove_absolute(ProjectSettings.globalize_path(path)) == OK
		return true
	DirAccess.make_dir_recursive_absolute(PRESET_DIR)
	var f := FileAccess.open(path, FileAccess.WRITE)
	if f == null:
		return false
	f.store_string(json)
	f.close()
	return true

## The overlay stack saved beside the look `name`, or `""` when the look has none (every look
## saved before this sidecar existed, and any look saved with an empty overlay stack).
func read_preset_overlay(name: String) -> String:
	var path := preset_overlay_path(name)
	if not FileAccess.file_exists(path):
		return ""
	return FileAccess.get_file_as_string(path)

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

## Physical GPUs the machine exposes, discrete first (`WorldGen.gpu_enumerate_devices`).
## Served from `_gpu_cache` while generating; `[]` without the GPU API.
func gpu_devices() -> Array:
	return _gpu_read("devices", [], func(): return world_gen.gpu_enumerate_devices())

## Device keys dispatch uses (`WorldGen.gpu_selected_devices`); empty means
## automatic. Cached while generating.
func gpu_selected_devices() -> PackedStringArray:
	return _gpu_read("selected", PackedStringArray(), func(): return world_gen.gpu_selected_devices())

## Chooses the dispatch device(s) and persists them in `DccSettings`. Silently
## refused while `generating` or without the GPU API; takes effect next generate.
func gpu_set_selected_devices(keys: PackedStringArray) -> void:
	if gpu_api and not generating:
		world_gen.gpu_set_selected_devices(keys)
		_gpu_cache["selected"] = keys
		DccSettings.set_gpu_devices(keys)

## Multi-GPU mode name (`WorldGen.gpu_multi_mode`); `"single_device"` is also the
## fallback when the API is absent or nothing is cached yet.
func gpu_multi_mode() -> String:
	return _gpu_read("mode", "single_device", func(): return String(world_gen.gpu_multi_mode()))

## Sets the multi-GPU mode and persists it. `false` while `generating`, without the
## API, or for a mode the engine rejects (unknown, or unimplemented `alternate_frames`).
func gpu_set_multi_mode(mode: String) -> bool:
	if not gpu_api or generating:
		return false
	var ok: bool = world_gen.gpu_set_multi_mode(mode)
	if ok:
		_gpu_cache["mode"] = mode
		DccSettings.set_gpu_mode(mode)
	return ok

## The VRAM cap in GB (`WorldGen.gpu_vram_budget_gb`). `0.0` means no cap, which is
## also the fallback with no API.
func gpu_vram_budget_gb() -> float:
	return _gpu_read("budget", 0.0, func(): return float(world_gen.gpu_vram_budget_gb()))

## Sets the VRAM cap (`0` removes it) and persists it. A silent no-op while
## `generating` or without the GPU API.
func gpu_set_vram_budget_gb(gb: float) -> void:
	if gpu_api and not generating:
		world_gen.gpu_set_vram_budget_gb(gb)
		_gpu_cache["budget"] = gb
		DccSettings.set_gpu_vram_budget_gb(gb)

## Over-budget fallback name (`WorldGen.gpu_vram_fallback`); `"cpu_tile_pass"` is the
## engine default and the fallback with no API.
func gpu_vram_fallback() -> String:
	return _gpu_read("fallback", "cpu_tile_pass", func(): return String(world_gen.gpu_vram_fallback()))

## Sets the over-budget fallback and persists it. `false` while `generating`, without
## the API, or for a name the engine rejects (unknown or unimplemented).
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

## Measured per-device GPU memory of the last GPU generate
## (`WorldGen.gpu_last_device_usage`). `[]` means none recorded, not zero usage.
func gpu_last_device_usage() -> Array:
	return _gpu_read("usage", [], func(): return world_gen.gpu_last_device_usage())

## The readback ban, and the one thing that lifts it.
##
## When a GPU readback fails, `cartalith-gpu` records that grid size against
## the adapter and refuses that size and every larger one on it for the rest of
## the process (`crates/cartalith-gpu/src/lib.rs:1217
## multi::readback_failure_cells` decides, `:1240 multi::note_readback_failure`
## records). There is no way back short of a restart, which is why
## `multi.rs::clear_readback_failures()`'s own doc asks for a "try the GPU
## again" affordance -- `menus.gd::_build_gpu_retry_row()` is it.
##
## Not routed through `_gpu_read()`: that helper answers its `busy_value` when
## `gpu_api` is false, and these two are a **separate binding pair** from the
## multi-GPU API `gpu_api` tests for. A build can have one without the other,
## and `_has()` is the question that fits.

## Whether anything is banned right now. `false` on an older cdylib, which is
## also the answer that keeps `menus.gd`'s row honestly dark -- it never claims
## a ban it has not been told about.
##
## Cached across a generation for `_gpu_read()`'s reason: the worker thread
## holds the engine mutably borrowed, and a `bind()` through it while it does
## is the failure `gpu_settings_locked()` exists to prevent.
func gpu_readback_failed() -> bool:
	if not _has("gpu_readback_failed"):
		return false
	if generating:
		return bool(_gpu_cache.get("readback_failed", false))
	var v: bool = world_gen.gpu_readback_failed()
	_gpu_cache["readback_failed"] = v
	return v

## The backend the **last generate actually opened** -- `"vulkan"`, `"dx12"`,
## `"metal"`, `"gl"` -- or `""` when it opened no GPU device at all and the run
## went to the CPU. `""` before the first generate of the session too.
##
## Measured, unlike `menus.gd::_active_backend()`, which reads
## `gpu_enumerate_devices()` and reports the backend a device request *would*
## prefer. That inference cannot notice that the request landed somewhere else,
## or that it opened nothing -- and on Android that is the entire question,
## since `wgpu` is compiled into the shipped `.so` with nothing gating it off
## and `_ready()` above turns `use_gpu` on at boot.
##
## `_has()`-guarded and cached across a generation for exactly the reasons
## `gpu_readback_failed()` above documents: an older cdylib lacks the binding,
## and the worker thread owns `world_gen` while a generate runs.
func gpu_last_backend() -> String:
	if not _has("gpu_last_backend"):
		return ""
	if generating:
		return String(_gpu_cache.get("last_backend", ""))
	var v := String(world_gen.gpu_last_backend())
	_gpu_cache["last_backend"] = v
	return v

## Clear the record. `true` if the call reached the engine; `false` if it could
## not (no binding, or a generation is running), which is the same answer the
## other GPU setters give and for the same reason.
func gpu_clear_readback_failures() -> bool:
	if generating or not _has("gpu_clear_readback_failures"):
		return false
	var ok: bool = world_gen.gpu_clear_readback_failures()
	if ok:
		_gpu_cache["readback_failed"] = false
	return ok

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

# -- CPU worker threads (SS2.5 Performance) -----------------------------------
#
# `WorldGen.cpu_*` are `&self` methods over a process-global in
# `cartalith-engine`, so they carry the same two hazards as the multi-GPU
# block above and are guarded the same way: `_has()`, because an older cdylib
# does not export them, and `generating`, because the worker thread holds
# `world_gen` mutably borrowed and a `bind()` through it while it does is the
# `Gd<T>::bind() failed, already bound` crash that block documents.
#
# **`cpu_thread_count_active()` is not cached** the way `_gpu_read` caches:
# the one number a reader wants from it is whether a preference has landed,
# and answering that from a snapshot taken before the pool existed is the
# same class of lie this pair was rewritten to stop telling. It answers `0`
# while a generation runs, which `menus.gd` renders as "not readable right
# now" rather than as "no pool".

## Logical cores this machine reports -- the ceiling a chooser clamps to.
## `0` when the binding is missing, which callers must read as "unknown".
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.
func cpu_logical_core_count() -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0
	if not _has("cpu_logical_core_count"):
		return 0
	return int(world_gen.cpu_logical_core_count())

## The preference the engine holds, `0` for "follow Rayon's own default".
## Not `DccSettings.cpu_thread_count()`: that is what is persisted on disk,
## this is what the running engine was actually told, and `_restore_cpu_pref`
## is the only thing that makes them agree.
func cpu_thread_count() -> int:
	if not _has("cpu_thread_count") or generating:
		return 0
	return int(world_gen.cpu_thread_count())

## What Rayon's global pool is **really** running, `0` before it exists (or
## while a generation makes it unreadable). Measured, not requested.
func cpu_thread_count_active() -> int:
	if not _has("cpu_thread_count_active") or generating:
		return 0
	return int(world_gen.cpu_thread_count_active())

## Store the preference and try to apply it. **Returns whether it actually
## took effect this process** -- `false` means "recorded, takes effect at next
## start", which is the honest answer any time the pool is already up, and it
## is up as soon as anything has drawn. Persisted either way: the whole value
## of the `false` answer is that the next launch honours it.
func cpu_set_thread_count(threads: int) -> bool:
	DccSettings.set_cpu_thread_count(threads)
	if not _has("set_cpu_thread_count") or generating:
		return false
	return bool(world_gen.set_cpu_thread_count(threads))

## Push the persisted worker count into the engine, **first thing at
## startup**, which is the only moment it can still change anything: Rayon
## builds its global pool once per process, implicitly, the first time any
## `par_iter()` runs anywhere -- and `render.rs`/`sample_bridge.rs`/`bake.rs`
## all reach one long before a settings row is opened. `_ready()` calls this
## before `_restore_gpu_prefs()` for that reason and no other.
##
## Unset (`0`) is left alone rather than pushed: `0` is the engine's own
## "follow Rayon's default" and setting it would build the pool early for no
## gain, taking the choice away from a later caller who might set a real one.
func _restore_cpu_pref() -> void:
	var n := DccSettings.cpu_thread_count()
	if n > 0 and _has("set_cpu_thread_count"):
		world_gen.set_cpu_thread_count(n)

# -- Files --------------------------------------------------------------------

## Bytes free on the volume `path` lives on, or `-1` when it cannot be
## determined -- an unmounted drive, a path whose parent does not exist, a
## platform whose statvfs equivalent refused, or a GDExtension too old to have
## the binding at all. **`-1` is the only "I do not know" answer**, so a caller
## must branch on it explicitly rather than treating it as "zero free" and
## refusing every save on an old build.
##
## Exists so a save or an export can refuse with a real sentence before it
## writes, instead of failing halfway and reporting "save failed -- see
## console". Nothing in this shell checked free space anywhere: `grep` for
## disk / space / ENOSPC / free-bytes across `shell/` found nothing at all.
##
## `path` rather than no argument because the answer is per-volume: a project
## on an external drive and the atlas cache under `user://` routinely sit on
## different filesystems, and the one that matters is whichever the bytes are
## about to be written to.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers -1.
func disk_free_bytes(path: String) -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return -1
	if not _has("disk_free_bytes"):
		return -1
	return int(world_gen.disk_free_bytes(path))

## `MVP_SCOPE.md` criterion 7: opens a real HTML-app `.zip` and renders that
## save's terrain. `load_save` reads the save's own stored fields directly --
## no `generate` call is involved, so nothing here goes through the worker.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func load_save(path: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
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
	var open_err := ""
	last_open_layout = ""
	last_open_warnings = PackedStringArray()
	last_open_restored = PackedStringArray()
	if _has("project_open"):
		var r: Dictionary = world_gen.project_open(path)
		ok = bool(r.get("ok", false))
		if ok:
			documents = r.get("documents", {})
			## `project_open` reports which of the two layouts it found, what
			## it could not use, and the archive's own format version. All four
			## were being discarded here, so a user opening a legacy flat
			## archive was told nothing -- even though the conversion is
			## ONE-WAY: this build reads both layouts and writes only the tree,
			## so a save round-tripped back to the browser app will not open
			## there. Kept for `_load_project` to say so.
			last_open_layout = String(r.get("layout", ""))
			var w = r.get("warnings", PackedStringArray())
			if w is PackedStringArray:
				last_open_warnings = w
			elif w is Array:
				for item in w:
					last_open_warnings.append(String(item))
			var rs = r.get("restored", PackedStringArray())
			if rs is PackedStringArray:
				last_open_restored = rs
			elif rs is Array:
				for item in rs:
					last_open_restored.append(String(item))
		else:
			open_err = String(r.get("error", "unknown"))
			push_warning("Cartalith: project_open could not read %s (%s) -- falling back to the flat reader"
				% [path, open_err])
	if not ok:
		ok = world_gen.load_save(path)
	last_documents = documents
	if not ok:
		note_error("open %s failed: %s" % [path,
			open_err if open_err != "" else "the engine's flat reader refused it too"])
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
		## `project_open` restored the archive's own Sculpt draft *inside* the
		## engine, so nothing on this side went through
		## `sculpt_restore_document()` and nothing emitted the signal
		## `world_workspace.gd::_refresh_sculpt_draft` listens on. Without this
		## the stamps are in the editor and the stack panel is empty until the
		## next edit -- which reads exactly like the loss this restore fixed.
		if last_open_restored.has("sculpt draft"):
			sculpt_draft_changed.emit()
	return ok

## `File ▸ Save project` / `Save as…` (`GUI_GAP_REGISTER.md` FI-01) — writes
## the current world to `path` as a `.zip` in the format
## `SAVEFILE_COMPAT.md` documents. Returns `false` (leaving any existing file
## untouched) when the engine has no writer, when there is no world, or when
## the write itself fails; the engine logs the reason.
## `preview_png` is optional PNG bytes for the archive's `preview.png`
## thumbnail slot (`SAVEFILE_COMPAT.md`) -- the caller captures and encodes
## it (`DccApp._capture_preview_png()`), because this bridge has no
## rendered world texture of its own. Empty writes no `preview.png` entry,
## same as omitting it.
## `include_lod_tiles` is owner rulings 28/29's optional stored LOD pyramid
## -- default `false` (`GUI_GAP_REGISTER.md`, `OUTSTANDING_WORK.md` §2.4).
## `true` synthesizes and writes `cartography/tiles/**`
## (`WorldGen::project_save_with_documents`'s own doc has the cost); an
## older binary with no `project_save_with_documents` degrades the same way
## `extra_documents`/`preview_png` already do, with a warning rather than a
## silent drop.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func save_project(path: String, extra_documents: Dictionary = {},
		preview_png: PackedByteArray = PackedByteArray(),
		include_lod_tiles: bool = false) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
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
	## store (a document, a preview thumbnail, OR the LOD-tile flag),
	## `project_save` otherwise -- the former is what the latter calls
	## anyway, so this is one branch for one guard rather than two paths.
	## `extra_documents` maps a registered slot to that document's JSON TEXT;
	## a Dictionary would go through Godot's JSON, which floats every integer,
	## and that is precisely how KV-04 discarded every knowledge link.
	var r: Dictionary
	var need_with_documents := not extra_documents.is_empty() or not preview_png.is_empty() \
		or include_lod_tiles
	if not need_with_documents or not _has("project_save_with_documents"):
		if include_lod_tiles and not _has("project_save_with_documents"):
			push_warning("Cartalith: this binary has no project_save_with_documents -- " +
				"the LOD tile pyramid was not written to %s" % path)
		r = world_gen.project_save(path)
	else:
		r = world_gen.project_save_with_documents(path, extra_documents, preview_png, include_lod_tiles)
	var ok: bool = bool(r.get("ok", false))
	if ok:
		_set_dirty(false)
		project_saved.emit(path)
	else:
		var reason := String(r.get("error", "unknown"))
		push_warning("Cartalith: could not write %s (%s)" % [path, reason])
		note_error("save %s failed: %s" % [path, reason])
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
	## The closed project's caller-owned documents go with it. Leaving them
	## here meant the *next* thing to read `last_documents` -- a generate, a
	## centring pass, anything that reached `DccApp`'s restore -- put the
	## closed project's journeys into an unrelated world, carrying route
	## indices that no longer address anything.
	last_documents = {}
	last_open_layout = ""
	last_open_warnings = PackedStringArray()
	last_open_restored = PackedStringArray()
	params_dirty = false
	_restore_gpu_prefs()
	_read_param_table()
	_read_landmark_kinds()
	world_loaded.emit()
	## After the emit, for the same reason `load_save` clears it there: the
	## handler above sets the flag, and an empty shell has nothing to save.
	_set_dirty(false)

## The only writer of `world_dirty`: sets it and emits `dirty_changed` on a real
## change. Callers use `mark_world_dirty()` for edits; this one also clears the flag.
func _set_dirty(value: bool) -> void:
	if world_dirty == value:
		return
	world_dirty = value
	dirty_changed.emit(value)

## Forwards to `WorldGen.load_asset_pack(path)`; emits `world_loaded` on success.
## `false` leaves the world untouched.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func load_asset_pack(path: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	var ok: bool = world_gen.load_asset_pack(path)
	if ok:
		world_loaded.emit()
	return ok

## `WorldGen.has_asset_pack`: whether a real asset pack is currently loaded.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func has_asset_pack() -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	return world_gen.has_asset_pack()

## `WorldGen::civ_trait_badge_row` -- a whole settlement pin's trait-badge row,
## laid out and resolved against the loaded pack's art, in one call. The
## `_has()` guard (rather than `has_asset_pack`'s bare forward above) matters
## here specifically: this is installed as a `Callable` in
## `viewport_host.gd::refresh_settlement_traits()` and held for the session,
## so an older GDExtension without this binding must degrade to the empty
## `Array` `map_overlay.gd::_trait_badge_art` already treats as "no resolver
## installed" -- not crash the first time a settlement redraws.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func civ_trait_badge_row(px: float, py: float, traits: PackedStringArray, sz: float, sc: float) -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("civ_trait_badge_row"):
		return []
	return world_gen.civ_trait_badge_row(px, py, traits, sz, sc)

## `WorldGen::icon_marker_art` (AP-3, Ruling CQ Q2) -- the loaded pack's own art
## for one placed settlement / POI / sea-mark / custom icon, as
## `{texture, aspect}`, or `{}` when there is none. Installed as a `Callable` by
## `viewport_host.gd::refresh_settlement_traits()` and held for the session,
## so it carries `civ_trait_badge_row`'s `_has()` guard for the same reason: an
## older GDExtension degrades to `{}`, which `map_overlay.gd::_marker_art`
## reads as "draw the glyph".
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func icon_marker_art(family: String, slot: String, set_name: String, x: float, y: float, screen_px: float) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("icon_marker_art"):
		return {}
	return world_gen.icon_marker_art(family, slot, set_name, x, y, screen_px)

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
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "reason": _BUSY_TEXT}.
func center_landmasses() -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "reason": _BUSY_TEXT}
	if not _has("center_landmasses"):
		return {"ok": false, "reason": "This build of the engine has no centring pass."}
	var r: Dictionary = world_gen.center_landmasses()
	if bool(r.get("ok", false)) and int(r.get("offset", 0)) != 0:
		world_loaded.emit()
	return r

## Overdeepen the glacially-carvable coastal valleys into fjords. Returns
## the engine's summary dictionary: `ok`, `cells_masked`, `cells_carved`,
## and `reason` when `ok` is false.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "reason": _BUSY_TEXT}.
func carve_fjords() -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "reason": _BUSY_TEXT}
	if not _has("carve_fjords"):
		return {"ok": false, "reason": "This build of the engine has no fjord pass."}
	var r: Dictionary = world_gen.carve_fjords()
	if bool(r.get("ok", false)) and int(r.get("cells_carved", 0)) > 0:
		world_loaded.emit()
	return r

## No `erode_defaults()` wrapper exists here, deliberately. `ErodeOpts`
## (`cartalith-engine/src/erode_op.rs`) is a separate struct from
## `WorldParams` on purpose -- droplet erosion is an op over the finished
## field, not a generation stage -- so its five exposed keys are not rows in
## `cartalith-godot/src/params.rs`' `PARAMS` table and `param_default()`
## returns `null` for them. There is no engine-side getter for them either.
## A wrapper guarded on `_has("erode_defaults")` would therefore push the
## stale-library warning above on every boot, telling the reader to rebuild
## for a symbol no rebuild can produce, and would leave that never-real name
## in `missing_bindings()`, which is meant to be the staleness fingerprint.
## `world_workspace.gd:_erode_defaults()` reads `param_default(key)` per key
## and falls back to its own transcription, saying which it used.

# -- Milestone F tool bindings ------------------------------------------------
#
# One thin wrapper per bound-but-unwired #[func], added together so no domain
# workspace needs to touch this file at all -- every wrapper follows sized_api's
# own established shape (`has_method` guard, safe default on an older binary),
# so a workspace built against these never crashes against a GDExtension that
# predates one specific tool's binding.
#
# **Call sites address `bridge.<name>()`, never `bridge.world_gen.<name>()`.**
# A caller that reaches around a wrapper gets none of what the wrapper is for:
# not the `_has()` guard, not the `missing_bindings()` staleness record, not
# the safe default, and since 2026-09-01 not `mark_world_dirty()` either -- so
# the edit it makes does not reach autosave. It also leaves the wrapper reading
# as dead code, which is how four of them were nearly deleted. Writing the
# `world_gen.has_method(...)` test at the call site is the tell: the wrapper
# already performed it.

# sculpt_bridge.rs
func get_sculpt_features() -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("get_sculpt_features"):
		return []
	return world_gen.get_sculpt_features()

## The eight one-click sculpt presets (`WorldGen.get_sculpt_presets`); applying one
## never paints. `[]` on a binary without the sculpt API.
func get_sculpt_presets() -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("get_sculpt_presets"):
		return []
	return world_gen.get_sculpt_presets()

## Range/default info for the shared brush/noise globals
## (`WorldGen.get_sculpt_globals_info`). `[]` without the sculpt API.
func get_sculpt_globals_info() -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("get_sculpt_globals_info"):
		return []
	return world_gen.get_sculpt_globals_info()

## Freehand's sub-mode keys in registry order (`WorldGen.get_sculpt_freehand_modes`).
## Empty without the sculpt API.
func get_sculpt_freehand_modes() -> PackedStringArray:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return PackedStringArray()
	if not _has("get_sculpt_freehand_modes"):
		return PackedStringArray()
	return world_gen.get_sculpt_freehand_modes()

## Current values of the shared sculpt globals (`WorldGen.sculpt_get_globals`).
## `{}` before any world or without the API.
func sculpt_get_globals() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not _has("sculpt_get_globals"):
		return {}
	return world_gen.sculpt_get_globals()

## Applies a partial dictionary of globals (`WorldGen.sculpt_set_globals`); the reply
## is `{rejected, clamped}`, so a key appears only when something went wrong with it.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func sculpt_set_globals(values: Dictionary) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("sculpt_set_globals"):
		return {}
	mark_world_dirty()
	return world_gen.sculpt_set_globals(values)

## Key of the feature the next stroke paints (`WorldGen.sculpt_get_feature`). `""`
## before any world or without the API.
func sculpt_get_feature() -> String:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return ""
	if not _has("sculpt_get_feature"):
		return ""
	return world_gen.sculpt_get_feature()

## Selects the next stroke's feature and resets its parameters to that feature's
## defaults (`WorldGen.sculpt_set_feature`). `false` for an unknown key.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func sculpt_set_feature(feature_key: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("sculpt_set_feature"):
		return false
	mark_world_dirty()
	return world_gen.sculpt_set_feature(feature_key)

## The current feature's live control values (`WorldGen.sculpt_get_feature_params`).
## `{}` before any world or without the API.
func sculpt_get_feature_params() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not _has("sculpt_get_feature_params"):
		return {}
	return world_gen.sculpt_get_feature_params()

## Applies a partial dictionary of the current feature's controls
## (`WorldGen.sculpt_set_feature_params`); the reply is `{rejected, clamped}`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func sculpt_set_feature_params(values: Dictionary) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("sculpt_set_feature_params"):
		return {}
	mark_world_dirty()
	return world_gen.sculpt_set_feature_params(values)

## Seeds the current feature, its parameters and the noise scale from preset `index`
## (`WorldGen.sculpt_apply_preset`). Never paints; `false` for a bad index.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func sculpt_apply_preset(index: int) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("sculpt_apply_preset"):
		return false
	mark_world_dirty()
	return world_gen.sculpt_apply_preset(index)

## Freehand's current sub-mode key (`WorldGen.sculpt_get_freehand_mode`). `""` when
## the feature is not Freehand, there is no world, or no API.
func sculpt_get_freehand_mode() -> String:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return ""
	if not _has("sculpt_get_freehand_mode"):
		return ""
	return world_gen.sculpt_get_freehand_mode()

## Sets Freehand's sub-mode (`WorldGen.sculpt_set_freehand_mode`). `false`, nothing
## changed, if the feature is not Freehand or the key is not one of the modes.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func sculpt_set_freehand_mode(mode_key: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("sculpt_set_freehand_mode"):
		return false
	mark_world_dirty()
	return world_gen.sculpt_set_freehand_mode(mode_key)

## The seed the next stroke will capture (`WorldGen.sculpt_get_seed`). **A negative value
## means "no seed"**: `-1` is what this wrapper returns for a missing binding AND what the
## engine itself reports before any `generate()` (`sculpt_seed_readout` in `lib.rs`). `0` is
## a REAL seed (a u32 seed of 0 is valid), so a caller must test `< 0`, never `== 0`, and dash
## a negative one (`world_workspace.gd::_build_brush_globals`). Corrected 2026-10-11 (R7): this
## doc used to say the engine reports `0` before any world.
func sculpt_get_seed() -> int:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return -1
	if not _has("sculpt_get_seed"):
		return -1
	return world_gen.sculpt_get_seed()

## Sets the seed the next stroke captures (`WorldGen.sculpt_set_seed`). The dice
## button's random pick belongs to the UI, not the engine.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func sculpt_set_seed(seed: int) -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if not _has("sculpt_set_seed"):
		return
	mark_world_dirty()
	world_gen.sculpt_set_seed(seed)

## Starts capturing a new stroke, discarding any uncommitted in-progress points
## (`WorldGen.sculpt_begin_stroke`). `false` before any world or without the API.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func sculpt_begin_stroke() -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("sculpt_begin_stroke"):
		return false
	return world_gen.sculpt_begin_stroke()

## True when the loaded extension carries both ET-D3 sculpt additions
## (`sculpt_begin_stroke_with` and `sculpt_add_point_p`). The sculpt handlers fall
## back to the plain `sculpt_begin_stroke`/`sculpt_add_point` pair when it is
## `false`. Checked with `has_method`, not `_has()`, so an older extension is not
## warned about on every press (the `has_paint_stroke_api` precedent). Never
## assume one of the two without the other: a stroke begun by one and fed by
## the other would still work, but the handlers' pressure bookkeeping would not.
func has_sculpt_stroke_api() -> bool:
	return world_gen != null and world_gen.has_method("sculpt_begin_stroke_with") \
		and world_gen.has_method("sculpt_add_point_p")

## ET-D3: `sculpt_begin_stroke` with options (`WorldGen.sculpt_begin_stroke_with`):
## `symmetry` ("none", "vertical", "horizontal", "dual", "radial"), `radial_count`,
## `symmetry_center` (Vector2), `symmetry_angle` (radians); every key optional, `{}`
## is "no symmetry". There is NO spacing key: a sculpt stroke is one continuous
## polyline per stamp. `false` means the call changed nothing (no world, no API, or a
## refused option) and any stroke already open is still open. Never pass a made-up
## default for a key the caller did not choose; omit it.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func sculpt_begin_stroke_with(options: Dictionary = {}) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("sculpt_begin_stroke_with"):
		return false
	return world_gen.sculpt_begin_stroke_with(options)

## Appends one grid-cell point to the in-progress stroke (`WorldGen.sculpt_add_point`)
## and returns the new point count; `-1` means the binding is missing.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers -1.
func sculpt_add_point(x: float, y: float) -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return -1
	if not _has("sculpt_add_point"):
		return -1
	return world_gen.sculpt_add_point(x, y)

## ET-D3: `sculpt_add_point` with the pointer's pressure (`WorldGen.sculpt_add_point_p`).
## `pressure` is a float in 0..1 **or `null` when the device reported none** (a mouse):
## absent is never encoded as 1.0 or 0.0, and one `null` anywhere in a stroke makes
## the whole stroke pressure-less (the engine refuses to invent the missing value).
## Returns the new point count, `-1` with no world or no binding.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers -1.
func sculpt_add_point_p(x: float, y: float, pressure: Variant) -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return -1
	if not _has("sculpt_add_point_p"):
		return -1
	return world_gen.sculpt_add_point_p(x, y, pressure)

## The grid-snap ladder in grid cells, in engine order. Empty when the
## binding is missing, which reads the same as "this build offers no snap
## steps" -- and a picker built from an empty ladder correctly shows only
## Off.
func get_sculpt_grid_snap_steps() -> PackedFloat64Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return PackedFloat64Array()
	if not _has("get_sculpt_grid_snap_steps"):
		return PackedFloat64Array()
	return world_gen.get_sculpt_grid_snap_steps()

## `{"step": <cells>}` while snapping is on, and an **empty** Dictionary
## while it is off -- ask `has("step")`, never compare against 0. An older
## extension with no binding also reads as off, which is what it is.
func sculpt_get_grid_snap() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not _has("sculpt_get_grid_snap"):
		return {}
	return world_gen.sculpt_get_grid_snap()

## Sets the snap step; anything off the ladder (0 included) turns snapping
## off. Returns the step now in effect, or 0 for off.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.0.
func sculpt_set_grid_snap(step: float) -> float:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0.0
	if not _has("sculpt_set_grid_snap"):
		return 0.0
	return world_gen.sculpt_set_grid_snap(step)

## One point through the engine's own snap, so a stroke preview draws the
## polyline that will actually be stamped. Returns the input unchanged when
## snapping is off *or* the binding is missing -- both of which mean "no
## snapping is happening", so the fallback is the truth rather than a guess.
##
## **This is for a preview, not for feeding `sculpt_add_point`.** The engine
## returns `PackedFloat64Array` precisely so an un-snapped point survives at
## full precision; narrowing to `Vector2` here costs f32 rounding, which is
## free only because the one consumer -- `world_workspace.gd`'s
## `_sculpt_stroke_points` -- is a `PackedVector2Array` and f32 already.
## Stroke capture still passes the raw pointer coordinates straight to
## `sculpt_add_point`, which does its own snapping in f64.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers Vector2(x, y).
func sculpt_snap(x: float, y: float) -> Vector2:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return Vector2(x, y)
	if not _has("sculpt_snap"):
		return Vector2(x, y)
	var p: PackedFloat64Array = world_gen.sculpt_snap(x, y)
	if p.size() < 2:
		return Vector2(x, y)
	return Vector2(p[0], p[1])

## **No caller.** The Sculpt tool needs the stroke's point *positions*, not
## its length -- it draws them as a path preview -- so
## `world_workspace.gd::_sculpt_stroke_points` keeps the array as the clicks
## arrive and the count falls out of it. Kept rather than deleted: the
## `#[func]` behind it is live, and it is the natural cross-check the next
## time that local copy and the engine's draft disagree.
func sculpt_stroke_point_count() -> int:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return -1
	if not _has("sculpt_stroke_point_count"):
		return -1
	return world_gen.sculpt_stroke_point_count()

## Drops the in-progress stroke's points without making a stamp
## (`WorldGen.sculpt_cancel_stroke`). Emits nothing: the draft did not change.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func sculpt_cancel_stroke() -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if not _has("sculpt_cancel_stroke"):
		return
	world_gen.sculpt_cancel_stroke()

## Freezes the stroke and tool state into one draft stamp (`WorldGen.sculpt_end_stroke`)
## and returns its index; `-1` for a zero-point stroke, no world or no binding. A draft
## push, not a commit; emits `sculpt_draft_changed` after the engine call.
## **ET-D3:** a stroke begun with a symmetry becomes N stamps; the index returned is the
## SOURCE stamp's, its mirrors follow at index+1..., the source is the selected one, and
## the whole stroke is ONE undo step (`sculpt_undo`). Callers that assumed one stamp per
## stroke must read `sculpt_stamp_count()` instead.
func sculpt_end_stroke() -> int:
	if not _has("sculpt_end_stroke"):
		return -1
	mark_world_dirty()
	## **After the engine call, not before it.** Every emit in this group used
	## to precede its own `world_gen.sculpt_*()`, so a listener that re-read
	## `sculpt_stamp_count()` inside the emit -- which is exactly what
	## `world_workspace.gd::_refresh_sculpt_draft` does -- got the count from
	## before the change. Measured 2026-09-05 (`_rdrefresh_probe.gd` T0): the
	## signal reported 0 stamps on the stroke that made the draft's first one.
	var stamp := world_gen.sculpt_end_stroke()
	sculpt_draft_changed.emit()
	return stamp

## Number of stamps on the draft (`WorldGen.sculpt_stamp_count`). `-1` means the
## binding is missing -- never read it as an empty draft.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func sculpt_stamp_count() -> int:
	if generating or not _has("sculpt_stamp_count"):
		return -1
	return world_gen.sculpt_stamp_count()

## The draft's stamps newest-first, each carrying its real draft `index`
## (`WorldGen.sculpt_list_stamps`). `[]` for an empty draft or no API. ET-D3: an entry
## carries a `pressure` PackedFloat32Array (one per point) **only when the device
## reported pressure for every point**; ask `has("pressure")`, never default it.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func sculpt_list_stamps() -> Array:
	if generating or not _has("sculpt_list_stamps"):
		return []
	return world_gen.sculpt_list_stamps()

## `MAP_CONTEXT_SCOPE.md` §9.2's stamp pick bound accessor: the stamp's own
## captured stroke polyline (grid coordinates), which `sculpt_list_stamps`'
## `point_count` cannot answer. Read-only -- selects nothing. Empty for an
## out-of-range index or before any `generate()` call.
func sculpt_stamp_points(index: int) -> PackedVector2Array:
	if not _has("sculpt_stamp_points"):
		return PackedVector2Array()
	return world_gen.sculpt_stamp_points(index)

## The primary selected stamp index (`WorldGen.sculpt_get_selected_stamp`); `-1`
## means none selected (or no binding).
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func sculpt_get_selected_stamp() -> int:
	if generating or not _has("sculpt_get_selected_stamp"):
		return -1
	return world_gen.sculpt_get_selected_stamp()

## Selects stamp `index` for re-tuning (`WorldGen.sculpt_select_stamp`). `false` for a
## bad index or no API. Emits nothing; the draft itself did not change.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func sculpt_select_stamp(index: int) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("sculpt_select_stamp"):
		return false
	return world_gen.sculpt_select_stamp(index)

## Every selected stamp's draft index, ascending. Falls back to whatever
## `sculpt_get_selected_stamp()` reports against a cdylib without the set.
##
## **No shell gesture builds a multi-selection of stamps yet.** A stamp is
## picked from the right dock's stack list rather than off the canvas, and that
## list is `right_dock.gd`'s. The engine-side set and `sculpt_select_set()`
## below are what a modifier click on those rows would drive.
func sculpt_get_selection() -> PackedInt64Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return PackedInt64Array()
	if not _has("sculpt_get_selection"):
		var one := sculpt_get_selected_stamp()
		return PackedInt64Array([one]) if one >= 0 else PackedInt64Array()
	return world_gen.sculpt_get_selection()

## Replaces the whole stamp selection (`WorldGen.sculpt_select_set`). Out-of-range
## indices are dropped and reported as `false`; an empty array deselects.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func sculpt_select_set(indices: PackedInt64Array) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("sculpt_select_set"):
		return false
	return world_gen.sculpt_select_set(indices)

## Selects every stamp on the draft; returns how many.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.
func sculpt_select_all_stamps() -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0
	if not _has("sculpt_select_all_stamps"):
		return 0
	return world_gen.sculpt_select_all_stamps()

## Hides or shows stamp `index` (`WorldGen.sculpt_set_stamp_hidden`); a hidden stamp
## is skipped by preview and commit but keeps its slot. `false` for a bad index.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func sculpt_set_stamp_hidden(index: int, hidden: bool) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("sculpt_set_stamp_hidden"):
		return false
	mark_world_dirty()
	return world_gen.sculpt_set_stamp_hidden(index, hidden)

## Moves stamp `index` one place earlier in bake order (`WorldGen.sculpt_move_stamp_up`)
## and emits `sculpt_draft_changed` after the call. `false` for a no-op move.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func sculpt_move_stamp_up(index: int) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("sculpt_move_stamp_up"):
		return false
	mark_world_dirty()
	var ok := world_gen.sculpt_move_stamp_up(index)
	sculpt_draft_changed.emit()
	return ok

## Moves stamp `index` one place later (`WorldGen.sculpt_move_stamp_down`) and emits
## `sculpt_draft_changed` after the call. `false` for a no-op move.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func sculpt_move_stamp_down(index: int) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("sculpt_move_stamp_down"):
		return false
	mark_world_dirty()
	var ok := world_gen.sculpt_move_stamp_down(index)
	sculpt_draft_changed.emit()
	return ok

## Removes stamp `index` from the draft (`WorldGen.sculpt_delete_stamp`) and emits
## `sculpt_draft_changed` after the call. `false` for a bad index.
func sculpt_delete_stamp(index: int) -> bool:
	if not _has("sculpt_delete_stamp"):
		return false
	mark_world_dirty()
	var ok := world_gen.sculpt_delete_stamp(index)
	sculpt_draft_changed.emit()
	return ok

## Whether `sculpt_undo` would do anything (`WorldGen.sculpt_can_undo`). Draft-scoped.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func sculpt_can_undo() -> bool:
	if generating or not _has("sculpt_can_undo"):
		return false
	return world_gen.sculpt_can_undo()

## Whether `sculpt_redo` would do anything (`WorldGen.sculpt_can_redo`). Draft-scoped.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func sculpt_can_redo() -> bool:
	if generating or not _has("sculpt_can_redo"):
		return false
	return world_gen.sculpt_can_redo()

## Reverts the last structural draft edit (`WorldGen.sculpt_undo`); never touches the
## real heightfield. Emits `sculpt_draft_changed` after the call; `false` if nothing to undo.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func sculpt_undo() -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("sculpt_undo"):
		return false
	mark_world_dirty()
	var ok := world_gen.sculpt_undo()
	sculpt_draft_changed.emit()
	return ok

## Re-applies the undone draft edit (`WorldGen.sculpt_redo`); `false` with nothing to
## redo. Emits `sculpt_draft_changed` after the call.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func sculpt_redo() -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("sculpt_redo"):
		return false
	mark_world_dirty()
	var ok := world_gen.sculpt_redo()
	sculpt_draft_changed.emit()
	return ok

## Live colour-plus-hillshade preview of the current draft
## (`WorldGen.build_sculpt_preview_texture`). `null` without a world or the API.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers null.
func build_sculpt_preview_texture() -> Texture2D:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return null
	if not _has("build_sculpt_preview_texture"):
		return null
	return world_gen.build_sculpt_preview_texture()

## Bakes the whole draft into the heightfield and re-runs hydrology and climate
## (`WorldGen.sculpt_commit(reason)`); returns the engine's summary dictionary, `{}`
## without the API. Emits `sculpt_draft_changed` after the call.
func sculpt_commit(reason: String) -> Dictionary:
	if not _has("sculpt_commit"):
		return {}
	mark_world_dirty()
	var summary: Dictionary = world_gen.sculpt_commit(reason)
	sculpt_draft_changed.emit()
	return summary

## Ruling AQ (2026-09-24): the Sea level control is LIVE -- it moves the
## current world's sea level and re-derives the coastline, lakes, biomes and
## the civilisation's water readings without regenerating, and re-stamps the
## sculpt draft (Ruling 17). Wraps `WorldGen.set_sea_level_live`
## (`cartalith-godot/src/sea_live_bridge.rs`), which says what it re-derives,
## what it only marks stale and what stays as generated.
##
## Returns the engine's reply, or `{}` when it could not be asked (an older
## cdylib without the binding, or a generation in flight -- the same window
## `param_set` refuses). A real move marks the WORLD dirty (the project now
## differs from its file) and deliberately not `params_dirty`: the map
## matches this dial again the moment the call returns, so the "a dial moved,
## regenerate" state would be false. The caller repaints.
func sea_level_live(level: float) -> Dictionary:
	if generating or not _has("set_sea_level_live"):
		return {}
	var r: Dictionary = world_gen.set_sea_level_live(level)
	if bool(r.get("ok", false)) and bool(r.get("changed", false)):
		mark_world_dirty()
	return r

## Drops the whole draft (`WorldGen.sculpt_discard`) and returns how many stamps went;
## `0` means an empty draft, `-1` that the binding is missing. Emits `sculpt_draft_changed`.
func sculpt_discard() -> int:
	if not _has("sculpt_discard"):
		return -1
	mark_world_dirty()
	var dropped := world_gen.sculpt_discard()
	sculpt_draft_changed.emit()
	return dropped

# -- Global heightmap undo (Edit ▸ Undo, Ctrl+Z) -------------------------------
#
# The reference's `pushUndo`/`undoLast`/`updateUndoUI`, register ED-01/PR-11.
# Deliberately NOT the same thing as `sculpt_undo`/`sculpt_redo` above: those
# pop a stamp off an uncommitted draft, these pop a whole committed height
# field. The reference keeps the same two apart under the same names.
#
# Since Ruling BA (2026-09-28) the same Undo also puts back a way deleted from
# the map context card (`way_delete` below) when that is the newer step --
# `undo_next_subsystem()` says which a press will revert.
#
# Same degrade-rather-than-crash `has_method` guard every wrapper here uses --
# an older cdylib without these five `#[func]`s reports "nothing to undo"
# rather than erroring.

## True when the height stack has a step to revert. **Guarded on `generating`** (found-bug
## sweep 2026-10-11, G6): `menus.gd::refresh_undo_rows` and `look_history.gd::wants_undo` both
## reach this on every look-history commit, and the worker holds the `WorldGen` for the whole
## run, so an unguarded call was the `Gd<WorldGen>::bind() failed` panic. Nothing can be undone
## mid-generation (a generate clears the stack anyway), so `false` is the true answer, the same
## reasoning `redo_available` records; the rows re-evaluate on `generation_finished`.
func can_undo() -> bool:
	if generating or not _has("can_undo"):
		return false
	return world_gen.can_undo()

## The operation `undo_last()` would revert ("Sculpt commit", "Carve fjords"),
## or "" when there is nothing to revert -- and "" while `generating` (same guard and reason
## as `can_undo`, G6).
func undo_label() -> String:
	if generating or not _has("undo_label"):
		return ""
	return String(world_gen.undo_label())

## Reverts the height field one step. Returns the reverted operation's label,
## or "" if nothing happened. The caller repaints -- the engine deliberately
## does not re-run flow/climate here (see `undo.rs`).
##
## **Refused while `generating`** (arming-panics sweep 2026-10-11, the sibling of `can_undo`'s
## G6 guard): the worker holds the `WorldGen`, so a real call is the `bind() failed` panic, and a
## generate clears the stack anyway. Answers `""` (nothing happened) and does NOT mark the world
## dirty -- nothing changed. `app.gd::_undo_last_impl` says why in the status bar before it gets here.
func undo_last() -> String:
	if generating or not _has("undo_last"):
		return ""
	mark_world_dirty()
	return String(world_gen.undo_last())

## Global redo. Separate `#[func]`s from `sculpt_redo` for exactly the reason
## the block header above gives: the sculpt pair cursors an uncommitted draft,
## these cursor the committed height stack.
##
## The three exist because the global stack used to POP rather than cursor --
## `menus.gd`'s `_edit()` Redo row (`:632` in this tree, and it moves) was
## drawn disabled with "global undo has no redo, in this port or the reference" and
## that reason was true and verified. Once the ledger became a cursor (it
## already recorded enough to revert *to* a point, which was most of the work),
## it stopped being true. These wrappers are what a shell meeting either binary
## uses: on an older cdylib `redo_available()` answers `false` and the menu
## item stays disabled with its old reason still honest.

## True when there is a step forward of the cursor to take.
##
## **Guarded on `generating`** (2026-09-24), like every read the shell polls
## per frame: `right_dock.gd::_process` asks this and `redo_labels()` every
## frame while HISTORY is the live context, and during `generate()` the worker
## thread holds `bind_mut` on the `WorldGen`, so the call panicked with
## `Gd<WorldGen>::bind() failed, already bound` once per frame of the run
## (traced from `_setxvert_probe`). Nothing can be redone mid-generation, so
## "no redo" is the true answer; the poll converges once the run ends.
func redo_available() -> bool:
	if generating or not _has("redo_available"):
		return false
	return world_gen.redo_available()

## The operation `redo_last()` would re-apply ("Sculpt commit", "Carve
## fjords"), or "" when the cursor is already at the top of the stack. Same
## shape as `undo_label()`, so a menu can label both from one code path.
func redo_label() -> String:
	if generating or not _has("redo_label"):
		return ""
	return String(world_gen.redo_label())

## Re-applies one step. `true` when the field changed; the caller repaints,
## exactly as it must after `undo_last()` -- the engine re-runs no flow or
## climate here either. **Refused while `generating`** (same reason and answer as `undo_last`:
## `false`, world not marked dirty).
func redo_last() -> bool:
	if generating or not _has("redo_last"):
		return false
	mark_world_dirty()
	return world_gen.redo_last()

## **Every** undone step still on the redo tail, in the order `redo_last()`
## would re-apply them: element 0 is `redo_label()`, element 1 is what a second
## press would put back. So a history panel drawing oldest-first can append this
## array straight after the last committed row and the ordinals continue.
##
## Empty exactly when `redo_available()` is `false` -- `RedoTail::labels()`
## carries the same `is_current()` gate `RedoTail::label()` does, so the list
## and the single label can never disagree about whether there is anything to
## redo. Also empty against a cdylib older than this binding, which is what
## keeps the panel's undone rows off the screen rather than drawing an
## unnamed one.
##
## **Not `undo_stats()["redo_depth"]`, and a caller must not count that
## instead.** That readout is raw buffer occupancy for `Preferences ▸ Memory`
## and is deliberately not gated on `redo_available()`, so a tail invalidated
## by a new commit still reports the bytes it is holding while offering
## nothing. The two agree exactly while `redo_available()` is `true`, which is
## the only state in which undone rows are drawn at all -- so the agreement is
## a coincidence of that state, not a licence to swap one for the other.
func redo_labels() -> PackedStringArray:
	if generating or not _has("redo_labels"):
		return PackedStringArray()
	return world_gen.redo_labels()

## Throw the redo tail away without committing anything -- the artboard's
## `✕ discard the N undone steps` row. Returns how many steps went, so the
## caller names the number it just acted on rather than re-reading and hoping.
## `0` against an older cdylib, and `0` when there was no live tail.
##
## Touches the tail and nothing else: not the height field, not the undo stack,
## not the ledger. That is what separates it from `clear_undo()` above, which
## drops the backward half too and is a `Preferences ▸ Memory` control.
##
## **Deliberately no `mark_world_dirty()`, unlike `undo_last()`/`redo_last()`
## either side of it.** Those two write the height field. This one changes no
## world state, and nothing the project `.zip` carries names the redo tail:
## `grep -n 'undo\|redo' crates/cartalith-godot/src/project_bridge.rs`
## (2026-09-06) returns two prose lines about `PassBuffer::discard` and no
## field. So marking would put an unsaved-changes star on the title bar over a
## change no save can record -- the reason `set_color_space()` further up does
## not mark either.
##
## **`0` while `generating`** (arming-panics sweep 2026-10-11): the tail lives on the `WorldGen`
## the worker holds, and a generate drops it anyway.
func discard_redo_tail() -> int:
	if generating or not _has("discard_redo_tail"):
		return 0
	return int(world_gen.discard_redo_tail())

## `depth`, `max_steps`, `bytes`, `budget_bytes`, `step_bytes`, `label` --
## the reference's `#undoMem` readout as data. Empty dictionary on an older
## cdylib.
##
## **While `generating` this answers the last value read** (`_undo_stats_cache`, the
## `grid_size()` precedent), because the worker holds the `WorldGen` and a real call is the
## `bind() failed` panic -- found-bug sweep 2026-10-11, G6: every look-history commit reaches
## this through `menus.gd::refresh_undo_rows`. The cached copy is the world as it stood before
## the run (a generate then clears the stack), so it is a STALE answer, shown only until the next
## real read after `generation_finished`; before any real read it is `{}`, exactly the "no
## binding" answer, which callers already treat as "unknown". Never write through it.
func undo_stats() -> Dictionary:
	if generating:
		return _undo_stats_cache.duplicate()
	if not _has("undo_stats"):
		return {}
	_undo_stats_cache = world_gen.undo_stats()
	return _undo_stats_cache.duplicate()

## The last `undo_stats()` read from the engine, served while `generating`. Not state of its
## own: it is refreshed by every real read and is never consulted otherwise.
var _undo_stats_cache: Dictionary = {}

## Re-budgets the engine's undo stack in MB (`WorldGen.set_undo_budget_mb`; the engine
## floors it at 4 MiB). A no-op on a binary without it.
##
## **Deferred while `generating`** (mid-generation sweep, 2026-10-12): the worker thread holds the
## engine, and reaching the `#[func]` is the `bind() failed` panic. The request is kept in
## `_pending_undo_budget_mb` (last one wins) and `_process` applies it on the first frame the run
## has finished, so a preference chosen mid-run is not lost. Its caller says so on the status line.
func set_undo_budget_mb(mb: int) -> void:
	if generating:
		_pending_undo_budget_mb = mb
		return
	if _has("set_undo_budget_mb"):
		world_gen.set_undo_budget_mb(mb)

## An undo budget (MB) asked for while `generating`, applied by `_process` once the run ends;
## `-1` is "none pending" (a real budget is never negative). See `set_undo_budget_mb`.
var _pending_undo_budget_mb := -1

## Drops every undo step (`WorldGen.clear_undo`), which is what Preferences > Memory
## offers to give memory back. A no-op without the binding.
func clear_undo() -> void:
	## Skipped while `generating` (arming-panics sweep 2026-10-11): the worker holds the engine,
	## and a generate clears the stack itself.
	if not generating and _has("clear_undo"):
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

## Declared but never read or assigned anywhere in the shell scripts (grep finds no other use);
## intent unverified, `atlas_ready` probes `_has("atlas_set_root")` directly instead.
var atlas_api := false
## True once `atlas_set_root` has succeeded in `atlas_ready`; answers later calls from the flag
## without re-calling the engine.
var _atlas_root_set := false

## True once the engine has somewhere to put baked chunks. Idempotent.
func atlas_ready() -> bool:
	if _atlas_root_set:
		return true
	# Mid-generation gate (caller-gate audit 2026-10-12): `atlas_set_root` below is a `WorldGen` call and the worker holds it.
	# `true` above is answered from the flag, so it stays honest mid-run; an unset root is simply not ready yet.
	if generating:
		return false
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
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not _has("atlas_status"):
		return {}
	atlas_ready()
	return world_gen.atlas_status()

## What a bake to `max_z` would cost: `tiles`, `already_baked`, `remaining`,
## `seconds`. Shown *before* the user commits, because depth 5 is 1365 tiles.
func bake_estimate(max_z: int) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not _has("bake_estimate"):
		return {}
	atlas_ready()
	return world_gen.bake_estimate(max_z)

## Bake every tile of every level 0..max_z. Synchronous and slow -- the caller
## must show a busy state. Returns `ok`, `baked`, `skipped`, `failed`, `total`,
## `seconds`, `error`.
func bake_all(max_z: int) -> Dictionary:
	## Mid-generation (2026-10-12): the worker holds `WorldGen`; refuse with the error the
	## Bake button already shows rather than panic on `atlas_ready()` / `bake_all`.
	if generating:
		return {"ok": false, "error": "the world is generating -- try again when it finishes"}
	if not _has("bake_all"):
		return {"ok": false, "error": "this build has no bake_all()"}
	if not atlas_ready():
		return {"ok": false, "error": "no atlas cache directory"}
	return world_gen.bake_all(max_z)

## Bake just the tiles a view rectangle (in coarse grid cells) touches.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func bake_visible(z: int, x0: float, y0: float, x1: float, y1: float) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("bake_visible"):
		return {"ok": false, "error": "this build has no bake_visible()"}
	if not atlas_ready():
		return {"ok": false, "error": "no atlas cache directory"}
	return world_gen.bake_visible(z, x0, y0, x1, y1)

## Throw away this world's baked chunks; returns how many went. Clears the
## finalize lock too -- a lock protecting nothing would strand the user.
## Is this pyramid chunk served from the baked atlas rather than synthesized?
##
## `PARITY_AUDIT.md` §23 F16 registered `atlas_is_covered` as **declined**, on
## the reasoning that *"both its jobs need a reader of baked imagery and the
## shell has none"* -- gating deep-zoom refinement on coverage would make the
## view show *less* detail, because nothing would draw the baked chunk in the
## refined tile's place. That reasoning still holds for those two jobs.
##
## It does not hold for a third one the audit did not consider: the chunk-debug
## overlay (`ViewportHost._draw_lod_debug`, the reference's
## `drawLODChunkDebug`) needs the *answer*, not the image -- it prints
## `baked` or `cached` beside a chunk id. So this is the function's first real
## consumer, and it does not reopen the refinement question.
## `Data ▸ World data ▸ Export heightmap`. Writes the committed height field
## as a 16-bit grayscale PNG -- the format `import_heightmap` has always been
## able to READ and nothing could write.
##
## Does not reflect an open Sculpt draft: that is uncommitted state on
## `SculptEditor`, not on `WorldState::field`. The caller says so on screen.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func export_heightmap_png(path: String, width: int) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("export_heightmap_png"):
		return {"ok": false, "error": "this build of the engine has no heightmap export"}
	return world_gen.export_heightmap_png(path, width)

## ET-E1: does this binary's `WorldGen.export_raster_png` take the optional 4th
## `color_space` argument? Read off the binding's own method list (`has_method`
## cannot see arity), so a build from before ET-E1 answers `false` instead of
## failing with a "too many arguments" script error. Never cached across a
## reload: it is cheap (one method-list walk per call, and callers are
## button presses). `false` also covers a missing binding.
func export_raster_takes_color_space() -> bool:
	if world_gen == null or not world_gen.has_method("export_raster_png"):
		return false
	for m in world_gen.get_method_list():
		if String(m.get("name", "")) == "export_raster_png":
			return (m.get("args", []) as Array).size() >= 4
	return false

## ET-E1: the colour spaces an EXPORT can be written in -- `["sRGB", "Display
## P3"]` from the engine's own `list_color_spaces()` when the export binding
## takes the 4th argument, else `["sRGB"]` (a pre-ET-E1 build writes sRGB-
## equivalent pixels and cannot embed a P3 profile). Deliberately separate from
## `color_spaces()` above, which is the MONITOR list: the two share a name list
## in the engine today, but an export is a document and the monitor is a
## device, and the shell must never route one through the other's setter.
func export_color_spaces() -> Array:
	if export_raster_takes_color_space() and _has("list_color_spaces"):
		return Array(world_gen.list_color_spaces())
	return ["sRGB"]

## `WorldGen.export_raster_png` for the colour raster, with ET-E1's export
## `color_space` ("sRGB" default, "Display P3"). **Never touches
## `set_color_space`**: that is the monitor, session state; this is one file's
## tag. The 4th argument is passed only when the binding accepts it
## (`export_raster_takes_color_space`), so the three-argument callers the shell
## had before ET-E1 keep working on an older binary; on one, a non-sRGB request
## is refused here with a message rather than silently written as sRGB.
## Refuses while a generation or landmark pass holds the engine
## (`generating`): the worker has `world_gen` mutably borrowed and a call from
## here would be a Rust panic. Returns the engine's `{ok, path, width, height,
## color_space, files, bytes, ms}` or `{ok: false, error}`; it never throws.
func export_raster_png(path: String, width: int, tiled: bool, color_space: String = "sRGB") -> Dictionary:
	if not _has("export_raster_png"):
		return {"ok": false, "error": "this build of the engine has no raster export"}
	if generating:
		return {"ok": false, "error": "a generation is still running -- wait for it to finish, then export"}
	if export_raster_takes_color_space():
		## `callv`, not a direct call: GDScript checks a direct call's argument
		## count against the loaded binary at PARSE time, so a four-argument call
		## would make this whole script fail to compile on an older binary
		## (found 2026-10-11 by re-running older probes against stale copies of
		## the DLL: 'Too many arguments ... Expected at most 3'). `callv` defers
		## the check to run time, where `export_raster_takes_color_space` has
		## already said the binding takes four.
		return world_gen.callv("export_raster_png", [path, width, tiled, color_space])
	if color_space.to_lower() != "srgb":
		return {"ok": false, "error": "this build of the engine cannot embed a %s profile -- rebuild cartalith-godot" % color_space}
	return world_gen.export_raster_png(path, width, tiled)

## ── The export overlay session (`EXPORT_SCOPE.md` §7 E4) ─────────────────────
##
## Thin, gated forwards to `WorldGen.export_session_*` for `ExportOverlay.run`
## (`shell/export_overlay.gd`), which is their only shell caller. Each:
##   * refuses while `generating` with `{"ok": false, "error": _BUSY_TEXT,
##     "busy": true}` -- the worker holds `world_gen` and a call from here would
##     panic -- and the driver waits and retries on `busy` rather than failing,
##     because the session itself renders from its own snapshot and is unharmed
##     by a run in between;
##   * refuses with a sentence when the loaded binary lacks the binding, and
##   * calls through `callv`, so this file still compiles against a binary built
##     before a binding existed (`export_raster_png` above says why).
## Return shapes are the `#[func]`s' own (`export_raster.rs`).

## `{ok:false, error, busy:true}` while generating, a missing-binding refusal for
## a binary without `method`, else `{}` (go ahead).
func _export_session_gate(method: String) -> Dictionary:
	if generating:
		return {"ok": false, "error": _BUSY_TEXT, "busy": true}
	if not _has(method):
		return {"ok": false, "error": "this build of the engine has no %s -- rebuild cartalith-godot" % method}
	return {}

## `WorldGen.export_session_begin(path, opts)`: snapshot the world and open the file.
func export_session_begin(path: String, opts: Dictionary) -> Dictionary:
	var g := _export_session_gate("export_session_begin")
	return g if not g.is_empty() else world_gen.callv("export_session_begin", [path, opts])

## `WorldGen.export_session_poll()`: `{ok, next_band, bands, band_ready}`, never blocking.
func export_session_poll() -> Dictionary:
	var g := _export_session_gate("export_session_poll")
	return g if not g.is_empty() else world_gen.callv("export_session_poll", [])

## `WorldGen.export_session_submit_tile(...)`: one premultiplied RGBA8 tile.
func export_session_submit_tile(band: int, x: int, y: int, w: int, h: int, rgba: PackedByteArray) -> Dictionary:
	var g := _export_session_gate("export_session_submit_tile")
	return g if not g.is_empty() else world_gen.callv("export_session_submit_tile", [band, x, y, w, h, rgba])

## `WorldGen.export_session_close()`: close the file without waiting -- `{ok: true,
## pending: true}` until it is complete, then the finished session's dictionary.
func export_session_close() -> Dictionary:
	var g := _export_session_gate("export_session_close")
	return g if not g.is_empty() else world_gen.callv("export_session_close", [])

## `WorldGen.export_session_abort()`: drop the session and remove its partial file.
func export_session_abort() -> Dictionary:
	var g := _export_session_gate("export_session_abort")
	return g if not g.is_empty() else world_gen.callv("export_session_abort", [])

## Is chunk `(z, col, row)`, or any ancestor, baked (`WorldGen.atlas_is_covered`)?
## `false` also means "no binding". Calls `atlas_ready()` first.
func atlas_is_covered(z: int, col: int, row: int) -> bool:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return false
	if not _has("atlas_is_covered"):
		return false
	atlas_ready()
	return bool(world_gen.atlas_is_covered(z, col, row))

## Throws away this world's baked chunks (`WorldGen.atlas_clear`) and returns how many
## were removed; `0` also covers a missing binding.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.
func atlas_clear() -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0
	if not _has("atlas_clear"):
		return 0
	atlas_ready()
	return int(world_gen.atlas_clear())

## Evict least-recently-used chunks until the store is at or under
## `max_bytes`. Returns the bytes actually freed -- `0` covers both "already
## under the cap" and "this build cannot evict", which is the right answer for
## a caller that only wants to know whether it should re-read `atlas_status()`.
##
## This is what makes Preferences' `Atlas cache ▸ Size cap · GB` a real
## control: until the engine grew per-chunk last-access and level accounting
## there was `atlas_status` and `atlas_clear` and nothing in between, so a cap
## could only ever have been enforced by throwing the whole cache away. The
## menu row's disabled reason said exactly that; it is only safe to enable the
## row on a binary where `_has("atlas_evict_to")` is true.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.
func atlas_evict_to(max_bytes: int) -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0
	if not _has("atlas_evict_to"):
		return 0
	if not atlas_ready():
		return 0
	return int(world_gen.atlas_evict_to(max_bytes))

## **No shell caller: `_bake_probe.gd` is the only one, and there is no UI.**
## A whole tile pyramid moved between machines as one file is a real
## capability with a real `#[func]` behind it, and nothing in the shell or in
## any document names it -- so it is recorded here, where the next person to
## read this pair will be.
##
## The route it wants is `data_manager_window.gd`'s `ROUTES` table, which
## already has an Import group and an Export group and carries a per-route
## `reason` string for exactly this kind of disclosure. Adding it is a UI
## decision (where a multi-hundred-megabyte cache export belongs, and what it
## should say about staleness), not a binding decision, which is why the
## binding waits here rather than being deleted.
func atlas_export_zip(gzip: bool = true) -> PackedByteArray:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return PackedByteArray()
	if not _has("atlas_export_zip"):
		return PackedByteArray()
	atlas_ready()
	return world_gen.atlas_export_zip(gzip)

## The other half of the pair above, and unreachable for the same reason.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func atlas_import_zip(bytes: PackedByteArray) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("atlas_import_zip"):
		return {"ok": false, "error": "this build has no atlas_import_zip()"}
	if not atlas_ready():
		return {"ok": false, "error": "no atlas cache directory"}
	return world_gen.atlas_import_zip(bytes)

## One baked chunk's stored visual as PNG bytes, or empty when that chunk was
## never baked -- in which case the caller falls through to live synthesis,
## exactly as the reference's `atlasLoadImg` does.
func atlas_tile_png(z: int, col: int, row: int) -> PackedByteArray:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return PackedByteArray()
	if not _has("atlas_tile_png"):
		return PackedByteArray()
	atlas_ready()
	return world_gen.atlas_tile_png(z, col, row)

## Pixel size a baked tile is synthesised at (`WorldGen.atlas_tile_size`). The `1024`
## fallback is the reference's `_lodTile` default, returned only without the binding.
func atlas_tile_size() -> int:
	if not _has("atlas_tile_size"):
		return 1024
	## Mid-generation (2026-10-12): the worker holds `WorldGen`. Serves the last size the engine
	## gave -- the size is a setting no generate changes -- never an invented one; before any
	## real read it is the same `1024` the no-binding fallback names.
	if generating:
		return _atlas_tile_size_seen
	_atlas_tile_size_seen = int(world_gen.atlas_tile_size())
	return _atlas_tile_size_seen

## The last `atlas_tile_size()` the engine answered, served while `generating`.
var _atlas_tile_size_seen := 1024

## Sets the baked tile size in pixels (`WorldGen.atlas_set_tile_size` -- the name
## differs). A no-op without the binding; it is part of the chunk key.
func set_atlas_tile_size(px: int) -> void:
	## Refused mid-generation (the worker holds `WorldGen`); `menus._on_tile_size` says so first.
	if generating:
		return
	if _has("atlas_set_tile_size"):
		world_gen.atlas_set_tile_size(px)

## `WorldGen.is_finalized`: whether the atlas bake has been finalized (the finalize
## lock `bake.finalized`). `false` also when the binding is missing.
func is_finalized() -> bool:
	if not _has("is_finalized"):
		return false
	## Mid-generation (2026-10-12): serves the last answer the engine gave (a run cannot
	## finalize the atlas; `set_finalized` is the only writer and is refused mid-run by
	## its callers), never a guessed one. `false` before any read, which is also "no world".
	if generating:
		return _finalized_seen
	_finalized_seen = bool(world_gen.is_finalized())
	return _finalized_seen

## The last `is_finalized()` the engine answered, served while `generating`.
var _finalized_seen := false

## Finalizing needs a non-empty atlas; un-finalizing always succeeds.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func set_finalized(on: bool) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("set_finalized"):
		return false
	mark_world_dirty()
	var ok: bool = world_gen.set_finalized(on)
	if ok:
		finalize_changed.emit(on)
	return ok

## "" when the change may proceed, otherwise the sentence to show. `kind` is
## `generation`, `height_edit` or `presentation`.
func finalize_check(kind: String) -> String:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return ""
	if not _has("finalize_check"):
		return ""
	return String(world_gen.finalize_check(kind))

## Emitted with the new state by `set_finalized`; declared next to `finalize_check`, which reads
## the engine's own answer.
signal finalize_changed(on: bool)


# icon_bridge.rs
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func icon_arm(family: String, variant: int, scale: float, rotation: float, jitter: float) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("icon_arm"):
		return false
	return world_gen.icon_arm(family, variant, scale, rotation, jitter)

## CM-3 residual (`MAP_CONTEXT_SCOPE.md` §5.1, CARTO's Icon▸ sub-ring): arms a
## `Custom`-family icon by explicit `slot`/`set` -- `icon_arm`'s numeric
## `variant` cannot address it (see that function's own Rust doc comment).
func icon_arm_custom(slot: String, set_name: String, scale: float, rotation: float, jitter: float) -> bool:
	if not _has("icon_arm_custom"):
		return false
	return world_gen.icon_arm_custom(slot, set_name, scale, rotation, jitter)

## The read side of `icon_arm_custom`: every `(set, slot)` pair the loaded
## pack actually carries at least one image for. `[]` before any pack is
## loaded, once one is loaded with no custom art, or against a GDExtension
## with no binding yet.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func icon_custom_slots() -> Array:
	if generating or not _has("icon_custom_slots"):
		return []
	return world_gen.icon_custom_slots()

## The armed icon chip contents (`WorldGen.icon_armed`). `{}` when nothing is armed
## (or no API).
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func icon_armed() -> Dictionary:
	if generating or not _has("icon_armed"):
		return {}
	return world_gen.icon_armed()

## Disarms the icon tool (`WorldGen.icon_disarm`): the next `icon_place` does nothing
## until re-armed.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func icon_disarm() -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if not _has("icon_disarm"):
		return
	world_gen.icon_disarm()

## Stamps the armed icon at grid `(gx, gy)` and selects it (`WorldGen.icon_place`),
## returning its index; `-1` means nothing was placed. Marks the world dirty first.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers -1.
func icon_place(gx: float, gy: float) -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return -1
	if not _has("icon_place"):
		return -1
	mark_world_dirty()
	return world_gen.icon_place(gx, gy)

## The density brush's three controls -- `icon_bridge/brush.rs`.
## `UNIFIED_TOOL_PLAN.md` milestone E's last open half: `_carIconBrushStamp`
## paints a blue-noise stand of the armed icon under the pointer, and is the
## larger of the reference's two *manual* placement paths (the other is
## `icon_place`). `false` against a cdylib without the binding, so the tool
## options row can hide the controls rather than draw three dead sliders.
func icon_brush_set(on: bool, radius: float, density: float) -> bool:
	if not _has("icon_brush_set"):
		return false
	return world_gen.icon_brush_set(on, radius, density)

## What the brush is set to -- `on` / `radius` / `density`. Empty before any
## world, and empty against a cdylib without the binding: `icon_armed()`'s own
## "there is nothing to report" convention, which callers already read with
## `is_empty()` rather than by comparing against invented defaults.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func icon_brush() -> Dictionary:
	if generating or not _has("icon_brush"):
		return {}
	return world_gen.icon_brush()

## One brush stamp -- how many icons it added. `0` is a real answer (a stamp
## entirely in water, or entirely inside the spacing of icons already there),
## and the reference uses it the same way: `if(_carIconBrushStamp(gx,gy))
## drawCivLayerAuto()` at line 9719, i.e. only to decide whether to redraw.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.
func icon_brush_stamp(gx: float, gy: float) -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0
	if not _has("icon_brush_stamp"):
		return 0
	mark_world_dirty()
	return world_gen.icon_brush_stamp(gx, gy)

## Clear the placed-icon selection -- `Edit > Deselect`. The icon half of
## `label_select(-1)`, which icons had no counterpart for; that asymmetry is
## the whole reason the Deselect row was disabled. Silent no-op against an
## older cdylib, which is correct here: nothing was selected in a build that
## cannot select.
func icon_deselect() -> void:
	if not _has("icon_deselect"):
		return
	world_gen.icon_deselect()

## `GUI_GAP_REGISTER.md` CA-05: `label_get_selected`'s own icon counterpart.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func icon_get_selected() -> int:
	if generating or not _has("icon_get_selected"):
		return -1
	return world_gen.icon_get_selected()

## Box hit test at grid `(gx, gy)` over every placed icon (`WorldGen.icon_hit_test`);
## selects and returns the hit index, `-1` for a miss. Handles are the caller's job.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func icon_hit_test(gx: float, gy: float) -> int:
	if generating or not _has("icon_hit_test"):
		return -1
	return world_gen.icon_hit_test(gx, gy)

## `icon_hit_test` with the modifier the click carried -- `SEL_REPLACE` /
## `SEL_TOGGLE` / `SEL_EXTEND` below. Step one of the owner's selection-sets
## ruling; the engine keeps a set per entity kind and `icon_get_selected()` is
## that set's primary.
##
## Falls back to the plain hit test against a cdylib that predates the mode
## binding, which is the right degradation: an older engine has no set to
## extend, so a modified click there behaves as a plain one rather than doing
## nothing.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func icon_hit_test_mode(gx: float, gy: float, mode: int) -> int:
	if generating or not _has("icon_hit_test_mode"):
		return icon_hit_test(gx, gy)
	return world_gen.icon_hit_test_mode(gx, gy, mode)

## `label_pick_all`'s icon twin (`icon_bridge/pick.rs`): every placed icon
## under `(gx, gy)`, topmost first, selecting nothing. Empty against a cdylib
## without the binding, for the same reason.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func icon_pick_all(gx: float, gy: float) -> PackedInt64Array:
	if generating or not _has("icon_pick_all"):
		return PackedInt64Array()
	return world_gen.icon_pick_all(gx, gy)

## Every selected icon's index, ascending. Empty against an older cdylib is a
## lie only in the "one icon is selected" case, so it falls back to whatever
## `icon_get_selected()` reports rather than to nothing.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func icon_get_selection() -> PackedInt64Array:
	if generating or not _has("icon_get_selection"):
		var one := icon_get_selected()
		return PackedInt64Array([one]) if one >= 0 else PackedInt64Array()
	return world_gen.icon_get_selection()

## Replaces the icon selection (`WorldGen.icon_select_set`). Out-of-range indices are
## dropped and reported as `false`; an empty array deselects.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func icon_select_set(indices: PackedInt64Array) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("icon_select_set"):
		return false
	return world_gen.icon_select_set(indices)

## Selects every placed icon; returns how many. `Edit > Select all`'s engine
## half -- the menu row itself is step three of the ruling and is still `_todo`.
func icon_select_all() -> int:
	if not _has("icon_select_all"):
		return 0
	return world_gen.icon_select_all()

## `GUI_GAP_REGISTER.md` CA-05: the selected icon's on-canvas resize-handle
## circle -- `label_handles()`'s own one-handle mirror.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func icon_handles(index: int, zoom: float) -> Dictionary:
	if generating or not _has("icon_handles"):
		return {}
	return world_gen.icon_handles(index, zoom)

## Applies one resize-drag sample to the selected icon (`WorldGen.icon_resize`);
## `index` must already be selected. `false` if nothing resized. Marks the world dirty.
func icon_resize(index: int, cx: float, cy: float, gx: float, gy: float, start_dist: float) -> bool:
	if not _has("icon_resize"):
		return false
	mark_world_dirty()
	return world_gen.icon_resize(index, cx, cy, gx, gy, start_dist)

## One placed icon's properties (`WorldGen.icon_get`: x, y, family, slot, scale,
## origin...). `{}` for a bad index or no API.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func icon_get(index: int) -> Dictionary:
	if generating or not _has("icon_get"):
		return {}
	return world_gen.icon_get(index)

## Removes placed icon `index` (`WorldGen.icon_delete`), fixing up the selection.
## `false` for a bad index. Marks the world dirty first.
func icon_delete(index: int) -> bool:
	if not _has("icon_delete"):
		return false
	mark_world_dirty()
	return world_gen.icon_delete(index)

## Every placed icon in placement order, each `icon_get`'s shape plus `index`,
## `selected` and `primary` (`WorldGen.icon_list`). `[]` for none or no API.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func icon_list() -> Array:
	if generating or not _has("icon_list"):
		return []
	return world_gen.icon_list()

## Drops every placed icon (`WorldGen.icon_clear_all`); the armed selection survives.
## Marks the world dirty first.
func icon_clear_all() -> void:
	if not _has("icon_clear_all"):
		return
	mark_world_dirty()
	world_gen.icon_clear_all()


# icon_bridge/generate.rs -- the generated placement pass (owner ruling
# 2026-09-02: "Build, and add a sea-marks asset family").

## The four placement families the ICONS panel draws chips for, each with its
## own vocabulary size and how much of it the loaded pack fills. A design table
## rather than world data, so it answers before any `generate()`.
##
## Empty against an older cdylib, which the panel reads as "this build has no
## placement pass" and says so -- rather than falling back to the transcribed
## design figures it used to carry, which would be a second source of truth for
## a number the engine now owns.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func icon_placement_families() -> Array:
	if generating or not _has("icon_placement_families"):
		return []
	return world_gen.icon_placement_families()

## Run the generated placement pass for one family and append what survives to
## the icon list. See `icon_bridge/generate.rs::icon_generate` for the option
## keys and the returned counters; `ok: false` carries a `reason`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "reason": _BUSY_TEXT}.
func icon_generate(options: Dictionary) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "reason": _BUSY_TEXT}
	if not _has("icon_generate"):
		return {"ok": false, "reason": "This build has no generated placement pass."}
	mark_world_dirty()
	return world_gen.icon_generate(options)


# civ_bridge.rs
## **No caller, and the shell picks settlements by a different rule.**
## `map_overlay.gd::_hit_test_settlement()` takes the settlement nearest the
## cursor within `_settlement_pin_radius(kind) + HOVER_RADIUS_PAD` -- a
## SCREEN-space test against the drawn pin. This binding is
## `cartalith_civ::civ_pick_place_at`'s weighted-nearest pick, where a bigger
## settlement outcompetes a closer small one, at a GRID-space radius
## (`tools.rs`, reference v1.88).
##
## The divergence is deliberate and it is the GUI's: a hit test must answer in
## the units the cursor is in, and only a screen radius stays constant as the
## map zooms -- a grid radius would grow and shrink under the same finger.
## What the engine's rule answers is a different question: "which settlement
## does this map coordinate belong to", which is the reference's own
## place-drop, not "which one did the user aim at". Left wrapped for that op;
## it is not a substitute for the hit test and must not be swapped in for one.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers -1.
func civ_pick_place_at(gx: float, gy: float) -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return -1
	if not _has("civ_pick_place_at"):
		return -1
	return world_gen.civ_pick_place_at(gx, gy)

## The Settlement tool's click (`WorldGen.civ_drop_settlement`): returns the new, or
## already-there, settlement's index; `-1` means nothing was placed (bad kind or
## coordinates, refused click, no world, or no binding).
## `kind` is a real tier, never `"metropolis"`; a blank `name` is engine-generated.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers -1.
func civ_drop_settlement(gx: float, gy: float, kind: String, faction: int, name: String, snap_to_water: bool) -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return -1
	if not _has("civ_drop_settlement"):
		return -1
	mark_world_dirty()
	return world_gen.civ_drop_settlement(gx, gy, kind, faction, name, snap_to_water)

## One Territory brush dab. Returns the engine's own answer: `true` when a dab
## was staged, `false` when nothing was -- no world yet, or an add dab whose
## `faction` the paint layer cannot hold (negative, or past 255), which the
## engine refuses rather than clamping to another faction
## (`WorldGen::civ_territory_paint_at`). `false` too on a binary without the
## call. The project is marked dirty only after a dab was actually staged.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func civ_territory_paint_at(gx: float, gy: float, faction: int, radius: float, subtract: bool) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("civ_territory_paint_at"):
		return false
	var staged: bool = world_gen.civ_territory_paint_at(gx, gy, faction, radius, subtract)
	if staged:
		mark_world_dirty()
	return staged

## The Territory lasso's one stamp: every cell whose centre is inside the
## ring, staged into the same draft `civ_territory_paint_at` feeds. Returns
## the number of cells staged; `0` = nothing pushed (no world, or a ring
## enclosing no cell centre); **`-1` = refused**: an add ring naming a faction
## the paint layer cannot hold (negative, or past 255), with nothing staged
## (`WorldGen::civ_territory_paint_polygon`). The project is marked dirty only
## when cells were staged.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.
func civ_territory_paint_polygon(points: PackedVector2Array, faction: int, subtract: bool) -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0
	if not _has("civ_territory_paint_polygon"):
		return 0
	var staged: int = world_gen.civ_territory_paint_polygon(points, faction, subtract)
	if staged > 0:
		mark_world_dirty()
	return staged

## Bakes the pending stroke into the claim grid. Returns the engine's `bool`:
## `false` when nothing was pending (or no world), and the project is marked
## dirty only when something was committed -- after the engine call, so a
## listener reads the committed grid, never the one before it.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func civ_territory_commit() -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("civ_territory_commit"):
		return false
	var committed: bool = world_gen.civ_territory_commit()
	if committed:
		mark_world_dirty()
	return committed

## Drops the in-progress territory draft, touching nothing committed
## (`WorldGen.civ_territory_discard`). Does not mark the world dirty.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func civ_territory_discard() -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if not _has("civ_territory_discard"):
		return
	world_gen.civ_territory_discard()

## Live territory stats for `faction` (`WorldGen.civ_faction_territory_stats`:
## `area_km2`, `claimed_cells`, ...). `{}` without the API; test with `has()`.
func civ_faction_territory_stats(faction: int) -> Dictionary:
	if not _has("civ_faction_territory_stats"):
		return {}
	return world_gen.civ_faction_territory_stats(faction)

## The assignable faction roster (`WorldGen.get_factions`), ids `1..=civ_faction_count()`.
## `[]` while `generating` (engine owned by the worker) or without a world/API.
func get_factions() -> Array:
	if not _has("get_factions"):
		return []
	## See `grid_size()` for why the in-flight guard is here rather than at
	## every caller. An empty roster is the same answer a world-less session
	## gives, which every caller already handles.
	if generating:
		return []
	return world_gen.get_factions()


# civ_roster_bridge.rs -- the place editor, the faction roster and the two
# readouts `PARITY_AUDIT.md` §5 items 3/4/7/9/10/12 found unported.

## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_settlement_details(index: int) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_settlement_details"):
		return {}
	return world_gen.civ_settlement_details(index)

## `fields` carries only the keys that changed -- see the Rust doc comment;
## an invalid value rejects the whole batch rather than half-applying it.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func civ_edit_settlement(index: int, fields: Dictionary) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("civ_edit_settlement"):
		return false
	mark_world_dirty()
	return world_gen.civ_edit_settlement(index, fields)

## FH-5 (`FACTION_HUB_DESIGN.md` §5.2 item 1): what moving settlement `index`
## to polity `to` would repaint, without changing anything. `{}` means the
## loaded engine has no such binding (an older DLL), which the caller must read
## as "unknown", never as a refusal. Otherwise `ok` is always present; `reason`
## and `message` only when `ok` is false. See the Rust doc comment
## (`WorldGen::civ_polity_reassign_preview`) for every key.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_polity_reassign_preview(index: int, to: int) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_polity_reassign_preview"):
		return {}
	return world_gen.civ_polity_reassign_preview(index, to)

## FH-5: the move itself -- the same plan the preview reports, applied as
## committed Territory paint plus the settlement's new `faction`. The project is
## marked dirty only when the engine says it applied, and only after the call
## (`MISTAKES.md`: emit/mark after the engine call).
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_polity_reassign(index: int, to: int) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_polity_reassign"):
		return {}
	var r: Dictionary = world_gen.civ_polity_reassign(index, to)
	if bool(r.get("ok", false)):
		mark_world_dirty()
	return r

## Toggles trait `key` on settlement `index` (`WorldGen.civ_settlement_toggle_trait`).
## `false` for a bad index or key, or no API. Marks the world dirty first.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func civ_settlement_toggle_trait(index: int, key: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("civ_settlement_toggle_trait"):
		return false
	mark_world_dirty()
	return world_gen.civ_settlement_toggle_trait(index, key)

## Re-rolls settlement `index`'s name from its own faction's culture
## (`WorldGen.civ_reroll_settlement_name`) and returns it; `""` means no new name.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers "".
func civ_reroll_settlement_name(index: int) -> String:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return ""
	if not _has("civ_reroll_settlement_name"):
		return ""
	mark_world_dirty()
	return world_gen.civ_reroll_settlement_name(index)

## Deletes settlement `index` (`WorldGen.civ_delete_settlement`); the shell owns the
## confirm prompt. `false` for a bad index. Marks the world dirty first.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func civ_delete_settlement(index: int) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("civ_delete_settlement"):
		return false
	mark_world_dirty()
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
	## Refuse while a generate or a landmark pass holds `world_gen` on the
	## worker thread. Without this the `bind_mut()` panics across the gdext
	## boundary -- `Gd<T>::bind_mut() failed, already bound` -- which
	## `cartalith-rust-conventions` says must never happen, since a panic
	## unwinding through a GDExtension callback can take the process down.
	##
	## This became reachable when the landmark pass moved off the main thread
	## (2026-09-02): the freeze used to make CIVIL's sibling buttons physically
	## unclickable, so responsiveness removed an accidental guard. Demonstrated
	## by `_poiverify_probe.tscn`, which presses this path mid-pass on purpose.
	if generating:
		return {"ok": false, "reason": "A generation or landmark pass is still running."}
	mark_world_dirty()
	return world_gen.recompute_civilisation()

## The other four re-entrant civ stages (`LARGE_ITEM_RULINGS.md`, owner
## 2026-08-31: "five re-entrant `#[func]`s over an existing world"). Every one
## of them carries the same two guards `civ_recompute` above does and for the
## same reasons -- a missing binding in an older extension, and the worker
## thread holding `world_gen` mid-generation -- so they are written the same
## way rather than each inventing its own refusal shape.
##
## `civ_populate` and `civ_auto_routes` return `civ_recompute`'s dictionary
## (`ok`/`ms`/`settlements`/`ways`/`provinces`/`recomputed`/`still_stale`/
## `reason`). The two clear operations return counts of what they removed and
## cannot fail, so they answer with zeroes rather than an `ok` flag.
func civ_populate() -> Dictionary:
	if not _has("civ_populate"):
		return {"ok": false, "reason": "This build's extension has no civ_populate."}
	if generating:
		return {"ok": false, "reason": "A generation or landmark pass is still running."}
	mark_world_dirty()
	return world_gen.civ_populate()

## Rebuilds the route network alone (`WorldGen.civ_auto_routes`); the reply carries
## `ok` and, when false, a `reason` (also given for no binding or a running generate).
func civ_auto_routes() -> Dictionary:
	if not _has("civ_auto_routes"):
		return {"ok": false, "reason": "This build's extension has no civ_auto_routes."}
	if generating:
		return {"ok": false, "reason": "A generation or landmark pass is still running."}
	mark_world_dirty()
	return world_gen.civ_auto_routes()

## Empties settlements and everything indexed by them (`WorldGen.civ_clear_places`).
## `{}` means this wrapper refused (no binding, or generating).
func civ_clear_places() -> Dictionary:
	if not _has("civ_clear_places") or generating:
		return {}
	mark_world_dirty()
	return world_gen.civ_clear_places()

## Empties the generated route network (`WorldGen.civ_clear_ways`). `{}` means
## this wrapper refused (no binding, or generating).
func civ_clear_ways() -> Dictionary:
	if not _has("civ_clear_ways") or generating:
		return {}
	mark_world_dirty()
	return world_gen.civ_clear_ways()

## Empties the claim map (`WorldGen.civ_clear_territory`). `{}` means refused (no
## binding, or generating).
func civ_clear_territory() -> Dictionary:
	if not _has("civ_clear_territory") or generating:
		return {}
	mark_world_dirty()
	return world_gen.civ_clear_territory()

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

## How many real, assignable factions exist (`WorldGen.civ_faction_count`);
## `0` is the engine's "before any generate" and also this stale-binary fallback.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.
func civ_faction_count() -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0
	if not _has("civ_faction_count"):
		return 0
	return world_gen.civ_faction_count()

## Appends one faction with the reference defaults (`WorldGen.civ_add_faction`) and
## returns its id; `-1` means none was added.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers -1.
func civ_add_faction() -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return -1
	if not _has("civ_add_faction"):
		return -1
	mark_world_dirty()
	return world_gen.civ_add_faction()

## Drops the last faction (`WorldGen.civ_remove_faction`), reverting its settlements
## and territory. `false` if nothing was removed.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func civ_remove_faction() -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("civ_remove_faction"):
		return false
	mark_world_dirty()
	return world_gen.civ_remove_faction()

## Writes one editable faction field (`WorldGen.civ_set_faction_field`): `key` is
## name, culture, religion, government or ag_tech. `false` for a bad faction or value.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func civ_set_faction_field(faction: int, key: String, value: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("civ_set_faction_field"):
		return false
	mark_world_dirty()
	return world_gen.civ_set_faction_field(faction, key, value)

## A faction's **identity colour** (`GUI_GAP_REGISTER.md` CV-21). Read back
## as `get_factions()`' `color_r/g/b`, with `color_custom` saying whether it
## is this or the palette rule. Every faction renderer -- the territory wash,
## the Political-control analysis field, the roster banner -- draws from it.
##
## Refused for faction 0 (Unclaimed, which nothing renders). The caller
## refreshes the map; nothing here invalidates a texture, the same contract
## every other roster edit has.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func civ_set_faction_color(faction: int, c: Color) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("civ_set_faction_color"):
		return false
	mark_world_dirty()
	return world_gen.civ_set_faction_color(faction,
		int(round(c.r * 255.0)), int(round(c.g * 255.0)), int(round(c.b * 255.0)))

## Back to the palette rule for this faction. See `civ_set_faction_color`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func civ_clear_faction_color(faction: int) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("civ_clear_faction_color"):
		return false
	mark_world_dirty()
	return world_gen.civ_clear_faction_color(faction)

## A faction's **currency** (Ruling R, kept by AR; Ruling AU: the user types
## the rate, the engine only converts). `key` is "name", "symbol" or "rate";
## the rate goes over as typed text and the engine parses it, so a typo is
## refused rather than read as 0 by `to_float()`. A blank value clears the
## member back to its default. False, changing nothing, for Unclaimed, an
## unknown faction or key, and a rate that is not a number above zero.
## Display only: nothing in the simulation reads a rate.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func civ_set_faction_currency(faction: int, key: String, value: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("civ_set_faction_currency"):
		return false
	var ok: bool = world_gen.civ_set_faction_currency(faction, key, value)
	if ok:
		mark_world_dirty()
	return ok

## `{name, symbol, rate}` plus a `*_default` flag per member (true = the
## user has not set it and the value is the stand-in: "<faction> currency",
## "¤", 1.0 at par with the world price index). `{}` for Unclaimed, an
## unknown faction, or no world -- no currency, not one at par.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_faction_currency(faction: int) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_faction_currency"):
		return {}
	return world_gen.civ_faction_currency(faction)

## An amount in world price-index units (a flow's `price` or `value` from
## `civ_trade_flows()`) in `faction`'s currency: `civ_faction_currency()`'s
## dictionary plus `amount`. `{}` when that faction has no currency; show the
## index value, labelled as the index, then.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_price_in_currency(index_amount: float, faction: int) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_price_in_currency"):
		return {}
	return world_gen.civ_price_in_currency(index_amount, faction)

## Sets the IN-13 tariff `importer` levies on goods from `exporter` (Ruling
## AE, `LARGE_ITEM_RULINGS.md`): a `0..=1` fraction of every crossing flow's
## volume; `0.0` clears it, `1.0` is an embargo. Directional -- the reverse
## pair is its own row. `false`, changing nothing, before any `generate()`,
## for an unknown faction on either side, for Unclaimed (`0`) as importer,
## for a faction taxing itself, and for a rate outside `0..=1`. Saved with
## the roster in `entities/factions.json`; nothing is recomputed here -- the
## next `civ_trade_flows()` read is what applies it.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func civ_set_trade_tariff(importer: int, exporter: int, rate: float) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("civ_set_trade_tariff"):
		return false
	var ok: bool = world_gen.civ_set_trade_tariff(importer, exporter, rate)
	if ok:
		mark_world_dirty()
	return ok

## The rate `importer` levies on goods from `exporter`; `0.0` when no tariff
## is set, for an unknown faction, and before any `generate()`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.0.
func civ_trade_tariff(importer: int, exporter: int) -> float:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0.0
	if not _has("civ_trade_tariff"):
		return 0.0
	return world_gen.civ_trade_tariff(importer, exporter)

## Whether any faction carries a user identity colour.
##
## **No caller.** The one surface that asks a question of this shape --
## `faction_roster_window.gd`'s Reset button -- needs the PER-FACTION answer
## and reads `color_custom` off the `get_factions()` row it already holds
## (`faction_roster_window.gd`, the `reset.disabled` line). The aggregate has
## no reader today; it is what a "reset every faction's colour" row would
## gate on, and that row does not exist yet.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func civ_has_faction_colors() -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("civ_has_faction_colors"):
		return false
	return world_gen.civ_has_faction_colors()

## Per-faction culture/terrain fit for every faction in one call
## (`WorldGen.civ_faction_terrain_fits`). `[]` without a world or the API.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func civ_faction_terrain_fits() -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("civ_faction_terrain_fits"):
		return []
	return world_gen.civ_faction_terrain_fits()

## `OUTSTANDING_WORK.md` §2.3's "`_civFactionAggregates`' resource- and
## density-fed half, as a *surfaced* readout". Two full-grid passes on the
## engine side (see the `#[func]`'s own doc), so this is a category-open call,
## never a per-frame one.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func civ_faction_economy() -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("civ_faction_economy"):
		return []
	return world_gen.civ_faction_economy()

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
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_military_summary() -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_military_summary"):
		return {}
	return world_gen.civ_military_summary()

## MM-8 (`MILITARY_MANPOWER_SCOPE.md` §5.7): the same answer read at the
## Timeline cursor's `year`, from the recorded snapshot in force. Adds
## `reading` (`live` / `recorded` / `none_in_force` / `unreadable`),
## `year_in_force` or `earliest_year`. Settlement rows carry MM-6's `garrison`
## keys when they have one (absent otherwise, never 0). `{}` on an older
## binary.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_military_summary_at(year: int) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_military_summary_at"):
		return {}
	return world_gen.civ_military_summary_at(year)

## `{"year": int}` -- the recorded year a reading at `year` uses -- or `{}`
## when there is none. Cheap; CIVIL ▸ Military reads it per cursor step to
## refill only when the year in force changes.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_year_in_force(year: int) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_year_in_force"):
		return {}
	return world_gen.civ_year_in_force(year)

## MM-6 for one settlement by `tid`, read at `year`: `garrison`,
## `garrison_share`, `border_exposure`, `garrison_multiplier`,
## `faction_garrison` -- or `absent` (`no_reading` / `not_recorded` /
## `unclaimed` / `no_claim_grid` / `no_standing`). Always `reading`. `{}` before a world or on an
## older binary.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_settlement_garrison(tid: int, year: int) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_settlement_garrison"):
		return {}
	return world_gen.civ_settlement_garrison(tid, year)


## Ruling AZ (2026-09-28): the garrison rule's border-exposure scale
## (`MILITARY_MANPOWER_SCOPE.md` §5.6), CIVIL ▸ Military ▸ Garrisons' own
## slider. `1.0` -- the rule's old fixed value -- on a binary too old to have
## the setter, so a stale build reads the same figure the rule always used.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 1.0.
func get_garrison_exposure_scale() -> float:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 1.0
	if not _has("get_garrison_exposure_scale"):
		return 1.0
	return world_gen.get_garrison_exposure_scale()


## Re-reads garrisons the next time one is asked for; regenerates nothing.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func set_garrison_exposure_scale(scale: float) -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if not _has("set_garrison_exposure_scale"):
		return
	world_gen.set_garrison_exposure_scale(scale)


# conflict_bridge.rs -- `STORY_PLANNING_SCOPE.md` SP-4, the conflict overlay.
# `conflict_add`/`conflict_update` return `{ok, id, error?}`; see the Rust
# doc comments for the field keys and the store's lifecycle.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers PackedStringArray().
func conflict_kinds() -> PackedStringArray:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return PackedStringArray()
	if not _has("conflict_kinds"):
		return PackedStringArray()
	return world_gen.conflict_kinds()

## Adds a conflict from `fields` (`WorldGen.conflict_add`); the reply is
## `{ok, id, error?}`. Only `ok: true` marks the world dirty; the stale-binary reply
## is `ok: false`, `id: 0`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "id": 0, "error": _BUSY_TEXT}.
func conflict_add(fields: Dictionary) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "id": 0, "error": _BUSY_TEXT}
	if not _has("conflict_add"):
		return {"ok": false, "id": 0, "error": "This build has no conflict_add binding."}
	var r: Dictionary = world_gen.conflict_add(fields)
	if r.get("ok", false):
		mark_world_dirty()
	return r

## Changes the fields present in `fields` on conflict `id` (`WorldGen.conflict_update`);
## reply `{ok, id, error?}`. Only `ok: true` marks the world dirty.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "id": id, "error": _BUSY_TEXT}.
func conflict_update(id: int, fields: Dictionary) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "id": id, "error": _BUSY_TEXT}
	if not _has("conflict_update"):
		return {"ok": false, "id": id, "error": "This build has no conflict_update binding."}
	var r: Dictionary = world_gen.conflict_update(id, fields)
	if r.get("ok", false):
		mark_world_dirty()
	return r

## Deletes conflict `id` (`WorldGen.conflict_delete`). `false` for an unknown id;
## only a `true` marks the world dirty.
func conflict_delete(id: int) -> bool:
	if not _has("conflict_delete"):
		return false
	var ok: bool = world_gen.conflict_delete(id)
	if ok:
		mark_world_dirty()
	return ok

## Every conflict in creation order (`WorldGen.conflict_list`), points already
## resolved. `[]` for none or no API.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func conflict_list() -> Array:
	if generating or not _has("conflict_list"):
		return []
	return world_gen.conflict_list()

# campaign_bridge.rs -- Ruling AW, the CARTO ▸ Conflict layer's rows for one
# year: siege rings, front segments and changed-hands cells of every conflict
# active that year. Keys that cannot be read are absent; see the Rust doc.
func conflict_campaigns(year: int) -> Array:
	if not _has("conflict_campaigns"):
		return []
	return world_gen.conflict_campaigns(year)

## One conflict as `conflict_list` describes it (`WorldGen.conflict_get`). `{}` for an
## unknown id.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func conflict_get(id: int) -> Dictionary:
	if generating or not _has("conflict_get"):
		return {}
	return world_gen.conflict_get(id)

## Ids of conflicts attached to settlement `tid` (`kind` `"settlement"`) or the
## province it seeds (`"province"`) (`WorldGen.conflicts_attached_to`). Empty for none.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers PackedInt64Array().
func conflicts_attached_to(kind: String, tid: int) -> PackedInt64Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return PackedInt64Array()
	if not _has("conflicts_attached_to"):
		return PackedInt64Array()
	return world_gen.conflicts_attached_to(kind, tid)

## SP-5: conflicts attached to settlement `tid` or to its province, each a
## `conflict_list()` row plus `via`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func conflicts_touching_settlement(tid: int) -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("conflicts_touching_settlement"):
		return []
	return world_gen.conflicts_touching_settlement(tid)

## One manpower row per side of conflict `id` (`WorldGen.conflict_sides_manpower`).
## Optional keys are omitted when the world has no row, so test with `has()`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func conflict_sides_manpower(id: int) -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("conflict_sides_manpower"):
		return []
	return world_gen.conflict_sides_manpower(id)

## CIVIL ▸ Relationships (`GUI_GAP_REGISTER.md` CV-26). One row per
## unordered faction pair, with the four terms beside the verdict. Derived
## and recomputed on every call; nothing here is stored or saved.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func civ_faction_relations() -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("civ_faction_relations"):
		return []
	return world_gen.civ_faction_relations()

## The "Land sustains ~N" population readout (`WorldGen.civ_agrarian_regional_total`).
## `{}` without a world or the API.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_agrarian_regional_total() -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_agrarian_regional_total"):
		return {}
	return world_gen.civ_agrarian_regional_total()

## Settlement trait vocabulary `{key, label, glyph}` (`WorldGen.civ_trait_vocabulary`);
## order is part of the save format. `[]` without the API.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func civ_trait_vocabulary() -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("civ_trait_vocabulary"):
		return []
	return world_gen.civ_trait_vocabulary()

## Specialisation `{key, label}` vocabulary (`WorldGen.civ_specialisation_vocabulary`).
## `[]` without the API.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func civ_specialisation_vocabulary() -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("civ_specialisation_vocabulary"):
		return []
	return world_gen.civ_specialisation_vocabulary()

## Built-in religion `{key, label}` vocabulary (`WorldGen.civ_religion_vocabulary`).
## `[]` without the API.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func civ_religion_vocabulary() -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("civ_religion_vocabulary"):
		return []
	return world_gen.civ_religion_vocabulary()

## `FACTION_HUB_DESIGN.md` §6 R1 (`civ_religion_bridge.rs`): the project's
## custom religions, each a named variant that BEHAVES AS its `base_key` --
## `{id, key, name, base_key, base_label, color_r/g/b, notes, used_by}`. `[]`
## before a world exists, and on a build without the binding (test
## `has_custom_religion_api()` to tell those apart).
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func civ_custom_religions() -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("civ_custom_religions"):
		return []
	return world_gen.civ_custom_religions()

## Whether this build's engine has the R1 library at all. The hub hides the
## library group (with a note) rather than drawing controls nothing answers.
func has_custom_religion_api() -> bool:
	return _has("civ_add_religion")

## The three R1 mutations. Each answers `{ok: true, ...}` or `{ok: false,
## error}` with a plain-English reason, and writes nothing on refusal. Every
## one is a saved edit, so the world is marked dirty -- the same rule as
## `civ_set_faction_field`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func civ_add_religion(religion_name: String, base_key: String, c: Color, notes: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("civ_add_religion"):
		return {"ok": false, "error": "This build's engine has no custom-religion binding."}
	var r: Dictionary = world_gen.civ_add_religion(religion_name, base_key, c.r8, c.g8, c.b8, notes)
	if bool(r.get("ok", false)):
		mark_world_dirty()
	return r

## Edits custom religion `id` (`WorldGen.civ_edit_religion`); reply `{ok, error?}`.
## Only `ok: true` marks the world dirty; no binding gives `ok: false` with a reason.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func civ_edit_religion(id: int, fields: Dictionary) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("civ_edit_religion"):
		return {"ok": false, "error": "This build's engine has no custom-religion binding."}
	var r: Dictionary = world_gen.civ_edit_religion(id, fields)
	if bool(r.get("ok", false)):
		mark_world_dirty()
	return r

## Deletes custom religion `id` unless a faction holds it (`WorldGen.civ_delete_religion`);
## reply `{ok, error?, used_by?}`. Only `ok: true` marks the world dirty.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func civ_delete_religion(id: int) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("civ_delete_religion"):
		return {"ok": false, "error": "This build's engine has no custom-religion binding."}
	var r: Dictionary = world_gen.civ_delete_religion(id)
	if bool(r.get("ok", false)):
		mark_world_dirty()
	return r

## `RELIGION_DIFFUSION_SCOPE.md` §3 milestone 1 (`lib.rs`'s `civ_belief_run`):
## seed the per-settlement adherence layer from the faction roster if it is
## missing or stale, then run `years` diffusion steps over the generated road
## network. One step is one Timeline year.
##
## **Not a read.** It mutates `CivData::belief`, and every other reader sees
## the result on its next call -- `settlements()` above starts emitting a
## `religion` key and an `adherents` dictionary per entry once this has run.
## Nothing is persisted: `CivData::belief`'s own doc states the cost plainly,
## a run does not survive a save/load, and re-running the same world
## reproduces it exactly because the model draws no random numbers.
##
## Returns `{settlements, seeded, years, any_faith}`, plus `reason` (String)
## **only** when `any_faith` is false. `{}` is not one of those states: it is
## "this build's native library has no such binding", the same answer
## `civ_military_summary()` gives, and it is why a caller must test
## `is_empty()` before reading `any_faith` -- an older cdylib would otherwise
## read as a world in which nothing has a religion, which is a real and
## different state with a fix the user can act on.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_belief_run(years: int) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_belief_run"):
		return {}
	return world_gen.civ_belief_run(years)

## Whether the loaded native library can run the diffusion at all, asked
## **without** running it.
##
## `civ_belief_run(0)` would answer the same question and is not free: a zero-
## year call still re-seeds a stale layer, which discards a previous run's
## diffused adherence. A panel that rebuilds on every roster edit must not do
## that as a side effect of drawing itself.
func has_belief_api() -> bool:
	return _has("civ_belief_run")

## Government `{key, label}` vocabulary (`WorldGen.civ_government_vocabulary`).
## `[]` without the API.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func civ_government_vocabulary() -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("civ_government_vocabulary"):
		return []
	return world_gen.civ_government_vocabulary()

## Agricultural-tech `{key, label, hint, farmers_per_urbanite}` vocabulary
## (`WorldGen.civ_ag_tech_vocabulary`). `[]` without the API.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func civ_ag_tech_vocabulary() -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("civ_ag_tech_vocabulary"):
		return []
	return world_gen.civ_ag_tech_vocabulary()

## The naming-culture keys (`WorldGen.civ_culture_vocabulary`); keys only, the
## reference has no labels. Empty without the API.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers PackedStringArray().
func civ_culture_vocabulary() -> PackedStringArray:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return PackedStringArray()
	if not _has("civ_culture_vocabulary"):
		return PackedStringArray()
	return world_gen.civ_culture_vocabulary()

## The same seven cultures as **rows**: `{id, key, name, terrain_affinity,
## faction_count, factions, settlement_count, population}` (`lib.rs`'s
## `get_cultures`, `GUI_GAP_REGISTER.md` CV-02).
##
## Non-empty **before** a world exists -- the seven are compile-time
## constants; only the three aggregates need a `civ` layer and come back
## zero without one. So a caller must not use emptiness as "no world":
## `[]` here means an engine too old to have the binding, nothing else.
##
## `id` is the 0-based `CIV_CULTURES` index, which is what the Markdown
## Vault addresses a culture by (`EntityKind::Culture`) and the one entity
## id in this port that survives both a regenerate and a save/load.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func get_cultures() -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("get_cultures"):
		return []
	return world_gen.get_cultures()


## Turns the biome carrying-capacity residual on/off (`WorldGen.set_biome_k_enabled`);
## takes effect on the next generate. No-op without the binding; marks the world dirty.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func set_biome_k_enabled(enabled: bool) -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if not _has("set_biome_k_enabled"):
		return
	mark_world_dirty()
	world_gen.set_biome_k_enabled(enabled)

## Whether the biome carrying-capacity residual is on (`WorldGen.get_biome_k_enabled`);
## `false` is also the stale-binary fallback.
func get_biome_k_enabled() -> bool:
	if not _has("get_biome_k_enabled"):
		return false
	return world_gen.get_biome_k_enabled()


# paint_bridge.rs
## While `generating` the worker thread holds `WorldGen`, so this answers its empty default without reaching the engine (any `#[func]` call then is the `bind() failed` panic; see `param_get`).
func get_paint_layers() -> PackedStringArray:
	if generating or not _has("get_paint_layers"):
		return PackedStringArray()
	return world_gen.get_paint_layers()

## Legal value range of paint `layer` as 1-based `{index, label}` rows
## (`WorldGen.get_paint_palette`). `[]` for an unknown layer or no API.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func get_paint_palette(layer: String) -> Array:
	if generating or not _has("get_paint_palette"):
		return []
	return world_gen.get_paint_palette(layer)

## Switches the layer `paint_stroke_at` writes to (`WorldGen.paint_set_layer`). `false`
## for an unknown layer name or no API.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func paint_set_layer(layer: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("paint_set_layer"):
		return false
	return world_gen.paint_set_layer(layer)

## Sets the paint brush (`WorldGen.paint_set_brush`: value, radius, hardness, softness,
## erase, land_only) and returns the engine's accepted values; `{}` without the API.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func paint_set_brush(value: int, radius: float, hardness: float, softness: float, erase: bool, land_only: bool) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("paint_set_brush"):
		return {}
	return world_gen.paint_set_brush(value, radius, hardness, softness, erase, land_only)

## One brush dab at grid `(gx, gy)` onto the active layer's draft
## (`WorldGen.paint_stroke_at`). Draft only until `paint_commit`; marks the world dirty.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func paint_stroke_at(gx: float, gy: float) -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if not _has("paint_stroke_at"):
		return
	mark_world_dirty()
	world_gen.paint_stroke_at(gx, gy)

## True when the loaded extension carries the whole ET-D2 stroke surface
## (`paint_stroke_begin`/`_sample`/`_end`). Callers fall back to the one-dab
## `paint_stroke_at` path when it is `false`; checked without `_has()` so an older
## extension is not warned about on every press.
func has_paint_stroke_api() -> bool:
	return world_gen != null and world_gen.has_method("paint_stroke_begin") 		and world_gen.has_method("paint_stroke_sample") and world_gen.has_method("paint_stroke_end")

## Opens a paint stroke on the engine's stroke engine (`WorldGen.paint_stroke_begin`).
## `options` takes the engine's own keys (spacing, smoothing, symmetry, scatter,
## pressure_size ...; every one optional, a wrong type or range refuses the whole
## call). The active brush and layer are snapshotted at this call. `false` on
## refusal or without the API; never marks the world dirty (no cell changed yet).
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func paint_stroke_begin(options: Dictionary = {}) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("paint_stroke_begin"):
		return false
	return world_gen.paint_stroke_begin(options)

## Feeds one pointer sample into the open stroke (`WorldGen.paint_stroke_sample`).
## `pressure` is a float in 0..1 or **`null`** (the device reported none: pass
## null, never 1.0 or 0.0); `tilt` is a `Vector2` or `null`. Returns the engine's
## reply: `{dabs: n}` plus the preview-patch keys (`texture`, `x`, `y`, `w`, `h`)
## when `n > 0`; **`{}` means refusal** (no open stroke, a bad argument). Test
## `has("texture")` before blitting. Marks the world dirty when dabs landed.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func paint_stroke_sample(gx: float, gy: float, pressure: Variant, tilt: Variant, t_ms: int) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("paint_stroke_sample"):
		return {}
	var reply: Dictionary = world_gen.paint_stroke_sample(gx, gy, pressure, tilt, t_ms)
	if int(reply.get("dabs", 0)) > 0:
		mark_world_dirty()
	return reply

## Closes the open stroke and flushes the smoothing tail
## (`WorldGen.paint_stroke_end`); same reply shape as `paint_stroke_sample`. `{}`
## without the API or with no stroke open. Must be called for every stroke
## `paint_stroke_begin` accepted, or the engine keeps it open.
func paint_stroke_end() -> Dictionary:
	if not _has("paint_stroke_end"):
		return {}
	var reply: Dictionary = world_gen.paint_stroke_end()
	if int(reply.get("dabs", 0)) > 0:
		mark_world_dirty()
	return reply

## Overlay of the active layer's committed paint plus this session's draft
## (`WorldGen.build_paint_preview_texture`). `null` without a world or the API.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func build_paint_preview_texture() -> Texture2D:
	if generating or not _has("build_paint_preview_texture"):
		return null
	return world_gen.build_paint_preview_texture()

## The same preview restricted to the rectangle the uncommitted draft touches
## -- `{texture, x, y, w, h}`, blitted at grid cell `(x, y)` by
## `ViewportHost.set_preview_patch()`. This is the call to prefer inside a
## drag; `build_paint_preview_texture()` above stays the one to take when
## there is no previous raster on screen to composite onto.
##
## **An empty Dictionary here always means the engine's own "nothing to draw
## at all"**, never "this build cannot answer". A library too old to have the
## method degrades to the full raster wearing the patch's shape instead --
## a full-grid window is a legal answer (`PaintEditor::preview_patch`'s own
## doc, row two of its table), so the caller keeps one code path and the one
## Dictionary that clears the preview keeps one meaning.
func build_paint_preview_patch() -> Dictionary:
	if _has("build_paint_preview_patch"):
		return world_gen.build_paint_preview_patch()
	var tex: Texture2D = build_paint_preview_texture()
	if tex == null:
		return {}
	return {"texture": tex, "x": 0, "y": 0, "w": tex.get_width(), "h": tex.get_height()}

## Per-class painted-cell counts for the active layer (`WorldGen.paint_painted_counts`).
## `{}` without the API.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func paint_painted_counts() -> Dictionary:
	if generating or not _has("paint_painted_counts"):
		return {}
	return world_gen.paint_painted_counts()

## `GUI_GAP_REGISTER.md` WW-13: the pending-draft dab count, which is what
## Commit / Discard act on. `paint_painted_counts()["total"]` above is the
## committed-plus-pending composite and is a legend's number, not a button's.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func paint_draft_count() -> int:
	if generating or not _has("paint_draft_count"):
		return 0
	return world_gen.paint_draft_count()

## GF-8 (`GEOLOGY_FIRST_SCOPE.md` §4.13): whether this world can take a Rock paint, and
## why not (`WorldGen.rock_paint_status`): `available`, `reason` (only when not
## available), `layers` (`[{key, label}]` for Surface / Beneath / Cap thickness),
## `cap_class_m`, `cap_max_m`, `sub_none`, `pad_cells`, `painted_cells` (only when the
## column has a painted mask), `save_note` and `saved` (true when a committed rock is
## written with the project, GF-8's save slice). **`{}` while `generating` or when the
## loaded extension predates GF-8** -- callers treat `{}` as "Rock is not offered" and
## say why with `rock_paint_unavailable_reason()`. Checked with `has_method` and called
## with `call()`, not `_has()` plus a direct call: every paint surface reads it on every
## rebuild, so a pre-GF-8 binary must neither warn per rebuild nor fail to parse.
func rock_paint_status() -> Dictionary:
	if generating or world_gen == null or not world_gen.has_method("rock_paint_status"):
		return {}
	return world_gen.call("rock_paint_status")

## The sentence a disabled Rock target carries: the engine's own refusal when it gave
## one, else why the shell could not ask. Always ends with the engine's save note, so
## the user is told before painting what a save keeps of Rock paint and what it does not.
func rock_paint_unavailable_reason(st: Dictionary) -> String:
	var why: String
	if generating:
		why = "A world is being generated."
	elif st.is_empty():
		why = "This build of the engine has no Rock paint (GF-8)."
	else:
		why = String(st.get("reason", ""))
	var note := String(st.get("save_note", ROCK_PAINT_SAVE_NOTE_FALLBACK))
	return (why + " " + note).strip_edges()

## The save caveat for a binary too old to send its own (`ROCK_PAINT_SAVE_NOTE` in
## `lib.rs`, which wins whenever the engine answers). Only a binary from before GF-8's
## save slice can fail to answer, and such a binary did not write the geology column, so
## this says so rather than repeating the current engine's sentence.
const ROCK_PAINT_SAVE_NOTE_FALLBACK := "This build of the engine did not say whether Rock paint is saved; a build from before 2026-10-18 does not save the geology column, so reopening the project loses the painted rock (and the whole column)."

## GF-8: the eleven rock classes with their `ROCK_PROPS` resistance values
## (`WorldGen.get_rock_classes`: `index`, `name`, `s`, `kappa`, `theta_c`, `gamma`,
## `rho`, `selby`, `soluble`). `[]` while `generating` or on a pre-GF-8 binary.
func get_rock_classes() -> Array:
	if generating or world_gen == null or not world_gen.has_method("get_rock_classes"):
		return []
	return world_gen.call("get_rock_classes")

## GF-8: whether the live world carries a geology column (`WorldGen.geology_available`).
## `false` while `generating` or on a pre-GF-8 binary -- never a guess that it does.
func geology_available() -> bool:
	if generating or world_gen == null or not world_gen.has_method("geology_available"):
		return false
	return bool(world_gen.call("geology_available"))

## Bakes every layer's draft into its override array (`WorldGen.paint_commit`); returns
## the engine's summary, `{}` without the API. Marks the world dirty first. With GF-8
## Rock dabs pending the summary also carries `rock` (the bake and the re-erosion of
## the painted window, or `{ok: false, reason}`; absent when no Rock dab was pending).
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func paint_commit() -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("paint_commit"):
		return {}
	mark_world_dirty()
	return world_gen.paint_commit()

## Drops every layer's pending draft (`WorldGen.paint_discard`) and returns how many
## dabs went; `0` means none, `-1` that the binding is missing.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers -1.
func paint_discard() -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return -1
	if not _has("paint_discard"):
		return -1
	mark_world_dirty()
	return world_gen.paint_discard()


# way_bridge.rs
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func way_begin(way_type: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("way_begin"):
		return false
	return world_gen.way_begin(way_type)

## Appends a waypoint, snapped to a nearby place or way, to the armed Way draft
## (`WorldGen.way_append_point`). `false` if no Way draft is armed.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func way_append_point(gx: float, gy: float) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("way_append_point"):
		return false
	return world_gen.way_append_point(gx, gy)

## Commits the Way draft by least-cost routing (`WorldGen.way_commit`); returns the new
## way's index, `-1` for no draft, under two waypoints or no world. Marks the world dirty.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers -1.
func way_commit() -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return -1
	if not _has("way_commit"):
		return -1
	mark_world_dirty()
	return world_gen.way_commit()

## Discards the in-progress way without committing (`WorldGen.way_discard`).
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func way_discard() -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if not _has("way_discard"):
		return
	world_gen.way_discard()

## ET-D4: true when the loaded extension has the whole drag-to-draw road API
## (`way_stroke_begin`/`_sample`/`_end`/`_cancel`). Callers fall back to the click path
## (`way_begin`/`way_append_point`/`way_commit`, which this API never touches) when it is
## `false`. Checked with `has_method` rather than `_has()` so an older extension is not
## warned about on every press (the `has_paint_stroke_api` precedent). Never `true` for a
## partial set: a drag whose end or cancel is missing could not be closed.
func has_way_stroke_api() -> bool:
	return world_gen != null and world_gen.has_method("way_stroke_begin") \
		and world_gen.has_method("way_stroke_sample") and world_gen.has_method("way_stroke_end") \
		and world_gen.has_method("way_stroke_cancel")

## ET-D4: opens a drag-to-draw road (`WorldGen.way_stroke_begin`). `options` takes the
## engine's own keys (`way_type` REQUIRED; `mode`, `spacing`, `smoothing`, `symmetry`,
## `smooth`, `smooth_spacing`, `guide`, `corridor`, `pull` ... all optional); a wrong type, an
## unknown name or an out-of-range value refuses the WHOLE call, so pass ints where ints are
## expected and floats where floats are expected. `true` when the drag is open; `false` (and
## nothing changed, an open drag surviving) without a world, without the API, on a refused
## option, or while a generation is running. Never marks the world dirty: nothing changes
## until `way_stroke_end` commits.
func way_stroke_begin(options: Dictionary) -> bool:
	if generating or not _has("way_stroke_begin"):
		return false
	return world_gen.way_stroke_begin(options)

## ET-D4: feeds one pointer sample, in grid cells, into the open drag
## (`WorldGen.way_stroke_sample`). Returns the waypoint count so far, or **`-1`** when no drag
## is open, there is no world, the API is absent, or a generation is running (the engine is
## not touched then). `-1` is the closed test: a probe's positive control is a count `>= 0`
## from an open drag. Nothing is committed here and the world is not marked dirty.
func way_stroke_sample(gx: float, gy: float, t_ms: int) -> int:
	if generating or not _has("way_stroke_sample"):
		return -1
	return world_gen.way_stroke_sample(gx, gy, t_ms)

## ET-D4: ends the drag and commits it as one unit (`WorldGen.way_stroke_end`). Returns the
## engine's reply untouched: `{}` when no drag was open (or no world, no API, or a
## generation is running: nothing was consumed); `{committed: true, first, count, mode,
## unreachable_legs, relaxed_legs}` on success; `{committed: false, refusal: String, ...}`
## when the drag was consumed but nothing was pushed (`too_few_waypoints`, `out_of_grid`,
## `symmetry`, `degenerate`, `no_world`, `water`). Marks the world dirty on success only.
func way_stroke_end() -> Dictionary:
	if generating or not _has("way_stroke_end"):
		return {}
	var reply: Dictionary = world_gen.way_stroke_end()
	if bool(reply.get("committed", false)):
		mark_world_dirty()
	return reply

## ET-D4: abandons the open drag, committing nothing (`WorldGen.way_stroke_cancel`). A no-op
## with no drag open, without the API, or while a generation is running (the caller then
## keeps a stale flag and cancels on the next safe call: see `infrastructure_workspace.gd`).
func way_stroke_cancel() -> void:
	if generating or not _has("way_stroke_cancel"):
		return
	world_gen.way_stroke_cancel()

## ET-D4: takes the last drag commit back as one step (`WorldGen.way_stroke_undo`): returns
## how many ways were removed, `0` when nothing was touched (nothing to undo, or the manual
## way list changed since). Single level, and NOT a row in the global Edit > Undo ledger:
## nothing in the shell calls this yet. Marks the world dirty when it removed something.
func way_stroke_undo() -> int:
	if generating or not _has("way_stroke_undo"):
		return 0
	var n: int = world_gen.way_stroke_undo()
	if n > 0:
		mark_world_dirty()
	return n


# route_bridge.rs
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func route_begin(mode: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("route_begin"):
		return false
	return world_gen.route_begin(mode)

## Appends a stop, snapped to a place or way, to the armed Route draft
## (`WorldGen.route_append_stop`). `false` if no Route draft is armed.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func route_append_stop(gx: float, gy: float) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("route_append_stop"):
		return false
	return world_gen.route_append_stop(gx, gy)

## Commits the Route draft (`WorldGen.route_commit`); returns the new route's index or
## `-1`, same convention as `way_commit`. Marks the world dirty.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers -1.
func route_commit() -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return -1
	if not _has("route_commit"):
		return -1
	mark_world_dirty()
	return world_gen.route_commit()

## Discards the in-progress route without committing (`WorldGen.route_discard`).
func route_discard() -> void:
	if not _has("route_discard"):
		return
	world_gen.route_discard()

## How many routes were committed this session (`WorldGen.route_count`); `0` also
## means no world or no binding.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func route_count() -> int:
	if generating or not _has("route_count"):
		return 0
	return world_gen.route_count()

## One committed route (`WorldGen.route_get`: points, brks, km, mode, name...). `{}`
## for an out-of-range index or no API.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func route_get(index: int) -> Dictionary:
	if generating or not _has("route_get"):
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
	mark_world_dirty()
	return world_gen.route_delete(index)

## CM-7 (`MAP_CONTEXT_SCOPE.md` §9.2, Ruling BA): the way under a grid point,
## within `radius_cells` of its drawn curve -- `{store, index, way_type, km,
## manual, x, y, dist}` plus `name` when it has one, or `{}`. Read-only.
## `store` is "generated", "sea_lane" or "manual"; `way_get`/`way_delete` take
## the same pair. `context_pick_bridge.rs` carries the rest.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func way_pick(gx: float, gy: float, radius_cells: float) -> Dictionary:
	if generating or not _has("way_pick"):
		return {}
	return world_gen.way_pick(gx, gy, radius_cells)

## One way, whole, with its stored control points -- `{}` for an unknown pair.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func way_get(store: String, index: int) -> Dictionary:
	if generating or not _has("way_get"):
		return {}
	return world_gen.way_get(store, index)

## Deletes one way and records an undo step for it (Ruling BA): Edit ▸ Undo
## puts it back at the same index. Later indices in the same store shift down.
func way_delete(store: String, index: int) -> bool:
	if not _has("way_delete"):
		return false
	var ok: bool = world_gen.way_delete(store, index)
	if ok:
		mark_world_dirty()
	return ok

## CM-7 / Ruling BA: the river under a grid point as "the branch to its
## mouth" -- `{cell, x, y, order, dist, branch, cells, km, terminus}`, or `{}`.
## No `name`: rivers are unnamed in this engine.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func river_pick(gx: float, gy: float, radius_cells: float) -> Dictionary:
	if generating or not _has("river_pick"):
		return {}
	return world_gen.river_pick(gx, gy, radius_cells)

## Every cell draining into `cell`, as a `gw * gh` byte mask (1 = drains here).
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers PackedByteArray().
func river_catchment(cell: int) -> PackedByteArray:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return PackedByteArray()
	if not _has("river_catchment"):
		return PackedByteArray()
	return world_gen.river_catchment(cell)

## A `w * h` 0/1 mask as an RGBA image, `color` where set -- for the overlay's
## catchment highlight. Null against an older cdylib or a mask of the wrong size.
func mask_rgba_image(mask: PackedByteArray, w: int, h: int, color: Color) -> Image:
	if not _has("mask_rgba_image"):
		return null
	return world_gen.mask_rgba_image(mask, w, h, color)

## What `undo_last()` would revert next: "height", "civ" (a way delete) or "". "" while
## `generating` (nothing can be reverted then, and the worker holds the engine: G6's guard,
## the same as `can_undo`).
func undo_next_subsystem() -> String:
	if generating:
		return ""
	if not _has("undo_next_subsystem"):
		return "height" if can_undo() else ""
	return String(world_gen.undo_next_subsystem())

## Renames committed route `index` (`WorldGen.route_set_name`). `false` for a bad
## index or no world. Marks the world dirty first.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func route_set_name(index: int, name: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("route_set_name"):
		return false
	mark_world_dirty()
	return world_gen.route_set_name(index, name)


## `STORY_PLANNING_SCOPE.md` SP-1. `route_index` is `route_get`'s own index
## space; the route's geometry is copied into the journey at save time, not
## referenced -- see `cartalith_civ::travel_library::JourneyRoute`'s doc
## comment. Returns the new journey's stable id, or `-1`.
func journey_save(name: String, party_preset: String, route_index: int, start_year: int) -> int:
	if not _has("journey_save"):
		return -1
	mark_world_dirty()
	return world_gen.journey_save(name, party_preset, route_index, start_year)

## Every saved journey in save order (`WorldGen.journey_list`: id, name, party preset,
## start year, km, mode). `[]` for none or no API.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func journey_list() -> Array:
	if generating or not _has("journey_list"):
		return []
	return world_gen.journey_list()

## Ruling BS: replaces the accepted resupply stops saved with journey `id`
## (`jp_compute`'s own `resupply_accepted` shape). `false` for an unknown id.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func journey_set_resupply(id: int, stops: Array) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("journey_set_resupply"):
		return false
	mark_world_dirty()
	return world_gen.journey_set_resupply(id, stops)

## Ruling AR: stores the whole planner plan with journey `id`, so it plans the
## same after a reopen (`entities/journeys.json`'s `plan` member). `request` is
## `jp_compute`'s own vocabulary -- `plan`, `stage_overrides`, `layovers`,
## `animal_entries`, `trim`, `auto_carriage`, `auto_stage` -- minus the route.
## Returns `{ok, error, rejected}`; an older binary without the call answers
## `{ok: false}` rather than raising, and the journey then keeps only its
## party preset exactly as before the ruling.
func journey_set_plan(id: int, request: Dictionary) -> Dictionary:
	if not _has("journey_set_plan"):
		return {"ok": false, "error": "journey_set_plan is not exposed by this build's GDExtension binary -- rebuild cartalith-godot.", "rejected": PackedStringArray()}
	mark_world_dirty()
	return world_gen.journey_set_plan(id, request)

## One journey's full detail by stable `id` (`WorldGen.journey_get`), including route
## points. `{}` for an unknown id.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func journey_get(id: int) -> Dictionary:
	if generating or not _has("journey_get"):
		return {}
	return world_gen.journey_get(id)

## Deletes the journey with stable `id` (`WorldGen.journey_delete`). `false` if no
## journey has that id. Marks the world dirty first.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func journey_delete(id: int) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("journey_delete"):
		return false
	mark_world_dirty()
	return world_gen.journey_delete(id)

## `STORY_PLANNING_SCOPE.md` SP-2 (`story_bridge.rs`): every saved journey at
## the cursor date -- position (`x`/`y`, absent when the plan is blocked),
## phase, supply used, the derived `arrival`. See the `#[func]`'s own doc.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func journey_positions() -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("journey_positions"):
		return []
	return world_gen.journey_positions()

## SP-3's journey-pass mark (`story_bridge.rs`): every saved journey whose
## route passes the settlement `tid` (the planner's own stop test), with the
## pass's `date`/`year`/`day_offset` -- absent, with an `error`, when the plan
## has no honest date. See the `#[func]`'s own doc.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func civ_settlement_journey_passes(tid: int) -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("civ_settlement_journey_passes"):
		return []
	return world_gen.civ_settlement_journey_passes(tid)

## What the last regenerate's re-snap did, dropped journeys included.
func journey_resnap_report() -> Array:
	if not _has("journey_resnap_report"):
		return []
	return world_gen.journey_resnap_report()

## The year cursor's day (Ruling AO): 0-based in the 365-day calendar.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func get_civ_day_of_year() -> int:
	if generating or not _has("get_civ_day_of_year"):
		return 0
	return world_gen.get_civ_day_of_year()

## Moves the timeline cursor's day, clamped to 0..=364 (`WorldGen.civ_set_day_of_year`),
## returning the day actually set. `0` also covers no binding; no-op before a world.
func civ_set_day_of_year(day_of_year: int) -> int:
	if not _has("civ_set_day_of_year"):
		return 0
	return world_gen.civ_set_day_of_year(day_of_year)

## `{year, month, day, day_of_year, text}`, `text` as `YYYY-MM-DD`.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func get_civ_date() -> Dictionary:
	if generating or not _has("get_civ_date"):
		return {}
	return world_gen.get_civ_date()


# measure_bridge.rs
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func measure_begin() -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if not _has("measure_begin"):
		return
	world_gen.measure_begin()

## Appends one raw point to the Measure chain (`WorldGen.measure_add_point`); no snapping.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func measure_add_point(gx: float, gy: float) -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if not _has("measure_add_point"):
		return
	world_gen.measure_add_point(gx, gy)

## The current chain's reading (`WorldGen.measure_result`: per-leg cells, km, bearing).
## `{}` without the API.
func measure_result() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not _has("measure_result"):
		return {}
	return world_gen.measure_result()

## Clears the Measure chain without ending the tool (`WorldGen.measure_clear`).
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func measure_clear() -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
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

## Cross-section profile between two grid points with `samples` samples
## (`WorldGen.measure_section`). `{}` when `measure_api` is false.
func measure_section(ax: float, ay: float, bx: float, by: float, samples: int) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not measure_api:
		return {}
	return world_gen.measure_section(ax, ay, bx, by, samples)

## Area reading over a closed ring of grid points (`WorldGen.measure_area`); under
## three vertices gives a zeroed reading. `{}` when `measure_api` is false.
func measure_area(points: PackedVector2Array) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not measure_api:
		return {}
	return world_gen.measure_area(points)

## Radius reading from centre `(cx, cy)` to rim `(px, py)` (`WorldGen.measure_radius`).
## `{}` when `measure_api` is false.
func measure_radius(cx: float, cy: float, px: float, py: float) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not measure_api:
		return {}
	return world_gen.measure_radius(cx, cy, px, py)

## Vertical-delta and 3D-distance reading for two points (`WorldGen.measure_vertical`).
## `{}` when `measure_api` is false.
func measure_vertical(ax: float, ay: float, bx: float, by: float) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not measure_api:
		return {}
	return world_gen.measure_vertical(ax, ay, bx, by)


# region_bridge.rs
func region_set(gx: float, gy: float, gw: float, gh: float) -> void:
	if not _has("region_set"):
		return
	world_gen.region_set(gx, gy, gw, gh)

## The current marquee in cells and km (`WorldGen.region_get`). `{}` means no marquee,
## no world or no API.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func region_get() -> Dictionary:
	if generating or not _has("region_get"):
		return {}
	return world_gen.region_get()

## Clears the marquee (`WorldGen.region_clear`). Does not mark the world dirty.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func region_clear() -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if not _has("region_clear"):
		return
	world_gen.region_clear()

## Exports the marquee as a tile zip (`WorldGen.region_export_tiles(opts)`). An empty
## `PackedByteArray` means nothing was produced, not an empty archive.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers PackedByteArray().
func region_export_tiles(opts: Dictionary) -> PackedByteArray:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return PackedByteArray()
	if not _has("region_export_tiles"):
		return PackedByteArray()
	return world_gen.region_export_tiles(opts)

## The whole world's LOD pyramid under XYZ / TMS / WMTS addressing, zipped
## (`lib.rs::slippy_export_tiles`, whose doc lists the `opts` keys). Empty on an
## older binary, like every wrapper here.
func slippy_export_tiles(opts: Dictionary) -> PackedByteArray:
	if not _has("slippy_export_tiles"):
		return PackedByteArray()
	return world_gen.slippy_export_tiles(opts)

## `Region ▸ New world from selection` (`ops_bridge.rs::region_new_world`, the
## reference's `#regionNewWorldBtn`) -- **replaces the current world** with a
## higher-resolution resample of the Region-select marquee.
##
## Threaded and signalled exactly like `import_heightmap()` above, and for the
## identical reason: it *is* a generate-scale call -- an amplify followed by the
## full substrate inversion, climate, flow and civilisation pipeline. Reusing
## `generation_started`/`generation_finished` means the status bar, the busy
## state and the `generating` guard all work with no extra wiring, and the shell
## can never start one while a generation is in flight.
##
## **This destroys the civilisation layer, labels, icons, hand-drawn ways and
## routes, paint and sculpt drafts, and every knowledge link**, because the
## resampled world is a different grid at a different scale and none of those
## coordinates mean anything over it -- the reference's own confirm() says the
## same in fewer words. A caller must ask first; this does not, for the same
## reason `generate()` does not.
##
## Returns nothing: the result arrives as `generation_finished(ok)`, and
## `last_summary` carries the engine's refusal reason when `ok` is false.
##
## **Primitives across the boundary, deliberately.** The engine-side call runs
## on the worker thread below, and without gdext's `experimental-threads` every
## `Dictionary`/`GString` operation there panics -- the same rule
## `landmark_run()` was rewritten to obey after it shipped a `dict!` reply from
## a thread. So the options go over as four scalars and the refusal reason
## comes back through `region_new_world_error()`, read on the main thread in
## `_finish_region_new_world`. `0`/`0.0` mean "use the engine's default"
## (`tile_size` 1024, the reference's `refSize`; `detail_freq` 1.0;
## `detail_amp` 0.14), which is what an unset caller passes.
func region_new_world(tile_size: int = 0, ridged: bool = false,
		detail_freq: float = 0.0, detail_amp: float = 0.0) -> void:
	if generating or not _has("region_new_world"):
		return
	## Snapshot the parameter table before the worker takes the engine, the
	## same contract `generate()`/`import_heightmap()` hold: from here until
	## `_finish_region_new_world`, `param_get` answers from this cache and
	## nothing reaches a `#[func]` on the borrowed object.
	if _params_available:
		_params_cache = world_gen.get_params()
	busy_kind = "resample"
	generating = true
	_gen_start_msec = Time.get_ticks_msec()
	generation_started.emit()
	_thread = Thread.new()
	_thread.start(_region_new_world_worker.bind(tile_size, ridged, detail_freq, detail_amp))

## Runs off the main thread. Touches only `world_gen`, never a node, and
## constructs no Godot value -- the same contract `_worker`/`_import_worker`
## above hold to, and the reason the engine side returns a bare `bool`.
func _region_new_world_worker(tile_size: int, ridged: bool,
		detail_freq: float, detail_amp: float) -> void:
	var ok: bool = world_gen.region_new_world(tile_size, ridged, detail_freq, detail_amp)
	_finish_region_new_world.call_deferred(ok)

## Main-thread completion of the new-world-from-marquee worker. Joins the thread,
## clears `generating`, and on failure reads `region_new_world_error()` into
## `last_summary`/`note_error`; success emits `generation_finished(true)`, `params_applied`
## and `world_loaded`.
func _finish_region_new_world(ok: bool) -> void:
	_thread.wait_to_finish()
	_thread = null
	generating = false
	last_generate_ms = Time.get_ticks_msec() - _gen_start_msec

	if not ok:
		## The engine's own words -- every refusal it can give names what to
		## change (no marquee, no world, a sub-4-cell aspect, or the finalize
		## lock). Read here rather than returned from the worker, because a
		## `GString` may not be constructed off the main thread. The previous
		## world is untouched and its marquee is still set.
		last_summary = world_gen.region_new_world_error() if _has("region_new_world_error") else ""
		if last_summary.is_empty():
			last_summary = "New world from selection failed -- see console"
		note_error(last_summary)
		generation_finished.emit(false)
		return

	last_width_km = world_gen.get_map_width_km() if sized_api else 0.0
	last_height_km = world_gen.get_map_height_km() if sized_api else 0.0
	has_world = true
	params_dirty = false
	## Names the provenance, because "where did this world come from?" is the
	## first question a resample raises and nothing else on screen answers it
	## -- `_finish_import`'s own reasoning, which names the source file.
	last_summary = "resampled region -- %d x %d cells, %.0f x %.0f km, inferred tectonics" % [
		world_gen.get_width(), world_gen.get_height(),
		last_width_km, last_height_km]
	generation_finished.emit(true)
	params_applied.emit()
	world_loaded.emit()


# geojson_bridge.rs -- GUI_GAP_REGISTER.md DM-03. Empty before the first
# generate()/load, and on a build whose GDExtension predates the binding.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers "".
func export_geojson() -> String:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return ""
	if not _has("export_geojson"):
		return ""
	return world_gen.export_geojson()

## `geojson_apply.rs` via `WorldGen::apply_geojson_document` --
## `LARGE_ITEM_RULINGS.md` Ruling V (2026-09-21), `GUI_GAP_REGISTER.md` DM-03's
## other half. Settlements and territory from `text` are placed into the live
## world; an imported feature naming a faction this world doesn't have
## creates it (never a fuzzy remap, never a silent drop to unclaimed -- the
## ruling's own words). See `geojson_apply.rs`'s own module doc for exactly
## which layers place and which are read but left unapplied.
##
## Always returns the dictionary `WorldGen::apply_geojson_document` builds
## (`"ok"` plus either `"error"` or the placement counts) except when this
## build's GDExtension predates the binding, which is reported the same way
## a parse fault is -- `"ok": false, "error": ...` -- rather than a silent
## no-op, so a caller has exactly one shape to branch on.
func apply_geojson_document(text: String) -> Dictionary:
	if not _has("apply_geojson_document"):
		return {"ok": false,
			"error": "this build's GDExtension predates apply_geojson_document -- rebuild cartalith-godot"}
	var result: Dictionary = world_gen.apply_geojson_document(text)
	if bool(result.get("ok", false)):
		mark_world_dirty()
	return result


# label_bridge/generate.rs -- CARTO's generated labelling pass
# (LARGE_ITEM_RULINGS.md, owner ruling 2026-08-31, all three steps).
#
# None of the four below marks the world dirty except `label_set_class`, and
# the asymmetry is the point: only the class field is written into the project
# archive (`project_bridge.rs`'s LabelDto). The generated run and the class
# typography table are derived from the world and are rebuilt on demand, so
# refreshing them must not make an untouched project look edited.

## The five label classes, their type specs (size/halo/tracking/italic/ink) and
## the three slider domains. Available before any generate() -- it is a design
## table, not world data. `{}` on a cdylib that predates the binding, which
## every caller must read as "draw the panel's own fallback", not as "no
## classes exist".
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func label_class_table() -> Dictionary:
	if generating or not _has("label_class_table"):
		return {}
	return world_gen.label_class_table()

## Run the pass and retain it. `options` is documented on the Rust side; every
## key is optional and an absent one keeps the last run's value.
func labels_generate(options: Dictionary) -> Dictionary:
	if not _has("labels_generate"):
		return {}
	return world_gen.labels_generate(options)

## The last run's per-class counts without re-running it. `[]` before the first
## run -- which the panel draws as `--`, not as `0`.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func labels_generated_counts() -> Array:
	if generating or not _has("labels_generated_counts"):
		return []
	return world_gen.labels_generated_counts()

## Everything the map should draw: the generated run first, hand-placed labels
## over it, each row carrying its class's resolved type spec.
##
## Falls back to `label_list()` rather than to `[]` on an older cdylib: losing
## the generated pass is a degrade, losing the user's own labels is a defect.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func labels_render_list() -> Array:
	if generating or not _has("labels_render_list"):
		return label_list()
	return world_gen.labels_render_list()

## One hand-placed label's class key, or `""` out of range / on an older
## cdylib -- which the panel resolves to the design's own default class.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func label_class_of(index: int) -> String:
	if generating or not _has("label_class_of"):
		return ""
	return world_gen.label_class_of(index)

## Reclass one hand-placed label. Writes a saved field, so it dirties.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func label_set_class(index: int, key: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("label_set_class"):
		return false
	mark_world_dirty()
	return world_gen.label_set_class(index, key)


# label_bridge.rs
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers -1.
func label_create(gx: float, gy: float, text: String) -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return -1
	if not _has("label_create"):
		return -1
	mark_world_dirty()
	return world_gen.label_create(gx, gy, text)

## Sets label `index`'s position directly, unclamped (`WorldGen.label_move`), once per
## drag sample. `false` for a bad index. Marks the world dirty first.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func label_move(index: int, gx: float, gy: float) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("label_move"):
		return false
	mark_world_dirty()
	return world_gen.label_move(index, gx, gy)

## Selects label `index` and snapshots its style fields (`WorldGen.label_select`).
## `false` for a bad index.
func label_select(index: int) -> bool:
	if not _has("label_select"):
		return false
	return world_gen.label_select(index)

## The primary selected label (`WorldGen.label_get_selected`); `-1` means none selected
## (or no binding).
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func label_get_selected() -> int:
	if generating or not _has("label_get_selected"):
		return -1
	return world_gen.label_get_selected()

## Ends the label edit session keeping edits (`WorldGen.label_confirm_edit`). Marks the
## world dirty first; no-op with nothing selected.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func label_confirm_edit() -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if not _has("label_confirm_edit"):
		return
	mark_world_dirty()
	world_gen.label_confirm_edit()

## Reverts the selected label's style fields to the session start (`WorldGen.label_cancel_edit`),
## not its position. `false` if there was no session.
func label_cancel_edit() -> bool:
	if not _has("label_cancel_edit"):
		return false
	return world_gen.label_cancel_edit()

## One label's full state (`WorldGen.label_get`: x, y, text, angle, arc, size, ...).
## `{}` for a bad index or no API.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func label_get(index: int) -> Dictionary:
	if generating or not _has("label_get"):
		return {}
	return world_gen.label_get(index)

## Every hand-placed label in storage order, each `label_get`'s shape plus `index`,
## `selected`, `primary` (`WorldGen.label_list`). `[]` for none or no API.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func label_list() -> Array:
	if generating or not _has("label_list"):
		return []
	return world_gen.label_list()

## Applies a partial dictionary of style fields to label `index` (`WorldGen.label_set`);
## returns the engine's reply, `{}` without the API. Marks the world dirty first.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func label_set(index: int, values: Dictionary) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("label_set"):
		return {}
	mark_world_dirty()
	return world_gen.label_set(index, values)

## Removes label `index` (`WorldGen.label_delete`). `false` for a bad index. Marks the
## world dirty first.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func label_delete(index: int) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("label_delete"):
		return false
	mark_world_dirty()
	return world_gen.label_delete(index)

## Drops every hand-placed label and ends any edit session (`WorldGen.label_clear_all`).
## The generated label run survives. Marks the world dirty first.
func label_clear_all() -> void:
	if not _has("label_clear_all"):
		return
	mark_world_dirty()
	world_gen.label_clear_all()

## `px_per_cell` sizes the hit box off `map_overlay.gd`'s own font model
## (`LARGE_ITEM_RULINGS.md` Ruling AG, 2026-09-23) -- pass
## `ViewportHost.label_px_per_cell()`. `zoom` is the live camera zoom
## (`ViewportHost.zoom()`): a "fixed" label holds a constant on-screen size
## (Ruling AP, 2026-09-23), so its box shrinks by `1 / max(zoom, 0.35)` -- the
## default of `1.0` is only for a caller with no camera.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func label_hit_test(gx: float, gy: float, px_per_cell: float, zoom: float = 1.0) -> int:
	if generating or not _has("label_hit_test"):
		return -1
	return world_gen.label_hit_test(gx, gy, px_per_cell, zoom)

## `label_hit_test` with the modifier the click carried -- see
## `icon_hit_test_mode` for the mode codes and the older-cdylib fallback.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func label_hit_test_mode(gx: float, gy: float, px_per_cell: float, mode: int, zoom: float = 1.0) -> int:
	if generating or not _has("label_hit_test_mode"):
		return label_hit_test(gx, gy, px_per_cell, zoom)
	return world_gen.label_hit_test_mode(gx, gy, px_per_cell, zoom, mode)

## Every label whose box contains `(gx, gy)`, topmost first, **selecting
## nothing** -- `MAP_CONTEXT_SCOPE.md` CM-1's multi-hit pick
## (`label_bridge/pick.rs`). Not `label_hit_test`, which selects its hit and
## answers one. Empty against a cdylib without the binding: that engine has no
## read-only pick, and falling back to the selecting one would move the
## selection on every right-click. `zoom` as on `label_hit_test`.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func label_pick_all(gx: float, gy: float, px_per_cell: float, zoom: float = 1.0) -> PackedInt64Array:
	if generating or not _has("label_pick_all"):
		return PackedInt64Array()
	return world_gen.label_pick_all(gx, gy, px_per_cell, zoom)

## Every selected label's index, ascending. Falls back to whatever
## `label_get_selected()` reports against a cdylib without the set.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func label_get_selection() -> PackedInt64Array:
	if generating or not _has("label_get_selection"):
		var one := label_get_selected()
		return PackedInt64Array([one]) if one >= 0 else PackedInt64Array()
	return world_gen.label_get_selection()

## Replaces the label selection (`WorldGen.label_select_set`). Out-of-range indices are
## dropped and reported as `false`; an empty array deselects.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func label_select_set(indices: PackedInt64Array) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("label_select_set"):
		return false
	return world_gen.label_select_set(indices)

## Selects every label; returns how many. `Edit > Select all`'s engine half.
func label_select_all() -> int:
	if not _has("label_select_all"):
		return 0
	return world_gen.label_select_all()

## `zoom` is the handles' own radius term (`label_bridge::handle_circles`)
## AND, since Ruling AP (2026-09-23), the camera zoom a "fixed" label's box
## shrinks by; `px_per_cell` sizes the box they sit on, off `map_overlay.gd`'s
## own font model (`LARGE_ITEM_RULINGS.md` Ruling AG, 2026-09-23) -- pass
## `ViewportHost.label_px_per_cell()`.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func label_handles(index: int, zoom: float, px_per_cell: float) -> Dictionary:
	if generating or not _has("label_handles"):
		return {}
	return world_gen.label_handles(index, zoom, px_per_cell)

## Per-glyph arc placement for label `index` at `zoom` given measured `char_widths`
## (`WorldGen.label_glyph_layout`). `[]` means no layout, not zero glyphs positioned.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func label_glyph_layout(index: int, zoom: float, char_widths: PackedFloat64Array, total_w: float) -> Array:
	if generating or not _has("label_glyph_layout"):
		return []
	return world_gen.label_glyph_layout(index, zoom, char_widths, total_w)

## The resize handle's live label size during a drag (`WorldGen.label_resize_size`);
## pure math. `0.0` is only the no-binding fallback here.
func label_resize_size(start_size: float, cx: float, cy: float, gx: float, gy: float, start_dist: float) -> float:
	if not _has("label_resize_size"):
		return 0.0
	return world_gen.label_resize_size(start_size, cx, cy, gx, gy, start_dist)

## The rotate handle's absolute label angle in degrees (`WorldGen.label_rotate_deg`);
## `0.0` is also the no-binding fallback.
func label_rotate_deg(cx: float, cy: float, gx: float, gy: float) -> float:
	if not _has("label_rotate_deg"):
		return 0.0
	return world_gen.label_rotate_deg(cx, cy, gx, gy)

## The arc handle's live value during a drag (`WorldGen.label_arc_value`); `0.0` is
## also the no-binding fallback.
func label_arc_value(cx: float, cy: float, grab_angle_deg: float, side: float, gx: float, gy: float) -> float:
	if not _has("label_arc_value"):
		return 0.0
	return world_gen.label_arc_value(cx, cy, grab_angle_deg, side, gx, gy)


# journey_bridge.rs
func jp_options() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not _has("jp_options"):
		return {}
	return world_gen.jp_options()

## The reference's default journey plan as flat keys (`WorldGen.jp_default_plan`), to
## seed a party form. `{}` without the API.
func jp_default_plan() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
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
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not _has("jp_plan_for_route"):
		return {}
	return world_gen.jp_plan_for_route(route_index)

## Plans one journey (`WorldGen.jp_compute(request)`: plan, verdict, confidence).
## `{}` without the API; read the reply's own keys for failure.
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
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func sample_cell(gx: int, gy: int) -> Dictionary:
	if generating or not _has("sample_cell"):
		return {}
	return world_gen.sample_cell(gx, gy)

## The Layers popover's grouped menu, in the reference's own `LAYER_GROUPS`
## order. Each item carries `available`, which is false for a view this
## particular world has no input for.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func debug_layers() -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("debug_layers"):
		return []
	return world_gen.debug_layers()

## One debug view as a grid-sized `Texture2D`. `null` for "off", an unknown
## id, or a view this world has no input for.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers null.
func debug_texture(view: String) -> Texture2D:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return null
	if not _has("build_debug_texture"):
		return null
	return world_gen.build_debug_texture(view)

## The wildlife ecoregion under a click, for the Wildlife view's roster
## popup (the reference's own `showWildInfo`). `{}` when the click missed
## every region marker, when the Wildlife view has no world to read, or on
## an engine build without the binding.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func wildlife_region_at(gx: float, gy: float) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
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
## that cost for this world. `{}` before any generate, on a world opened
## without its substrate (SAVEFILE_COMPAT.md §8.3), and on a world with no
## capital.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_territory_influence() -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
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
## `{}` before any generate, on a world opened without its substrate
## (SAVEFILE_COMPAT.md §8.3), and on a world with no settlements.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_trade_flows() -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_trade_flows"):
		return {}
	return world_gen.civ_trade_flows()

## Every settlement's food-shed capacity -- `_civFoodShed` run once per
## settlement: its own catchment ceiling, the countryside within land reach,
## and genuine spare capacity imported from elsewhere over the cheapest mode
## both ends share (`ECONOMY_SCOPE.md` milestone 2).
##
## **Computed on demand and held nowhere**, the same contract
## `civ_trade_flows()` above ships on -- `CivData` gains no field and
## nothing is saved. `TradeStore` is what keeps the result on the shell
## side, alongside the trade-flow match.
##
## `{}` before any generate, on a world opened without its substrate
## (SAVEFILE_COMPAT.md §8.3), and on a world with no settlements.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_food_shed() -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_food_shed"):
		return {}
	return world_gen.civ_food_shed()

## Every settlement's smelting budget -- `_civPlaceSmelting` run once per
## settlement: the iron a catchment's ore and fuel can jointly sustain, and
## which of the two binds (`ECONOMY_SCOPE.md` EC-2).
##
## **Computed on demand and held nowhere**, the same contract
## `civ_food_shed()` above ships on -- `CivData` gains no field and nothing
## is saved. `TradeStore` is what keeps the result on the shell side,
## alongside the trade-flow match and the food shed.
##
## `{}` before any generate, on a world opened without its substrate
## (SAVEFILE_COMPAT.md §8.3), and on a world with no settlements.
##
## **This wrapper was missing until 2026-09-05**, and its absence was not a
## degrade: `#[func] civ_place_smelting` has been on `WorldGen` since
## 2026-09-02 (`civ_trade_bridge.rs`) and `TradeStore.refresh()` has called
## it through this node since the same day, so every `Match trade flows`
## press aborted with `Nonexistent function 'civ_place_smelting' in base
## 'Node (EngineBridge)'` before it could return. The engine half was never
## the problem -- see `menus.gd::_engine_has()` for why "the bridge has a
## wrapper" and "the cdylib exports the method" are two separate questions,
## and this file only ever answered the second.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_place_smelting() -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_place_smelting"):
		return {}
	return world_gen.civ_place_smelting()

## Every settlement's salt access -- `_civSaltAccess` run once per
## settlement: whether sea salt, a salt deposit or a salt lake is actually in
## reach, and which (`ECONOMY_SCOPE.md` EC-7).
##
## Same held-nowhere contract, same `{}` cases and the same missing-wrapper
## history as `civ_place_smelting()` immediately above -- `TradeStore.refresh()`
## calls the two on consecutive lines, so the abort at the first one is the
## only reason this one was never reached to abort as well.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_salt_access() -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_salt_access"):
		return {}
	return world_gen.civ_salt_access()

## The coordinate frame this world's fields and its GeoJSON export are in
## (`GUI_GAP_REGISTER.md` WW-15). `{}` before any generate.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func world_crs() -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
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
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func ecology_summary() -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
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
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func civ_add_year(year: int) -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if not _has("civ_add_year"):
		return
	mark_world_dirty()
	world_gen.civ_add_year(year)

## `civGotoYear`: moves the active-year cursor. A recorded year loads its
## snapshot's territory and makes it the Territory tool's base (paint and any
## pending stroke dropped); an unrecorded year leaves territory, base and paint
## untouched (Ruling AT, 2026-09-24 -- the reference zeroed it). Never touches
## settlements/ways. A no-op before any generate.
func civ_goto_year(year: int) -> void:
	if not _has("civ_goto_year"):
		return
	world_gen.civ_goto_year(year)

## The recorded year the claim grid last loaded from, as `{"year": int}`, or
## `{}` when it did not come from one (a fresh world, a recompute, a project
## saved before 2026-09-24; a later project reopens with the year it was saved
## holding). An unrecorded year keeps the claims it was reached with, so
## this is the year the timeline strip's "territory holds at ..." names. `{}`
## on a binary without it, and before any generate.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func get_civ_territory_year() -> Dictionary:
	if generating or not _has("get_civ_territory_year"):
		return {}
	return world_gen.get_civ_territory_year()

## `civRemoveYear`: deletes a recorded year. If it was the active year, falls
## back to the earliest remaining one (or year 0 if none remain). A no-op
## before any generate or for a year that was never recorded.
func civ_remove_year(year: int) -> void:
	if not _has("civ_remove_year"):
		return
	mark_world_dirty()
	world_gen.civ_remove_year(year)

## The active timeline cursor (reference `civYear`). `0` before any
## generate/`civ_add_year` call.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func get_civ_year() -> int:
	if generating or not _has("get_civ_year"):
		return 0
	return world_gen.get_civ_year()

## Every recorded timeline year, ascending -- the pill list's own data source.
##
## Answers its empty default while `generating`: the worker holds `WorldGen`, and any
## `#[func]` call then is the `bind() failed` panic (see `param_get`).
func get_civ_timeline_years() -> PackedInt64Array:
	if generating or not _has("get_civ_timeline_years"):
		return PackedInt64Array()
	return world_gen.get_civ_timeline_years()

## Re-derive `CivData::next_tid` from the live settlements/ways AND every
## recorded timeline snapshot's own, returning the new value.
##
## `cartalith_civ::timeline::civ_resync_next_tid_with_timeline` is the
## milestone-4 function; the milestone-1 `civ_resync_next_tid` beside it scans
## only the live arrays, because `TimelineSnapshot` did not exist when it was
## written. That difference is the whole point of binding this one: a reseed
## that ignores the timeline can hand out a tid a historical year already
## holds, and the collision only shows up as two records claiming one id long
## after the year that created it was recorded.
##
## Returns the resynced `next_tid`, or `0` on a binary that has neither
## resync bound -- `0` is not a value the function can legitimately produce
## (it is always `max(tid) + 1`, so at minimum `1`), so a caller can tell
## "unavailable" from "resynced to the first free id".
## **No caller.** Nothing in the shell reseeds `next_tid` by hand: the engine
## does it wherever it hands one out, and the collision this guards against
## needs a timeline snapshot written by a build whose live arrays had already
## moved past it. Kept because that is exactly the repair a Timeline panel
## would offer once one exists, and because deleting it would leave the
## milestone-1 `civ_resync_next_tid` as the only reachable resync -- the one
## that ignores the snapshots and can therefore hand out a tid a recorded year
## already holds.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.
func civ_resync_next_tid_with_timeline() -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0
	if not _has("civ_resync_next_tid_with_timeline"):
		return 0
	mark_world_dirty()
	return int(world_gen.civ_resync_next_tid_with_timeline())

## `_civYearDiff`: `{"present": PackedInt64Array, "removed": PackedInt64Array,
## "added": PackedInt64Array}` of settlement/way tids, diffing `year` against
## the chronologically-previous recorded year -- the ghost/highlight/
## exist-only overlay's own data source. Empty sets (not an error) before any
## generate or for an unrecorded year.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_year_diff(year: int) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_year_diff"):
		return {}
	return world_gen.civ_year_diff(year)

## The settlements `civ_year_diff(year).removed` names, read from the PREVIOUS
## recorded year's snapshot -- the only place a removed settlement's position
## still exists. Rows are `{tid, x, y, name, kind, faction, capital,
## population}`, the same keys `settlements()` uses, so the map can draw a ghost
## with its ordinary pin code. Way tids are dropped (a way is not a pin). Empty
## (not an error) before any generate, for an unrecorded year, for the first
## recorded year, and on a binary older than the `#[func]` -- none of which has
## a removed settlement to show.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func civ_year_diff_removed(year: int) -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("civ_year_diff_removed"):
		return []
	return world_gen.civ_year_diff_removed(year)

## The ways `civ_year_diff(year).removed` names, from the PREVIOUS recorded
## year's snapshot: `{tid, points, brks, way_type, village_addon}` rows in the
## same render geometry `roads()` uses, so the map can trace a ghost along the
## curve the live way was drawn with. A way that was `hidden` (consolidated
## away, never drawn) is never returned. Empty in the same cases as
## `civ_year_diff_removed()`, and on a binary older than the `#[func]`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func civ_year_diff_removed_ways(year: int) -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("civ_year_diff_removed_ways"):
		return []
	return world_gen.civ_year_diff_removed_ways(year)

## The Settlement Editor's "Political history" tab (`lazy-riding-piglet.md`
## Batch C): derived, read-only ownership spans for one settlement's `tid`,
## walking every recorded timeline year. One entry per contiguous span,
## oldest first: `{"start_year": int, "faction_id": int, "current": bool,
## "end_year": int}` -- `end_year` is present only when `current` is
## `false`. Empty before any generate/`civ_add_year` call, for `tid <= 0`,
## or for a `tid` the timeline never recorded.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func civ_settlement_ownership_periods(tid: int) -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("civ_settlement_ownership_periods"):
		return []
	return world_gen.civ_settlement_ownership_periods(tid)

## The Settlement Editor's "Political history" tab, population/tier
## trajectory sub-section (`OUTSTANDING_WORK.md` §2.3 SP-3): derived,
## read-only population/tier readings for one settlement's `tid`, one entry
## per recorded year the settlement was actually present in (a gap year
## contributes no entry -- not carried forward, not zero), oldest first:
## `{"year": int, "pop": int, "kind": String}`, plus `"fortified": bool` and
## `"ruins": bool` only for a year a collapse/recovery run wrote -- absent
## means not recorded, so test with `has()`. `kind` is the same
## vocabulary `get_settlements()` uses
## ("metropolis"/"capital"/"city"/"town"/"village"/"hamlet"). Empty before
## any generate/`civ_add_year` call, for `tid <= 0`, or for a `tid` the
## timeline never recorded.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func civ_settlement_population_trajectory(tid: int) -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("civ_settlement_population_trajectory"):
		return []
	return world_gen.civ_settlement_population_trajectory(tid)

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
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func civ_run_collapse_simulation(request: Dictionary) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("civ_run_collapse_simulation"):
		return {"ok": false, "error": "civ_run_collapse_simulation not available on this binary"}
	mark_world_dirty()
	return world_gen.civ_run_collapse_simulation(request)


# travel_bridge.rs / lib.rs's Travel Library #[func] block (TRAVEL_LIBRARY_SPEC.md,
# GUI_GAP_REGISTER.md DM-15/O1). `kind` is one of "animal"/"vehicle"/"vessel"/"preset"
# throughout. `has_method` guards match every wrapper above: a binary built before this
# landed simply has no `tl_*` methods, and `travel_library_window.gd` falls back to an
# empty library rather than erroring.

## `{kind: {"total": int, "custom": int, "stock": int}}` for all four definition types.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func tl_counts() -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("tl_counts"):
		return {}
	return world_gen.tl_counts()

## Every entry of one definition type, stock-then-custom order, each row carrying
## `id`/`name`/`origin`/`editable`/`subtitle`/`species_key`/`validation_state`/
## `validation_missing`/`validation_conflicts`/`usage_presets`/`usage_journeys`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers [].
func tl_list(kind: String) -> Array:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return []
	if not _has("tl_list"):
		return []
	return world_gen.tl_list(kind)

## One entry's full detail -- `tl_list`'s own per-row keys plus every field
## `TRAVEL_LIBRARY_SPEC.md` §3 lists for `kind`. An unset optional field is simply
## absent from the returned Dictionary -- test `has()`, don't assume a default.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false}.
func tl_get(kind: String, id: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false}
	if not _has("tl_get"):
		return {"ok": false}
	return world_gen.tl_get(kind, id)

## Clones `id` (stock or custom) into a new editable custom entry.
## `{"ok": true, "id": new_id}` or `{"ok": false, "error": ...}`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func tl_duplicate(kind: String, id: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("tl_duplicate"):
		return {"ok": false, "error": "tl_duplicate not available on this binary"}
	mark_world_dirty()
	return world_gen.tl_duplicate(kind, id)

## A brand-new custom entry with every field unset. `{"ok": true, "id": new_id}`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func tl_add_blank(kind: String, name: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("tl_add_blank"):
		return {"ok": false, "error": "tl_add_blank not available on this binary"}
	mark_world_dirty()
	return world_gen.tl_add_blank(kind, name)

## Deletes a custom entry. No-op on an unknown id or a stock one.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false}.
func tl_delete(kind: String, id: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false}
	if not _has("tl_delete"):
		return {"ok": false}
	mark_world_dirty()
	return world_gen.tl_delete(kind, id)

## Discards every custom entry of one kind, restoring the stock-only bootstrap.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false}.
func tl_reset_to_stock(kind: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false}
	if not _has("tl_reset_to_stock"):
		return {"ok": false}
	mark_world_dirty()
	return world_gen.tl_reset_to_stock(kind)

## Applies a partial `fields` Dictionary onto an existing custom entry (stock entries
## are read-only -- duplicate first). Returns `{"ok", "error", "rejected",
## "validation_state", "validation_missing", "validation_conflicts"}`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT, "rejected": []}.
func tl_edit(kind: String, id: String, fields: Dictionary) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT, "rejected": []}
	if not _has("tl_edit"):
		return {"ok": false, "error": "tl_edit not available on this binary", "rejected": []}
	mark_world_dirty()
	return world_gen.tl_edit(kind, id, fields)

## "Capture party from planner": a new custom party preset from `plan`, in
## `jp_default_plan()`/`jp_compute`'s own `plan` key vocabulary.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func tl_capture_preset_from_plan(name: String, plan: Dictionary) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("tl_capture_preset_from_plan"):
		return {"ok": false, "error": "tl_capture_preset_from_plan not available on this binary"}
	mark_world_dirty()
	return world_gen.tl_capture_preset_from_plan(name, plan)


# `asset_bridge.rs` / `lib.rs`'s Asset Library #[func] block (`GUI_GAP_REGISTER.md`
# AS-01..AS-08/AS-13, DM-05). `has_method` guards match every wrapper above: a binary
# built before this landed simply has no `as_*` methods, and `asset_library_window.gd`
# falls back to an empty/disabled state rather than erroring.

## Decode `bytes` as a PNG and add it as a new item on `uid`. `{"ok": true}` or
## `{"ok": false, "error": ...}`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func as_import_item(uid: String, item_name: String, bytes: PackedByteArray) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("as_import_item"):
		return {"ok": false, "error": "as_import_item not available on this binary"}
	mark_world_dirty()
	return world_gen.as_import_item(uid, item_name, bytes)

## AP-1: open a pack `.zip`'s bytes into the library (`WorldGen.as_import_pack`).
## `mode` is "dry_run" (report conflicts, change nothing), "keep" (fill empty
## slots only) or "replace" (the user confirmed). See the Rust doc for the
## returned keys. A dry run does not mark the world dirty; the other modes do.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func as_import_pack(bytes: PackedByteArray, source: String, mode: String) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("as_import_pack"):
		return {"ok": false, "error": "as_import_pack not available on this binary"}
	if mode != "dry_run":
		mark_world_dirty()
	return world_gen.as_import_pack(bytes, source, mode)

## AP-2: the pack report (`WorldGen.as_pack_report`). `{"ok": true, "findings":
## [{kind, severity, label, text, uid?, index?, colour?}]}`; read-only.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func as_pack_report() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("as_pack_report"):
		return {"ok": false, "error": "as_pack_report not available on this binary"}
	return world_gen.as_pack_report()

## The largest pack file `as_import_pack` will read, in bytes, or -1 when this
## binary cannot say (an older build, or mid-generation). The Rust cap is the
## authority; the shell only uses this to avoid reading a larger file first.
func as_pack_archive_cap() -> int:
	# Mid-generation gate: the worker holds `WorldGen`.
	if generating:
		return -1
	if not _has("as_pack_archive_cap"):
		return -1
	return int(world_gen.as_pack_archive_cap())

## Add (or return the existing) custom slot. `{"ok": true, "uid": ...}`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func as_add_custom_slot(slot_name: String, set_name: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("as_add_custom_slot"):
		return {"ok": false, "error": "as_add_custom_slot not available on this binary"}
	mark_world_dirty()
	return world_gen.as_add_custom_slot(slot_name, set_name)

## Every slot in `family_key`'s registry with real fill state -- each row carries
## `uid`/`id`/`name`/`item_count`/`filled`/`has_dupe`. Empty on an older binary.
func as_family_slots(family_key: String) -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("as_family_slots"):
		return []
	return world_gen.as_family_slots(family_key)

## One slot's inspector detail: id/name/family/set, tags, collections, meta fields.
func as_slot_summary(uid: String) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {"ok": false}
	if not _has("as_slot_summary"):
		return {"ok": false}
	return world_gen.as_slot_summary(uid)

## One item's inspector detail: name, scale/pan_x/pan_y, decoded w/h, hash.
func as_item_summary(uid: String, index: int) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {"ok": false}
	if not _has("as_item_summary"):
		return {"ok": false}
	return world_gen.as_item_summary(uid, index)

## A real, baked PNG thumbnail for one stored item. Empty on a miss or an older binary.
func as_thumbnail_png(uid: String, index: int, size: int) -> PackedByteArray:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return PackedByteArray()
	if not _has("as_thumbnail_png"):
		return PackedByteArray()
	return world_gen.as_thumbnail_png(uid, index, size)

## Pack-level metadata and totals: name/author/license/total_items.
func as_pack_info() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {"name": "", "author": "", "license": "", "total_items": 0}
	if not _has("as_pack_info"):
		return {"name": "", "author": "", "license": "", "total_items": 0}
	return world_gen.as_pack_info()

## Sets the pack's name/author/license fields directly.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func as_set_pack_info(pack_name: String, author: String, license: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("as_set_pack_info"):
		return false
	mark_world_dirty()
	return world_gen.as_set_pack_info(pack_name, author, license)

## AP-4: the seam checker (`WorldGen.as_seam_check`) for one ground-texture
## variant's bake. `{"ok": true, "edge", "interior", "seam"}` or
## `{"ok": false, "error"}`; read-only.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func as_seam_check(uid: String, index: int) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("as_seam_check"):
		return {"ok": false, "error": "as_seam_check not available on this binary"}
	return world_gen.as_seam_check(uid, index)

## AP-4: make one ground-texture variant seamless as a NEW variant
## (`WorldGen.as_make_seamless`, Ruling CQ Q7). `{"ok": true, "changed",
## "before_edge", "before_interior", "after_edge", "after_interior", "fixed_x",
## "fixed_y", "index"?}` or `{"ok": false, "error"}`. Marks the world dirty
## only when a variant was added.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func as_make_seamless(uid: String, index: int) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("as_make_seamless"):
		return {"ok": false, "error": "as_make_seamless not available on this binary"}
	var r: Dictionary = world_gen.as_make_seamless(uid, index)
	if bool(r.get("changed", false)):
		mark_world_dirty()
	return r

## Removes one item from a slot.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func as_remove_item(uid: String, index: int) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("as_remove_item"):
		return false
	mark_world_dirty()
	return world_gen.as_remove_item(uid, index)

## Resets the whole session to a fresh, empty library.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func as_clear_library() -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("as_clear_library"):
		return false
	mark_world_dirty()
	return world_gen.as_clear_library()

## `AssetValidator.run()`'s real, ordered warning strings.
func as_validate() -> PackedStringArray:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return PackedStringArray()
	if not _has("as_validate"):
		return PackedStringArray()
	return world_gen.as_validate()

## Bakes every stored item and writes the pack `.zip` bytes.
## `{"ok": true, "name": ..., "bytes": PackedByteArray}` or `{"ok": false, "error": ...}`.
func as_export_pack_bytes() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("as_export_pack_bytes"):
		return {"ok": false, "error": "as_export_pack_bytes not available on this binary"}
	return world_gen.as_export_pack_bytes()

## Compiles the current session into a pack and loads it straight into the renderer
## -- the reference's own `applyToMap()`, same bake `as_export_pack_bytes` does.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func as_apply_to_map() -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("as_apply_to_map"):
		return {"ok": false, "error": "as_apply_to_map not available on this binary"}
	mark_world_dirty()
	var result: Dictionary = world_gen.as_apply_to_map()
	if bool(result.get("ok", false)):
		world_loaded.emit()
	return result

## Comma-separated `tags_csv` onto every uid in `uids`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false}.
func as_batch_tag(uids: PackedStringArray, tags_csv: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false}
	if not _has("as_batch_tag"):
		return {"ok": false}
	mark_world_dirty()
	return world_gen.as_batch_tag(uids, tags_csv)

## Adds every uid in `uids` to collection `coll_name`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false}.
func as_batch_collect(uids: PackedStringArray, coll_name: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false}
	if not _has("as_batch_collect"):
		return {"ok": false}
	mark_world_dirty()
	return world_gen.as_batch_collect(uids, coll_name)

## `{base}_01`, `{base}_02`, ... over `uids` in order. `remap` carries
## `old_uid -> new_uid` for every custom slot whose uid changed.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "renamed": 0, "remap": {}}.
func as_batch_rename(uids: PackedStringArray, base: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "renamed": 0, "remap": {}}
	if not _has("as_batch_rename"):
		return {"ok": false, "renamed": 0, "remap": {}}
	mark_world_dirty()
	return world_gen.as_batch_rename(uids, base)

## Clones every slot in `uids` carrying at least one item into a new custom slot.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "made": 0}.
func as_batch_duplicate(uids: PackedStringArray) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "made": 0}
	if not _has("as_batch_duplicate"):
		return {"ok": false, "made": 0}
	mark_world_dirty()
	return world_gen.as_batch_duplicate(uids)

## Custom slots in `uids` are removed entirely; frozen slots have their items cleared.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "deleted": 0}.
func as_batch_delete(uids: PackedStringArray) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "deleted": 0}
	if not _has("as_batch_delete"):
		return {"ok": false, "deleted": 0}
	mark_world_dirty()
	return world_gen.as_batch_delete(uids)

## Decodes a sprite sheet and holds it on the session for slicing (AS-09).
## `{"ok": true, "w", "h", "name"}` or `{"ok": false, "error": ...}`. PNG only.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT, "w": 0, "h": 0}.
func as_load_sheet(sheet_name: String, bytes: PackedByteArray) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT, "w": 0, "h": 0}
	if not _has("as_load_sheet"):
		return {"ok": false, "error": "as_load_sheet not available on this binary", "w": 0, "h": 0}
	return world_gen.as_load_sheet(sheet_name, bytes)

## Drops the loaded sheet (the slicer modal closing).
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func as_clear_sheet() -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("as_clear_sheet"):
		return false
	return world_gen.as_clear_sheet()

## The real `N cells detected · M non-empty` pass plus the overlay's grid lines
## (AS-09). `{"ok", "total", "non_empty", "usable", "col_x0"/"col_x1"/"row_y0"/
## "row_y1"}` -- the four span arrays are in sheet pixels, engine-computed, so
## the overlay draws exactly the cells `as_slice_apply` will cut.
func as_slice_preview(opts: Dictionary) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {"ok": false, "error": "as_slice_preview not available on this binary"}
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
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers its unavailable default.
func as_slice_apply(opts: Dictionary) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT, "added": 0, "skipped_blank": 0, "unplaced": 0, "uids": PackedStringArray()}
	if not _has("as_slice_apply"):
		return {"ok": false, "error": "as_slice_apply not available on this binary",
			"added": 0, "skipped_blank": 0, "unplaced": 0, "uids": PackedStringArray()}
	mark_world_dirty()
	return world_gen.as_slice_apply(opts)

## AS-07: writes one item's scale/pan directly. `false` for an unknown
## uid/index or an older binary.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func as_set_item_transform(uid: String, index: int, scale: float, pan_x: float, pan_y: float) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("as_set_item_transform"):
		return false
	mark_world_dirty()
	return world_gen.as_set_item_transform(uid, index, scale, pan_x, pan_y)

## AS-07: resets one item's transform to identity, re-fitting to the slot's
## family when `fit` is true. `{"ok", "scale", "pan_x", "pan_y"}` or
## `{"ok": false}` for an unknown uid/index or an older binary.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false}.
func as_reset_item_transform(uid: String, index: int, fit: bool) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false}
	if not _has("as_reset_item_transform"):
		return {"ok": false}
	mark_world_dirty()
	return world_gen.as_reset_item_transform(uid, index, fit)

## AS-17: moves interior line `index` of `lines` to `frac`, clamped strictly
## between its neighbours -- `lines` unchanged on an older binary.
func as_slicer_move_line(lines: PackedFloat64Array, index: int, frac: float) -> PackedFloat64Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return lines
	if not _has("as_slicer_move_line"):
		return lines
	return world_gen.as_slicer_move_line(lines, index, frac)

## AS-17: the uniform `n+1`-line array a fresh grid (or a cols/rows edit)
## falls back to. Empty on an older binary.
func as_uniform_lines(n: int) -> PackedFloat64Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return PackedFloat64Array()
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

## Failure Dictionary (`ok: false` plus an error string) that the vault wrappers return when the
## build has no Markdown Vault support, so callers need no second branch (see the section comment
## above).
const _VAULT_UNAVAILABLE := {
	"ok": false,
	"error": "This engine build has no Markdown Vault support — rebuild the GDExtension.",
}

## Addressable landmasses, largest first (`get_continents()`).
## `id` is a **rank by area**, not a persistent identity — see the engine's
## own doc comment on that binding before storing one anywhere.
func continents() -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("get_continents"):
		return []
	return world_gen.get_continents()

## Binds `path` as this device's Markdown Vault (`WorldGen.vault_connect`); reply
## `{ok, error, ...}`. Returns `_VAULT_UNAVAILABLE` without the API. Marks the world dirty.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func vault_connect(path: String, display_name: String = "") -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("vault_connect"):
		return _VAULT_UNAVAILABLE
	mark_world_dirty()
	return world_gen.vault_connect(path, display_name)

## Drops the device-local vault binding (`WorldGen.vault_disconnect`); links survive.
## No-op without the API.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func vault_disconnect() -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if _has("vault_disconnect"):
		mark_world_dirty()
		world_gen.vault_disconnect()

## `MARKDOWN_VAULT_SCOPE.md` milestone 4 -- the Android Storage Access
## Framework half of `vault_connect`. `tree_uri` is a `content://` document
## tree Android has already granted (this call binds it, it does not request
## or persist the grant); `dispatch` is `Callable(handler, "_saf_dispatch")`
## for a GDScript object speaking the op/args/result contract
## `crates/cartalith-godot/src/vault_saf.rs`'s own module doc specifies in
## full -- read that before writing `handler`, since it is also where the
## real Android work this call cannot do on its own is named plainly.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func vault_connect_saf(tree_uri: String, display_name: String, dispatch: Callable) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("vault_connect_saf"):
		return _VAULT_UNAVAILABLE
	mark_world_dirty()
	return world_gen.vault_connect_saf(tree_uri, display_name, dispatch)

## `{bound, root, display_name, vault_id, link_count}` (`WorldGen.vault_info`). The
## no-API fallback reports `bound: false` and `link_count: 0`.
func vault_info() -> Dictionary:
	if not _has("vault_info"):
		return {"bound": false, "root": "", "display_name": "", "vault_id": "", "link_count": 0}
	## Mid-generation (2026-10-12): the worker holds `WorldGen`. Serves the last answer the
	## engine gave -- the vault binding is not a thing a generate changes -- rather than a
	## fabricated `bound: false`, which would tell the vault windows the user's vault vanished.
	if generating:
		return _vault_info_seen.duplicate()
	_vault_info_seen = world_gen.vault_info()
	return _vault_info_seen.duplicate()

## The last `vault_info()` the engine answered, served while `generating`. Empty before any
## read, which callers already read as "unbound" via `.get("bound", false)`.
var _vault_info_seen: Dictionary = {}

## Markdown files in the bound vault, sorted, capped at `limit` (`WorldGen.vault_list_files`).
## Empty for an unbound vault or no API.
func vault_list_files(limit: int = 2000) -> PackedStringArray:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return PackedStringArray()
	if not _has("vault_list_files"):
		return PackedStringArray()
	return world_gen.vault_list_files(limit)

## The templates in the connected vault (`GUI_GAP_REGISTER.md` VA-02), each
## `{rel, label}`. Ruling BF: the files under the folder Obsidian's Templates
## plugin is set to (`.obsidian/templates.json`), else Templater's; only when
## neither is set, every `.md` with "template" in its path. Cartalith ships
## none of its own.
func vault_templates() -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("vault_templates"):
		return []
	return world_gen.vault_templates()

## Where `vault_templates()` looked: `{ok, source, describe}` and `folder` when
## a setting named one (absent for the fallback). `source` is `obsidian`,
## `templater` or `fallback`.
func vault_template_source() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {"ok": false, "error": "this engine build predates Ruling BF"}
	if not _has("vault_template_source"):
		return {"ok": false, "error": "this engine build predates Ruling BF"}
	return world_gen.vault_template_source()

## Obsidian's Insert template: `{ok, text, caret}` -- `note_text` with the
## filled template inserted at `caret` (characters) and its properties merged
## into the note's. Writes nothing.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func vault_insert_template(template_rel: String, note_text: String, caret: int, title: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("vault_insert_template"):
		return {"ok": false, "error": "this engine build has no Insert template"}
	return world_gen.vault_insert_template(template_rel, note_text, caret, title)

## Where a new note for this entity goes -- v3's `Settlements/{name}.md`
## convention, generalised to every kind. A suggestion, not a rule.
func vault_suggested_path(kind: String, name: String) -> String:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return ""
	if not _has("vault_suggested_path"):
		return ""
	return world_gen.vault_suggested_path(kind, name)

## Creates a note from a template with Obsidian's placeholders filled
## (`{{title}}` = the new file's name, `{{date}}`/`{{time}}` = now, in the
## vault's formats) and `name` in the template's `{{…Name}}`/`[Name]` tokens;
## Templater's `<% %>` is left as written. Refuses an existing path; never
## overwrites. `{ok, path, text}` or `{ok: false, error}`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func vault_create_from_template(template_rel: String, rel: String, name: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("vault_create_from_template"):
		return {"ok": false, "error": "this engine build has no note creator"}
	return world_gen.vault_create_from_template(template_rel, rel, name)

## Headings of file `rel` as `{level, title}` (`WorldGen.vault_file_headings`). `[]`
## for an unreadable file, which callers read as "whole document only".
func vault_file_headings(rel: String) -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("vault_file_headings"):
		return []
	return world_gen.vault_file_headings(rel)

## Raw text of vault file `rel` (`WorldGen.vault_read_file`). `""` means unreadable
## or no API, not necessarily an empty file.
func vault_read_file(rel: String) -> String:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return ""
	if not _has("vault_read_file"):
		return ""
	return world_gen.vault_read_file(rel)

## The browse panel's edit path: a whole file's raw text plus the hash it was
## read at (`{ok, text, hash}`), for a note picked but not attached to any
## entity. `hash` must be handed back to `vault_write_file` unchanged.
func vault_read_file_for_edit(rel: String) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return _VAULT_UNAVAILABLE
	if not _has("vault_read_file_for_edit"):
		return _VAULT_UNAVAILABLE
	return world_gen.vault_read_file_for_edit(rel)

## Writes a whole file with no attached link -- `vault_write_section`'s
## simpler cousin. Refuses with the file unchanged if it was edited since
## `expect_hash` was read. `{ok, hash}` on success, `{ok: false, error}`
## otherwise.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func vault_write_file(rel: String, text: String, expect_hash: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("vault_write_file"):
		return _VAULT_UNAVAILABLE
	mark_world_dirty()
	return world_gen.vault_write_file(rel, text, expect_hash)

## Attaches an entity to a document or heading (`WorldGen.vault_attach`); reply
## `{ok, error, ...}`. `_VAULT_UNAVAILABLE` without the API; marks the world dirty.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func vault_attach(kind: String, entity_id: int, label: String, rel: String, heading: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("vault_attach"):
		return _VAULT_UNAVAILABLE
	mark_world_dirty()
	return world_gen.vault_attach(kind, entity_id, label, rel, heading)

## Removes link `link_id` (`WorldGen.vault_detach`). `false` for an unknown id. Marks
## the world dirty first.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func vault_detach(link_id: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("vault_detach"):
		return false
	mark_world_dirty()
	return world_gen.vault_detach(link_id)

## Every link on one entity, each with live status (`WorldGen.vault_links_for`).
## `[]` for none or no API.
func vault_links_for(kind: String, entity_id: int) -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("vault_links_for"):
		return []
	return world_gen.vault_links_for(kind, entity_id)

## Every link in the store for the vault window overview (`WorldGen.vault_all_links`).
## `[]` for none or no API.
func vault_all_links() -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("vault_all_links"):
		return []
	return world_gen.vault_all_links()

## `{link_count, status}` for one entity (`WorldGen.vault_entity_summary`); `status`
## is the worst link status, `""` when none. No-API fallback is `link_count: 0`.
func vault_entity_summary(kind: String, entity_id: int) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {"link_count": 0, "status": ""}
	if not _has("vault_entity_summary"):
		return {"link_count": 0, "status": ""}
	return world_gen.vault_entity_summary(kind, entity_id)

## The Cartalith-side working text of link `link_id` (`WorldGen.vault_link_text`).
## `""` for an unknown link or no API.
func vault_link_text(link_id: String) -> String:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return ""
	if not _has("vault_link_text"):
		return ""
	return world_gen.vault_link_text(link_id)

## Records an edit to link `link_id`'s working copy (`WorldGen.vault_set_link_text`).
## `false` for an unknown link. Marks the world dirty first.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func vault_set_link_text(link_id: String, text: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("vault_set_link_text"):
		return false
	mark_world_dirty()
	return world_gen.vault_set_link_text(link_id, text)

## Re-reads the link's source selection, discarding the local copy
## (`WorldGen.vault_reload_link`). `_VAULT_UNAVAILABLE` without the API. Not marked dirty.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func vault_reload_link(link_id: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("vault_reload_link"):
		return _VAULT_UNAVAILABLE
	return world_gen.vault_reload_link(link_id)

## Preview and source `hash` for writing link `link_id` back (`WorldGen.vault_preview_section_write`);
## hand the hash to `vault_write_section`. Writes nothing.
func vault_preview_section_write(link_id: String) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return _VAULT_UNAVAILABLE
	if not _has("vault_preview_section_write"):
		return _VAULT_UNAVAILABLE
	return world_gen.vault_preview_section_write(link_id)

## Writes one edited section back to the source file (`WorldGen.vault_write_section`);
## refused if `expect_hash` no longer matches. Reply `{ok, error, ...}`; marks the world dirty.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func vault_write_section(link_id: String, expect_hash: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("vault_write_section"):
		return _VAULT_UNAVAILABLE
	mark_world_dirty()
	return world_gen.vault_write_section(link_id, expect_hash)

## Every entity kind this build can address in a vault, in the order the
## docks list them. `faction` and `culture` both joined
## `settlement`/`province`/`continent` on 2026-08-25 (`GUI_GAP_REGISTER.md`
## CV-22 and CV-02); `poi` is deliberately absent and will stay absent while
## this port has no point-of-interest entity (CV-01).
##
## **Build every kind list from this, never from a literal.** The fallback
## below carried the pre-2026-08-25 three for months after the engine grew
## five, so a transcribed copy of it was wrong in two places at once -- which
## is the whole reason `vault_bridge.rs` exports the list at all.
func vault_entity_kinds() -> PackedStringArray:
	# Mid-generation gate: answered from the constant list (needs no `WorldGen`).
	if generating:
		return PackedStringArray(["settlement", "province", "continent", "faction", "culture"])
	if not _has("vault_entity_kinds"):
		## Matches `vault_bridge.rs::vault_entity_kinds` exactly. A binary old
		## enough to be missing the binding may also reject `faction` and
		## `culture` at `vault_attach`; that call answers for itself, and
		## under-reporting the list here would hide two kinds from a binary
		## that does support them.
		return PackedStringArray(["settlement", "province", "continent", "faction", "culture"])
	return world_gen.vault_entity_kinds()

## Fields offerable for one entity as `{key, group, label}` (`WorldGen.vault_export_fields`),
## filtered to those with a value. `[]` for none or no API.
func vault_export_fields(kind: String, entity_id: int) -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("vault_export_fields"):
		return []
	return world_gen.vault_export_fields(kind, entity_id)

## Every value held for one entity keyed by export-field key (`WorldGen.vault_entity_values`).
## `{}` for none or no API.
func vault_entity_values(kind: String, entity_id: int) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not _has("vault_entity_values"):
		return {}
	return world_gen.vault_entity_values(kind, entity_id)

## Markdown body of the Cartalith block for one entity and checkbox selection
## (`WorldGen.vault_block_body`); pure. `""` also means no API.
func vault_block_body(kind: String, entity_id: int, selected: PackedStringArray) -> String:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return ""
	if not _has("vault_block_body"):
		return ""
	return world_gen.vault_block_body(kind, entity_id, selected)

## §21's three radii, and whether each has been generated for this entity.
## `[]` on an older binary, which is what the panel checks before drawing the
## Map section at all — an empty list is "this build cannot make snapshots",
## not "this world has none".
func vault_snapshot_radii(kind: String, entity_id: int) -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("vault_snapshot_radii"):
		return []
	return world_gen.vault_snapshot_radii(kind, entity_id)

## Renders one entity's map snapshot into `subdir` inside the bound vault and
## files it on the link store (`MARKDOWN_VAULT_SCOPE.md` milestone 2).
##
## `mark_world_dirty()` for the same reason the two block writers above have
## it: the snapshot's path is part of the link store, and the link store is
## part of the project archive (`vault.json`).
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func vault_snapshot(kind: String, entity_id: int, radius: String, subdir: String, size: int) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("vault_snapshot"):
		return _VAULT_UNAVAILABLE
	mark_world_dirty()
	return world_gen.vault_snapshot(kind, entity_id, radius, subdir, size)

## Previews inserting or replacing the machine-owned block in file `rel`
## (`WorldGen.vault_preview_block`): `{ok, error, preview, hash, action, ...}`. Writes nothing.
func vault_preview_block(rel: String, kind: String, entity_id: int, body: String) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return _VAULT_UNAVAILABLE
	if not _has("vault_preview_block"):
		return _VAULT_UNAVAILABLE
	return world_gen.vault_preview_block(rel, kind, entity_id, body)

## Writes the machine-owned block into file `rel` (`WorldGen.vault_write_block`);
## refused if `expect_hash` is stale. Reply `{ok, error, ...}`; marks the world dirty.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func vault_write_block(rel: String, kind: String, entity_id: int, body: String, expect_hash: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("vault_write_block"):
		return _VAULT_UNAVAILABLE
	mark_world_dirty()
	return world_gen.vault_write_block(rel, kind, entity_id, body, expect_hash)

## Previews filling the author's own template fields (`WorldGen.vault_preview_field_fill`);
## reply carries per-field outcomes. Writes nothing.
func vault_preview_field_fill(rel: String, kind: String, entity_id: int, overwrite: bool) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return _VAULT_UNAVAILABLE
	if not _has("vault_preview_field_fill"):
		return _VAULT_UNAVAILABLE
	return world_gen.vault_preview_field_fill(rel, kind, entity_id, overwrite)

## Fills the author's template fields in file `rel` (`WorldGen.vault_write_field_fill`);
## refused if `expect_hash` is stale. Reply `{ok, error, ...}`; marks the world dirty.
func vault_write_field_fill(rel: String, kind: String, entity_id: int, overwrite: bool, expect_hash: String) -> Dictionary:
	if not _has("vault_write_field_fill"):
		return _VAULT_UNAVAILABLE
	mark_world_dirty()
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
##
## **`[]` while `generating`** (arming-panics sweep 2026-10-11): the ledger is on the `WorldGen`
## the worker holds, so a real read is the `bind() failed` panic; the History panel redraws on
## the run's finish signals.
func undo_ledger() -> Array:
	if generating or not _has("undo_ledger"):
		return []
	return world_gen.undo_ledger()

## Roll back to a ledger row, popping every height snapshot above it as well.
## Returns how many steps were reverted; `0` when the row is gone or was never
## reversible, in which case the caller should re-read `undo_ledger()`.
## **`0` while `generating`** (arming-panics sweep 2026-10-11, the write twin of `undo_ledger`'s
## guard: nothing was reverted, the world is not marked dirty).
func undo_revert_to(seq: int) -> int:
	if generating or not _has("undo_revert_to"):
		return 0
	return world_gen.undo_revert_to(seq)


## Refreshes the backlink index, the only operation that scans the vault
## (`WorldGen.vault_refresh_backlinks`, capped at `limit`). Reply `{ok, error, ...}`.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func vault_refresh_backlinks(limit: int = 2000) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("vault_refresh_backlinks"):
		return {"ok": false, "error": "this build predates the backlink index"}
	return world_gen.vault_refresh_backlinks(limit)

## Throw the index away so the next refresh re-reads everything.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): does nothing.
func vault_rebuild_backlinks() -> void:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return
	if _has("vault_rebuild_backlinks"):
		world_gen.vault_rebuild_backlinks()

## `{built, notes, links, entities, broken, orphans, bytes, refreshed_at}`.
## `built` is the one every reader must branch on: "no notes" and "nothing
## indexed" are opposite statements on screen.
func vault_backlink_stats() -> Dictionary:
	if not _has("vault_backlink_stats"):
		return {"built": false}
	## Mid-generation (2026-10-12): serves the last stats the engine gave (see `vault_info`);
	## `{"built": false}` before any read, the same "nothing indexed" the no-binding case says.
	if generating:
		return _vault_backlink_stats_seen.duplicate()
	_vault_backlink_stats_seen = world_gen.vault_backlink_stats()
	return _vault_backlink_stats_seen.duplicate()

## The last `vault_backlink_stats()` the engine answered, served while `generating`.
var _vault_backlink_stats_seen: Dictionary = {"built": false}

## Every note that references this entity -- `{rel, form, count}`, `form` one
## of `wiki`, `markdown`, `block`. A `block` row is a note carrying this
## entity's own Cartalith block, which finds the entity even when it has no
## note of its own.
func vault_entity_backlinks(kind: String, entity_id: int) -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("vault_entity_backlinks"):
		return []
	return world_gen.vault_entity_backlinks(kind, entity_id)

## Notes that name this entity in prose and do not link to it --
## `{rel, excerpt}`. A guess, and the panel draws it as one.
func vault_entity_mentions(kind: String, entity_id: int, name: String, max_rows: int = 12) -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("vault_entity_mentions"):
		return []
	return world_gen.vault_entity_mentions(kind, entity_id, name, max_rows)

## Every note that links to `rel` directly -- `{rel, form, count}` -- for a
## file being browsed rather than attached to an entity, where
## `vault_entity_backlinks` above has no `(kind, entity_id)` to look up.
func vault_file_backlinks(rel: String) -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("vault_file_backlinks"):
		return []
	return world_gen.vault_file_backlinks(rel)

## Notes that name this file's own title in prose and do not link to it --
## `{rel, excerpt}` -- the path-keyed analogue of `vault_entity_mentions`.
func vault_file_mentions(rel: String, max_rows: int = 12) -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("vault_file_mentions"):
		return []
	return world_gen.vault_file_mentions(rel, max_rows)

## `{built, broken: [{source, target}], orphans: [rel]}` -- both halves of
## Data ▸ Missing & orphan notes report…, from the one index.
func vault_backlink_report(limit: int = 200) -> Dictionary:
	## Mid-generation (2026-10-12): `{built: false}` is this report's own "nothing indexed"
	## answer, and the only caller (`vault_window`) shows it as such rather than as data.
	if generating or not _has("vault_backlink_report"):
		return {"built": false, "broken": [], "orphans": PackedStringArray()}
	return world_gen.vault_backlink_report(limit)

## The index as JSON. `VaultStore` writes it beside the link store -- and
## separately from it, because the store is portable project data and this is
## a cache of somebody's folder.
func vault_backlink_index_json() -> String:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return ""
	if not _has("vault_backlink_index_json"):
		return ""
	return world_gen.vault_backlink_index_json()

## Restores a saved backlink index from JSON (`WorldGen.vault_restore_backlink_index`).
## `false` on malformed JSON, leaving the in-memory index alone.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func vault_restore_backlink_index(json: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("vault_restore_backlink_index"):
		return false
	return world_gen.vault_restore_backlink_index(json)


## The whole vault link store as JSON for the shell to persist (`WorldGen.vault_state_json`).
## `""` means unavailable, not an empty store.
func vault_state_json() -> String:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return ""
	if not _has("vault_state_json"):
		return ""
	return world_gen.vault_state_json()

## Restores a link store from JSON (`WorldGen.vault_restore_state`), keeping the current
## binding. `false` on malformed JSON, changing nothing.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func vault_restore_state(json: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
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
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {"ok": false, "error": "vault_search not available on this binary", "hits": []}
	if not _has("vault_search"):
		return {"ok": false, "error": "vault_search not available on this binary", "hits": []}
	return world_gen.vault_search(query, limit, max_reads)

## One note's frontmatter and template fields, read from disk right now and
## stored nowhere -- what a search result or an attach dialog shows *before*
## the user commits to anything.
func vault_file_data(rel: String) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {"ok": false, "error": "vault_file_data not available on this binary"}
	if not _has("vault_file_data"):
		return {"ok": false, "error": "vault_file_data not available on this binary"}
	return world_gen.vault_file_data(rel)

## Every linked note's imported data for one entity, one Dictionary per link.
func vault_entity_data(kind: String, entity_id: int) -> Array:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return []
	if not _has("vault_entity_data"):
		return []
	return world_gen.vault_entity_data(kind, entity_id)

## SP-3 authored events (Ruling AM): every ` ```chronos ` block in the notes
## attached to this entity, parsed. `{ok, error, notes: [{rel, live, blocks,
## skipped}], events: [{rel, kind, start, end?, color?, group?, name,
## description?}]}` -- `events` sorted by start year; absent keys are absent,
## never zero. `live` is false when the note was read from the saved copy
## because the vault is not connected.
func vault_entity_chronos(kind: String, entity_id: int) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {"ok": false, "error": "vault_entity_chronos not available on this binary"}
	if not _has("vault_entity_chronos"):
		return {"ok": false, "error": "vault_entity_chronos not available on this binary"}
	return world_gen.vault_entity_chronos(kind, entity_id)

## Writes one authored event into the note `rel` as a Chronos line (Ruling AM),
## appended to its ```chronos block (created if absent). `event` uses
## `vault_entity_chronos`'s own event keys -- `start`, `name`, and optional
## `end`/`color`/`group`/`description`, absent when not given. `expect_hash`
## is `vault_read_file_for_edit(rel).hash`. `{ok, error, hash, conflict}`:
## `conflict` is true when the note changed since that read, and nothing was
## written.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func vault_add_chronos_event(rel: String, expect_hash: String, event: Dictionary) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("vault_add_chronos_event"):
		return {"ok": false, "error": "vault_add_chronos_event not available on this binary"}
	return world_gen.vault_add_chronos_event(rel, expect_hash, event)

## One link's imported data.
func vault_link_data(link_id: String) -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {"ok": false, "error": "vault_link_data not available on this binary"}
	if not _has("vault_link_data"):
		return {"ok": false, "error": "vault_link_data not available on this binary"}
	return world_gen.vault_link_data(link_id)

## Which write confirmations the user has switched off: `{section, block,
## field_fill}`, all bool. These suppress the DIALOG, never the guard -- a
## caller with a preference set must still call the matching `vault_preview_*`
## (that is where `expect_hash` comes from) and simply not show it.
func vault_write_prefs() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not _has("vault_write_prefs"):
		return {}
	return world_gen.vault_write_prefs()

## Sets one of them. `path` is "section", "block" or "field_fill"; anything
## else returns false and changes nothing, so a typo cannot quietly disarm a
## confirmation.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func vault_set_write_pref(path: String, value: bool) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("vault_set_write_pref"):
		return false
	mark_world_dirty()
	return world_gen.vault_set_write_pref(path, value)

## The preferences as JSON text, for the save file. Text, not a Dictionary --
## Godot's JSON floats every integer on the way through (GUI_GAP_REGISTER.md
## KV-04), so engine JSON travels as a string in both directions.
func vault_prefs_json() -> String:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return ""
	if not _has("vault_prefs_json"):
		return ""
	return world_gen.vault_prefs_json()

## Restores vault preferences from JSON (`WorldGen.vault_restore_prefs`). `false` on
## malformed JSON, leaving the safe ask-every-time defaults.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func vault_restore_prefs(json: String) -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("vault_restore_prefs"):
		return false
	return world_gen.vault_restore_prefs(json)

## Removes a Cartalith-written block from a note. `expect_hash` comes from the
## matching preview and is what makes a note edited in between refuse instead
## of being overwritten.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func vault_remove_block(rel: String, kind: String, entity_id: int, expect_hash: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("vault_remove_block"):
		return {"ok": false, "error": "vault_remove_block not available on this binary"}
	mark_world_dirty()
	return world_gen.vault_remove_block(rel, kind, entity_id, expect_hash)

# -- F10 · project documents -------------------------------------------------

## `project_save` plus the caller's own documents: `{slot: json_text}` over the
## slots `project_document_slots()` lists. Text rather than a Dictionary for
## the same KV-04 reason as `vault_prefs_json` above.
##
## **`save_project()` is still the entry point File ▸ Save uses**; this one
## exists for the single caller that must write the same archive *without*
## the bookkeeping `save_project()` performs -- `DccApp._autosave_tick()`,
## which must not clear the dirty flag and must not emit `project_saved`,
## because the project the user chose to keep is still unwritten.
##
## An older binary with no `project_save_with_documents` degrades to
## `project_save` rather than refusing the write outright: an autosave that
## carries the world but not the drafts still beats no autosave. It says so,
## because silently dropping documents is exactly the failure this pair of
## functions was added to end.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func project_save_with_documents(path: String, extra_documents: Dictionary) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("project_save_with_documents"):
		if not _has("project_save"):
			return {"ok": false, "error": "project_save_with_documents not available on this binary"}
		if not extra_documents.is_empty():
			push_warning("Cartalith: this binary has no project_save_with_documents -- %d caller-owned document(s) were not written to %s"
				% [extra_documents.size(), path])
		return world_gen.project_save(path)
	return world_gen.project_save_with_documents(path, extra_documents)

## Every document the ENGINE can build for a caller-owned slot, keyed by slot
## name (`project_bridge.rs::project_engine_built_documents`): the paint
## layers, the sculpt draft, the Asset Library and the Travel Library. Merge
## it into whatever the shell owns and hand the union to `save_project()`;
## the two sets never collide, because none of these four is a slot GDScript
## writes.
##
## A slot with nothing to write is **absent**, not empty -- that side's own
## contract, and the reason this returns a Dictionary to merge rather than
## four strings to test.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func project_engine_built_documents() -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("project_engine_built_documents"):
		return {}
	return world_gen.project_engine_built_documents()

## Puts a `drafts/sculpt.json` back into the live Sculpt editor. Returns
## `{ok, error, stamps}`.
##
## **Not the call the load path makes.** This used to carry a note saying to
## expect `ok == false` after opening a project, because the engine dropped
## the Sculpt editor on every load and the draft was never re-applied. That
## was the defect, and `project_open` now restores the archive's own draft
## itself, building the editor to hold it. What is left for this function is
## the *import* case: a draft read out of a different file with
## `project_read_document` and applied to the world already on screen, which
## is why the grid mismatch is still a refusal rather than a resize.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func sculpt_restore_document(text: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("sculpt_restore_document"):
		return {"ok": false, "error": "sculpt_restore_document not available on this binary"}
	var restored: Dictionary = world_gen.sculpt_restore_document(text)
	sculpt_draft_changed.emit()
	return restored

## Puts a `drafts/paint.json` back into the live Paint editor. Returns
## `{ok, error, biome, terrain, splat}` -- the painted-cell count each layer
## came back with.
##
## The exact mirror of `sculpt_restore_document()` above, down to which caller
## it is for: the load path does not call it, because `project_open` applies
## the archive's own copy. This is the import route.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func paint_restore_document(text: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("paint_restore_document"):
		return {"ok": false, "error": "paint_restore_document not available on this binary"}
	return world_gen.paint_restore_document(text)

## Puts a `library/assets.json` back. Returns `{ok, error, slots, items}`.
## `items` is `0` on this build by design -- the record comes back, the
## decoded pixels do not (`project_bridge.rs`'s own note on
## `asset_library_document_json`).
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func asset_library_restore_document(text: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("asset_library_restore_document"):
		return {"ok": false, "error": "asset_library_restore_document not available on this binary"}
	return world_gen.asset_library_restore_document(text)

## Puts a `library/travel.json` back. Returns `{ok, error, restored,
## rejected}`. Replaces the custom half of every set and leaves the stock
## entries alone, so opening a project cannot leave the previous one's pack
## mule behind.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func travel_library_restore_document(text: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
	if not _has("travel_library_restore_document"):
		return {"ok": false, "error": "travel_library_restore_document not available on this binary"}
	return world_gen.travel_library_restore_document(text)

## Every slot the format defines.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers PackedStringArray().
func project_document_slots() -> PackedStringArray:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return PackedStringArray()
	if not _has("project_document_slots"):
		return PackedStringArray()
	return world_gen.project_document_slots()

## The subset the engine writes itself, which a caller may not supply.
##
## **No caller.** `DccApp._project_documents()` is the only place that builds
## a document map, and it cannot produce an engine-owned slot: its two sources
## are `project_engine_built_documents()` (which names its own four
## caller-owned slots) and the one literal `entities/journeys.json`. This is
## what a check would test against if that ever stops being true -- a slot
## name typed by hand is discovered "as a failed save" otherwise, which is
## the failure `project_document_slots()` was added for.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers PackedStringArray().
func project_engine_owned_slots() -> PackedStringArray:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return PackedStringArray()
	if not _has("project_engine_owned_slots"):
		return PackedStringArray()
	return world_gen.project_engine_owned_slots()

## The `format_version` this build writes (`WorldGen.project_format_version`). `0`
## means no binding, never a real version.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.
func project_format_version() -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0
	if not _has("project_format_version"):
		return 0
	return world_gen.project_format_version()

# -- F14 · read engine state back instead of trusting a shell copy -----------
#
# The setters were wired and these were not, so the shell kept its own copy of
# each toggle and nothing re-read the engine after a load. Same shape as the
# bug `route_get`'s `mode` carried until 2026-08-26.

## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers true.
func get_villages_enabled() -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return true
	if not _has("get_villages_enabled"):
		return true
	return world_gen.get_villages_enabled()

## Whether imperial-seat promotion is on (`WorldGen.get_metropolis_enabled`). The stale-binary
## fallback is `false`: it is the engine's own default (`cartalith_engine::WorldParams::
## defaults()`'s `civ.metropolis: false`, checked 2026-10-11), so a binary without this getter shows the
## state a generate would really use. It was `true` -- a guess that made the New World dialog
## offer promotion the engine would not apply (found-bug sweep 2026-10-11, R7).
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers false.
func get_metropolis_enabled() -> bool:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return false
	if not _has("get_metropolis_enabled"):
		return false
	return world_gen.get_metropolis_enabled()

## The recovery phase the next generate applies: 0 Stable .. 4 Mature
## (`WorldGen.get_recovery_phase`). `0` is also the stale-binary fallback.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers 0.
func get_recovery_phase() -> int:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return 0
	if not _has("get_recovery_phase"):
		return 0
	return world_gen.get_recovery_phase()

## The engine's own ceiling for `lod_level_for_zoom`, so a caller clamps
## against the engine rather than against a number copied into GDScript.
func lod_max_level() -> int:
	if not _has("lod_max_level"):
		return 0
	return world_gen.lod_max_level()

## Owner rulings 28/29's *"the size shown at save time"* -- the raw bytes a
## stored LOD tile pyramid would cost this project right now, at the depth
## `save_project()`'s `include_lod_tiles` flag actually stores
## (`lod_bridge::SAVE_PYRAMID_MAX_LEVEL`). Exact and instant, so the save
## dialog can show it before the user opts in. `0` against a binary built
## before this exists, or before any world -- the save flow reads that as
## "nothing to show", not as a real zero-byte pyramid.
func lod_save_pyramid_estimate_bytes() -> int:
	if not _has("lod_save_pyramid_estimate_bytes") or not has_world:
		return 0
	return int(world_gen.lod_save_pyramid_estimate_bytes())

## **LOD-D3's morph parameter**: how far pyramid level `z` has faded in at
## `px_per_cell` screen pixels per coarse cell. `0.0` means "draw this tile's
## parent", `1.0` means "draw this tile" (`LOD_DETAIL_SCOPE.md` LOD-D3;
## `lod_bridge::morph_for_zoom` carries the argument and the tests).
##
## Asked of the engine rather than written out in `ViewportHost` because it is
## `lod_level_for_zoom`'s own expression minus the rounding: two copies of it
## could disagree about where a level boundary is, and that disagreement is
## visible as exactly the pop the fade exists to remove.
##
## `1.0` against a binary built before this milestone -- which is the
## pre-LOD-D3 picture exactly (every tile drawn whole, no blend), so the
## compositor degrades to the previous behaviour rather than to an empty
## layer.
func lod_morph(px_per_cell: float, z: int) -> float:
	if not _has("lod_morph"):
		return 1.0
	return world_gen.lod_morph(px_per_cell, z)

# -- LOD-D6 · tile synthesis off the main thread ------------------------------
#
# `lod_synthesize_tile()` above stays exactly what it was: synchronous, and
# what `_lodsweep_probe.gd`'s metric 5 times. The five below are the
# non-blocking route -- prepare, request, drain, inspect, release. Both end in
# the same coloriser in Rust (`lod_worker::LodSnapshot::render_tile`), so a
# worker-built tile and a main-thread one are the same bytes by construction.

## Whether this binary can synthesise tiles off the main thread at all.
##
## **The gate the shell must use, and not `has_method` on this object.** These
## forwarders are shipped GDScript and always exist, so `has_method` on the
## bridge answers a question about *this file* rather than about the loaded
## `.dll`. Only `_has()` reaches through to the extension, and an older
## extension has to degrade to the pre-LOD-D6 synchronous path rather than to
## a pyramid that never fills in.
func lod_async_available() -> bool:
	return _has("lod_prepare") and _has("lod_request_tile") and _has("lod_take_ready_tiles")

## Start, or confirm, the background build of the tile context -- the
## `RenderCtx` precomputes and `TileFields` that LOD-D2 measured at 201.5 ms +
## 278.2 ms at 2048x1311 and that every tile of a zoom notch shares.
##
## `0` unavailable (no world), `1` ready (tiles can be requested), `2`
## building. **`0` against a binary without it**, which is the same answer as
## "no world" and is what makes the caller fall back rather than wait forever
## -- callers gate on `lod_async_available()` first.
func lod_prepare() -> int:
	if not _has("lod_prepare"):
		return 0
	return world_gen.lod_prepare()

## Queue one chunk for background synthesis. `false` when it was not queued:
## no context yet, the same chunk already in flight, or the quality tier's
## in-flight cap full. Not an error -- the caller asks again next frame.
func lod_request_tile(z: int, col: int, row: int) -> bool:
	if not _has("lod_request_tile"):
		return false
	return world_gen.lod_request_tile(z, col, row)

## **EF-9.4.** `lod_request_tile` for a tile drawn beside coarser ground:
## `conform` is its 8-neighbour mask from `lod_ef9_select()` (`WorldGen`'s
## `ef9_bridge.rs`). `false` against a `.dll` without it -- the caller then
## asks again next frame, as for any refusal, and never draws a conformed key
## from a plain request.
func lod_request_tile_conformed(z: int, col: int, row: int, conform: int) -> bool:
	if not _has("lod_request_tile_conformed"):
		return false
	return world_gen.lod_request_tile_conformed(z, col, row, conform)

## **EF-9.4.** The synchronous `lod_synthesize_tile` for a conformed tile;
## `null` against a `.dll` without it, which `_build_lod_tile` reads as "no tile".
func lod_synthesize_tile_conformed(z: int, col: int, row: int, conform: int) -> Texture2D:
	if not _has("lod_synthesize_tile_conformed"):
		return null
	return world_gen.lod_synthesize_tile_conformed(z, col, row, conform)

## **EF-9.4.** EF-9.2's tile selection for the view the viewport is drawing --
## `{basis, promoted, tiles, order, fallbacks, terms_missing, policy}`, see
## `WorldGen::lod_ef9_select`. `{}` against a `.dll` without it, which the
## viewport reads as "draw today's way".
func lod_ef9_select(z0: int, z_cap: int, c0: int, c1: int, r0: int, r1: int) -> Dictionary:
	if not _has("lod_ef9_select"):
		return {}
	return world_gen.lod_ef9_select(z0, z_cap, c0, c1, r0, r1)

## **EF-9.4, continuous model** (`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §8.5).
## The zoom's unrounded pyramid level (`lod_bridge::continuous_level`), the
## number `lod_level_for_zoom` rounds. `-1.0` against a `.dll` without it, which
## no real level is; the viewport reads it as "no continuous model" and draws
## today's way.
func lod_continuous_level(px_per_cell: float) -> float:
	if not _has("lod_continuous_level"):
		return -1.0
	return world_gen.lod_continuous_level(px_per_cell)

## **EF-9.4, continuous model.** The boost's fade-in after LOD entry
## (`continuous_lod::entry_gain`): 0 at `zoom_entry`, 1 one octave past it.
## `0.0` (no boost) against a `.dll` without it.
func lod_ef9_gain(zoom: float, zoom_entry: float) -> float:
	if not _has("lod_ef9_gain"):
		return 0.0
	return world_gen.lod_ef9_gain(zoom, zoom_entry)

## **EF-9.4, continuous model.** The boost above which a `z0 + 1` tile must be
## live, or `null` when none is needed (`continuous_lod::plus_one_threshold`).
## `null` against a `.dll` without it.
func lod_ef9_plus_one_threshold(lam_zoom: float, gain: float, z0: int, z_cap: int) -> Variant:
	if not _has("lod_ef9_plus_one_threshold"):
		return null
	return world_gen.lod_ef9_plus_one_threshold(lam_zoom, gain, z0, z_cap)

## **EF-9.4, continuous model.** `{key, n, any}` describing this world's boost
## field; `{"pending": true}` while the field is being built on its own thread
## (the main thread never waits for it -- ask again); `{}` before a world,
## when the build failed, or against a `.dll` without it.
func lod_ef9_boost() -> Dictionary:
	if not _has("lod_ef9_boost"):
		return {}
	return world_gen.lod_ef9_boost()

## **EF-9.4, continuous model.** The boost field as an `n x n` `FORMAT_RF`
## texture for `lod_tile.gdshader`'s `boost_tex`; `null` before a world or
## against a `.dll` without it.
func lod_ef9_boost_texture() -> Texture2D:
	if not _has("lod_ef9_boost_texture"):
		return null
	return world_gen.lod_ef9_boost_texture()

## **EF-9.4, continuous model.** `col, row` pairs of the level-`z` tiles in the
## range whose boost exceeds `thr` somewhere; empty against a `.dll` without it.
func lod_ef9_boost_tiles(z: int, c0: int, c1: int, r0: int, r1: int, thr: float) -> PackedInt32Array:
	if not _has("lod_ef9_boost_tiles"):
		return PackedInt32Array()
	return world_gen.lod_ef9_boost_tiles(z, c0, c1, r0, r1, thr)

## Collect up to `max` finished tiles as `{z, col, row, tex}`. The textures
## are created inside the engine call, on the calling (main) thread -- the
## scope's *"the main thread uploads the texture"*.
func lod_take_ready_tiles(max: int) -> Array:
	if not _has("lod_take_ready_tiles"):
		return []
	return world_gen.lod_take_ready_tiles(max)

## The background synthesiser's own state: `{ready, building, in_flight,
## waiting, pending, built, dropped, retained_bytes}`. `{}` against a binary
## without it, so a caller tests the key it wants rather than a version.
func lod_worker_stats() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not _has("lod_worker_stats"):
		return {}
	return world_gen.lod_worker_stats()

## The quality tier's LOD policy: `{tier, tiles_per_update, tiles_per_catchup,
## max_level, cache_tiles, max_in_flight}` (`lod_worker::budget_for_tier`).
##
## `{}` against a binary without it, which is what makes `ViewportHost` fall
## back to its own pre-LOD-D6 constants rather than to zero -- a budget of
## zero tiles is an empty pyramid, and a silent one.
func lod_budget() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not _has("lod_budget"):
		return {}
	return world_gen.lod_budget()

## Drop the tile context and everything queued against it. The retained clone
## of the world's four fields is the largest single thing the LOD path holds
## (43 MB at 2048x1311), so this is what a caller leaving deep zoom for good,
## or a probe wanting a cold start, uses.
func lod_release_worker() -> void:
	if not _has("lod_release_worker"):
		return
	world_gen.lod_release_worker()

# -- LOD-D7 (Ruling AV) · deep-zoom tiles for the info views -------------------
# The same snapshot, worker and in-flight cap as the terrain tiles above. Gate on
# `info_tiles_available()`, which asks the loaded `.dll` (`_has`), not this file.

## Whether the loaded extension can draw info tiles at all.
func info_tiles_available() -> bool:
	return _has("info_request_tile") and _has("info_take_ready_tiles") and _has("info_tile_tag")

## `{views: {id: "refined"|"interpolated"|"mixed"}, deferred: {group: reason},
## version}` -- which info views get a deep-zoom tile and how each relates to
## the map raster. `{}` against a binary without it.
func info_tile_views() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not _has("info_tile_views"):
		return {}
	return world_gen.info_tile_views()

## The cache tag tiles of `view` from the current snapshot carry, or `""` when
## the view is not tileable, no snapshot is built yet or its inputs are missing.
## `""` is the ONLY "no value" answer; a real tag is never empty.
func info_tile_tag(view: String) -> String:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return ""
	if not _has("info_tile_tag"):
		return ""
	var t: Variant = world_gen.info_tile_tag(view)
	return str(t) if t != null else ""

## Queue one info tile; `false` when it was not queued (ask again next frame).
func info_request_tile(view: String, z: int, col: int, row: int) -> bool:
	if not _has("info_request_tile"):
		return false
	return world_gen.info_request_tile(view, z, col, row)

## Collect up to `max` finished info tiles as `{view, tag, z, col, row, tex}`.
func info_take_ready_tiles(max: int) -> Array:
	if not _has("info_take_ready_tiles"):
		return []
	return world_gen.info_take_ready_tiles(max)

## `{in_flight, waiting, pending, built, dropped}`; `{}` against an older binary.
func info_worker_stats() -> Dictionary:
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
	if not _has("info_worker_stats"):
		return {}
	return world_gen.info_worker_stats()

# -- F13 · the two ops_bridge bindings the shell reaches for -----------------

## `_civRegionalPopulation` (reference line 23297): the modeled persons/km²
## field integrated over land, plus the painted-territory share. The
## reference's OTHER regional population figure -- distinct from
## `civ_agrarian_regional_total`'s settlement-sizing ceiling, and it never
## feeds back into it. `{total, land_km2, claimed}`, empty before a world.
##
## Recomputed fresh on every call, and comparable in cost to a real slice of
## Recompute civilisation -- call it from a button, never from a panel refresh.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {}.
func civ_regional_population() -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {}
	if not _has("civ_regional_population"):
		return {}
	return world_gen.civ_regional_population()

## Drops a whole asset collection by name. The grouping only -- the assets in
## it are untouched. A no-op, not an error, for a name that does not exist.
## Refused while `generating` (the worker holds `WorldGen`; see `settlements()`): answers {"ok": false, "error": _BUSY_TEXT}.
func as_drop_collection(name: String) -> Dictionary:
	# Mid-generation gate (caller-gate audit 2026-10-12): the worker holds `WorldGen`.
	if generating:
		return {"ok": false, "error": _BUSY_TEXT}
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
	# Mid-generation gate: the worker holds `WorldGen` (see `settlements()`).
	if generating:
		return {}
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

# -- Landmark generation (`landmark_bridge.rs`, `LANDMARK_UI_DESIGN.md` §9) --
#
# `landmark_*` is a SEPARATE accessor set from `param_*`/`get_param_info`,
# on purpose: a WORLD parameter's `on_release` write triggers a full
# `generate()` (`param_set` above, and `world_workspace.gd:1113`'s own call
# site), and a landmark cap is downstream of terrain, never an input to it
# -- it must never trigger one, and none of the writers below call
# `mark_dirty()` or touch `param_set`.
#
# Every row here is guarded on `generating`, not only `_has()`, the same
# discipline `stale_stages()` and `_gpu_read()` above already use and for
# the same reason (`_gpu_read`'s own doc comment, ~line 895, has the
# 360-panic measurement): `generate()` holds the whole `WorldGen` mutably
# borrowed on a worker thread for its run's duration, and any `#[func]`
# reached from the main thread meanwhile fails its own `Gd<T>::bind()` --
# a Rust panic per call. `landmark_run()` is threaded too, since 2026-09-01,
# and sets the SAME `generating` flag while it runs -- so the guard on every
# row here does double duty: it holds the panel off the engine during a
# generate, and off the engine during a landmark pass. Neither can start
# while the other is in flight, for the same reason.

## Static for the session -- filled once by `_read_landmark_kinds()` (called
## from `_ready()` and after `close_world()` swaps in a fresh `WorldGen`),
## so this never itself crosses the `Gd<T>` boundary and needs no
## `generating` guard.
var _landmark_kinds_cache: Array = []
## Landmark settings Dictionary read once by `_read_landmark_kinds` (from
## `world_gen.landmark_settings()`); served by `landmark_settings` without crossing into
## `WorldGen`.
var _landmark_settings_cache: Dictionary = {}

## Refreshes `_landmark_kinds_cache` and `_landmark_settings_cache` from the engine
## (`WorldGen.landmark_kinds`/`landmark_settings`). Main thread, not while a worker owns it.
func _read_landmark_kinds() -> void:
	_landmark_kinds_cache = world_gen.landmark_kinds() if _has("landmark_kinds") else []
	_landmark_settings_cache = world_gen.landmark_settings() if _has("landmark_settings") else {}

## The ~49-row type registry: `[{key,label,family,class,default_cap,
## needs_viewshed,buildable,not_built}, …]` -- `not_built` is the engine's own
## reason a `buildable: false` type is not placed, empty for the rest, and the
## text the panels show for that row. Served from the startup cache -- safe to
## call at any time, including mid-generation.
func landmark_kinds() -> Array:
	return _landmark_kinds_cache

## `{caps:{key:int}, armed:{key:bool}, crowding:float,
## class_radius_km:[f,f,f,f], cross_type_competition:bool}`. Cached read-
## through, the same shape `_gpu_read()` uses: mid-generation this answers
## from the last-known cache rather than reaching a borrowed `world_gen`.
func landmark_settings() -> Dictionary:
	if not _has("landmark_settings"):
		return {}
	if generating:
		return _landmark_settings_cache
	_landmark_settings_cache = world_gen.landmark_settings()
	return _landmark_settings_cache

## Writes one landmark type's cap (`WorldGen.landmark_set_cap`) and re-reads the cached
## settings. `false` while `generating`, without the binding, or for an unknown key.
func landmark_set_cap(key: String, v: int) -> bool:
	if generating or not _has("landmark_set_cap"):
		return false
	var ok: bool = world_gen.landmark_set_cap(key, v)
	if ok:
		_landmark_settings_cache = world_gen.landmark_settings()
	return ok

## Arms/disarms one landmark type (`WorldGen.landmark_set_armed`), then re-reads the
## cached settings. `false` while `generating`, without the binding, or for an unknown key.
func landmark_set_armed(key: String, on: bool) -> bool:
	if generating or not _has("landmark_set_armed"):
		return false
	var ok: bool = world_gen.landmark_set_armed(key, on)
	if ok:
		_landmark_settings_cache = world_gen.landmark_settings()
	return ok

## Sets the Crowding dial (`WorldGen.landmark_set_crowding`; the engine clamps rather
## than rejects), then re-reads the cache. `false` while `generating` or no binding.
func landmark_set_crowding(v: float) -> bool:
	if generating or not _has("landmark_set_crowding"):
		return false
	var ok: bool = world_gen.landmark_set_crowding(v)
	if ok:
		_landmark_settings_cache = world_gen.landmark_settings()
	return ok

## `class_key` is `"continental"` / `"regional"` / `"local"` / `"cultural"`.
func landmark_set_class_radius(class_key: String, km: float) -> bool:
	if generating or not _has("landmark_set_class_radius"):
		return false
	var ok: bool = world_gen.landmark_set_class_radius(class_key, km)
	if ok:
		_landmark_settings_cache = world_gen.landmark_settings()
	return ok

## Toggles "types compete with each other" (`WorldGen.landmark_set_cross_competition`),
## then re-reads the cache. `false` while `generating` or without the binding.
func landmark_set_cross_competition(on: bool) -> bool:
	if generating or not _has("landmark_set_cross_competition"):
		return false
	var ok: bool = world_gen.landmark_set_cross_competition(on)
	if ok:
		_landmark_settings_cache = world_gen.landmark_settings()
	return ok

## Restores every landmark setting to the engine default (`WorldGen.landmark_reset_settings`);
## the last run's placements stay. No-op while `generating`.
func landmark_reset_settings() -> void:
	if generating or not _has("landmark_reset_settings"):
		return
	world_gen.landmark_reset_settings()
	_landmark_settings_cache = world_gen.landmark_settings() if _has("landmark_settings") else {}

## Threaded, and `await`ed by its caller -- `generate()`'s pattern above, in
## this subject. **It used to be synchronous**, and that was the whole of the
## owner's 2026-09-01 report *"the new point of interest function seems to
## make the program freeze"*: the pass is full-raster terrain analysis costing
## seconds (measured in `_poifreeze_probe.gd`: 1.2 s at 1024x768, 4.4 s at the
## shipping 2048 default, 23 s at 4096), and running it inside a `#[func]` on
## the main thread served **zero** frames for that entire time. The control in
## the same probe run is `generate()`, four times the work, which served 255 --
## because it is threaded. Nothing about the pass needed to be on the main
## thread; it was simply never given the treatment `generate()` already had.
##
## Reuses `generating` and `_thread` rather than adding a second busy flag:
## `generating` is what holds every row in this block, `_gpu_read()`,
## `param_get()` and `ViewportHost._engine_readable()` off a mutably-borrowed
## `WorldGen`, which is exactly the crash this block's header describes. A
## private flag would have re-opened it for the landmark pass alone.
##
## `generation_started`/`generation_finished` are deliberately NOT reused, the
## opposite call from `import_heightmap()` below and for the opposite reason:
## that one *is* a generate, and this is not. Thirty-odd listeners read those
## two as "the world was replaced" and clear the trade store, the planner's
## journeys, the water animation and the wind field. A landmark pass changes
## none of those. It gets one signal of its own, declared at the head of this
## file with the rest.
##
## **Every caller must `await` this.** It is a coroutine on both paths -- the
## two refusals return without ever reaching the `await`, which `await` on a
## plain value passes straight through, so `await bridge.landmark_run()` is
## correct for a refusal too and no caller needs to know which it got.
func landmark_run() -> Dictionary:
	if generating:
		return {
			"ok": false, "placed": 0, "seconds": 0.0,
			"error": "A world generation is already running.", "funnels": [],
		}
	if not _has("landmark_run") or not _has("landmark_last_run"):
		return {
			"ok": false, "placed": 0, "seconds": 0.0,
			"error": "This build's extension has no landmark_run.", "funnels": [],
		}
	busy_kind = "landmarks"
	generating = true
	_thread = Thread.new()
	_thread.start(_landmark_worker)
	return await landmark_finished

## Runs off the main thread. Touches only `world_gen`, never a node -- the
## contract `_worker` above holds to, and the reason the `_finish`-shaped
## hand-back through `call_deferred` is not optional.
##
## `landmark_run()` returns a bare `bool` and the REPLY is fetched on the main
## thread by `_finish_landmark` below. That division is not stylistic: without
## gdext's `experimental-threads` feature every Godot `Dictionary`, `Array` and
## `GString` operation asserts the main thread, so a `#[func]` that builds its
## own reply dictionary panics when called from here -- *"attempted to access
## binding from different thread than main thread; this is UB"*, measured, with
## the pass itself completing and its reply arriving as nulls. `generate_sized`
## has always been safe on this thread for exactly the same reason: primitives
## in, nothing Godot-shaped out.
func _landmark_worker() -> void:
	world_gen.landmark_run()
	_finish_landmark.call_deferred()

## Main-thread completion of the landmark worker: joins the thread, clears `generating`
## and emits `landmark_finished` with `WorldGen.landmark_last_run()`.
func _finish_landmark() -> void:
	_thread.wait_to_finish()
	_thread = null
	generating = false
	## Read AFTER `generating` is cleared, on the main thread: this one does
	## build a Dictionary, so it may not run anywhere but here.
	landmark_finished.emit(world_gen.landmark_last_run())

## **Ruling AT's manual viewshed refine** -- the higher-fidelity visibility
## analysis over one cell rectangle (the current view), stored with the project
## and read by the next landmark pass where it is still fresh. Threaded and
## `await`ed exactly as `landmark_run()` is, and for the same reason (a large
## view is seconds of work; the same `generating` flag keeps it off a
## mutably-borrowed `WorldGen`). Returns `landmark_refine_status()` as it stands
## after the attempt, plus `ok`/`error` for the attempt itself -- so a refusal
## (`ok: false`, `error` says why) leaves the previous result and its state
## visible in the same dictionary.
##
## **Every caller must `await` this**; the two refusals return without
## reaching the `await`, which passes a plain value straight through.
var _refine_rect := Rect2i()
## Starts a viewshed refine of the given cell rect on a worker thread and awaits
## `landmark_refine_finished`, returning its status dictionary. Immediately returns
## `{ok: false, state: "never", error}` while generating or without the binding.
func landmark_refine_view(x: int, y: int, w: int, h: int) -> Dictionary:
	if generating:
		return {"ok": false, "state": "never", "error": "A world generation is already running."}
	if not _has("landmark_refine_view") or not _has("landmark_refine_status"):
		return {"ok": false, "state": "never", "error": "This build's extension has no viewshed refine."}
	_refine_rect = Rect2i(x, y, w, h)
	busy_kind = "viewshed"
	generating = true
	_thread = Thread.new()
	_thread.start(_landmark_refine_worker)
	return await landmark_refine_finished

## Runs off the main thread: primitives in, a bool out, no Godot value built --
## `_landmark_worker`'s contract. The reply is read on the main thread.
func _landmark_refine_worker() -> void:
	world_gen.landmark_refine_view(_refine_rect.position.x, _refine_rect.position.y,
		_refine_rect.size.x, _refine_rect.size.y)
	_finish_landmark_refine.call_deferred()

## Main-thread completion of the viewshed-refine worker: joins the thread, clears
## `generating`, emits `landmark_refine_finished` with `WorldGen.landmark_refine_status()`.
func _finish_landmark_refine() -> void:
	_thread.wait_to_finish()
	_thread = null
	generating = false
	landmark_refine_finished.emit(world_gen.landmark_refine_status())

## The refined viewshed's state now, judged against the live world by content
## key: `{state, ok, error[, observers, radius_cells, radius_km, x, y, w, h]}`
## with `state` one of `never` / `fresh` / `stale_terrain` / `stale_observers` /
## `stale_both`. The geometry keys are absent for `never`. `{}` while a pass
## holds the engine or on a build without it -- callers treat that as "unknown",
## not as "never refined".
func landmark_refine_status() -> Dictionary:
	if generating or not _has("landmark_refine_status"):
		return {}
	return world_gen.landmark_refine_status()

## Drops the stored refined viewshed (back to the coarse field).
func landmark_refine_clear() -> void:
	if generating or not _has("landmark_refine_clear"):
		return
	world_gen.landmark_refine_clear()

## The last `landmark_run()`'s placements: `[{id,kind,class,x,y,elevation,
## score,importance,causal:Array}, …]`.
func landmarks() -> Array:
	if generating or not _has("landmarks"):
		return []
	return world_gen.landmarks()

## The last `landmark_run()`'s per-type funnel -- the "why fewer than I
## asked for" popover's own data (`LANDMARK_UI_DESIGN.md` §5): `[{kind,
## candidates,rejected_constraint,rejected_score,rejected_spacing,cap,
## placed,limit}, …]`, `limit` a String.
func landmark_funnels() -> Array:
	if generating or not _has("landmark_funnels"):
		return []
	return world_gen.landmark_funnels()

## The last `landmark_run()`'s **rejected** candidates -- every cell the pass
## offered and did not place: `[{kind,x,y,score,reason,needs_crowding}, …]`.
##
## `reason` is `"score"` / `"spacing"` / `"cap"`. `needs_crowding` is the
## smallest Crowding at which that candidate would have cleared the ring that
## blocked it (see `landmark_bridge/rejects.rs`), and is **absent from the
## dictionary entirely** when there is no such figure -- use `has()`, not a
## default. *This line said `0.0` meant "does not apply" until 2026-09-03; it
## was true of the encoding and the encoding was the bug, since `0.0` is a
## plausible Crowding rather than an obvious sentinel and 44 of 614 spacing rows
## on a real world carried it.*
## Capped at the best-scoring 256 per kind -- `landmark_funnels()` still carries
## the true totals, so "showing 256 of 39 999" needs no second call.
##
## **Prefer `landmark_reject_points()` for drawing.** Measured at the shipping
## 2048x1311 default: this returns 3 216 rows in 6.0 ms, the packed form returns
## the same positions in 0.41 ms.
func landmark_rejects() -> Array:
	if generating or not _has("landmark_rejects"):
		return []
	return world_gen.landmark_rejects()

## The same rejections' positions alone, as one packed buffer -- the renderer's
## half. `reason` is `""` for all, or one of `"score"` / `"spacing"` / `"cap"`;
## an unrecognised string returns an empty buffer rather than everything.
##
## Cells, as `Vector2(x, y)` -- the space `landmarks()` reports and
## `map_overlay.gd::_cell_to_screen` consumes.
func landmark_reject_points(reason: String) -> PackedVector2Array:
	if generating or not _has("landmark_reject_points"):
		return PackedVector2Array()
	return world_gen.landmark_reject_points(reason)

## The headroom line (`LANDMARK_UI_DESIGN.md` §4.4): `{caps_total:int,
## room_estimate:int, last_placed:int}`.
func landmark_headroom() -> Dictionary:
	if generating or not _has("landmark_headroom"):
		return {}
	return world_gen.landmark_headroom()
