extends Node
## CIVIL ▸ Landmarks probe — `design/landmark-generation/LANDMARK_UI_DESIGN.md`.
##
##   Godot_v4.7.1 --path . --resolution 1600x900 --rendering-driver opengl3 \
##       _landmark_probe.tscn
##
## Windowed, never `--headless`: the dummy rasterizer returns null textures and
## the shell boot walks straight into them.
##
## Two halves, because the landmark bridge is being written by a concurrent
## pass and may not be present:
##
##   A — the LIVE shell. Asserts the category exists, that the old
##       "Points of interest" stub and its two notes are gone, and that the
##       panel degrades to a *disclosed* empty state rather than to nothing.
##   B — the same workspace built against `StubBridge`, a GDScript subclass of
##       `EngineBridge` implementing the locked contract over fixtures. This is
##       what exercises the wiring the live bridge cannot yet: the rows, the
##       viewshed tag, the disabled unbuildable row, the class chips, the run
##       button, and the crux — a `spacing`-limited row whose placed bar is
##       genuinely shorter than its cap bar while an `at cap` row's two bars are
##       flush.
##
## Half B is a real exercise of this file's code against the contract, not a
## substitute for the live bridge. Which of the two ran is printed, loudly.

var _vp: SubViewport
var _fail := 0

# -- the contract, as fixtures ------------------------------------------------

class StubBridge extends EngineBridge:
	## `landmark_kinds()`'s shape, exactly: two families, all four classes, one
	## `needs_viewshed` type, one `buildable: false` type.
	const KINDS := [
		{"key": "peak", "label": "Peak", "family": "physical", "class": "regional",
			"default_cap": 24, "needs_viewshed": true, "buildable": true},
		{"key": "waterfall", "label": "Waterfall", "family": "physical", "class": "regional",
			"default_cap": 40, "needs_viewshed": false, "buildable": true},
		{"key": "cliff", "label": "Cliff", "family": "physical", "class": "local",
			"default_cap": 30, "needs_viewshed": false, "buildable": true},
		{"key": "ancient_forest", "label": "Ancient forest", "family": "physical",
			"class": "continental", "default_cap": 2, "needs_viewshed": false,
			"buildable": true},
		{"key": "ice_shelf", "label": "Ice shelf", "family": "physical",
			"class": "continental", "default_cap": 3, "needs_viewshed": true,
			"buildable": false},
		{"key": "mountain_pass", "label": "Mountain pass", "family": "transportation",
			"class": "regional", "default_cap": 16, "needs_viewshed": false,
			"buildable": true},
		{"key": "pilgrim_way", "label": "Pilgrim way", "family": "transportation",
			"class": "cultural", "default_cap": 12, "needs_viewshed": false,
			"buildable": true},
	]

	var caps := {"peak": 24, "waterfall": 40, "cliff": 30, "ancient_forest": 2,
		"ice_shelf": 3, "mountain_pass": 16, "pilgrim_way": 12}
	var armed := {"peak": true, "waterfall": true, "cliff": true,
		"ancient_forest": false, "ice_shelf": true, "mountain_pass": true,
		"pilgrim_way": false}
	var crowding := 1.0
	var radii := [180.0, 34.0, 9.0, 6.0]
	var cross := true
	var ran := false
	## Every setter call, in order, so the probe can assert what was written --
	## and, just as importantly, what was NOT.
	var writes: Array = []

	func landmark_kinds() -> Array:
		return KINDS.duplicate(true)

	func landmark_settings() -> Dictionary:
		return {"caps": caps.duplicate(), "armed": armed.duplicate(),
			"crowding": crowding, "class_radius_km": radii.duplicate(),
			"cross_type_competition": cross}

	func landmark_set_cap(key: String, v: int) -> bool:
		writes.append("cap:%s=%d" % [key, v])
		caps[key] = v
		return true

	func landmark_set_armed(key: String, on: bool) -> bool:
		writes.append("armed:%s=%s" % [key, str(on)])
		armed[key] = on
		return true

	func landmark_set_crowding(v: float) -> bool:
		writes.append("crowding=%.2f" % v)
		crowding = v
		return true

	func landmark_set_class_radius(class_key: String, km: float) -> bool:
		writes.append("radius:%s=%.1f" % [class_key, km])
		return true

	func landmark_set_cross_competition(on: bool) -> bool:
		writes.append("cross=%s" % str(on))
		cross = on
		return true

	func landmark_reset_settings() -> void:
		writes.append("reset")

	func landmark_run() -> Dictionary:
		ran = true
		return {"ok": true, "placed": 44, "seconds": 2.4, "error": "",
			"funnels": landmark_funnels()}

	func landmarks() -> Array:
		return []

	func landmark_funnels() -> Array:
		if not ran:
			return []
		return [
			{"kind": "peak", "candidates": 300, "rejected_constraint": 120,
				"rejected_score": 40, "rejected_spacing": 116, "cap": 24,
				"placed": 24, "limit": "at_cap"},
			{"kind": "waterfall", "candidates": 1284, "rejected_constraint": 1149,
				"rejected_score": 0, "rejected_spacing": 124, "cap": 40,
				"placed": 11, "limit": "spacing"},
			{"kind": "cliff", "candidates": 90, "rejected_constraint": 84,
				"rejected_score": 0, "rejected_spacing": 0, "cap": 30,
				"placed": 6, "limit": "no_terrain"},
			{"kind": "mountain_pass", "candidates": 3, "rejected_constraint": 0,
				"rejected_score": 0, "rejected_spacing": 0, "cap": 16,
				"placed": 3, "limit": "candidates"},
			{"kind": "ice_shelf", "candidates": 0, "rejected_constraint": 0,
				"rejected_score": 0, "rejected_spacing": 0, "cap": 3,
				"placed": 0, "limit": "not_buildable"},
		]

	func landmark_headroom() -> Dictionary:
		return {"caps_total": 110, "room_estimate": 64, "last_placed": 44 if ran else 0}

