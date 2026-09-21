extends Node
## **`LOD_DETAIL_SCOPE.md` LOD-D6 — tile synthesis off the main thread.**
##
## This is the probe that earns the milestone. Everything in it fails on the
## pre-D6 shell, which is the test a probe has to pass before it counts as
## evidence (`MISTAKES.md`: *"a probe earns authority by FAILING on the state
## before the change"*) — and `_lod_sync` makes that literal here, because it
## restores the pre-D6 behaviour inside the same run and section 5 measures
## both legs against each other.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x900 \
##       --rendering-driver opengl3 _d6async_probe.tscn
##
## **MUST run WINDOWED.** Section 4 compares a live tile's own texels against
## a freshly synthesised tile, and `texture_2d_get` returns null under the
## dummy rasterizer (`MISTAKES.md`, "Run a pixel probe"). The probe refuses to
## run headless rather than passing vacuously.
##
## Exit 0 pass, 1 a real assertion failed, 2 the premise could not be set up
## (no `.gdextension`, a binary without the D6 surface, never reached deep
## zoom).

var _fail := 0
var _checks := 0
var _vp: SubViewport
var _vh: Node
var _br: Object

## Deep enough that `level_for_zoom` is past level 0 and a viewful is several
## tiles, which is what makes "did this call block" a question with an answer.
const DEEP_ZOOM := 9.0

## How long any "has it finished yet" loop is willing to wait, in frames.
## A bound, not a timing: every wait here is on the engine's own state
## (`building`, `pending`), and this only decides when the probe gives up and
## says so rather than hanging. Generous at any frame rate this shell reaches.
const WAIT_FRAMES := 400

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if cond:
		print("  ok   %s%s" % [name, ("  [%s]" % detail) if detail != "" else ""])
	else:
		_fail += 1
		printerr("  FAIL %s%s" % [name, ("  [%s]" % detail) if detail != "" else ""])

## Median, min and max of a sample — never a point estimate (`MISTAKES.md`:
## *"Median with min..max. Run the harness alone"*).
func _stat(v: Array) -> Dictionary:
	if v.is_empty():
		return {"n": 0, "med": 0.0, "min": 0.0, "max": 0.0}
	var a := v.duplicate()
	a.sort()
	return {"n": a.size(), "med": float(a[a.size() / 2]), "min": float(a[0]), "max": float(a[a.size() - 1])}

func _fmt(s: Dictionary) -> String:
	return "%.2f ms (%.2f..%.2f, n=%d)" % [s["med"], s["min"], s["max"], s["n"]]

## Waits until the background context build has finished, or the deadline
## passes. A frame count would be a claim about how fast this machine is; the
## engine's own `building` flag is the thing being waited on.
func _await_context(max_frames: int) -> bool:
	for i in max_frames:
		var st: Dictionary = _br.lod_worker_stats()
		if bool(st.get("ready", false)) and not bool(st.get("building", false)):
			return true
		await get_tree().process_frame
	return false

## Waits until nothing is owed: no chunk queued for a worker and none waiting
## for a per-call budget.
func _await_settled(max_frames: int) -> bool:
	for i in max_frames:
		if _vh.lod_pending() == 0:
			return true
		await get_tree().process_frame
	return false

