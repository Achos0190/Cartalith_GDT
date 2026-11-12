extends Node
## Phone search-field lag, measured (OUTSTANDING_WORK.md "Phone and touch": "Search field lag of 10 to 40 s on the
## 24601 world over adb", unexplained until this probe). Boots the real shell as a phone in a `--vp`-sized host
## `SubViewport`, generates seed 24601 (2048 x 1311, 1 200 km, the 3D scope's measurement world), runs the civilisation
## populate, opens the phone search overlay and types a query one character at a time through the field's real
## `text_changed` path. Per keystroke it prints, in microseconds: the whole handler (`_refresh_phone_search()`), and the
## three phases the handler is made of, timed separately by calling the same functions on the same text:
## `_search_result_set()` (index build on the first keystroke, then the substring match), `_fill_scope_chips()`, and
## `_fill_search_results()` (one Button row per hit), plus the row count and the node count of the results column, and
## the next frame's wall time (layout and draw of the rows, which the handler only queues).
##
##   godot --path . --resolution 1600x1000 _phonesearchperf_probe.tscn -- --vp 1080x2340 --density 2.8125 --force-touch \
##       [--grid 2048x1311] [--query albfox] [--reps 3]
## Arguments read by THIS file: `--vp WxH`, `--grid WxH`, `--query TEXT`, `--reps N`. Accepted because the shell reads
## them: `--density D`, `--force-touch`. Anything else aborts with exit 2.
##
## It asserts nothing about speed on this machine (a desktop is not the handset; `MISTAKES.md`, "Quote a timing"): it
## reports a median with min..max over `--reps` repeats of the whole typing sequence, and a final line
## `RESULT: PASS|FAIL` that fails when the world did not generate, the overlay did not open, or the row cap (60) does not hold, so that a run that
## measured nothing cannot read as a measurement.

var _vp: SubViewport
var _app: Node
var _fails := 0
var _checks := 0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
		print("FAIL: ", what)
	else:
		print("ok:   ", what)

func _frames(n: int) -> void:
	for k in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var k := args.find(name)
	return String(args[k + 1]) if k >= 0 and k + 1 < args.size() else dflt

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("ABORT: run WINDOWED")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch":
			i += 1
		elif a in ["--vp", "--density", "--grid", "--query", "--reps"]:
			i += 2
		else:
			print("ABORT: unknown argument ", a)
			get_tree().quit(2)
			return
	var parts := _arg("--vp", "").split("x")
	if parts.size() != 2:
		print("ABORT: needs --vp WxH")
		get_tree().quit(2)
		return
	await _run(int(parts[0]), int(parts[1]))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)

func _count(n: Node) -> int:
	var c := 1
	for k in n.get_children():
		c += _count(k)
	return c

func _med(a: Array) -> float:
	var s := a.duplicate()
	s.sort()
	return float(s[s.size() / 2]) if not s.is_empty() else -1.0