# -- harness ------------------------------------------------------------------

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

## Every `Label`/`Button` text under a node, joined -- the only honest way to ask
## "is this sentence on screen" without hard-coding a tree shape.
static func _texts(n: Node, out: Array) -> Array:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	for c in n.get_children():
		_texts(c, out)
	return out

static func _blob(n: Node) -> String:
	return "\n".join(_texts(n, []))

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(50)
	print("[BOOT] shell up")

	var live_bridge = app.get("bridge")
	var live_api: bool = live_bridge != null and live_bridge.has_method("landmark_kinds")
	print("\n############################################################")
	if live_api:
		print("# LIVE LANDMARK BRIDGE PRESENT -- half B runs against the")
		print("# stub anyway, so the fixtures stay deterministic.")
	else:
		print("# NO LIVE LANDMARK BRIDGE. EngineBridge has no landmark_kinds().")
		print("# Half A therefore asserts the DISCLOSED EMPTY STATE, and every")
		print("# wired assertion below it runs against StubBridge, which")
		print("# implements the locked contract. Nothing here proves the real")
		print("# engine works -- only that this panel does, against that shape.")
	print("############################################################")

	# ── A · the live dock ────────────────────────────────────────────────────
	print("\n=== A1: the category exists and the stub is gone ===")
	var civ: Node = app.find_child("CivilizationWorkspace", true, false)
	_ok("the CIVIL workspace is registered", civ != null, true)
	if civ == null:
		print("_landmark_probe: FAILURE (no workspace)"); get_tree().quit(1); return
	var titles: Array = []
	for e in (civ.get("categories") as Array):
		titles.append(String((e as Dictionary)["title"]))
	print("  info CIVIL categories: ", titles)
	_ok("CIVIL has a Landmarks category", titles.has("Landmarks"), true)
	_ok("v3's 'Points of interest' category is gone", titles.has("Points of interest"), false)

	var civ_blob := _blob(civ)
	_ok("the old 'not a ported concept' note is gone",
		civ_blob.find("not a ported concept") >= 0, false)
	_ok("the old 'Not built' section title is gone",
		civ_blob.find("Not built") >= 0, false)

	print("\n=== A2: the live panel is either wired or DISCLOSED, never quiet ===")
	var live_kinds: Array = live_bridge.landmark_kinds() if live_api else []
	var live_rows_dict: Dictionary = civ.get("_lm_rows")
	print("  info live landmark_kinds(): ", live_kinds.size(), " types")
	print("  info live rows built: ", live_rows_dict.size())
	if live_kinds.is_empty():
		## Either no wrapper at all, or a wrapper over a cdylib that has none.
		_ok("it names the missing binding by name",
			civ_blob.find("landmark_kinds()") >= 0
			or civ_blob.find("reports no types at all") >= 0, true)
		_ok("it lists what it expects", civ_blob.find("landmark_headroom") >= 0, true)
		_ok("hand-stamped icons are still signposted",
			civ_blob.find("Cartography ▸ Assets & landmarks") >= 0, true)
		_ok("...and it drew no row it could not back", live_rows_dict.size(), 0)
	else:
		_ok("a row per live kind", live_rows_dict.size(), live_kinds.size())
		var live_fams: Array = []
		var live_vs := 0
		var live_nb := 0
		for k in live_kinds:
			var kd: Dictionary = k
			if not live_fams.has(String(kd.get("family", ""))):
				live_fams.append(String(kd.get("family", "")))
			if bool(kd.get("needs_viewshed", false)):
				live_vs += 1
			if not bool(kd.get("buildable", true)):
				live_nb += 1
		print("  info live families: ", live_fams)
		print("  info live needs_viewshed: %d - not buildable: %d" % [live_vs, live_nb])
		_ok("the run button is live too", (civ.get("_lm_run_btn") as Button) != null, true)
		_ok("...and pressable with a world absent or present",
			(civ.get("_lm_run_btn") as Button).disabled, false)
		if live_vs > 0:
			_ok("the live panel tags its viewshed types on the row",
				civ_blob.find("[no viewshed]") >= 0, true)
		if live_nb > 0:
			_ok("the live panel says how many types it cannot build",
				civ_blob.find("are listed and disabled") >= 0, true)

	# ── B · the same panel against the locked contract ───────────────────────
	print("\n=== B0: rebuilding CIVIL ▸ Landmarks against StubBridge ===")
	var stub := StubBridge.new()
	stub.has_world = true
	var ws := CivilizationWorkspace.new()
	ws.app = app
	ws.bridge = stub
	ws.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_vp.add_child(ws)
	ws.call("_build_landmarks")
	await _frames(4)
	var rows: Dictionary = ws.get("_lm_rows")
	print("  info rows built: ", rows.size(), " -> ", rows.keys())
	_ok("every kind from landmark_kinds() has a row", rows.size(), StubBridge.KINDS.size())
	var all_present := true
	for k in StubBridge.KINDS:
		if not rows.has(String((k as Dictionary)["key"])):
			all_present = false
			print("  MISSING ", (k as Dictionary)["key"])
	_ok("...and each by its own key", all_present, true)

	print("\n=== B1: needs_viewshed shows ON THE ROW ===")
	var vs_tagged := 0
	var vs_expected := 0
	for k in StubBridge.KINDS:
		var kd: Dictionary = k
		if not bool(kd["needs_viewshed"]):
			continue
		vs_expected += 1
		var rr: Dictionary = rows[String(kd["key"])]
		if _blob(rr["row"]).find("[no viewshed]") >= 0:
			vs_tagged += 1
		else:
			print("  UNTAGGED ", kd["key"])
	_ok("there ARE viewshed types in the fixture", vs_expected > 0, true)
	_ok("every one carries the tag beside its name", vs_tagged, vs_expected)
	var ws_blob := _blob(ws)
	_ok("and § TYPES says how many, and what it costs them",
		ws_blob.find("no viewshed]") >= 0 and ws_blob.find("0.20") >= 0, true)

	print("\n=== B2: buildable:false is listed, disabled, WITH a reason ===")
	var ice: Dictionary = rows["ice_shelf"]
	_ok("its slider is not editable", (ice["slider"] as HSlider).editable, false)
	_ok("its second line states the reason",
		String((ice["count"] as Label).text), "not buildable")
	_ok("its row tooltip carries the why",
		(ice["row"] as Control).tooltip_text.find("no placement rule") >= 0, true)
	_ok("it is dimmed rather than hidden", (ice["row"] as Control).visible, true)
	_ok("§ TYPES says how many are in that state",
		ws_blob.find("are listed and disabled") >= 0, true)

	print("\n=== B3: the four class filter chips ===")
	var chips: Dictionary = ws.get("_lm_chips")
	var chip_text: Array = []
	for k in chips:
		chip_text.append(String((chips[k] as Button).text))
	chip_text.sort()
	print("  info chips: ", chip_text)
	_ok("all + one per class present in the fixture", chips.size(), 5)
	for want in ["continental", "regional", "local", "cultural"]:
		_ok("chip for %s" % want, chips.has(want), true)
	## The badge is derived, not tabulated.
	_ok("the regional chip is badged REG",
		String((chips["regional"] as Button).text).begins_with("REG"), true)

	print("\n=== B4: the run button ===")
	var run_btn: Button = ws.get("_lm_run_btn")
	_ok("it exists", run_btn != null, true)
	_ok("it is not disabled with a world loaded", run_btn.disabled, false)
	_ok("it is labelled for the pass, not for a recompute",
		run_btn.text, "Run landmark pass")

	print("\n=== B5: before a run, rows say so rather than showing zeros ===")
	_ok("Waterfall reads 'not run yet'",
		String((rows["waterfall"]["count"] as Label).text), "not run yet")
	_ok("...and no under-bar is drawn yet",
		(rows["waterfall"]["under"] as Control).visible, false)

	print("\n=== B6 · THE CRUX: after a run, spacing shows and the bars differ ===")
	ws.call("_lm_run")
	await _frames(6)
	_ok("the pass actually ran", stub.ran, true)

	var wf: Dictionary = rows["waterfall"]
	var pk: Dictionary = rows["peak"]
	var track := float(CivilizationWorkspace._lm_track_w())
	var steps := float(CivilizationWorkspace.LM_LADDER.size() - 1)

	var wf_rung := float((wf["slider"] as HSlider).value)
	var wf_cap_px: float = round(wf_rung / steps * track)      ## the slider's own fill
	var wf_bar_px := (wf["bar"] as ColorRect).custom_minimum_size.x
	var pk_rung := float((pk["slider"] as HSlider).value)
	var pk_cap_px: float = round(pk_rung / steps * track)
	var pk_bar_px := (pk["bar"] as ColorRect).custom_minimum_size.x
	print("  info waterfall 40 max / 11 placed -> cap bar %.0f px, placed bar %.0f px"
		% [wf_cap_px, wf_bar_px])
	print("  info peak      24 max / 24 placed -> cap bar %.0f px, placed bar %.0f px"
		% [pk_cap_px, pk_bar_px])

	_ok("Waterfall's token is the spacing word",
		String((wf["token"] as Button).text), "spacing")
	_ok("...and its count is the placed count",
		String((wf["count"] as Label).text), "11 placed ·")
	_ok("...its under-bar is drawn", (wf["under"] as Control).visible, true)
	_ok("...and it is SHORTER than the cap bar", wf_bar_px < wf_cap_px, true)
	_ok("...visibly so, not by a rounding pixel", wf_bar_px < wf_cap_px * 0.5, true)

	_ok("Peak's token is 'at cap'", String((pk["token"] as Button).text), "at cap")
	_ok("...and its two bars are flush", absf(pk_bar_px - pk_cap_px) <= 1.0, true)
	_ok("'at cap' is the only reason drawn in accent",
		(pk["token"] as Button).get_theme_color("font_color"), DccTheme.c("accent"))
	_ok("...spacing is not",
		(wf["token"] as Button).get_theme_color("font_color") == DccTheme.c("accent"), false)

	_ok("Cliff reports no terrain", String((rows["cliff"]["token"] as Button).text), "no terrain")
	_ok("Mountain pass reports candidates",
		String((rows["mountain_pass"]["token"] as Button).text), "candidates")

	print("\n=== B7: group headers count while collapsed, headroom explains ===")
	var groups: Dictionary = ws.get("_lm_groups")
	var phys := String((groups["physical"]["button"] as Button).text)
	print("  info physical header: ", phys)
	_ok("the family header carries armed-of-total", phys.find("of 5 armed") >= 0, true)
	_ok("...and the family's placed total", phys.find("41 placed") >= 0, true)
	var head := String((ws.get("_lm_head_note") as Label).text)
	print("  info headroom: ", head)
	_ok("the headroom line is the engine's three numbers",
		head.find("caps total 110") >= 0 and head.find("about 64") >= 0
		and head.find("placed 44") >= 0, true)
	_ok("...and it says 'about'", head.find("about") >= 0, true)
	var crowd := String((ws.get("_lm_crowd_note") as Label).text)
	print("  info crowding: ", crowd)
	_ok("the crowding dial converts to km on the map",
		crowd.find("34 km") >= 0, true)

	print("\n=== B8: the zero stop disarms and KEEPS the number ===")
	stub.writes.clear()
	var wsl: HSlider = wf["slider"]
	wsl.drag_started.emit()
	wsl.value = 0.0
	wsl.drag_ended.emit(true)
	await _frames(2)
	print("  info writes: ", stub.writes)
	_ok("it wrote armed=false", stub.writes.has("armed:waterfall=false"), true)
	_ok("it did NOT write a zero cap", stub.writes.has("cap:waterfall=0"), false)
	_ok("the readout reads off", String((wf["readout"] as Label).text), "off")
	_ok("the row promises the number survived",
		String((wf["count"] as Label).text), "was 40")

	print("\n=== B9: dragging up from off resumes at the retained number ===")
	stub.writes.clear()
	wsl.drag_started.emit()
	wsl.value = 1.0                      ## the first detent off the zero stop
	wsl.drag_ended.emit(true)
	await _frames(2)
	print("  info writes: ", stub.writes)
	_ok("it resumed at 40, not at 1", stub.writes.has("cap:waterfall=40"), true)
	_ok("...and re-armed", stub.writes.has("armed:waterfall=true"), true)
	_ok("the readout agrees", String((wf["readout"] as Label).text), "40 max")
	## 40 is NOT a rung -- the ladder runs ... 20 · 30 · 50 ... -- so this is the
	## exact assertion that caught the first build quantising a retained cap down
	## to 30. The handle sits on the nearest detent; the number does not move.
	_ok("...while the handle sits on the nearest detent (30's)",
		int((wf["slider"] as HSlider).value), CivilizationWorkspace.LM_LADDER.find(30))
	_ok("40 really is off-ladder, so that assertion means something",
		CivilizationWorkspace.LM_LADDER.has(40), false)

	print("\n=== B10: the class filter dims rather than hides ===")
	ws.call("_lm_set_filter", "local")
	await _frames(2)
	_ok("a matching row is at full ink", (rows["cliff"]["row"] as Control).modulate.a, 1.0)
	_ok("a non-matching row is dimmed",
		(rows["peak"]["row"] as Control).modulate.a < 0.5, true)
	_ok("...and still on screen", (rows["peak"]["row"] as Control).visible, true)
	ws.call("_lm_set_filter", "")
	await _frames(2)
	_ok("clearing the filter restores it", (rows["peak"]["row"] as Control).modulate.a, 1.0)

	print("\n=== B11: the funnel popover proves it in numbers ===")
	ws.call("_lm_open_funnel", "waterfall")
	await _frames(4)
	var fp = ws.get("_lm_funnel")
	_ok("a popover was built", fp != null, true)
	if fp != null:
		var fblob := _blob(fp)
		print("  info funnel:\n    ", fblob.replace("\n", "\n    "))
		_ok("it opens the candidate count", fblob.find("1284") >= 0, true)
		_ok("it names the spacing rejection", fblob.find("rejected by spacing") >= 0, true)
		_ok("'cap 40 · not reached' earns its row",
			fblob.find("cap 40") >= 0 and fblob.find("not reached") >= 0, true)
		_ok("it ends on the placed total", fblob.find("11 placed") >= 0, true)
		_ok("and on the unused cap", fblob.find("29 of the cap unused") >= 0, true)
		fp.hide()

	print("\n=== B12: bulk arm/off over a family ===")
	stub.writes.clear()
	ws.call("_lm_bulk", "transportation", true)
	await _frames(2)
	print("  info writes: ", stub.writes)
	_ok("the disarmed Pilgrim way was armed", stub.writes.has("armed:pilgrim_way=true"), true)
	_ok("...at its default cap", stub.writes.has("cap:pilgrim_way=12"), true)
	ws.call("_lm_bulk", "transportation", false)
	await _frames(2)
	_ok("off disarms the family", String((rows["mountain_pass"]["readout"] as Label).text), "off")

	# ── C · Assets ▸ Landmark types ▸ ────────────────────────────────────────
	print("\n=== B13: every placement setter actually writes, on release ===")
	stub.writes.clear()
	var cs := _find_row_slider(ws, "Crowding")
	_ok("the Crowding dial exists", cs != null, true)
	cs.value = 0.70
	cs.drag_ended.emit(true)
	await _frames(2)
	print("  info writes: ", stub.writes)
	_ok("landmark_set_crowding on release, not per tick",
		stub.writes, ["crowding=0.70"])
	_ok("...and the km sentence followed the dial (34 x 0.70)",
		String((ws.get("_lm_crowd_note") as Label).text).find("24 km") >= 0, true)

	stub.writes.clear()
	var rs := _find_row_slider(ws, "Regional")
	_ok("the per-class radius rows exist under + advanced", rs != null, true)
	rs.value = 50.0
	rs.drag_ended.emit(true)
	await _frames(2)
	print("  info writes: ", stub.writes)
	_ok("landmark_set_class_radius carries the CLASS KEY, not an index",
		stub.writes.has("radius:regional=50.0"), true)

	stub.writes.clear()
	var cb := _find_toggle(ws, "Types compete with each other")
	_ok("the cross-type competition toggle exists", cb != null, true)
	cb.button_pressed = false
	await _frames(2)
	print("  info writes: ", stub.writes)
	_ok("landmark_set_cross_competition was written", stub.writes.has("cross=false"), true)

	print("\n=== C1: the cascade, live (no bridge -> one disclosed row) ===")
	var live_pop := _find_popup(app, "LandmarkTypes")
	_ok("Assets ▸ Landmark types ▸ exists", live_pop != null, true)
	if live_pop != null:
		var live_rows: Array = []
		for i in live_pop.item_count:
			live_rows.append("%s%s" % [live_pop.get_item_text(i),
				"  [disabled]" if live_pop.is_item_disabled(i) else ""])
		print("  info rows: ", live_rows)
		_ok("its two real destinations are there",
			_pop_has(live_pop, "Landmark icons…") and _pop_has(live_pop, "Landmark label style…"),
			true)
		if not live_api:
			_ok("the missing vocabulary is one disabled row",
				_pop_has(live_pop, "No landmark types"), true)
			var idx := _pop_index(live_pop, "No landmark types")
			_ok("...disabled", live_pop.is_item_disabled(idx), true)
			_ok("...with a reason attached, per menus.gd's own rule",
				live_pop.get_item_tooltip(idx).find("landmark_kinds()") >= 0, true)

	print("\n=== C2: the cascade against the contract ===")
	var m := DccMenus.new()
	m.set("_shell", app)
	m.set("_bridge", stub)
	m.set("_host", app)
	var host_pop := PopupMenu.new()
	app.add_child(host_pop)
	m.call("_build_landmark_types_menu", host_pop)
	await _frames(2)
	var pop: PopupMenu = m.get("_landmark_popup")
	var pop_rows: Array = []
	for i in pop.item_count:
		pop_rows.append(pop.get_item_text(i))
	print("  info level 2: ", pop_rows)
	_ok("a row per family, plus hand-placed and the two destinations",
		pop.item_count >= 5, true)
	_ok("the family row carries its counts",
		_pop_has_sub(pop, "Physical") and _pop_find(pop, "Physical").find("armed") >= 0, true)
	print("  info physical row: ", _pop_find(pop, "Physical"))
	## 3, not 4: the fixture arms five Physical types but one of them is
	## `buildable: false`, and neither readout counts a type the engine cannot
	## place as armed. The dock's own header says 3 too (B7 above), which is the
	## point -- two readouts of one fact that disagreed would be worse than
	## either being wrong alone.
	_ok("...armed-of-total is the engine's, not a guess",
		_pop_find(pop, "Physical").find("3 of 5 armed") >= 0, true)
	_ok("...and it agrees with the dock's own header",
		String((groups["physical"]["button"] as Button).text).find("3 of 5 armed") >= 0, true)
	_ok("...and its placed total", _pop_find(pop, "Physical").find("41 placed") >= 0, true)

	var fams: Array = m.get("_landmark_families")
	_ok("two families were built", fams.size(), 2)
	var sub: PopupMenu = (fams[0] as Dictionary)["popup"]
	var sub_rows: Array = []
	for i in sub.item_count:
		sub_rows.append(sub.get_item_text(i))
	print("  info level 3: ", sub_rows)
	var sub_blob := "\n".join(sub_rows)
	_ok("a leaf shows cap · placed · reason",
		sub_blob.find("40 max · 11 placed · spacing") >= 0, true)
	_ok("a disarmed leaf shows its retained number",
		sub_blob.find("off · was 2") >= 0, true)
	_ok("an unbuildable leaf says so instead of a cap it cannot honour",
		sub_blob.find("Ice shelf   not buildable") >= 0, true)
	_ok("the family's own destination row is there",
		sub_blob.find("Open Physical in the dock") >= 0, true)
	_ok("and the read-only signpost", sub_blob.find("do not arm a type") >= 0, true)

	print("\n=== C3: picking a family opens the dock at it ===")
	## The live dock, not the stub one -- this is the shipped path.
	m.set("_bridge", live_bridge)
	m.call("_open_landmark_dock", "physical")
	await _frames(4)
	var open_titles: Array = []
	for e in (civ.get("categories") as Array):
		var ee: Dictionary = e
		if (ee["body"] as Control).visible:
			open_titles.append(String(ee["title"]))
	print("  info open CIVIL categories: ", open_titles)
	_ok("CIVIL ▸ Landmarks is the open category", open_titles.has("Landmarks"), true)

	print("\n=== D: reset calls the engine and rebuilds from what it answers ===")
	stub.writes.clear()
	ws.call("_lm_reset")
	await _frames(4)
	print("  info writes: ", stub.writes)
	_ok("landmark_reset_settings() was called", stub.writes.has("reset"), true)
	_ok("...and the panel rebuilt itself from the engine afterwards",
		(ws.get("_lm_rows") as Dictionary).size(), StubBridge.KINDS.size())

	# -- E - the real engine, end to end -------------------------------------
	print("\n=== E: a real world, and the LIVE landmark pass over it ===")
	if not live_api or live_kinds.is_empty():
		print("  SKIPPED -- no live landmark vocabulary in this build.")
	else:
		live_bridge.generate({
			"seed": 483920, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
			"archetype": "", "villages": true, "sea_level": 0.45,
		})
		while live_bridge.generating:
			await get_tree().create_timer(0.25).timeout
		await get_tree().create_timer(1.0).timeout
		if app.open_project_dialog:
			app.open_project_dialog.hide()
		await _frames(8)
		_ok("a world generated", live_bridge.has_world, true)
		var live_btn: Button = civ.get("_lm_run_btn")
		_ok("the run button is pressable with a world up", live_btn.disabled, false)
		await civ.call("_lm_run")
		await _frames(8)
		print("  info run note: ", String((civ.get("_lm_run_note") as Label).text))
		print("  info headroom: ", String((civ.get("_lm_head_note") as Label).text))
		var live_funnels: Array = live_bridge.landmark_funnels()
		print("  info live funnels: ", live_funnels.size())
		_ok("the pass returned per-type funnels", live_funnels.size() > 0, true)
		_ok("...and placements", live_bridge.landmarks().size() > 0, true)
		## Every reason the live run actually produced, and the row that shows it.
		var seen := {}
		var live_rows2: Dictionary = civ.get("_lm_rows")
		var mismatched := 0
		for f in live_funnels:
			var fd: Dictionary = f
			var lim := String(fd.get("limit", ""))
			seen[lim] = int(seen.get(lim, 0)) + 1
			var rr: Dictionary = live_rows2.get(String(fd.get("kind", "")), {})
			if rr.is_empty():
				continue
			var tk: Button = rr["token"]
			if tk.visible and tk.text != String(
					CivilizationWorkspace.LM_LIMIT_WORD.get(lim, lim)):
				mismatched += 1
				print("  MISMATCH %s: row says '%s', engine says '%s'" % [
					fd.get("kind", ""), tk.text, lim])
		print("  info limiting reasons across the live run: ", seen)
		_ok("no row disagrees with the engine's own token", mismatched, 0)
		_ok("the run note carries a placed count",
			String((civ.get("_lm_run_note") as Label).text).find("Placed") >= 0, true)

		## The assertion that caught the engine spelling its tokens with spaces.
		## `at cap` reading correctly was NOT enough -- the fallback prints an
		## unknown token verbatim, so the word looked right while §2.2's accent
		## and the tooltip behind it were both silently dead. Colour and tooltip
		## are the parts a lookup miss actually breaks, so they are what is
		## asserted.
		var at_cap_rows := 0
		var accented := 0
		var silent := 0
		for f2 in live_funnels:
			var fd2: Dictionary = f2
			var rr2: Dictionary = live_rows2.get(String(fd2.get("kind", "")), {})
			if rr2.is_empty():
				continue
			var tk2: Button = rr2["token"]
			if not tk2.visible:
				continue
			if tk2.tooltip_text.strip_edges() == "" \
					or tk2.tooltip_text.strip_edges() == "Click for the funnel.":
				silent += 1
				print("  SILENT TOKEN  %s -> '%s'" % [fd2.get("kind", ""), tk2.text])
			if CivilizationWorkspace._lm_limit_key(String(fd2.get("limit", ""))) == "at_cap":
				at_cap_rows += 1
				if tk2.get_theme_color("font_color") == DccTheme.c("accent"):
					accented += 1
		print("  info at-cap rows: %d, drawn in accent: %d" % [at_cap_rows, accented])
		_ok("the live run produced at-cap rows to test", at_cap_rows > 0, true)
		_ok("every one is drawn in accent, per 2.2", accented, at_cap_rows)
		_ok("no displayed token hovers to nothing", silent, 0)

	print("\n_landmark_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	if not live_api:
		print("_landmark_probe: NOTE -- the live landmark bridge was ABSENT.")
		print("  Half A tested the disclosed empty state; halves B and C tested")
		print("  this panel against StubBridge. The real engine was not exercised.")
	get_tree().quit(1 if _fail > 0 else 0)

# -- popup helpers ------------------------------------------------------------

## A dock parameter row is an `HBoxContainer` holding its label and its control,
## so a row is found by the pair rather than by an index into a tree this file
## does not own.
static func _find_row_slider(n: Node, label_text: String) -> HSlider:
	if n is HBoxContainer:
		var named := false
		var s: HSlider = null
		for c in n.get_children():
			if c is Label and (c as Label).text == label_text:
				named = true
			elif c is HSlider:
				s = c
		if named and s != null:
			return s
	for c in n.get_children():
		var r := _find_row_slider(c, label_text)
		if r != null:
			return r
	return null

static func _find_toggle(n: Node, label_text: String) -> CheckBox:
	if n is HBoxContainer:
		var named := false
		var b: CheckBox = null
		for c in n.get_children():
			if c is Label and (c as Label).text == label_text:
				named = true
			elif c is CheckBox:
				b = c
		if named and b != null:
			return b
	for c in n.get_children():
		var r := _find_toggle(c, label_text)
		if r != null:
			return r
	return null

func _find_popup(n: Node, want: String) -> PopupMenu:
	if n is PopupMenu and n.name == want:
		return n
	for c in n.get_children(true):
		var r := _find_popup(c, want)
		if r != null:
			return r
	return null

static func _pop_has(p: PopupMenu, prefix: String) -> bool:
	return _pop_index(p, prefix) >= 0

static func _pop_index(p: PopupMenu, prefix: String) -> int:
	for i in p.item_count:
		if p.get_item_text(i).begins_with(prefix):
			return i
	return -1

static func _pop_find(p: PopupMenu, prefix: String) -> String:
	var i := _pop_index(p, prefix)
	return p.get_item_text(i) if i >= 0 else ""

static func _pop_has_sub(p: PopupMenu, prefix: String) -> bool:
	var i := _pop_index(p, prefix)
	return i >= 0 and p.get_item_submenu(i) != ""
