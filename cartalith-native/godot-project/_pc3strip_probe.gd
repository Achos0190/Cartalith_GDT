extends Node
## PC-3 probe: the year strip at program scope (`PC_SHELL_PROPOSAL.md` 3.2, milestone PC-3, CV-24), inside the
## real shell, with real mouse clicks through `Input.parse_input_event()`.
##
##   desktop: godot --path . --resolution 1600x1000 _pc3strip_probe.tscn -- --shots <dir>
##   touch:   godot --path . --resolution 1600x1000 _pc3strip_probe.tscn -- --force-touch --shots <dir>
##
## **Windowed, never `--headless`**, from a project copy with its own `config/name` and a wiped
## `app_userdata/<name>` (`MISTAKES.md`: a probe that reads `user://` can be green for reasons that have
## nothing to do with the code). Reads `--force-touch` and `--shots <dir>`; anything else exits 2.
## The phone (393x852) is not driven: it keeps its shipped strip (CIVIL only), which the pixel-identity
## captures of the PC-3 commit cover.
##
## What it protects (PC-3's bar and the fault PC-2 found):
##   A. a fresh world has NO range: `tl_range()` answers the fallback axis with `from_world` false, and the
##      strip's end literals say `(default)` -- a track drawn from nothing must not pass for the world's;
##   B. the strip is on screen in WORLD, CIVIL and CARTO, and a REAL CLICK at 5 %, 50 % and 95 % of the
##      collapsed strip's width expands it in each (the spacer used to swallow the middle of the strip);
##   C. a scrub of the track moves the year (`tl_set_year`) in each domain, to the year the world's range
##      maps that x to; + Record year records the SHOWN year from the strip; a year recorded in CIVIL's dock
##      moves the range and the strip's literals and adds a pill to the strip; a strip pill jumps the year;
##   D. the three filters flip the CIVIL workspace's flags from the strip (and the dock shows them), and the
##      speeds change the speed, from CARTO;
##   E. a chronos entry in the selected place's note is drawn as an accent tick above the rail at its year
##      (pixel read), listed in the event row, and a click on its row moves the year; deselecting removes both.
## Exit 0 pass, 1 fail, 2 bad argument, watchdog or no world.

var _fails := 0
var _checks := 0
var _mode := "desktop"
var _shots := ""
var _app: Node
var _bridge
var _ws: Node

const VAULT_DIR := "user://_pc3_vault"
const _LA := preload("res://shell/llm_actions.gd")


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
		var path := _shots.path_join("pc3_%s_%s.png" % [_mode, tag])
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
	await _settle(8)


func _to_root(p: Vector2, c: Control) -> Vector2:
	var w := c.get_window()
	return p if w == null or w == get_tree().root else p + Vector2(w.position)


func _click_control(c: Control) -> void:
	var sc := c.get_parent()
	while sc != null and not (sc is ScrollContainer):
		sc = sc.get_parent()
	if sc != null:
		(sc as ScrollContainer).ensure_control_visible(c)
		await _settle(4)
	await _click_at(_to_root(c.get_global_rect().get_center(), c))


## A visible Button under `root` whose text is `text`, or null.
func _button(root: Node, text: String) -> Button:
	for b in root.find_children("*", "Button", true, false):
		if (b as Button).text == text and (b as Control).is_visible_in_tree():
			return b
	return null


## The strip's visible Button named `name` (the nodes the strip names), or null.
func _named(root: Node, node_name: String) -> Button:
	for b in root.find_children(node_name, "Button", true, false):
		if (b as Control).is_visible_in_tree():
			return b
	return null


## The strip's visible collapsed-strip hit button, or null when the strip is expanded or hidden.
func _hit() -> Button:
	for b in _app.timeline_row.find_children("*", "Button", true, false):
		if (b as Button).tooltip_text.begins_with("Expand the timeline") and (b as Control).is_visible_in_tree():
			return b
	return null