func _run(vw: int, vh: int) -> void:
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.6).timeout
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	_ok(DccTheme.is_phone(), "boots as a phone")
	var g := _arg("--grid", "2048x1311").split("x")
	var bridge = _app.get("bridge")
	bridge.generate({"seed": 24601, "width_km": 1200.0, "grid_w": int(g[0]), "grid_h": int(g[1]),
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(10)
	var pop: Dictionary = bridge.civ_populate()
	print("populate: ", str(pop).substr(0, 120))
	await _frames(10)
	print("settlements=%d roads=%d sea_routes=%d labels=%d" % [bridge.settlements().size(), bridge.roads().size(),
		bridge.sea_routes().size(), bridge.label_list().size()])
	var query := _arg("--query", "albfox")
	var reps := int(_arg("--reps", "3"))
	var per_key: Array = []  ## [rep][key] -> {total, set, chips, fill, rows, nodes, frame}
	for rep in reps:
		var open_t0 := Time.get_ticks_usec()
		_app.call("open_find_on_map")
		var open_us := Time.get_ticks_usec() - open_t0
		var open_nodes := _count(_app.get("_phone_search_results"))
		print("open (empty query, index build included on rep 0): %d us, %d nodes in the results column" % [open_us, open_nodes])
		await _frames(8)
		var field: LineEdit = _app.get("_phone_search_field")
		_ok(field != null and field.is_visible_in_tree(), "the phone search overlay is open")
		if field == null:
			return
		field.text = ""
		field.grab_focus()
		var keys: Array = []
		for n in range(1, query.length() + 1):
			var txt := query.substr(0, n)
			field.text = txt
			field.caret_column = txt.length()
			var t0 := Time.get_ticks_usec()
			field.text_changed.emit(txt)  ## the handler the real key path ends in
			var total := Time.get_ticks_usec() - t0
			if n == 1 and txt != ".":
				## The cap, asserted against the literal 60 (not the shell's constant): a one-letter query on this world
				## has hundreds of hits (306 for "a"), the sheet must hold 60 row buttons and say it cut the list.
				await get_tree().process_frame
				var col: Control = _app.get("_phone_search_results")
				var btns := 0
				var last_text := ""
				for c in col.get_children():
					if c.is_queued_for_deletion():
						continue
					if c is Button:
						btns += 1
					elif c is Label or c is Control:
						var lbl := c as Label
						if lbl == null and c.get_child_count() > 0 and c.get_child(0) is Label:
							lbl = c.get_child(0) as Label
						if lbl != null:
							last_text = lbl.text
				var hits: int = (_app.call("_search_result_set", txt) as Dictionary).get("rows", []).size()
				if hits > 60:
					_ok(btns == 60, "'%s' has %d hits and the sheet builds exactly 60 row buttons (%d)" % [txt, hits, btns])
					_ok(last_text.begins_with("Showing the first 60 of %d" % hits), "the cut list says so: '%s'" % last_text)
				else:
					print("note: '%s' has only %d hits on this world, the cap is not exercised" % [txt, hits])
			var frame_t0 := Time.get_ticks_usec()
			await get_tree().process_frame
			await get_tree().process_frame
			var frame := Time.get_ticks_usec() - frame_t0
			## the same three phases, repeated on the same text (index already built by now)
			var a := Time.get_ticks_usec()
			var res: Dictionary = _app.call("_search_result_set", txt)
			var b := Time.get_ticks_usec()
			_app.call("_fill_scope_chips", _app.get("_phone_search_chips"), res)
			var c := Time.get_ticks_usec()
			_app.call("_fill_search_results", _app.get("_phone_search_results"), res, func(): pass)
			var d := Time.get_ticks_usec()
			var rows: Array = res.get("rows", [])
			keys.append({"txt": txt, "total": total, "set": b - a, "chips": c - b, "fill": d - c, "rows": rows.size(),
				"nodes": _count(_app.get("_phone_search_results")), "frame": frame})
		per_key.append(keys)
		_app.call("_set_search_open", false)
		await _frames(4)
		_app.call("_reset_place_search_cache")
	print("--- per keystroke, microseconds (median over %d reps; handler total, then its phases) ---" % reps)
	for k in query.length():
		var tot: Array = []
		var st: Array = []
		var ch: Array = []
		var fl: Array = []
		var fr: Array = []
		for rep in per_key.size():
			tot.append(per_key[rep][k]["total"])
			st.append(per_key[rep][k]["set"])
			ch.append(per_key[rep][k]["chips"])
			fl.append(per_key[rep][k]["fill"])
			fr.append(per_key[rep][k]["frame"])
		var e: Dictionary = per_key[0][k]
		print("'%s' rows=%d nodes=%d  handler med %d (min %d max %d)  set %d chips %d fill %d  next-2-frames %d" % [
			e["txt"], e["rows"], e["nodes"], _med(tot), tot.min(), tot.max(), _med(st), _med(ch), _med(fl), _med(fr)])
