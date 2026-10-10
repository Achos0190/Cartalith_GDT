extends Node
## Ruling BZ item 3 (2026-10-10): the generate freeze. Moving the render's
## world-derived inputs (water flood, river network, valley shading, river
## colour field) onto the generation worker (`WorldGen.warm_render_caches`,
## called from `engine_bridge.gd::_worker`). This probe protects three things.
##
## MUST run windowed: it times frames and hashes textures, and both are
## meaningless under `--headless` (`ImageTexture.update()` is a no-op there,
## `frame_post_draw` never fires).
##   Godot_v4.7.1-stable_win64_console.exe --path . _genasync_probe.tscn -- --mode time --grid 2048x1311 --reps 3
##   Godot_v4.7.1-stable_win64_console.exe --path . _genasync_probe.tscn -- --mode identity --grid 1024x656
##   Godot_v4.7.1-stable_win64_console.exe --path . _genasync_probe.tscn -- --mode stale --grid 1024x656
## Flags: --mode time|identity|stale, --grid WxH, --seed N, --reps N, --sync
## (time mode only: turn the warm off, to measure the synchronous path in the
## same binary). An unknown flag fails the run.
##
## `--mode time`: the main thread's longest stall during a generate -- the
## largest gap between two consecutive `_process` ticks from the `generate()`
## call until one second after the first frame drawn with the new world -- and
## the total time to that frame. A measurement, not an assertion.
##
## `--mode identity`: the same seed generated twice through the real shell,
## once with `EngineBridge.prewarm_render` off (everything computed on the main
## thread at the first repaint, as before the ruling) and once on. Asserts the
## map texture, the river colour texture, the water mask, the shore field and
## the drawn water classification (`WorldGen.drawn_water_digest`) are the same
## bytes, and -- the positive control that the warm really ran where it should
## -- that the valley shading was built OFF the main thread on the warmed run
## and ON it on the synchronous one (`valley_shade_stats().main_thread`).
##
## `--mode stale`: a warmed result must never be drawn after the world moved
## under it. (1) A sea-level change while a generate runs is refused
## (`sea_level_live` returns `{}`), so a mid-job edit cannot reach the engine.
## (2) The key check itself: warm at sea level 0.42, move the sea to 0.5, then
## repaint; every texture must equal a synchronous repaint at 0.5 of the same
## world, and differ from the 0.42 one (positive control that the move is
## visible). Without the key check the shore field is drawn from the stale
## 0.42 flood and (2) fails.

var _mode := "time"
var _grid := Vector2i(1024, 656)
var _seed := 24601
var _reps := 3
var _sync := false
var _app: Node
var _br: Node
var _vh: Control
var _fails := 0

var _tracking := false
var _last_us := 0
var _gaps: Array = []