## The strip's pill for recorded year `y` (a segment tagged with the `tl_year` meta in the RECORDED row).
func _pill(y: int) -> Button:
	var rec: Node = _app.timeline_row.find_child("TimelineRecords", true, false)
	if rec == null:
		return null
	for b in rec.find_children("*", "Button", true, false):
		if (b as Button).has_meta("tl_year") and int((b as Button).get_meta("tl_year")) == y and (b as Control).is_visible_in_tree():
			return b
	return null


func _years() -> Array:
	var out: Array = []
	for y in _bridge.get_civ_timeline_years():
		out.append(int(y))
	return out


## Clicks the expanded strip's scrub track at fraction `f` of its width and checks the year landed where the
## world's range maps that x. `where` names the domain for the log.
func _scrub(f: float, where: String) -> void:
	var track: Control = _app._tl_track
	_ok(track != null and track.is_visible_in_tree() and track.size.x > 0.0, "%s: the scrub track is on screen" % where)
	if track == null or not track.is_visible_in_tree():
		return
	var rng: Dictionary = _app.tl_range()
	var lo := int(rng["min"])
	var span := float(int(rng["max"]) - lo)
	var r := track.get_global_rect()
	await _click_at(_to_root(Vector2(r.position.x + r.size.x * f, r.get_center().y), track))
	var want := lo + int(round(f * span))
	var tol := int(ceil(span / r.size.x)) + 1
	var got: int = _app.tl_year()
	_ok(abs(got - want) <= tol, "%s: a scrub at %d%% moved the year to %d (range %d..%d wants %d, within %d)" % [where, int(f * 100.0), got, lo, lo + int(span), want, tol])
	_ok(int(_bridge.get_civ_year()) == got, "%s: the engine holds the year the strip names" % where)


## Whether any pixel in a 5x5 box around `at` (viewport pixels) is within `tol` of `col`.
func _near_colour(img: Image, at: Vector2, col: Color, tol: float) -> bool:
	for dy in range(-2, 3):
		for dx in range(-2, 3):
			var x := int(at.x) + dx
			var y := int(at.y) + dy
			if x < 0 or y < 0 or x >= img.get_width() or y >= img.get_height():
				continue
			var p := img.get_pixel(x, y)
			if absf(p.r - col.r) <= tol and absf(p.g - col.g) <= tol and absf(p.b - col.b) <= tol:
				return true
	return false


