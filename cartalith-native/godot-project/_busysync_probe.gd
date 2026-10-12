extends Node
## Protects: the busy overlay around the SYNCHRONOUS main-thread actions (2026-10-12) --
## `busy_overlay.gd::busy_run` and every call site wired through `DccApp.busy_run`. Pins that the
## card is visible, painted and blocking DURING the blocked call (state read from inside the call
## through `busy_run`'s `work_started` / `work_finished` hooks and through the bridge's own
## `project_saved` / `world_loaded`, which fire inside the engine call), that input pushed during
## the call and input arriving right after it (deferred out of the work) is swallowed, that the
## overlay is gone and the depth counter is 0 after every exit path -- success, early return,
## failure, a runtime script error inside the work, an invalid callable -- and that no engine panic
## is logged. Also measures how long each wired action blocks and how long `jp_compute` takes (the
## journey planner was deliberately NOT wired; this is the number behind that).
##
## Boots the REAL shell in a SubViewport (the `_busyov_probe.gd` pattern) and drives the real
## handlers: File ▸ Save (`menus._on_file`), Recent worlds (`menus._on_recent_world`),
## `open_project_busy`, the Data manager's `_busy` export runner, `world_workspace._on_bake_all` /
## `_run_erode_busy`, `civilization_workspace._recompute_civ` / `_populate_world_now`,
## `infrastructure_workspace._generate_roads`, `menus._refine_current_view_busy`.
## `_populate_world_now` is called past its confirm and the Data manager exports past their file
## picker -- DISCLOSED setup: a synthetic click cannot reach an embedded dialog (MISTAKES.md).
##
## Run WINDOWED from a scratch copy, never headless (pixel reads):
##   godot --path <copy> res://_busysync_probe.tscn -- [--vp WxH] [--grid WxH]
## Exit 0 = every check held; 1 = a check failed; 2 = could not run.

var app: Node
var bridge: Node
var ov = null
var _vp: SubViewport
var _checks := 0
var _fail := 0
var _grid := Vector2i(512, 320)

## Counts engine panics and runtime SCRIPT errors from the log (the busyov probe's logger).
class PanicLogger extends Logger:
	var hits := 0
	var script_errors := 0
	var lines: PackedStringArray = PackedStringArray()
	func _log_error(function: String, _file: String, _line: int, code: String, rationale: String, _editor_notify: bool, error_type: int, _bt: Array[ScriptBacktrace]) -> void:
		var s := "%s %s %s" % [function, code, rationale]
		if "bind() failed" in s or "cannot borrow" in s or "panicked" in s or "already bound" in s or "already mutably" in s:
			hits += 1
			if lines.size() < 8:
				lines.append(s.substr(0, 160))
		if error_type == 2:
			script_errors += 1
			if lines.size() < 12:
				lines.append("SCRIPT " + s.substr(0, 160))
	func _log_message(_m: String, _e: bool) -> void:
		pass

## Counts every input callback a node inside the app receives: zero while the overlay blocks.
class Sentinel extends Node:
	var n := 0
	func _input(_e: InputEvent) -> void:
		n += 1
	func _shortcut_input(_e: InputEvent) -> void:
		n += 1
	func _unhandled_input(_e: InputEvent) -> void:
		n += 1

var _pl: PanicLogger
var _sent: Sentinel
var _rec: Dictionary = {}
var _effect: Callable = Callable()
var _before_img: Image = null
var _runs := 0


func _log(s: String) -> void:
	print("[busysync] %s" % s)


func _check(ok: bool, what: String) -> void:
	_checks += 1
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])


func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	return String(args[i + 1]) if i >= 0 and i + 1 < args.size() else dflt


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


## True only when `ImageTexture.update` really uploads (it is a no-op headless).
func _windowed() -> bool:
	var img := Image.create_empty(2, 2, false, Image.FORMAT_RGBA8)
	img.fill(Color(0, 0, 0, 0))
	var t := ImageTexture.create_from_image(img)
	img.set_pixel(1, 1, Color(1, 0, 0, 1))
	t.update(img)
	return t.get_image().get_pixel(1, 1).a > 0.0


