extends Node
## Reproduces the owner's 2026-09-01 report — *"the new point of interest
## function seems to make the program freeze and doesn't render on the map"* —
## in a real Godot process, and times the two halves of it separately.
##
##   Godot_v4.7.1 --headless --path . _poifreeze_probe.tscn -- 256 192
##   Godot_v4.7.1 --headless --path . _poifreeze_probe.tscn -- 512 384
##   Godot_v4.7.1 --headless --path . _poifreeze_probe.tscn -- 1024 768
##
## Grid comes from user args (after `--`) so each size runs in its OWN process:
## a hang at 1024 must not lose the numbers 256 already produced.
##
## Three things are measured, not one. **All three assertions failed before the
## 2026-09-01 fix and must keep passing after it** — that is what this probe is
## for now: it is the regression check for both halves of the owner's report.
##
##   1. WHETHER THE MAIN LOOP KEEPS TICKING during the pass. `landmark_run()`
##      was a synchronous `#[func]` on the main thread and served **0 frames**
##      at every grid size (1.2 s at 1024x768, 4.4 s at the 2048 default, 23 s
##      at 4096) while `generate()` — four times the work, on a `Thread` —
##      served 255 in the same process. It is threaded now, so the assertion
##      `the main thread kept painting during the pass` is the freeze's own
##      regression test. Wall time is still printed: the growth curve across
##      grids is the evidence, one number is not.
##   2. Whether the placements ever reach `map_overlay.gd`. `_lm_run()` refreshed
##      its own rows, its groups, its headroom and its stale badge and told the
##      map nothing; `ViewportHost` now connects itself to
##      `EngineBridge.landmark_finished`. The probe reads
##      `ViewportHost.overlay._landmarks` before and after.
##   3. Whether a REGENERATE clears them again. `WorldGen` invalidates its
##      landmark store on every generate, but `MapOverlay._landmarks` is
##      shell-side state that nothing was clearing, so world A's rings stayed
##      drawn over world B. `ViewportHost.refresh()` pushes the annotation
##      layers now; STEP 14 regenerates and asserts the overlay went back to 0.
##
## A Timer watchdog is here because every probe in this folder has one, but note
## honestly what it can and cannot do: a Timer is serviced by the main loop, so
## it CANNOT fire while the main thread is blocked inside a synchronous Rust
## call. It catches a stall in GDScript-land; it cannot catch the freeze this
## report is about. That is why the runner also wraps the process in an OS-level
## timeout — see the pass notes.

const WATCHDOG_S := 900.0

var _app: Node
var _gw := 256
var _gh := 192
var _fails := 0

## Frames the main loop actually served. The whole "frozen window" claim reduces
## to one question — did the main loop tick while `landmark_run()` was running? —
## and this counter answers it without needing a window to look at.
var _ticks := 0

