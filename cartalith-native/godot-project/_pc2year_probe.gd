extends Node
## PC-2 probe: one year (`PC_SHELL_PROPOSAL.md` §2 rule 2, §3.2, milestone PC-2), inside the real shell,
## with real mouse clicks through `Input.parse_input_event()`.
##
##   desktop: godot --path . --resolution 1600x1000 _pc2year_probe.tscn -- --shots <dir>
##   touch:   godot --path . --resolution 1600x1000 _pc2year_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a wiped
## `app_userdata/<name>` (`MISTAKES.md`: a probe that reads `user://` can be green for reasons that have
## nothing to do with the code). Reads `--force-touch` and `--shots <dir>`; anything else exits 2.
## The phone (393x852) is not driven: its strip is reached through the phone menu's sheet, which this
## probe does not open, and the writer under test is the same function (`DccShell.tl_set_year`).
##
## What it protects (PC-2's bar). A small world is generated (seed 4242, 256 x 192 cells), three years
## are recorded (100, 300 and 5000), and the year is then moved from THREE writers, each by a real click:
##   1. CIVIL ▸ Timeline: a year pill (300), which before PC-2 called the engine directly;
##   2. the strip: the scrub track, clicked at the x of a year (700), the writer that always emitted;
##   3. the Place editor's Timeline tab: a recorded-year chip (100), and Go to year with 9000, a value
##      outside the strip's track (the world's range, 100..5000 here since PC-3) (so the writer must not clamp it), both of which called
##      the engine directly before PC-2.
## After EACH write every surface that shows the year must agree on it: the engine and `tl_year()`; the
## strip's year labels; CIVIL's pills (the active one) and its "Active year" note; the right dock's
## Settlement Garrison row, which is read at the cursor, and which must have been rebuilt; the Place
## editor's "Cursor: year" line (while open); and the territory texture's year (`get_civ_territory_year`
## against what CIVIL last repainted). `timeline_changed` must have been emitted at least once per write.
## Controls: the same readings with NO write must not move, and the writer clamps by default but not
## with `exact`. Last, a scan of the shell's source: the only code line that calls `civ_goto_year(` outside
## the bridge's own wrapper is in `dcc_shell.gd`.
## Exit 0 pass, 1 fail, 2 bad argument, watchdog or no world.

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
var _ws: Node
var _emits := 0


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(frames: int = 4) -> void:
	for _i in frames:
		await get_tree().process_frame


func _shot(tag: String) -> void:
	if _shots == "":
		return
	await _settle(3)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if img != null:
		var path := _shots.path_join("pc2_%s_%s.png" % [_mode, tag])
		img.save_png(path)
		print("SHOT %s" % path)


func _click_at(at: Vector2) -> void:
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	mm.relative = Vector2(0, 2)
	Input.parse_input_event(mm)
	await _settle(2)
	for down in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		Input.parse_input_event(ev)
		await _settle(2)
	await _settle(6)


func _to_root(p: Vector2, c: Control) -> Vector2:
	var w := c.get_window()
	return p if w == null or w == get_tree().root else p + Vector2(w.position)


## A visible Button under `root` whose text is `text`, or null.
func _button(root: Node, text: String) -> Button:
	for b in root.find_children("*", "Button", true, false):
		if (b as Button).text == text and (b as Control).is_visible_in_tree():
			return b
	return null


func _label_with(root: Node, needle: String) -> Label:
	for l in root.find_children("*", "Label", true, false):
		if (l as Label).text.contains(needle) and (l as Control).is_visible_in_tree():
			return l
	return null


## Scrolls `c` into view in its nearest ScrollContainer, then clicks it for real.
func _click_control(c: Control) -> void:
	var sc := c.get_parent()
	while sc != null and not (sc is ScrollContainer):
		sc = sc.get_parent()
	if sc != null:
		(sc as ScrollContainer).ensure_control_visible(c)
		await _settle(4)
	await _click_at(_to_root(c.get_global_rect().get_center(), c))


## The year-keyed text the dock draws for a settlement's Garrison, as the dock would build it.
func _garrison_expected(tid: int, y: int) -> String:
	return String(CivilizationWorkspace.garrison_row(_bridge.civ_settlement_garrison(tid, y), y)["value"])


## Every surface's answer to "what year is it", as one record, read now.
func _state() -> Dictionary:
	var strip: Array = []
	for l in _app._tl_year_labels:
		if is_instance_valid(l) and (l as Label).is_visible_in_tree():
			strip.append((l as Label).text)
	var pills_active := -99999
	var active_note := ""
	if _ws._tl_body != null and is_instance_valid(_ws._tl_body):
		var n := _label_with(_ws._tl_body, "Active year: ")
		active_note = "" if n == null else n.text
		for b in _ws._tl_body.find_children("*", "Button", true, false):
			var t := (b as Button).text
			if not (t.ends_with(" AD") or t.ends_with(" BC")):
				continue
			var sb := (b as Button).get_theme_stylebox("normal")
			if sb is StyleBoxFlat and (sb as StyleBoxFlat).bg_color == DccTheme.c("accent"):
				pills_active = int(t.split(" ")[0]) * (-1 if t.ends_with("BC") else 1)
	var terr: Dictionary = _bridge.get_civ_territory_year()
	return {
		"tl_year": int(_app.tl_year()), "engine": int(_bridge.get_civ_year()),
		"strip": strip, "pill": pills_active, "note": active_note,
		"terr": int(terr["year"]) if terr.has("year") else null,
		"terr_painted": _ws._tl_last_territory_year,
		"dock_first": _dock_first_id(),
	}