func _capture() -> Image:
	var img := _vp.get_texture().get_image()
	if img != null and img.get_format() != Image.FORMAT_RGBA8:
		img.convert(Image.FORMAT_RGBA8)
	return img


## Mean absolute RGB difference (0..255) of two captures inside `r`, sampled every 4 px.
func _diff(a: Image, b: Image, r: Rect2) -> float:
	if a == null or b == null:
		return -1.0
	var sum := 0.0
	var n := 0
	var x0 := int(clampf(r.position.x, 0, a.get_width() - 1))
	var y0 := int(clampf(r.position.y, 0, a.get_height() - 1))
	var x1 := int(clampf(r.end.x, 0, a.get_width() - 1))
	var y1 := int(clampf(r.end.y, 0, a.get_height() - 1))
	for y in range(y0, y1, 4):
		for x in range(x0, x1, 4):
			var ca := a.get_pixel(x, y)
			var cb := b.get_pixel(x, y)
			sum += (absf(ca.r - cb.r) + absf(ca.g - cb.g) + absf(ca.b - cb.b)) / 3.0 * 255.0
			n += 1
	return sum / float(maxi(1, n))


func _map_point() -> Vector2:
	return (app.viewport.get_global_rect() as Rect2).get_center()


## Four events at the map: a left click and a wheel notch. Returns how many were pushed.
func _push_small() -> int:
	var mp := _map_point()
	for b in [MOUSE_BUTTON_LEFT, MOUSE_BUTTON_WHEEL_UP]:
		for pressed in [true, false]:
			var e := InputEventMouseButton.new()
			e.button_index = b
			e.pressed = pressed
			e.position = mp
			e.global_position = mp
			_vp.push_input(e, true)
	return 4


## `work_started`: the blocked call is about to run. Records the overlay's state, the action's
## effect so far, the frame on screen, and pushes input that must reach nothing.
func _on_started(label: String) -> void:
	_runs += 1
	var s0 := _sent.n
	var w0: int = ov.swallowed
	_rec = {"label": label, "vis": ov.visible, "block": ov.is_blocking(), "depth": ov.depth(),
		"cat": ov.message_category(), "t0": Time.get_ticks_usec(),
		"eff0": _effect.call() if _effect.is_valid() else null}
	var dmw = app.get("data_manager_window")
	_rec["dm_muted"] = dmw != null and dmw.visible and dmw.gui_disable_input
	_rec["mirrors"] = ov.mirror_count() if ov.has_method("mirror_count") else -1
	var img := _capture()
	_rec["card_diff"] = _diff(_before_img, img, ov.layout()["card"]) if _before_img != null else -1.0
	_rec["img"] = img
	var pushed := _push_small()
	_rec["pushed"] = pushed
	_rec["leaked"] = _sent.n - s0
	_rec["swallowed"] = ov.swallowed - w0


## `work_finished`: the blocked call returned. Records the effect and the duration.
func _on_finished(label: String) -> void:
	_rec["end_label"] = label
	_rec["ms"] = float(Time.get_ticks_usec() - int(_rec.get("t0", 0))) / 1000.0
	_rec["eff1"] = _effect.call() if _effect.is_valid() else null
	_rec["vis1"] = ov.visible and ov.is_blocking()


## Arms the recorder for one action: `effect` is read at start and end of the blocked call.
func _arm(effect: Callable) -> void:
	_rec = {}
	_effect = effect
	await RenderingServer.frame_post_draw
	_before_img = _capture()