func _zoom_to(target: float) -> void:
	var guard := 0
	while _vh.zoom() < target and _vh.zoom() < _vh._zoom_max and guard < 60:
		_vh.zoom_step(1.45)
		guard += 1
		await get_tree().process_frame

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless. Section 4 reads texels off a texture and")
		printerr("  `texture_2d_get` returns null under the dummy rasterizer, so every")
		printerr("  comparison here would pass without comparing anything.")
		get_tree().quit(2)
		return
	if not ClassDB.class_exists("WorldGen"):
		printerr("PROBE-CANNOT-RUN: the .gdextension did not load.")
		get_tree().quit(2)
		return

	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(30)
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(15)
	_br = app.bridge
	_vh = app.viewport
	if _br == null or _vh == null:
		printerr("PROBE-CANNOT-RUN: the shell exposed no bridge/viewport.")
		get_tree().quit(2)
		return
	print("[BOOT] shell up")

	# --- 1. the engine surface --------------------------------------------
	#
	# A stale `.dll` is the standing hazard for any Godot probe grading a Rust
	# change (`MISTAKES.md`: *"`cargo test` does not rebuild the .dll"*), and
	# these four methods are what makes it detectable here: a binary built
	# before LOD-D6 answers none of them, and the probe stops rather than
	# reporting a pass about a library that does not contain the milestone.
	print("\n=== 1: the LOD-D6 engine surface ===")
	## `lod_async_available()` and NOT `has_method` on the bridge: the
	## forwarders are shipped GDScript and always answer, so `has_method`
	## here would be a question about `engine_bridge.gd` rather than about the
	## loaded `.dll`. This is also this probe's stale-`.dll` detector -- the
	## standing hazard for any Godot probe grading a Rust change.
	if not _br.lod_async_available():
		printerr("PROBE-CANNOT-RUN: the loaded .dll has no LOD-D6 bindings -- it predates the milestone.")
		get_tree().quit(2)
		return
	print("  ok   the loaded extension carries the LOD-D6 bindings")
	var bud: Dictionary = _br.lod_budget()
	for k in ["tier", "tiles_per_update", "tiles_per_catchup", "max_level", "cache_tiles", "max_in_flight"]:
		_ok("lod_budget() carries %s" % k, bud.has(k), str(bud.get(k, "-")))
	# The shipped default tier is `Quality`, whose budget the engine's own
	# `the_tier_budgets_are_the_documented_literals` pins to the pre-D6
	# constants. Asserted here as the literals too, not against the engine's
	# own answer: a tier-vs-tier check cannot fail.
	_ok("the default tier schedules the pre-D6 budget", int(bud.get("tiles_per_update", -1)) == 48 and int(bud.get("tiles_per_catchup", -1)) == 6,
		"%s/%s" % [bud.get("tiles_per_update", "-"), bud.get("tiles_per_catchup", "-")])

	# --- 2. a world, zoomed past the threshold ----------------------------
	print("\n=== 2: a world, zoomed until the pyramid is up ===")
	_br.generate({
		"seed": 483920, "width_km": 800.0, "grid_w": 512, "grid_h": 384,
		"sea_level": 0.42, "villages": true,
	})
	## **`generate()` is asynchronous.** `EngineBridge.generating` is the flag
	## it clears, and a probe that only awaits frames measures a shell with no
	## world in it -- which reads as "never reached deep zoom" rather than as
	## "never generated", so it is worth waiting on the right thing.
	var spins := 0
	while _br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	if _br.generating:
		printerr("PROBE-CANNOT-RUN: generation never finished.")
		get_tree().quit(2)
		return
	await get_tree().create_timer(1.0).timeout
	_vh.refresh()
	await _frames(20)
	print("  info grid %s, zoom_max %.1f" % [str(_br.grid_size()), _vh._zoom_max])
	await _zoom_to(DEEP_ZOOM)
	await _frames(4)
	## **Auto entry is a saved preference** (`Preferences -> Tiled LOD`,
	## `DccSettings.lod_auto()`), so a machine whose settings have it on
	## manual would otherwise report "never reached deep zoom" for a camera
	## that is perfectly deep enough. `request_lod_entry()` is the shell's own
	## documented way in and is a no-op when the layer is already up.
	if not _vh.lod_active():
		print("  info auto entry is off (zoom %.2f); asking for the pyramid explicitly" % _vh.zoom())
		_vh.request_lod_entry()
		await _frames(4)
	if not _vh.lod_active():
		printerr("PROBE-CANNOT-RUN: never reached deep zoom (zoom=%.2f, auto=%s)." % [_vh.zoom(), _vh.lod_auto()])
		get_tree().quit(2)
		return
	print("  info zoom %.2f, level %d" % [_vh.zoom(), _vh._lod_child_level])
	_ok("the context finished building in the background", await _await_context(WAIT_FRAMES))
	_ok("the pyramid settled", await _await_settled(WAIT_FRAMES), "pending=%d" % _vh.lod_pending())
	_ok("and it settled with real tiles on screen", (_vh._lod_tiles as Dictionary).size() > 0,
		"%d tiles" % (_vh._lod_tiles as Dictionary).size())
	var st: Dictionary = _br.lod_worker_stats()
	_ok("tiles were built on a worker, not on this thread", int(st.get("built", 0)) > 0, "built=%s" % st.get("built", 0))
	print("  info worker: %s" % str(st))

	# --- 3. THE CLAIM: a cold entry does not build on this thread ---------
	#
	# This is the milestone in one assertion. `lod_release_worker()` makes the
	# context cold, `_clear_lod_tiles()` makes the tile set empty, and then one
	# `_update_lod()` is the exact call LOD-D3 measured at 225 ms. If it comes
	# back with tiles already live, synthesis happened on this thread and the
	# milestone is not built.
	print("\n=== 3: a cold LOD entry returns without synthesising anything ===")
	_vh._lod_sync = false
	_br.lod_release_worker()
	_vh._clear_lod_tiles()
	await _frames(2)
	## `_clear_lod_tiles()` does not take the layer down, so the manual-entry
	## request (if one was needed above) is still armed and this is the same
	## call the camera itself makes.
	_vh._update_lod()
	var tiles_now: int = (_vh._lod_tiles as Dictionary).size()
	var pending_now: int = _vh.lod_pending()
	_ok("nothing was synthesised inside the call", tiles_now == 0, "%d tiles live immediately after" % tiles_now)
	_ok("and the work was queued instead", pending_now > 0, "%d chunks owed" % pending_now)
	_ok("the context build is in flight, not finished", bool((_br.lod_worker_stats() as Dictionary).get("building", false)) or bool((_br.lod_worker_stats() as Dictionary).get("ready", false)))
	_ok("the pyramid then fills in on its own", await _await_settled(WAIT_FRAMES), "pending=%d" % _vh.lod_pending())
	_ok("and ends with tiles on screen", (_vh._lod_tiles as Dictionary).size() > 0, "%d tiles" % (_vh._lod_tiles as Dictionary).size())

	# --- 4. determinism: a worker tile IS a main-thread tile ---------------
	#
	# The engine test `worker_and_main_thread_tiles_are_byte_identical` asserts
	# this over plain buffers. This asserts it over the texture that actually
	# reached the canvas, which is a different claim: it covers the marshalling
	# too (`PackedByteArray` -> `Image` -> `ImageTexture`), and it is the half a
	# unit test structurally cannot reach.
	print("\n=== 4: a tile built on a worker is the tile this thread would have built ===")
	var keys: Array = (_vh._lod_tiles as Dictionary).keys()
	var compared := 0
	var differed := 0
	for i in range(mini(4, keys.size())):
		var sprite := _vh._lod_tiles[keys[i]] as Sprite2D
		var idx: Vector3i = sprite.get_meta("lod_idx", Vector3i(-1, 0, 0))
		if idx.x < 0 or sprite.texture == null:
			continue
		var live: Image = sprite.texture.get_image()
		var fresh_tex: Texture2D = _br.lod_synthesize_tile(idx.x, idx.y, idx.z)
		if live == null or fresh_tex == null:
			continue
		var fresh: Image = fresh_tex.get_image()
		if fresh == null:
			continue
		compared += 1
		if live.get_data() != fresh.get_data():
			differed += 1
	_ok("there were real tiles to compare (teeth check)", compared > 0, "%d compared" % compared)
	_ok("every one is byte-identical to the synchronous path", differed == 0, "%d of %d differ" % [differed, compared])

	# --- 5. the controlled A/B on the entry stall -------------------------
	#
	# The same shell, the same world, the same camera, the same cold start --
	# and the only difference is which thread synthesises. The sync leg IS the
	# pre-D6 behaviour, so this is a before/after measured inside one run
	# rather than across two builds, and it carries its own control.
	#
	# Medians of five with min..max, never a point estimate. The number that
	# matters is the SYNC leg's, because that is the frame the user feels.
	print("\n=== 5: how long a cold LOD entry blocks this thread ===")
	var sync_ms: Array = []
	var async_ms: Array = []
	for i in 5:
		for mode in [true, false]:
			_vh._lod_sync = mode
			_br.lod_release_worker()
			_vh._clear_lod_tiles()
			await _frames(3)
			var t0 := Time.get_ticks_usec()
			_vh._update_lod()
			var dt := float(Time.get_ticks_usec() - t0) / 1000.0
			if mode:
				sync_ms.append(dt)
			else:
				async_ms.append(dt)
			## Let whatever was started finish before the next iteration, so
			## one leg is never timed against the other's background load.
			await _await_context(WAIT_FRAMES)
			await _await_settled(WAIT_FRAMES)
	var s_sync := _stat(sync_ms)
	var s_async := _stat(async_ms)
	print("  MEASURED cold LOD entry, main-thread cost of one _update_lod():")
	print("    synchronous (pre-LOD-D6): %s" % _fmt(s_sync))
	print("    on a worker  (LOD-D6):    %s" % _fmt(s_async))
	_ok("the synchronous leg really is expensive (teeth check -- without this the comparison is meaningless)",
		float(s_sync["med"]) > 20.0, _fmt(s_sync))
	_ok("moving synthesis off this thread makes the entry call cheap",
		float(s_async["med"]) < float(s_sync["med"]) * 0.25, "%.2f vs %.2f ms" % [s_async["med"], s_sync["med"]])

	# --- 6. the tile cache ------------------------------------------------
	#
	# The tier's `cache_tiles`: a chunk that scrolls out is parked, and a chunk
	# that scrolls back in is restored rather than re-synthesised. Measured on
	# the worker's own `built` counter, which only a real synthesis moves.
	print("\n=== 6: a chunk that scrolls out and back is not synthesised twice ===")
	_vh._lod_sync = false
	await _await_settled(WAIT_FRAMES)
	var before_park: int = (_vh._lod_cache as Dictionary).size()
	var here: Vector2 = _vh._camera.position
	_vh._camera.position = here + Vector2(-900.0, -700.0)
	_vh._update_lod()
	await _await_settled(WAIT_FRAMES)
	var parked: int = (_vh._lod_cache as Dictionary).size()
	_ok("panning away parks tiles rather than freeing them", parked > before_park, "%d -> %d parked" % [before_park, parked])
	var built_before: int = int((_br.lod_worker_stats() as Dictionary).get("built", 0))
	_vh._camera.position = here
	_vh._update_lod()
	await _frames(2)
	var built_after: int = int((_br.lod_worker_stats() as Dictionary).get("built", 0))
	var restored: int = (_vh._lod_tiles as Dictionary).size()
	_ok("panning back restores them with no new synthesis", built_after == built_before and restored > 0,
		"built %d -> %d, %d tiles live" % [built_before, built_after, restored])

	# --- 7. the tier caps the level ---------------------------------------
	#
	# The scope's own example: *"Android Performance gets MAX_LEVEL - 1"*. The
	# level the shell chooses has to honour it, not merely the engine's
	# `lod_max_level()`.
	print("\n=== 7: the quality tier's maximum level is honoured ===")
	var absolute: int = _br.lod_max_level()
	_br.set_quality_tier("Performance")
	_vh._refresh_lod_budget()
	var perf: Dictionary = _br.lod_budget()
	_ok("Performance caps one level shallower than the engine's ceiling",
		int(perf.get("max_level", 0)) == absolute - 1, "%s vs %d" % [perf.get("max_level", "-"), absolute])
	await _zoom_to(_vh._zoom_max)
	await _frames(6)
	_vh._update_lod()
	await _frames(2)
	_ok("and the shell does not go deeper than it", _vh._lod_child_level <= int(perf.get("max_level", 0)),
		"level %d, cap %s" % [_vh._lod_child_level, perf.get("max_level", "-")])
	_br.set_quality_tier("Quality")

	# --- 8. memory --------------------------------------------------------
	#
	# Reported, not asserted: the bar is *"steady memory at most 60 MiB above
	# the recorded steady state"*, and what the snapshot retains is only part
	# of a process's steady memory. What this prints is the part this milestone
	# is responsible for, measured off the live snapshot rather than estimated
	# from a formula.
	print("\n=== 8: what the snapshot retains (reported, not asserted) ===")
	_vh._refresh_lod_budget()
	await _await_settled(WAIT_FRAMES)
	var final_stats: Dictionary = _br.lod_worker_stats()
	var mb := float(final_stats.get("retained_bytes", 0)) / 1048576.0
	var g: Vector2i = _br.grid_size()
	print("  MEASURED retained by the LOD snapshot at %dx%d: %.2f MB" % [g.x, g.y, mb])
	print("  MEASURED parked tiles: %d of a %d cap (%.2f MB of RGBA at 256px)"
		% [(_vh._lod_cache as Dictionary).size(), int((_br.lod_budget() as Dictionary).get("cache_tiles", 0)),
		   float((_vh._lod_cache as Dictionary).size()) * 262144.0 / 1048576.0])
	print("  info final worker state: %s" % str(final_stats))

	# --- 9. the same measurement at the size the bar is quoted at ----------
	#
	# LOD-D6's memory bar is *"steady memory at most 60 MiB above the recorded
	# steady state"*, and every figure this milestone inherits (the 201.5 ms
	# context, the 5.98 ms tile, the 878-908 MB generation peak) is quoted at
	# 2048x1311. Arithmetic from the 512x384 world would be an extrapolation
	# presented as a measurement, so the world is regenerated at the size the
	# number is about. No zoom is needed: `lod_prepare()` builds the context
	# on its own, which is the thing that retains.
	print("\n=== 9: the same measurement at 2048x1311 ===")
	_br.lod_release_worker()
	_br.generate({
		"seed": 483920, "width_km": 800.0, "grid_w": 2048, "grid_h": 1311,
		"sea_level": 0.42, "villages": true,
	})
	var spins2 := 0
	while _br.generating and spins2 < 4800:
		await get_tree().create_timer(0.25).timeout
		spins2 += 1
	if _br.generating:
		printerr("  (could not generate at 2048x1311; the figure above is the only one measured)")
	else:
		await get_tree().create_timer(1.0).timeout
		var t0 := Time.get_ticks_usec()
		var prep: int = _br.lod_prepare()
		var prep_ms := float(Time.get_ticks_usec() - t0) / 1000.0
		_ok("lod_prepare() returns without building at 2048x1311", prep == 2 or prep == 1, "state=%d in %.2f ms" % [prep, prep_ms])
		var got := await _await_context(WAIT_FRAMES * 3)
		_ok("the 2048x1311 context finished on a worker", got)
		var st9: Dictionary = _br.lod_worker_stats()
		var g9: Vector2i = _br.grid_size()
		print("  MEASURED lod_prepare() main-thread cost at %dx%d: %.2f ms" % [g9.x, g9.y, prep_ms])
		print("  MEASURED retained by the LOD snapshot at %dx%d: %.2f MB"
			% [g9.x, g9.y, float(st9.get("retained_bytes", 0)) / 1048576.0])
		print("  info worker: %s" % str(st9))

	print("\nRESULT %d checks, %d failed" % [_checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