func _process(_d: float) -> void:
	_ticks += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fails += 1
	print("  %s %s   got=%s want=%s" % ["ok  " if good else "FAIL", name, got, want])

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	if args.size() >= 2:
		_gw = int(args[0])
		_gh = int(args[1])
	print("=== _poifreeze_probe  grid %dx%d ===" % [_gw, _gh])

	var wd := Timer.new()
	wd.wait_time = WATCHDOG_S
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT after %.0f s (main loop still alive, probe stalled)" % WATCHDOG_S)
		get_tree().quit(3))
	wd.start()

	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(1)
		return

	print("[STEP 1] instantiating res://shell/app.tscn")
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	print("[STEP 2] shell up")

	var bridge = _app.bridge
	if bridge == null:
		print("[FATAL] app.bridge is null")
		get_tree().quit(1)
		return

	print("[STEP 3] generating world %dx%d" % [_gw, _gh])
	var gticks := _ticks
	var t_gen := Time.get_ticks_msec()
	bridge.generate({
		"seed": 483920, "width_km": 2400.0, "grid_w": _gw, "grid_h": _gh,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.1).timeout
	var gen_ms := Time.get_ticks_msec() - t_gen
	## The control. `generate()` is the same size of work on the same data and
	## it does NOT starve the loop, because it runs on a `Thread` and reports
	## through `bridge.generating`. Whatever number this prints is what
	## `landmark_run()`'s own line should have printed and did not.
	print("[STEP 3b] CONTROL — main loop frames served during generate(): %d"
		% (_ticks - gticks))
	await get_tree().create_timer(0.5).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(4)
	print("[STEP 4] world generated in %d ms  has_world=%s" % [gen_ms, bridge.has_world])
	_ok("a world exists to place landmarks on", bridge.has_world, true)

	## What the overlay holds BEFORE any landmark pass. `map_overlay.gd`'s own
	## `_landmarks` starts `[]` and `_landmarks_visible` starts `true`.
	var overlay = _app.viewport.overlay if _app.viewport != null else null
	var before_ov := -1
	var vis_flag = null
	if overlay != null:
		before_ov = (overlay.get("_landmarks") as Array).size()
		vis_flag = overlay.get("_landmarks_visible")
	print("[STEP 5] overlay before run: _landmarks=%d  _landmarks_visible=%s" % [before_ov, vis_flag])

	## The vocabulary, and how much of it is armed — the run's own workload.
	var kinds: Array = bridge.landmark_kinds() if bridge.has_method("landmark_kinds") else []
	var settings: Dictionary = bridge.landmark_settings() if bridge.has_method("landmark_settings") else {}
	var armed_map: Dictionary = settings.get("armed", {})
	var armed_n := 0
	var buildable_armed := 0
	for k in kinds:
		var kd: Dictionary = k
		if bool(armed_map.get(String(kd.get("key", "")), false)):
			armed_n += 1
			if bool(kd.get("buildable", true)):
				buildable_armed += 1
	print("[STEP 6] vocabulary: %d kinds, %d armed, %d of those buildable"
		% [kinds.size(), armed_n, buildable_armed])
	print("         caps_total=%s" % str(settings.get("caps", {}).size()))

	# ── THE MEASUREMENT ──────────────────────────────────────────────────────
	print("[STEP 7] >>> calling bridge.landmark_run() NOW — if the process dies")
	print("         or is killed by the outer timeout, it died HERE.")
	var ticks0 := _ticks
	var t0 := Time.get_ticks_usec()
	## `await` is mandatory since 2026-09-01: the bridge runs the pass on a
	## `Thread` and hands the reply back through `landmark_finished`. A bare
	## call returns the coroutine, not the dictionary.
	var r: Dictionary = await bridge.landmark_run()
	var us := Time.get_ticks_usec() - t0
	var served := _ticks - ticks0
	print("[STEP 8] <<< landmark_run() RETURNED after %.1f ms (%.3f s)"
		% [us / 1000.0, us / 1000000.0])
	## At 60 Hz a healthy %.1f ms window would have served roughly %d frames.
	print("         MAIN LOOP FRAMES SERVED DURING THE CALL: %d  (a responsive"
		% served)
	print("         main loop would have served about %d in %.1f ms)"
		% [int(us / 16666.0), us / 1000.0])
	_ok("the main thread kept painting during the pass", served > 0, true)
	print("         reply: ok=%s placed=%s seconds=%s error='%s' funnels=%d"
		% [r.get("ok"), r.get("placed"), r.get("seconds"), r.get("error", ""),
			(r.get("funnels", []) as Array).size()])
	print("RESULT_ROW\t%dx%d\tgen_ms=%d\tlandmark_ms=%.1f\tplaced=%s"
		% [_gw, _gh, gen_ms, us / 1000.0, r.get("placed")])

	_ok("the pass reported ok", r.get("ok"), true)

	# ── WHAT CAME BACK ───────────────────────────────────────────────────────
	var lms: Array = bridge.landmarks()
	print("[STEP 9] landmarks(): %d entries" % lms.size())
	_ok("the pass placed something", lms.size() > 0, true)
	var by_kind := {}
	var no_coord := 0
	var out_of_grid := 0
	for l in lms:
		var ld: Dictionary = l
		var kk := String(ld.get("kind", "?"))
		by_kind[kk] = int(by_kind.get(kk, 0)) + 1
		if not ld.has("x") or not ld.has("y"):
			no_coord += 1
			continue
		var x := float(ld["x"])
		var y := float(ld["y"])
		if x < 0.0 or y < 0.0 or x > float(_gw) or y > float(_gh):
			out_of_grid += 1
	print("         by kind: ", by_kind)
	if lms.size() > 0:
		print("         sample[0]: ", lms[0])
	_ok("every placement carries x/y", no_coord, 0)
	_ok("every placement lies inside the grid", out_of_grid, 0)

	var funnels: Array = bridge.landmark_funnels()
	var limits := {}
	for f in funnels:
		var lim := String((f as Dictionary).get("limit", ""))
		limits[lim] = int(limits.get(lim, 0)) + 1
	print("[STEP 10] funnels: %d, limiting reasons: %s" % [funnels.size(), str(limits)])

	# ── DID THE MAP EVER HEAR ABOUT IT? ──────────────────────────────────────
	## The shipped path is `CivilizationWorkspace._lm_run()`, not the bare
	## bridge call above. Run THAT, then re-read the overlay: if the engine has
	## placements and the overlay still holds none, nothing in the UI path
	## pushed them at the map, which is the report's second symptom exactly.
	var civ: Node = _app.find_child("CivilizationWorkspace", true, false)
	print("[STEP 11] CivilizationWorkspace present: %s" % (civ != null))
	if civ != null:
		var t1 := Time.get_ticks_usec()
		await civ.call("_lm_run")
		var us2 := Time.get_ticks_usec() - t1
		await _frames(4)
		print("          the SHIPPED UI path (_lm_run) took %.1f ms" % (us2 / 1000.0))
		var note = civ.get("_lm_run_note")
		if note != null:
			print("          run note: ", String(note.text))

	var after_ov := -1
	if overlay != null:
		after_ov = (overlay.get("_landmarks") as Array).size()
	print("[STEP 12] overlay after the UI run: _landmarks=%d  (engine has %d)"
		% [after_ov, lms.size()])
	_ok("the map overlay received the placements", after_ov, lms.size())
	## `_draw_landmarks` maps a cell through `_cell_to_screen`, which divides by
	## `MapOverlay._gw`/`_gh`. Those default to 0 and are written only by
	## `set_civ_data()`, so a `refresh_annotations()` that ever reached the
	## overlay before a `refresh()` would put every ring at INF and draw
	## nothing, silently — `interior.has_point(INF, INF)` is simply false. The
	## ordering inside `ViewportHost.refresh()` (civ data first, annotations
	## last) is what makes that unreachable; this asserts the ordering rather
	## than trusting it.
	if overlay != null:
		print("          overlay grid for _cell_to_screen: _gw=%s _gh=%s"
			% [overlay.get("_gw"), overlay.get("_gh")])
		_ok("the overlay knows the grid, so rings map to real pixels",
			int(overlay.get("_gw")) > 0 and int(overlay.get("_gh")) > 0, true)

	## And if it did not — prove the overlay is capable of drawing them, so the
	## finding is "nobody handed them over", not "the overlay is broken".
	if overlay != null and after_ov != lms.size():
		print("[STEP 13] hand-feeding the overlay to isolate the fault")
		if _app.viewport.has_method("refresh_annotations"):
			_app.viewport.refresh_annotations()
			await _frames(2)
			var forced := (overlay.get("_landmarks") as Array).size()
			print("          after an explicit ViewportHost.refresh_annotations(): %d" % forced)
			print("          => the overlay works; the landmark pass never called it.")
			_ok("refresh_annotations() does deliver them", forced, lms.size())
		overlay.queue_redraw()
		await _frames(2)

	# ── AND DOES A REGENERATE TAKE THEM BACK DOWN? ───────────────────────────
	## The sibling of the missing wire. `WorldGen` invalidates its landmark
	## store on every `generate`, so the panel correctly goes back to "not run";
	## `MapOverlay._landmarks` is shell-side and was never cleared, which left
	## world A's rings drawn over world B's terrain. A second world here, one
	## seed apart, and the overlay must be empty again afterwards.
	print("[STEP 14] regenerating with a different seed — the rings must go")
	bridge.generate({
		"seed": 483921, "width_km": 2400.0, "grid_w": _gw, "grid_h": _gh,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.1).timeout
	await _frames(6)
	var stale := -1
	if overlay != null:
		stale = (overlay.get("_landmarks") as Array).size()
	print("          engine landmarks() after regenerate: %d" % bridge.landmarks().size())
	print("          overlay _landmarks after regenerate: %d" % stale)
	_ok("a regenerate clears the previous world's rings", stale, 0)

	print("=== SUMMARY %dx%d fails=%d ===" % [_gw, _gh, _fails])
	get_tree().quit(1 if _fails > 0 else 0)