## The common assertions for one wired action, after its handler has returned and settled.
func _assert_run(name: String, label: String, category: String, want_effect: bool = true) -> void:
	await _frames(4)
	_check(String(_rec.get("label", "")) == label and String(_rec.get("end_label", "")) == label,
		"%s: busy_run ran with label '%s' (got '%s')" % [name, label, _rec.get("label", "<never ran>")])
	_check(bool(_rec.get("vis", false)) and bool(_rec.get("block", false)) and bool(_rec.get("vis1", false)),
		"%s: overlay visible AND blocking at the start and the end of the blocked call" % name)
	_check(String(_rec.get("cat", "")) == category, "%s: message category '%s' (got '%s')" % [name, category, _rec.get("cat", "")])
	_check(int(_rec.get("leaked", -1)) == 0 and int(_rec.get("swallowed", 0)) >= int(_rec.get("pushed", 1)),
		"%s: %d events pushed inside the call, %d swallowed, %d reached the app" % [name, int(_rec.get("pushed", 0)), int(_rec.get("swallowed", 0)), int(_rec.get("leaked", -1))])
	if want_effect:
		_check(_rec.get("eff0") != _rec.get("eff1"), "%s: the action's effect happened INSIDE the blocked call (%s -> %s)" % [name, str(_rec.get("eff0")), str(_rec.get("eff1"))])
	_check(not ov.visible and not ov.is_blocking() and ov.depth() == 0, "%s: afterwards hidden, not blocking, depth 0 (depth=%d)" % [name, ov.depth()])
	_log("  info %s blocked %.1f ms; card-rect frame diff vs before %.1f" % [name, float(_rec.get("ms", -1.0)), float(_rec.get("card_diff", -1.0))])


func _png(bytes: PackedByteArray) -> Image:
	if bytes.is_empty():
		return null
	var img := Image.new()
	if img.load_png_from_buffer(bytes) != OK:
		return null
	img.convert(Image.FORMAT_RGBA8)
	return img


## Waits for the armed run's `work_finished` (bounded: a call site that never reaches `busy_run`
## must FAIL its checks, not hang the probe), then returns.
func _await_finished(max_frames: int = 1800) -> void:
	var f := 0
	while not _rec.has("end_label") and f < max_frames:
		await _frames(1)
		f += 1


func _hint() -> String:
	var labels = app.get("_status_labels")
	if labels is Dictionary and (labels as Dictionary).has("hint"):
		return String((labels["hint"] as Label).text)
	return ""


func _ready() -> void:
	if not _windowed():
		_log("ABORT run WINDOWED")
		get_tree().quit(2)
		return
	var g: PackedStringArray = _arg("--grid", "512x320").split("x")
	_grid = Vector2i(int(g[0]), int(g[1]))
	_pl = PanicLogger.new()
	OS.add_logger(_pl)
	var parts: PackedStringArray = _arg("--vp", "1600x900").split("x")
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	var picker = app.get("phone_project_picker")
	if picker != null and picker.visible:
		picker.hide()
	await _frames(6)
	bridge = app.bridge
	ov = app.get("busy_overlay")
	_log("user dir %s; DLL mtime %d" % [ProjectSettings.globalize_path("user://"),
		FileAccess.get_modified_time(ProjectSettings.globalize_path("res://").path_join("../target/debug/cartalith_godot.dll"))])
	if ov == null or not ov.has_signal("work_started"):
		_log("ABORT the shell has no busy overlay with busy_run hooks -- stale shell copy")
		get_tree().quit(2)
		return
	ov.work_started.connect(_on_started)
	ov.work_finished.connect(_on_finished)
	_sent = Sentinel.new()
	app.add_child(_sent)
	bridge.generate({"seed": 4242, "width_km": 1200.0, "grid_w": _grid.x, "grid_h": _grid.y, "sea_level": 0.5, "villages": true})
	var f := 0
	while bridge.generating and f < 6000:
		await _frames(1)
		f += 1
	await _frames(30)
	_check(bridge.has_world and not ov.visible and ov.depth() == 0, "setup: a %dx%d world, overlay idle" % [_grid.x, _grid.y])
	await _leg_helper()
	await _leg_journey_timing()
	await _leg_civ()
	await _leg_erode()
	await _leg_export()
	await _leg_bake()
	await _leg_save_open()
	await _frames(10)
	_check(not ov.visible and not ov.is_blocking() and ov.depth() == 0 and ov.watchdog_releases == 0,
		"end: hidden, not blocking, depth 0, watchdog never fired (%d)" % ov.watchdog_releases)
	var s0 := _sent.n
	_push_small()
	await _frames(2)
	_check(_sent.n - s0 > 0, "positive control: with the overlay hidden the same events DO reach the app (%d)" % (_sent.n - s0))
	_log("RESULT busysync checks=%d fail=%d engine-panic-hits=%d script-errors=%d (1 expected: the deliberate error leg)" % [_checks, _fail, _pl.hits, _pl.script_errors])
	if not _pl.lines.is_empty():
		_log("logged: %s" % str(_pl.lines))
	get_tree().quit(1 if _fail > 0 or _pl.hits > 0 or _pl.script_errors != 1 else 0)