func _process(_d: float) -> void:
	if not _tracking:
		return
	var now := Time.get_ticks_usec()
	_gaps.append(now - _last_us)
	_last_us = now

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless"); get_tree().quit(2); return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--mode": _mode = args[i + 1]; i += 1
			"--seed": _seed = int(args[i + 1]); i += 1
			"--reps": _reps = int(args[i + 1]); i += 1
			"--sync": _sync = true
			"--grid":
				var p := args[i + 1].split("x")
				_grid = Vector2i(int(p[0]), int(p[1])); i += 1
			_:
				printerr("unknown arg ", args[i]); get_tree().quit(2); return
		i += 1
	DisplayServer.window_set_size(Vector2i(1600, 1000))
	await get_tree().process_frame
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_br = _app.bridge
	_vh = _app.viewport
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if not _br.has_method("_warm_wanted") or not _br.world_gen.has_method("warm_render_caches"):
		print("PROBE-RESULT: ABORT this shell or extension has no render warm")
		get_tree().quit(2); return
	match _mode:
		"time":
			_br.prewarm_render = not _sync
			for r in _reps:
				await _timed(r)
		"identity":
			await _identity()
		"stale":
			await _stale()
		_:
			printerr("unknown mode ", _mode); get_tree().quit(2); return
	print("PROBE-RESULT: %s fails=%d" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(1 if _fails > 0 else 0)

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fails += 1
	print("  %s %s   got=%s want=%s" % ["ok  " if good else "FAIL", name, got, want])

## Generates through the real shell and returns once the first frame with the
## new world has been drawn.
func _generate(sea: float) -> bool:
	var done := [false, false]
	_br.generation_finished.connect(func(ok: bool): done[0] = true; done[1] = ok, CONNECT_ONE_SHOT)
	_br.generate({"seed": _seed, "width_km": 1200.0, "grid_w": _grid.x, "grid_h": _grid.y,
		"archetype": "", "villages": true, "sea_level": sea})
	while not done[0]:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	return done[1]

func _timed(rep: int) -> void:
	await get_tree().process_frame
	_gaps = []
	_last_us = Time.get_ticks_usec()
	var t0 := _last_us
	_tracking = true
	var ok := await _generate(0.42)
	var t_drawn := Time.get_ticks_usec()
	var t_end := t_drawn + 1000000
	while Time.get_ticks_usec() < t_end:
		await get_tree().process_frame
	_tracking = false
	var g := _gaps.duplicate()
	g.sort()
	g.reverse()
	var top := []
	for k in mini(4, g.size()):
		top.append("%.0f" % (g[k] / 1000.0))
	print("GENASYNC_RESULT rep=%d grid=%dx%d warm=%s ok=%s max_gap_ms=%.1f top_gaps_ms=%s frames=%d to_drawn_ms=%.0f" % [
		rep, _grid.x, _grid.y, str(not _sync), str(ok), g[0] / 1000.0, ",".join(top), _gaps.size(),
		(t_drawn - t0) / 1000.0])

## Every texture the first repaint makes, plus the drawn water classification.
func _hashes() -> Dictionary:
	return {"map": _sha(_vh.map_view.texture), "river": _sha(_br.river_color_texture()),
		"mask": _sha(_br.river_water_mask()), "shore": _sha(_br.shore_field_texture()),
		"water": String(_br.world_gen.drawn_water_digest().get("fnv1a", "none"))}

func _identity() -> void:
	print("\n== identity: seed %d grid %dx%d ==" % [_seed, _grid.x, _grid.y])
	_br.prewarm_render = false
	_ok("synchronous generate succeeded", await _generate(0.42), true)
	var sync_h := _hashes()
	var sync_vs: Dictionary = _br.world_gen.valley_shade_stats()
	_br.prewarm_render = true
	_ok("warmed generate succeeded", await _generate(0.42), true)
	var warm_h := _hashes()
	var warm_vs: Dictionary = _br.world_gen.valley_shade_stats()
	print("  sync  ", sync_h, "\n  warm  ", warm_h)
	for k in sync_h:
		_ok("%s identical warmed vs synchronous" % k, warm_h[k], sync_h[k])
	_ok("textures are real (not 'none')", sync_h["map"] != "none" and sync_h["shore"] != "none" and sync_h["water"] != "none", true)
	_ok("sync run built the valley shade on the main thread", sync_vs.get("main_thread"), true)
	_ok("warmed run built the valley shade off the main thread", warm_vs.get("main_thread"), false)

func _stale() -> void:
	print("\n== stale: seed %d grid %dx%d ==" % [_seed, _grid.x, _grid.y])
	## (1) A mid-job sea move is refused.
	_br.prewarm_render = true
	var done := [false]
	_br.generation_finished.connect(func(_ok_g: bool): done[0] = true, CONNECT_ONE_SHOT)
	_br.generate({"seed": _seed, "width_km": 1200.0, "grid_w": _grid.x, "grid_h": _grid.y,
		"archetype": "", "villages": true, "sea_level": 0.42})
	await get_tree().process_frame
	_ok("a generate is running", _br.generating, true)
	_ok("sea_level_live mid-generate is refused", _br.sea_level_live(0.5), {})
	while not done[0]:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	var at42 := _hashes()
	## (2a) Reference: the synchronous repaint at 0.5 of the same world.
	_br.prewarm_render = false
	await _generate(0.42)
	_ok("same world regenerated (sync) draws the same 0.42 map", _hashes()["map"], at42["map"])
	var r: Dictionary = _br.sea_level_live(0.5)
	_ok("sea move accepted on an idle world", r.get("ok", false), true)
	_vh.map_view.texture = _br.color_texture()
	var ref50 := _hashes()
	_ok("positive control: sea 0.5 draws a different map than 0.42", ref50["map"] != at42["map"], true)
	_ok("positive control: sea 0.5 draws a different shore field", ref50["shore"] != at42["shore"], true)
	## (2b) Warm at 0.42, move to 0.5, repaint: must equal the reference.
	await _generate(0.42)
	_ok("warm at 0.42 ran", _br.world_gen.warm_render_caches(), true)
	_br.sea_level_live(0.5)
	_vh.map_view.texture = _br.color_texture()
	var got50 := _hashes()
	print("  ref50 ", ref50, "\n  got50 ", got50)
	for k in ref50:
		_ok("%s after a warm then a sea move equals the fresh build" % k, got50[k], ref50[k])
	## (3) A warm left over from ANOTHER world. The sea move above cannot catch
	## a missing key check on its own: `sea_level_live` re-floods the
	## classification cache itself, and an inland lake's fill level does not
	## depend on the sea, so a stale fill moves no byte there. A whole world
	## changing under the warm does: world B is drawn once clean (reference),
	## then world A is warmed by hand and B generated over it without a warm.
	## Measured 2026-10-10: with both key checks removed, B was drawn with A's
	## water and this leg failed; with them, it passes.
	var seed_a := _seed
	_seed = seed_a + 1
	_br.prewarm_render = false
	await _generate(0.42)
	var ref_b := _hashes()
	_seed = seed_a
	await _generate(0.42)
	_ok("world A warmed by hand", _br.world_gen.warm_render_caches(), true)
	_seed = seed_a + 1
	await _generate(0.42)
	var got_b := _hashes()
	_seed = seed_a
	_ok("positive control: worlds A and B draw different maps", ref_b["map"] != at42["map"], true)
	for k in ref_b:
		_ok("%s of world B equals its clean build despite A's leftover warm" % k, got_b[k], ref_b[k])

## sha256 of a texture's pixel bytes (prefix), `"none"` for a null texture.
func _sha(t: Texture2D) -> String:
	if t == null:
		return "none"
	var img := t.get_image()
	if img == null:
		return "no-image"
	var ctx := HashingContext.new()
	ctx.start(HashingContext.HASH_SHA256)
	ctx.update(img.get_data())
	return ctx.finish().hex_encode().substr(0, 16)