func _dock_first_id() -> int:
	var body: Control = _app.right_dock_body
	return 0 if body.get_child_count() == 0 else body.get_child(0).get_instance_id()


func _dock_texts() -> Array:
	var out: Array = []
	for l in _app.right_dock_body.find_children("*", "Label", true, false):
		out.append((l as Label).text)
	return out


## Every surface must name `y` after a write by `way`; `before_dock` is the dock's first node before it.
func _agree(y: int, way: String, before_dock: int, tid: int, expect_emit_from: int) -> void:
	await _settle(8)
	var s := _state()
	_ok(s["tl_year"] == y and s["engine"] == y, "%s: tl_year() and the engine say %d (got %s / %s)" % [way, y, str(s["tl_year"]), str(s["engine"])])
	var want: String = _app._tl_format_year(y)
	_ok(not (s["strip"] as Array).is_empty() and (s["strip"] as Array).all(func(t): return t == want),
		"%s: the strip's year labels read '%s' (got %s)" % [way, want, str(s["strip"])])
	_ok(String(s["note"]).ends_with(_ws._tl_format_year(y)),
		"%s: CIVIL's 'Active year' note reads %s (got '%s')" % [way, _ws._tl_format_year(y), s["note"]])
	var recorded: bool = _bridge.get_civ_timeline_years().has(y)
	_ok((s["pill"] == y) if recorded else (s["pill"] == -99999),
		"%s: CIVIL's active pill is %s (got %s)" % [way, str(y) if recorded else "none (an unrecorded year)", str(s["pill"])])
	_ok(s["dock_first"] != before_dock,
		"%s: the right dock's Settlement context was rebuilt for the new year" % way)
	var gv := _garrison_expected(tid, y)
	_ok(_dock_texts().has(gv), "%s: the dock's Garrison row reads '%s' for year %d" % [way, gv, y])
	_ok(s["terr"] == s["terr_painted"] or s["terr"] == null,
		"%s: the territory texture was last repainted for the engine's territory year (%s vs %s)" % [way, str(s["terr"]), str(s["terr_painted"])])
	_ok(_emits > expect_emit_from, "%s: timeline_changed was emitted (%d emits so far, %d before)" % [way, _emits, expect_emit_from])