# ------------------------------------------------------------------ H: the helper's contract ----

func _leg_helper() -> void:
	_log("== H busy_run contract ==")
	# H1 pass-through, state inside, the card painted before the work.
	await _arm(Callable())
	var inside := {}
	var r = await app.busy_run("saving project…", func():
		inside["vis"] = ov.visible
		inside["block"] = ov.is_blocking()
		inside["depth"] = ov.depth()
		inside["img"] = _capture()
		return 42)
	_check(r == 42, "H1 busy_run returns work's value (%s)" % str(r))
	_check(bool(inside.get("vis")) and bool(inside.get("block")) and int(inside.get("depth", -1)) == 1,
		"H1 inside the work: visible, blocking, depth 1 (%s)" % str([inside.get("vis"), inside.get("block"), inside.get("depth")]))
	var cd := _diff(_before_img, inside.get("img"), ov.layout()["card"])
	_check(cd > 20.0, "H1 the card was PAINTED before the work ran: card-rect diff %.1f > 20 (0 = nothing drawn)" % cd)
	await _frames(2)
	_check(not ov.visible and ov.depth() == 0, "H1 hidden and depth 0 afterwards")
	# H2 an early return.
	r = await app.busy_run("exporting…", func():
		if true:
			return
		return 1)
	_check(r == null and not ov.visible and ov.depth() == 0, "H2 early return: null, hidden, depth 0")
	# H3 a runtime script error inside the work (the one expected SCRIPT ERROR).
	var se0 := _pl.script_errors
	var holder := {"o": null}
	r = await app.busy_run("baking atlas…", func():
		var o: Object = holder["o"]
		return o.get_class())
	_check(_pl.script_errors == se0 + 1, "H3 the deliberate script error was logged (%d)" % (_pl.script_errors - se0))
	_check(r == null and not ov.visible and not ov.is_blocking() and ov.depth() == 0, "H3 a script error inside the work still hides: null, hidden, depth 0")
	# H4 an invalid callable.
	r = await app.busy_run("exporting…", Callable())
	_check(r == null and ov.depth() == 0 and not ov.visible, "H4 invalid callable: null, depth 0, hidden")
	# H5 input arriving right after the freeze (deferred out of the work) is swallowed.
	var s0 := _sent.n
	var w0: int = ov.swallowed
	await app.busy_run("exporting…", func(): call_deferred("_push_small"))
	await _frames(3)
	_check(_sent.n == s0 and ov.swallowed - w0 >= 4, "H5 input delivered right after the work returns is swallowed (reached %d, swallowed %d)" % [_sent.n - s0, ov.swallowed - w0])
	# H6 nesting under another source.
	ov.show_busy("working…")
	await app.busy_run("exporting…", func(): return 0)
	_check(ov.depth() == 1 and ov.visible, "H6 nested under another source: depth back to 1, still visible")
	ov.hide_busy()
	await _frames(2)
	_check(ov.depth() == 0 and not ov.visible, "H6 the outer hide releases it")


# ---------------------------------------------------------------- J: journey planner timing ----