## Whether the track shows an accent chronos tick above the rail at `year`, read from the rendered frame.
func _tick_drawn(year: int) -> bool:
	await _settle(4)
	await RenderingServer.frame_post_draw
	var track: Control = _app._tl_track
	var img := get_viewport().get_texture().get_image()
	var scale := Vector2(img.get_size()) / get_viewport().get_visible_rect().size
	var r := track.get_global_rect()
	var col := int(_app.tl_mark_column(year, r.size.x))
	## The tick is drawn 8..4 px above the rail's centre (mid - 8 .. mid - 4): sample its middle.
	var at := Vector2(r.position.x + float(col) + 0.5, r.position.y + r.size.y * 0.5 - 6.0) * scale
	return _near_colour(img, at, DccTheme.c("accent"), 0.12)


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
	get_tree().create_timer(420.0).timeout.connect(func() -> void:
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
	_ws = _app._workspaces.filter(func(w): return w is CivilizationWorkspace)[0]

	## --- A. a fresh world has no range ----------------------------------------------------------------
	print("== A. a fresh world: no range, so the default axis, said so ==")
	var r0: Dictionary = _app.tl_range()
	_ok(_years().is_empty(), "control: the fresh world records no year")
	_ok(not bool(r0["from_world"]), "a world with no recorded year and no conflict reports from_world = false")
	_ok(int(r0["min"]) == -400 and int(r0["max"]) == 1200, "...and the fallback axis is -400..1200 (got %d..%d)" % [int(r0["min"]), int(r0["max"])])

	## --- B. the strip in every domain; a click across the full width expands it -------------------------
	print("== B. the strip in WORLD, CIVIL and CARTO; the collapsed strip is clickable across its width ==")
	for d in ["world", "civilization", "cartography"]:
		_app.select_domain(d)
		_app.arm_tool("inspect")
		await _settle(8)
		_ok(_app.timeline_bar.is_visible_in_tree() and _app.timeline_bar.size.y > 8.0, "%s: the strip is on screen" % d)
		var inside: bool = _app.timeline_bar.get_global_rect().end.y <= get_viewport().get_visible_rect().size.y + 1.0
		_ok(inside, "%s: the strip sits inside the window" % d)
		for frac in [0.05, 0.5, 0.95]:
			_app._tl_expanded = false
			_app._fill_timeline_strip()
			await _settle(6)
			var hit := _hit()
			_ok(hit != null and hit.size.x > 500.0, "%s: the collapsed strip is one wide button (%.0f px)" % [d, 0.0 if hit == null else hit.size.x])
			if hit == null:
				continue
			var rr := hit.get_global_rect()
			await _click_at(_to_root(Vector2(rr.position.x + rr.size.x * frac, rr.get_center().y), hit))
			_ok(bool(_app._tl_expanded), "%s: a click at %d%% of the collapsed strip's width expands it" % [d, int(frac * 100.0)])
	_app.select_domain("world")
	_app.arm_tool("inspect")
	await _settle(8)
	## Whatever B found, section C needs the expanded strip: a FAIL above must not crash the rest of the
	## probe (a script error in `_ready` ends the coroutine and the process then waits for the watchdog).
	if not _app._tl_expanded:
		_app._tl_expanded = true
		_app._fill_timeline_strip()
		await _settle(6)
	_ok(_app._tl_expanded and _app._tl_track != null, "WORLD: the expanded strip has its track")
	var lo_txt: String = (_app._tl_min_label as Label).text
	_ok(lo_txt.contains("(default)") and (_app._tl_max_label as Label).text.contains("(default)"),
		"the end literals say (default) with no range ('%s' / '%s')" % [lo_txt, (_app._tl_max_label as Label).text])
	await _shot("world_default")

	## --- C. scrub and record from each domain -----------------------------------------------------------
	print("== C. scrub, + Record year and the range, from each domain ==")
	await _scrub(0.5, "WORLD")
	var y1: int = _app.tl_year()
	var rec := _button(_app.timeline_row, "+ Record year")
	_ok(rec != null, "WORLD: the strip draws + Record year")
	if rec != null:
		await _click_control(rec)
	_ok(_years() == [y1], "WORLD: + Record year recorded the SHOWN year %d from the strip (recorded %s)" % [y1, str(_years())])
	var r1: Dictionary = _app.tl_range()
	_ok(bool(r1["from_world"]) and int(r1["min"]) == y1 - 50 and int(r1["max"]) == y1 + 50,
		"WORLD: one recorded year makes a 100-year range around it (got %d..%d from_world=%s)" % [int(r1["min"]), int(r1["max"]), str(r1["from_world"])])
	_ok((_app._tl_min_label as Label).text == "YEAR %d" % int(r1["min"]) and (_app._tl_max_label as Label).text == "YEAR %d" % int(r1["max"]),
		"WORLD: the end literals follow the world's range ('%s' / '%s')" % [(_app._tl_min_label as Label).text, (_app._tl_max_label as Label).text])
	_ok(_pill(y1) != null, "WORLD: the strip draws a pill for the recorded year")
	await _shot("world_recorded")

	## CIVIL: record 100 through the DOCK's Add year; the strip follows the world.
	_app.select_domain("civilization")
	_app.arm_tool("inspect")
	await _settle(8)
	_ws.open_category("Timeline")
	await _settle(8)
	var add := _button(_ws._tl_body, "Add year")
	_ok(add != null, "CIVIL: the dock draws Add year")
	if add != null:
		await _click_control(add)
	_ok(_years().has(100) and _years().has(y1), "CIVIL: the dock recorded 100 (recorded %s)" % str(_years()))
	var r2: Dictionary = _app.tl_range()
	_ok(int(r2["min"]) == 100 and int(r2["max"]) == y1, "CIVIL: the range follows the recorded years: 100..%d (got %d..%d)" % [y1, int(r2["min"]), int(r2["max"])])
	await _settle(8)
	_ok((_app._tl_min_label as Label).text == "YEAR 100" and (_app._tl_max_label as Label).text == "YEAR %d" % y1,
		"CIVIL: the strip's literals follow a year recorded in the dock ('%s' / '%s')" % [(_app._tl_min_label as Label).text, (_app._tl_max_label as Label).text])
	_ok(_pill(100) != null and _pill(y1) != null, "CIVIL: the strip draws a pill for each recorded year")
	## AC-3's catalogue: the bounds it hands the model are the world's range, and a card's year outside it is
	## refused by the shell's preflight with the same bounds (llm_actions.gd), so the two cannot disagree.
	var facts: Dictionary = _LA.facts(_app, _bridge)
	_ok(facts.has("timeline") and int(facts["timeline"]["min"]) == 100 and int(facts["timeline"]["max"]) == y1,
		"the assistant catalogue's year bounds are the world's range 100..%d (got %s)" % [y1, str(facts.get("timeline"))])
	var why := _LA.preflight(_app, _bridge, [{"id": "timeline.set_year", "route": "year", "args": {"year": y1 + 1}}])
	_ok(why.contains("outside the track"), "a card year of %d, one past the world's range, is refused ('%s')" % [y1 + 1, why])
	why = _LA.preflight(_app, _bridge, [{"id": "timeline.set_year", "route": "year", "args": {"year": y1}}])
	_ok(not why.contains("outside the track"), "...and the range's own last year is accepted ('%s')" % why)
	await _scrub(0.25, "CIVIL")
	var p100 := _pill(100)
	if p100 != null:
		await _click_control(p100)
	_ok(_app.tl_year() == 100 and int(_bridge.get_civ_year()) == 100, "CIVIL: a strip pill jumped the year to 100 (got %d)" % _app.tl_year())

	## CARTO: scrub, the three filters and a speed.
	_app.select_domain("cartography")
	_app.arm_tool("inspect")
	await _settle(8)
	_ok(_app.timeline_bar.is_visible_in_tree() and _app._tl_expanded, "CARTO: the strip is expanded")
	await _scrub(0.75, "CARTO")
	await _shot("carto_strip")

	## --- D. filters and speeds from CARTO -----------------------------------------------------------------
	print("== D. the three filters and the speeds, from CARTO ==")
	for pair in [["exist", "Exist only"], ["ghost", "Ghost removed"], ["highlight", "Highlight new"]]:
		var key: String = pair[0]
		var seg := _named(_app.timeline_row, "Filter_" + key)
		_ok(seg != null and seg.text == String(pair[1]), "CARTO: the strip draws the %s filter" % pair[1])
		_ok(not _ws.tl_filter(key), "control: %s starts off" % pair[1])
		if seg != null:
			await _click_control(seg)
		_ok(_ws.tl_filter(key), "CARTO: a click on the strip's %s turned the CIVIL workspace's flag on" % pair[1])
		seg = _named(_app.timeline_row, "Filter_" + key)
		if seg != null:
			await _click_control(seg)
		_ok(not _ws.tl_filter(key), "CARTO: a second click turned it off")
	var ex := _named(_app.timeline_row, "Filter_exist")
	if ex != null:
		await _click_control(ex)
	_app.select_domain("civilization")
	await _settle(8)
	_ws.open_category("Timeline")
	await _settle(8)
	var pressed := 0
	for cb in _ws._tl_body.find_children("*", "CheckBox", true, false):
		if (cb as CheckBox).button_pressed:
			pressed += 1
	_ok(pressed == 1, "CIVIL: the dock's Filters show the one filter the strip turned on (%d pressed)" % pressed)
	_ws.tl_set_filter("exist", false)
	_app.select_domain("cartography")
	await _settle(8)
	var seg100 := _button(_app.timeline_row, "×100")
	_ok(seg100 != null, "CARTO: the strip draws the speeds")
	if seg100 != null:
		await _click_control(seg100)
	_ok(_app.tl_speed == 100, "CARTO: the x100 speed took (speed %d)" % _app.tl_speed)

	## --- E. the selection's chronos ----------------------------------------------------------------------
	print("== E. a chronos entry of the selected place: tick, event row, click ==")
	var places: Array = _bridge.settlements()
	var idx := -1
	for i in places.size():
		if int((places[i] as Dictionary).get("tid", 0)) > 0 and String((places[i] as Dictionary).get("name", "")).strip_edges() != "":
			idx = i
			break
	if idx < 0:
		print("[FATAL] no settlement with a tid")
		get_tree().quit(2)
		return
	var tid := int((places[idx] as Dictionary).get("tid", 0))
	var pname := String((places[idx] as Dictionary).get("name", "?"))
	DirAccess.make_dir_recursive_absolute(VAULT_DIR + "/Settlements")
	var f := FileAccess.open(VAULT_DIR + "/Settlements/Note.md", FileAccess.WRITE)
	f.store_string("# %s\n\nA river town.\n\n```chronos\n- [200] #red The first wall\n- [250~320] #blue The long siege | held\n```\n" % pname)
	f.close()
	var info: Dictionary = _bridge.vault_connect(ProjectSettings.globalize_path(VAULT_DIR), "PC3 probe vault")
	_ok(bool(info.get("ok", false)), "the probe vault connected (%s)" % str(info.get("error", "")))
	var att: Dictionary = _bridge.vault_attach("settlement", tid, pname, "Settlements/Note.md", "")
	_ok(bool(att.get("ok", false)), "the note attached to the settlement (%s)" % str(att.get("error", "")))
	_app.tl_set_year(100, true)
	await _settle(4)
	_ok(_app.tl_selection_events().is_empty(), "control: with nothing selected the strip has no chronos entries")
	_app._queue_tl_rows()
	await _settle(8)
	_ok(not await _tick_drawn(200), "control: no accent tick at 200 before the place is selected")
	_app.select("settlement", idx)
	await _settle(10)
	var evs: Array = _app.tl_selection_events()
	_ok(evs.size() == 2 and int(evs[0]["start"]) == 200 and int(evs[1]["start"]) == 250 and int(evs[1]["end"]) == 320,
		"the selected place yields its two chronos entries (got %s)" % str(evs))
	_ok(await _tick_drawn(200), "the track draws an accent tick above the rail at 200")
	_ok(await _tick_drawn(280), "...and the span 250~320 (sampled at 280)")
	await _shot("carto_chronos")
	var ev_btn: Button = null
	var evrow: Node = _app.timeline_row.find_child("TimelineEvents", true, false)
	if evrow != null:
		for b in evrow.find_children("*", "Button", true, false):
			if (b as Button).has_meta("tl_year") and int((b as Button).get_meta("tl_year")) == 200:
				ev_btn = b
	_ok(ev_btn != null and ev_btn.text.contains("The first wall") and ev_btn.text.contains("Note"), "the event row lists the entry with its source note ('%s')" % ("" if ev_btn == null else ev_btn.text))
	if ev_btn != null:
		await _click_control(ev_btn)
	_ok(_app.tl_year() == 200, "a click on the event row moved the year to 200 (got %d)" % _app.tl_year())
	_app.clear_selection()
	await _settle(10)
	_ok(_app.tl_selection_events().is_empty(), "deselecting empties the chronos entries")
	## The playhead is also accent and stands at 200 after the row click; move it off the sampled column.
	_app.tl_set_year(100, true)
	await _settle(8)
	_ok(not await _tick_drawn(200), "...and the tick is gone")
	_bridge.vault_disconnect()
	DirAccess.remove_absolute(ProjectSettings.globalize_path(VAULT_DIR + "/Settlements/Note.md"))

	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