func _scan_writers() -> void:
	print("== source scan: who calls civ_goto_year( ==")
	var hits: Array = []
	for dir in ["res://shell", "res://shell/workspaces"]:
		for f in DirAccess.get_files_at(dir):
			## `engine_bridge.gd` holds the wrapper itself (`world_gen.civ_goto_year(`), the one place
			## the shell reaches the engine; the writer is the only caller of the wrapper.
			if not f.ends_with(".gd") or f == "engine_bridge.gd":
				continue
			var lines := FileAccess.get_file_as_string(dir.path_join(f)).split("\n")
			for i in lines.size():
				var code := String(lines[i]).split("#")[0]
				if code.contains("civ_goto_year(") and not code.contains("func civ_goto_year"):
					hits.append("%s:%d" % [f, i + 1])
	print("  info writers: ", str(hits))
	_ok(hits.size() == 1 and String(hits[0]).begins_with("dcc_shell.gd:"),
		"exactly one code line outside the bridge's wrapper calls civ_goto_year(, in dcc_shell.gd (got %s)" % str(hits))


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var k := 0
	while k < args.size():
		var a := args[k]
		if a == "--force-touch":
			pass
		elif a == "--shots" and k + 1 < args.size():
			k += 1
			_shots = args[k]
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
		k += 1
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(2)
		return
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	_mode = "tablet" if DccTheme.is_tablet() else "desktop"
	print("== mode %s ==" % _mode)
	_bridge = _app.bridge
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _settle(2)
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _settle(10)
	await _settle(30)
	if not bool(_bridge.has_world):
		print("[FATAL] no world")
		get_tree().quit(2)
		return
	_scan_writers()
	var civ = _app._workspaces.filter(func(w): return w is CivilizationWorkspace)
	_ws = civ[0]
	_app.timeline_changed.connect(func(): _emits += 1)
	## Three recorded years (setup only: the writers under test are the clicks below). 5000 sets the world's range top (PC-3; it was
	## outside the old fixed -400..1200 track, on purpose).
	for y in [100, 300, 5000]:
		_bridge.civ_add_year(y)
	_app.tl_set_year(100, true)
	_app.select_domain("civilization")
	_app.arm_tool("inspect")
	await _settle(6)
	_ws.open_category("Timeline")
	await _settle(8)
	## A settlement for the dock (the garrison row is read at the cursor). Any named one.
	var all: Array = _bridge.settlements()
	var idx := -1
	for i in all.size():
		if String((all[i] as Dictionary).get("name", "")).strip_edges() != "":
			idx = i
			break
	if idx < 0:
		print("[FATAL] no named settlement")
		get_tree().quit(2)
		return
	var tid := int((all[idx] as Dictionary).get("tid", idx))
	_app.select("settlement", idx)
	await _settle(8)
	_ok(_app.right_dock_ctrl.current_settlement_index() == idx, "the dock holds the settlement (control)")

	## --- controls: no write, no movement ---------------------------------------------------------
	print("== controls ==")
	var s0 := _state()
	var e0 := _emits
	await _settle(30)
	var s1 := _state()
	_ok(s1["tl_year"] == s0["tl_year"] and s1["dock_first"] == s0["dock_first"] and _emits == e0,
		"with no write the year, the dock and the emit count do not move")
	## PC-3: the track is the world's range, here 100..5000 (the three recorded years), so the default write
	## clamps to 5000 and only an `exact` write may pass it.
	_app.tl_set_year(99999)
	_ok(_app.tl_year() == 5000, "control: the writer clamps a default write of 99999 to the track's 5000 (got %d)" % _app.tl_year())
	_app.tl_set_year(99999, true)
	_ok(_app.tl_year() == 99999, "control: the writer does not clamp an exact write of 99999 (got %d)" % _app.tl_year())
	_app.tl_set_year(100, true)
	await _settle(10)

	## --- 1. CIVIL pill -----------------------------------------------------------------------------
	print("== 1. CIVIL ▸ Timeline pill (300 AD) ==")
	var pill := _button(_ws._tl_body, _ws._tl_format_year(300))
	_ok(pill != null, "the Timeline draws a pill for 300 AD")
	if pill != null:
		var dock0 := _dock_first_id()
		var e1 := _emits
		await _click_control(pill)
		await _agree(300, "CIVIL pill", dock0, tid, e1)
		await _shot("pill")

	## --- 2. the strip's track ----------------------------------------------------------------------
	print("== 2. the strip's scrub track (700 AD) ==")
	## The strip starts collapsed to one row whose whole width is a button; a real click expands it.
	for b in _app.timeline_row.find_children("*", "Button", true, false):
		if (b as Button).tooltip_text.begins_with("Expand the timeline") and (b as Control).is_visible_in_tree():
			## The centre of the row, over the spacer: PC-3 fixed the spacer swallowing the click (PC-2 had to
			## click on the word TIMELINE instead). `_pc3strip_probe` pins the fix across the full width.
			var rr := (b as Control).get_global_rect()
			await _click_at(_to_root(rr.get_center(), b))
			break
	await _settle(8)
	var track: Control = _app._tl_track
	_ok(track != null and track.is_visible_in_tree() and track.size.x > 0.0, "the strip's scrub track is on screen")
	if track != null and track.is_visible_in_tree():
		var rng: Dictionary = _app.tl_range()
		var span := float(int(rng["max"]) - int(rng["min"]))
		var f := float(700 - int(rng["min"])) / span
		var r := track.get_global_rect()
		var dock1 := _dock_first_id()
		var e2 := _emits
		await _click_at(_to_root(Vector2(r.position.x + r.size.x * f, r.get_center().y), track))
		## The click lands on a pixel, so the year is within a pixel's worth (1600 / track width) of 700.
		var got: int = _app.tl_year()
		var tol := int(ceil(span / r.size.x)) + 1
		_ok(abs(got - 700) <= tol, "the track click at 700's x moved the year to %d (within %d of 700)" % [got, tol])
		await _agree(got, "strip", dock1, tid, e2)
		await _shot("strip")

	## --- 3. the Place editor -----------------------------------------------------------------------
	print("== 3. the Place editor's Timeline tab ==")
	var ped = _app.place_editor_window
	ped.open_for_tab(idx, "timeline")
	await _settle(10)
	_ok(ped.visible, "the Place editor is open on its Timeline tab")
	var chip := _button(ped, "y100")
	_ok(chip != null, "the editor draws a y100 chip")
	if chip != null:
		var dock2 := _dock_first_id()
		var e3 := _emits
		await _click_control(chip)
		await _agree(100, "Place chip", dock2, tid, e3)
		_ok(_label_with(ped, "Cursor: year 100") != null, "Place chip: the editor's own 'Cursor: year' line reads 100")
		await _shot("chip")
	ped._tl_target_year = 9000
	var go := _button(ped, "Go to year")
	_ok(go != null, "the editor draws Go to year")
	if go != null:
		var dock3 := _dock_first_id()
		var e4 := _emits
		await _click_control(go)
		await _agree(9000, "Place Go to year", dock3, tid, e4)
		_ok(_label_with(ped, "Cursor: year 9000") != null, "Place Go to year: the editor's 'Cursor: year' line reads 9000, not a clamped 5000")
		await _shot("goto")
	ped.hide()
	await _settle(6)
	## The assistant's place still names the one selection after all of the above (PC-1 still holds).
	_ok(int(_app.assistant_window._current_place().get("index", -1)) == idx, "the assistant's context still names the selection")
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