func _leg_journey_timing() -> void:
	_log("== J jp_compute timing (not wired; this is why) ==")
	var towns: Array = bridge.settlements()
	if towns.size() < 2:
		_check(false, "J needs two settlements (got %d)" % towns.size())
		return
	var far := 1
	var best := 0.0
	for i in range(1, towns.size()):
		var d := Vector2(float(towns[0]["x"]), float(towns[0]["y"])).distance_to(Vector2(float(towns[i]["x"]), float(towns[i]["y"])))
		if d > best:
			best = d
			far = i
	bridge.route_begin("mixed")
	bridge.route_append_stop(float(towns[0]["x"]), float(towns[0]["y"]))
	bridge.route_append_stop(float(towns[far]["x"]), float(towns[far]["y"]))
	var ridx: int = bridge.route_commit()
	var ms: Array = []
	var ok := true
	for k in 5:
		var t0 := Time.get_ticks_usec()
		var r: Dictionary = bridge.jp_compute({"route": ridx})
		ms.append(float(Time.get_ticks_usec() - t0) / 1000.0)
		ok = ok and bool(r.get("ok", false))
	ms.sort()
	_check(ok, "J jp_compute planned the longest route from town 0 (%.0f cells)" % best)
	_log("  info jp_compute ms over 5 runs: median %.2f (%.2f..%.2f)" % [ms[2], ms[0], ms[4]])


# ------------------------------------------------------------------------------- civ legs ----

func _civ_ws() -> Node:
	for w in app._workspaces:
		if w.has_method("_recompute_civ"):
			return w
	return null


func _leg_civ() -> void:
	_log("== C civilisation ==")
	var ws := _civ_ws()
	if ws == null:
		_check(false, "C found the civilisation workspace")
		return
	await _arm(func(): return _hint())
	await ws._recompute_civ()
	await _assert_run("C recompute", "recomputing civilisation…", "civilisation", false)
	_check(_hint().begins_with("Recomputed") or _hint().begins_with("Not recomputed"), "C recompute: the outcome reached the status line after (%s)" % _hint().substr(0, 60))
	await _arm(func(): return bridge.settlements().size())
	await ws._populate_world_now()
	await _assert_run("C populate", "populating world…", "civilisation", false)
	_check(_hint().begins_with("Populated"), "C populate: outcome on the status line (%s)" % _hint().substr(0, 60))
	var infra = ws.get("_infra")
	if infra == null:
		_check(false, "C found the infrastructure workspace")
		return
	await _arm(func(): return bridge.roads().size())
	await infra._generate_roads()
	await _assert_run("C roads", "generating roads…", "civilisation", false)
	_check(_hint().begins_with("Network rebuilt") or _hint().begins_with("No roads"), "C roads: outcome on the status line (%s)" % _hint().substr(0, 60))


# ------------------------------------------------------------------------------- erosion ----

func _leg_erode() -> void:
	_log("== E erosion ==")
	var ws = app._world_workspace()
	await _arm(func(): return _hint().begins_with("Eroded"))
	app.set_status("hint", "", "text_dim")
	await ws._run_erode_busy()
	await _assert_run("E erode", "erosion pass…", "erode")
	# The status bar's stale-stage Recompute: the erode above left the derived stages stale.
	if bridge.world_gen.has_method("recompute_stale_stages"):
		await _arm(Callable())
		await app._recompute_stale()
		await _assert_run("E stale-stage Recompute", "finishing…", "validation", false)
	else:
		_check(false, "E this DLL has no recompute_stale_stages -- stale binary")
	var shot: Image = _rec.get("img")
	if shot != null:
		var sp := ProjectSettings.globalize_path("res://").path_join("../busysync_erode_inside_call.png")
		shot.save_png(sp)
		_log("  shot %s (the frame on screen while the erosion call blocks)" % sp)


# --------------------------------------------------------------------------------- export ----

