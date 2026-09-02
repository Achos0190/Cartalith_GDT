extends Node
## INDEPENDENT adversarial verification of the 2026-09-01 POI fix.
## Written by the verifying agent, not the fixing agent. Deliberately does NOT
## reuse `_poifreeze_probe.gd`'s assertions; it exercises the paths that probe
## does not:
##
##   A. the REAL button press (`_lm_run_btn.pressed.emit()`), not `_lm_run()`
##      called by name.
##   B. the refusal path -- pressing Run while `bridge.generating` is true.
##   C. a double press (the button is disabled, but the signal is emitted twice
##      anyway to prove the bridge itself refuses re-entry rather than starting
##      a second Thread over a mutably-borrowed WorldGen).
##   D. mid-pass engine reads that a live, responsive UI now makes reachable and
##      a frozen one did not: `get_factions()` (UNGUARDED on `generating`),
##      `param_get()`, `landmarks()`, `ViewportHost.refresh_annotations()`.
##      If any of these panics the process dies here and that is the finding.
##   E. `landmark_last_run()` directly, before any pass and after a regenerate.
##   F. that the rings map to on-screen pixels (`_cell_to_screen` finite).

const WATCHDOG_S := 900.0
var _app: Node
var _gw := 512
var _gh := 384
var _fails := 0
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
	print("=== _poiverify_probe  grid %dx%d ===" % [_gw, _gh])

	var wd := Timer.new()
	wd.wait_time = WATCHDOG_S
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT"); get_tree().quit(2))
	add_child(wd)
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await _frames(6)
	var bridge = _app.bridge
	if bridge == null:
		print("[FATAL] app.bridge null"); get_tree().quit(1); return

	# ── E1: last_run before ANY pass ─────────────────────────────────────────
	var pre: Dictionary = bridge.world_gen.landmark_last_run()
	print("[E1] landmark_last_run() with no world at all: ", pre)
	_ok("E1 reports not-run rather than a hollow ok", pre.get("ok"), false)

	# ── B: press Run WHILE a generate is in flight ───────────────────────────
	print("[B] starting a generate, then pressing Run mid-generate")
	bridge.generate({"seed": 771002, "width_km": 2400.0, "grid_w": _gw,
		"grid_h": _gh, "archetype": "", "villages": true, "sea_level": 0.45})
	await _frames(2)
	_ok("B generate is actually in flight", bridge.generating, true)
	## THE CONTROL for [D] below. If these same unguarded reads already panic
	## during a plain `generate()`, the landmark pass's version of it is an
	## INHERITED condition, not a regression this fix introduced.
	print("[D0] CONTROL -- the same unguarded reads during a plain generate():")
	print("     get_factions() -> %d" % (bridge.get_factions() as Array).size())
	print("     grid_size() -> %s" % str(bridge.grid_size()))
	print("[D0] control done")
	var civ: Node = _app.find_child("CivilizationWorkspace", true, false)
	if civ == null:
		print("[FATAL] no CivilizationWorkspace"); get_tree().quit(1); return
	## Force the Landmarks panel to exist so `_lm_run_btn` is real.
	_app.select_domain_category("civilization", "Landmarks")
	await _frames(3)
	var btn = civ.get("_lm_run_btn")
	print("[B] _lm_run_btn present: %s  disabled=%s"
		% [btn != null and is_instance_valid(btn),
		   (btn.disabled if btn != null and is_instance_valid(btn) else "n/a")])
	_ok("B the real button exists", btn != null and is_instance_valid(btn), true)
	if btn != null and is_instance_valid(btn):
		btn.pressed.emit()          ## THE REAL CONTROL, pressed mid-generate.
		await _frames(3)
		var note_b = civ.get("_lm_run_note")
		print("[B] note after a mid-generate press: '%s'"
			% (String(note_b.text) if note_b != null else "n/a"))
		_ok("B the refusal re-enabled the button", btn.disabled, false)
		_ok("B the refusal said why", String(note_b.text).begins_with("Not run."), true)
	while bridge.generating:
		await get_tree().create_timer(0.1).timeout
	await _frames(4)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	print("[B] world up: has_world=%s" % bridge.has_world)

	# ── E2: last_run after a generate, before any pass ───────────────────────
	var mid: Dictionary = bridge.world_gen.landmark_last_run()
	print("[E2] landmark_last_run() after generate, before any pass: ", mid)
	_ok("E2 still reports not-run", mid.get("ok"), false)

	var overlay = _app.viewport.overlay
	_ok("E2 overlay holds nothing yet", (overlay.get("_landmarks") as Array).size(), 0)

	# ── A + C: the REAL button, pressed twice ────────────────────────────────
	btn = civ.get("_lm_run_btn")
	print("[A] pressing the REAL 'Run landmark pass' button")
	var t0 := Time.get_ticks_usec()
	var ticks0 := _ticks
	btn.pressed.emit()
	await _frames(1)
	_ok("A the press started a pass", bridge.generating, true)
	_ok("A the button went busy", btn.disabled, true)
	print("[C] pressing it a SECOND time while the first is in flight")
	btn.pressed.emit()             ## must not start a second Thread.
	await _frames(1)

	# ── D: mid-pass engine reads a live UI now makes reachable ───────────────
	print("[D] mid-pass reads (a panic here kills the process -- that IS the finding)")
	var f = bridge.get_factions()
	print("     get_factions() -> %d entries (UNGUARDED on `generating`)" % (f as Array).size())
	var pv = bridge.param_get("seaLevel")
	print("     param_get('seaLevel') -> %s" % str(pv))
	var lm_mid: Array = bridge.landmarks()
	print("     landmarks() mid-pass -> %d" % lm_mid.size())
	_app.viewport.refresh_annotations()
	print("     refresh_annotations() mid-pass survived")
	var gs = bridge.grid_size()
	print("     grid_size() -> %s" % str(gs))
	## The concretely NEW hazard: the fix left `_recompute_civ()` synchronous
	## and `civ_recompute()` unguarded on `generating`. Before the fix the two
	## buttons could never overlap (both blocked the main thread); now the
	## landmark pass leaves the UI live, so the sibling button is pressable
	## during it.
	print("[D-sib] pressing the SIBLING 'Recompute civilisation' path mid-pass")
	var rc: Dictionary = bridge.civ_recompute()
	print("     civ_recompute() mid-landmark-pass -> %s" % str(rc))
	## And the panel's own setters, which return false while `generating`.
	if bridge.has_method("landmark_set_cap"):
		var took = bridge.landmark_set_cap("peak", 7)
		print("     landmark_set_cap('peak',7) mid-pass -> %s (false = silently refused)" % str(took))
	print("[D] survived every mid-pass read")

	while bridge.generating:
		await get_tree().create_timer(0.05).timeout
	await _frames(6)
	var us := Time.get_ticks_usec() - t0
	var served := _ticks - ticks0
	print("[A] pass finished in %.1f ms, main loop served %d frames" % [us / 1000.0, served])
	_ok("A the main loop kept ticking through the real button press", served > 0, true)
	_ok("A the button came back", btn.disabled, false)

	var note = civ.get("_lm_run_note")
	print("[A] run note: '%s'" % String(note.text))
	_ok("A the note reports a real placement count",
		String(note.text).begins_with("Placed "), true)

	var engine_n: int = (bridge.landmarks() as Array).size()
	var ov_n: int = (overlay.get("_landmarks") as Array).size()
	print("[A] engine=%d overlay=%d visible=%s" % [engine_n, ov_n, overlay.get("_landmarks_visible")])
	_ok("A the button press alone reached the map", ov_n, engine_n)
	_ok("A something was actually placed", engine_n > 0, true)
	_ok("A the layer is visible by default", overlay.get("_landmarks_visible"), true)

	# ── F: do the rings land on real pixels? ─────────────────────────────────
	var gwo := int(overlay.get("_gw"))
	var gho := int(overlay.get("_gh"))
	print("[F] overlay grid _gw=%d _gh=%d" % [gwo, gho])
	_ok("F the overlay knows the grid", gwo > 0 and gho > 0, true)
	var bad := 0
	if gwo > 0 and gho > 0 and overlay.has_method("_cell_to_screen"):
		var rect: Rect2 = overlay.get_rect()
		for l in (overlay.get("_landmarks") as Array):
			var d: Dictionary = l
			var p: Vector2 = overlay.call("_cell_to_screen",
				Vector2(float(d.get("x", 0)), float(d.get("y", 0))), rect)
			if not (is_finite(p.x) and is_finite(p.y)):
				bad += 1
		print("[F] non-finite ring positions: %d of %d" % [bad, ov_n])
		_ok("F every ring maps to a finite pixel", bad, 0)
	else:
		print("[F] _cell_to_screen not reachable from here -- skipped")

	# ── E3: after a regenerate ───────────────────────────────────────────────
	print("[E3] regenerating")
	bridge.generate({"seed": 771003, "width_km": 2400.0, "grid_w": _gw,
		"grid_h": _gh, "archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.1).timeout
	await _frames(6)
	var post: Dictionary = bridge.world_gen.landmark_last_run()
	print("[E3] landmark_last_run() after regenerate: ", post)
	_ok("E3 a regenerate makes the last run not-ok again", post.get("ok"), false)
	_ok("E3 the rings came down",
		(overlay.get("_landmarks") as Array).size(), 0)

	## And the panel's own note after a regenerate + a fresh press.
	print("[E4] pressing Run on the NEW world")
	btn = civ.get("_lm_run_btn")
	btn.pressed.emit()
	while bridge.generating:
		await get_tree().create_timer(0.05).timeout
	await _frames(6)
	var n2: int = (bridge.landmarks() as Array).size()
	var o2: int = (overlay.get("_landmarks") as Array).size()
	print("[E4] engine=%d overlay=%d" % [n2, o2])
	_ok("E4 the second world's pass also reaches the map", o2, n2)
	_ok("E4 and it placed something", n2 > 0, true)

	print("=== SUMMARY %dx%d fails=%d ===" % [_gw, _gh, _fails])
	get_tree().quit(1 if _fails > 0 else 0)