func _leg_export() -> void:
	_log("== X export ==")
	var dm = app.data_manager_window
	var hp := ProjectSettings.globalize_path("user://busysync_height.png")
	# A FAILING export: 1024 px is not an offered width, so the engine refuses inside the work.
	dm.set("_wd_width", 1024)
	DirAccess.remove_absolute(hp)
	await _arm(func(): return FileAccess.file_exists(hp))
	dm._busy("exporting heightmap…", dm._run_heightmap_export.bind(hp))
	await _await_finished()
	await _assert_run("X heightmap (engine refuses 1024 px)", "exporting heightmap…", "export", false)
	_check(not FileAccess.file_exists(hp) and _hint().findn("failed") >= 0, "X the refusal reached the status line and wrote nothing (%s)" % _hint().substr(0, 60))
	dm.set("_wd_width", 2048)
	await _arm(func(): return FileAccess.file_exists(hp))
	dm._busy("exporting heightmap…", dm._run_heightmap_export.bind(hp))
	await _await_finished()
	await _assert_run("X heightmap 2048", "exporting heightmap…", "export")
	# The same run with the Data manager OPEN -- where a real export is pressed from.
	app.open_data_manager()
	await _frames(10)
	var dmr := Rect2(Vector2(dm.position), Vector2(dm.size))
	var card: Rect2 = ov.layout()["card"]
	_log("  info Data manager rect %s in a %s viewport, card rect %s, card inside window: %s" % [str(dmr), str(_vp.size), str(card), str(dmr.encloses(card))])
	DirAccess.remove_absolute(hp)
	await _arm(func(): return FileAccess.file_exists(hp))
	dm._busy("exporting heightmap…", dm._run_heightmap_export.bind(hp))
	await _await_finished()
	await _frames(4)
	_check(bool(_rec.get("vis", false)) and bool(_rec.get("block", false)) and bool(_rec.get("dm_muted", false)),
		"X (window open): overlay up and blocking, and the Data manager's input muted during the call")
	_check(int(_rec.get("leaked", -1)) == 0, "X (window open): events pushed during the call reach neither the app nor the window (%d leaked; the muted window absorbs them, overlay swallowed %d)" % [int(_rec.get("leaked", -1)), int(_rec.get("swallowed", 0))])
	_check(_rec.get("eff0") != _rec.get("eff1") and not ov.visible and ov.depth() == 0 and not dm.gui_disable_input,
		"X (window open): exported inside the call; afterwards hidden, depth 0, window input restored")
	var img: Image = _rec.get("img")
	var outside := Rect2(Vector2(4, 4), Vector2(_vp.size.x - 8, maxf(8.0, dmr.position.y - 8.0)))
	_log("  info with the window open: card-rect diff %.1f (0 = no card visible); strip above the window diff %.1f"
		% [_diff(_before_img, img, card), _diff(_before_img, img, outside)])
	# The card above the embedded window (2026-10-12, `busy_overlay.gd::_WindowMirror`): the card rect lies
	# INSIDE the window, so a diff of 0.0 meant the card was hidden under it. Mutation: stop `_mute_subwindows`
	# creating the mirror and the diff returns to 0.0 and this goes red. The threshold (4.0) is a labelled
	# judgement: the card-rect diff measured 10.3 with a mirror (1600x900) and exactly 0.0 without, so anything between works.
	_check(dmr.encloses(card) and int(_rec.get("mirrors", 0)) == 1 and _diff(_before_img, img, card) > 4.0,
		"X (window open): the busy card is VISIBLE above the Data manager (mirrors %d, card-rect diff %.1f > 4.0)" % [int(_rec.get("mirrors", 0)), _diff(_before_img, img, card)])
	await _frames(3)
	_check(ov.mirror_count() == 0 and dm.find_child("BusyOverlayMirror", true, false) == null,
		"X (window open): afterwards the window carries no mirror")
	dm.hide()
	await _frames(4)


# ----------------------------------------------------------------------------------- bake ----

func _leg_bake() -> void:
	_log("== B bake ==")
	var ws = app._world_workspace()
	if bridge.is_finalized():
		bridge.set_finalized(false)
	# The atlas store is on disk and keyed by world: a previous run's chunks would make the bake a
	# skip-everything no-op, so start from an empty store (setup, disclosed).
	bridge.atlas_clear()
	await _arm(func(): return int(bridge.atlas_status().get("chunks", 0)))
	await ws._on_bake_all()
	await _assert_run("B bake all", "baking atlas…", "bake")
	# Atlas export through the menu's lifted writer.
	var ap := ProjectSettings.globalize_path("user://busysync_atlas.zip")
	DirAccess.remove_absolute(ap)
	await _arm(func(): return FileAccess.file_exists(ap))
	await app.busy_run("exporting atlas cache…", app.menus._write_atlas_export.bind(ap))
	await _assert_run("B atlas export", "exporting atlas cache…", "export")
	bridge.set_finalized(false)
	# Refine at a fitted view: the pyramid is not up, so the gate must NOT flash the card.
	var runs0 := _runs
	app.set_status("hint", "", "text_dim")
	await app.menus._refine_current_view_busy()
	await _frames(3)
	_check(_runs == runs0 and ov.depth() == 0 and _hint().findn("refine") >= 0,
		"B refine at a fitted view: no card, the refusal on the status line (%s)" % _hint().substr(0, 60))


# ------------------------------------------------------------------------------ save/open ----

func _leg_save_open() -> void:
	_log("== S save / O open ==")
	var p := ProjectSettings.globalize_path("user://busysync_a.ctl")
	DirAccess.remove_absolute(p)
	app.current_project_path = p
	var saved_inside := {}
	var on_saved := func(_path: String): saved_inside["vis"] = ov.visible and ov.is_blocking()
	bridge.project_saved.connect(on_saved)
	await _arm(func(): return FileAccess.file_exists(p))
	app.menus._on_file(DccMenus.ID_SAVE)
	await _await_finished()
	await _assert_run("S File > Save", "saving project…", "save")
	bridge.project_saved.disconnect(on_saved)
	_check(bool(saved_inside.get("vis", false)), "S project_saved fired inside the call with the overlay up")
	_check(_hint().begins_with("saved"), "S status line says saved (%s)" % _hint().substr(0, 40))
	# The thumbnail must be the map, not the card. Control: a preview captured WITH the card up.
	await RenderingServer.frame_post_draw
	var ref := _png(app._capture_preview_png())
	ov.show_busy("saving project…")
	await _frames(2)
	await RenderingServer.frame_post_draw
	var carded := _png(app._capture_preview_png())
	ov.hide_busy()
	await _frames(2)
	var saved: Image = null
	var zr := ZIPReader.new()
	if zr.open(p) == OK:
		if zr.get_files().has("preview.png"):
			saved = _png(zr.read_file("preview.png"))
		zr.close()
	var full := Rect2(Vector2.ZERO, Vector2(ref.get_size()) if ref != null else Vector2.ONE)
	var d_saved := _diff(ref, saved, full) if saved != null and ref != null and saved.get_size() == ref.get_size() else -1.0
	var d_card := _diff(ref, carded, full) if carded != null and ref != null and carded.get_size() == ref.get_size() else -1.0
	_check(d_saved >= 0.0 and d_saved < 3.0 and d_card > 10.0,
		"S the saved preview.png is the map, not the card: diff to a card-free capture %.1f (< 3), card-up control %.1f (> 10)" % [d_saved, d_card])
	# A failing save: an unwritable path.
	var bad := "Q:/no_such_drive_busysync/x.ctl"
	app.current_project_path = bad
	await _arm(Callable())
	app.menus._on_file(DccMenus.ID_SAVE)
	await _frames(8)
	_check(not ov.visible and not ov.is_blocking() and ov.depth() == 0, "S failing save: hidden, not blocking, depth 0 (hint: %s)" % _hint().substr(0, 50))
	app.current_project_path = p
	# Open through Recent worlds.
	var recents: Array = DccSettings.recent_projects()
	var idx := recents.find(p)
	_check(idx >= 0, "O the save is in Recent worlds (index %d)" % idx)
	var loaded_inside := {}
	var on_loaded := func(): loaded_inside["vis"] = ov.visible and ov.is_blocking()
	bridge.world_loaded.connect(on_loaded)
	await _arm(func(): return loaded_inside.has("vis"))
	app.menus._on_recent_world(idx)
	await _await_finished()
	await _assert_run("O Recent worlds", "opening project…", "load")
	bridge.world_loaded.disconnect(on_loaded)
	_check(bool(loaded_inside.get("vis", false)), "O world_loaded fired inside the call with the overlay up")
	# A failing open.
	var ok: bool = await app.open_project_busy(ProjectSettings.globalize_path("user://no_such_busysync.ctl"))
	await _frames(3)
	_check(not ok and not ov.visible and ov.depth() == 0, "O failing open: false, hidden, depth 0")
