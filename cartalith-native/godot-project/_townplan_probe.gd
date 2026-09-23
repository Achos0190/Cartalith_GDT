extends Node
## Ruling J probe -- the City Viewer's `§ TOWN PLAN` section, driven through
## its own controls on a real generated world.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _townplan_probe.tscn
##
## Windowed (not --headless) so the before/after screenshots are real pixels.
## Asserts, per the brief:
##  (a) the target settlement's layout changes under culture / rule set /
##      Regenerate, and returns byte-identical to the untouched layout when
##      every override is set back;
##  (b) every other settlement's layout, the settlement list, roads, trade
##      and the stale-stage report are byte-identical before and after, and
##      the map's town cache loses exactly the one entry;
##  (c) closing and reopening the window shows the stored override state.

var app: Node
var _fail := 0
const OUT := "user://townplan_"

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("TPL %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _find(n: Node, pred: Callable) -> Node:
	for c in n.get_children():
		if pred.call(c):
			return c
		var r := _find(c, pred)
		if r != null:
			return r
	return null

func _option(cv: Node, tip_prefix: String) -> OptionButton:
	## `DccWidgets.choice` puts the tooltip on its row, not on the dropdown.
	return _find(cv, func(c): return c is OptionButton \
		and String((c.get_parent() as Control).tooltip_text).begins_with(tip_prefix)) as OptionButton

func _button(cv: Node, text: String) -> Button:
	return _find(cv, func(c): return c is Button and not (c is OptionButton) and c.text == text) as Button

func _bytes(v) -> PackedByteArray:
	return var_to_bytes(v)

func _layout_bytes(i: int) -> PackedByteArray:
	return _bytes(app.bridge.urban_layouts(PackedInt32Array([i])))

func _sig(l: Dictionary) -> String:
	return "edges=%d blocks=%d parcels=%d buildings=%d wall=%s" % [int(l.get("edge_count", 0)),
		(l.get("blocks", []) as Array).size(), (l.get("parcels", []) as Array).size(),
		(l.get("buildings", []) as Array).size(), str(l.has("wall_ring"))]

func _pick(ob: OptionButton, idx: int) -> void:
	ob.select(idx)
	ob.item_selected.emit(idx)

func _shot(cv: Window, name: String) -> void:
	await RenderingServer.frame_post_draw
	var img: Image = cv.get_texture().get_image()
	var path := OUT + name + ".png"
	img.save_png(path)
	print("TPL shot %s -> %s (%dx%d)" % [name, ProjectSettings.globalize_path(path), img.get_width(), img.get_height()])


func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 3000:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	if not app.bridge.has_world:
		print("TPL  !! generate failed")
		get_tree().quit(1)
		return
	var places: Array = app.bridge.settlements()
	var n := mini(places.size(), 40)
	var with_layout: Array = []
	for i in n:
		if app.bridge.urban_layouts(PackedInt32Array([i])).size() > 0:
			with_layout.append(i)
	print("TPL world: %d settlements, %d of the first %d have a layout" % [places.size(), with_layout.size(), n])
	_check("at least three settlements have layouts", with_layout.size() >= 3)
	if with_layout.size() < 3:
		get_tree().quit(1)
		return
	## A town rather than a hamlet: its plan has the most to change.
	var k: int = with_layout[0]
	var best_pop := -1
	for i in with_layout:
		var p := int((places[i] as Dictionary).get("population", 0))
		if p > best_pop and p < 20000:
			best_pop = p
			k = i
	print("TPL target: #%d %s (%s, pop %d)" % [k, places[k].get("name", "?"), places[k].get("kind", "?"), best_pop])

	# -- snapshots of everything that must NOT move ---------------------------
	var before := {}
	for i in with_layout:
		before[i] = _layout_bytes(i)
	var world_before := {
		"settlements": _bytes(app.bridge.settlements()),
		"roads": _bytes(app.bridge.roads()),
		"trade": _bytes(app.bridge.trade_balances()),
		"stale": _bytes(app.bridge.stale_stages()),
	}
	var details_before := {}
	for i in n:
		if i != k:
			details_before[i] = _bytes(app.bridge.civ_settlement_details(i))
	## Seed the map's town cache with every layout, so the test can tell a
	## one-entry invalidation from a wholesale one.
	var ov = app.viewport.overlay
	var idx := PackedInt32Array(with_layout)
	ov.set_urban_layouts(idx, app.bridge.urban_layouts(idx))

	# -- open, screenshot the untouched town -------------------------------
	app.open_city_viewer(k)
	await _frames(8)
	var cv: Window = app.city_viewer_window
	var t0: Dictionary = cv._layout
	print("TPL original: %s" % _sig(t0))
	print("TPL canvas width %.0f of %d" % [cv._canvas.size.x, cv.size.x])
	var culture := _option(cv, "The culture profile")
	var rules := _option(cv, "World rules follows")
	var walls := _option(cv, "Auto is _umWallSpec")
	var regen := _button(cv, "Regenerate")
	_check("Town plan section has culture, rule set, walls and Regenerate",
		culture != null and rules != null and walls != null and regen != null)
	if culture == null or rules == null or regen == null:
		get_tree().quit(1)
		return
	_check("an unedited settlement opens on medieval / World rules",
		culture.get_item_text(culture.selected) == "Medieval" and rules.selected == 0,
		"%s / %s" % [culture.get_item_text(culture.selected), rules.get_item_text(rules.selected)])
	## Control measured at HEAD (no Town plan): 676 of 940. With the dropdowns
	## sized to their longest item it was 594 (and 436 before clip_text).
	_check("the Town plan did not widen the side column (plan keeps > 70% of the window)",
		cv._canvas.size.x > cv.size.x * 0.7, "canvas %.0f of %d" % [cv._canvas.size.x, cv.size.x])
	var wide: Array = []
	var st: Array = [cv._info, cv._legend]
	while not st.is_empty():
		var nd: Node = st.pop_back()
		if nd is Control and (nd as Control).get_combined_minimum_size().x > 262:
			wide.append("%s:%s %.0f" % [nd.get_class(), String(nd.get("text")) if nd.get("text") != null else "", (nd as Control).get_combined_minimum_size().x])
		st.append_array(nd.get_children())
	print("TPL wide leaves: %s" % [wide])
	await _shot(cv, "1_before")

	# -- culture -> venus ---------------------------------------------------
	var venus_i := -1
	for j in culture.item_count:
		if culture.get_item_text(j) == "Venus":
			venus_i = j
	_pick(culture, venus_i)
	await _frames(6)
	var t1: Dictionary = cv._layout
	print("TPL venus:    %s" % _sig(t1))
	_check("(a) culture=venus changed the town", _bytes(t1) != _bytes(t0))
	_check("(b) the map cache lost the target only",
		not ov._urban_layouts.has(k) and ov._urban_layouts.size() == with_layout.size() - 1,
		"cache=%d of %d" % [ov._urban_layouts.size(), with_layout.size()])
	await _shot(cv, "2_venus")

	# -- Regenerate (new variant) -------------------------------------------
	regen = _button(cv, "Regenerate")
	regen.pressed.emit()
	await _frames(6)
	var t2: Dictionary = cv._layout
	print("TPL venus #1: %s" % _sig(t2))
	_check("(a) Regenerate produced a different town", _bytes(t2) != _bytes(t1))
	_check("variant stored as 1", int(app.bridge.civ_settlement_details(k).get("variant", -1)) == 1)
	await _shot(cv, "3_venus_regenerated")

	# -- rule set -> walled market town (back on medieval) ------------------
	culture = _option(cv, "The culture profile")
	_pick(culture, 1 - venus_i if culture.item_count == 2 else 0)
	await _frames(4)
	rules = _option(cv, "World rules follows")
	var mt := -1
	for j in rules.item_count:
		if rules.get_item_text(j).findn("market") >= 0:
			mt = j
	_pick(rules, mt)
	await _frames(6)
	var t3: Dictionary = cv._layout
	print("TPL medieval #1 market_town: %s" % _sig(t3))
	_check("(a) rule set=market_town changed the town", _bytes(t3) != _bytes(t2))
	await _shot(cv, "4_market_town")

	# -- (c) round trip: close and reopen ------------------------------------
	cv.hide()
	await _frames(3)
	app.open_city_viewer(k)
	await _frames(8)
	culture = _option(cv, "The culture profile")
	rules = _option(cv, "World rules follows")
	var vtexts: Array = []
	var stack: Array = [cv]
	while not stack.is_empty():
		var nd: Node = stack.pop_back()
		if nd is Label:
			vtexts.append((nd as Label).text)
		stack.append_array(nd.get_children())
	_check("(c) reopened on the stored rule set",
		rules.get_item_text(rules.selected).findn("market") >= 0, rules.get_item_text(rules.selected))
	_check("(c) reopened on the stored culture (medieval, explicit)",
		culture.get_item_text(culture.selected) == "Medieval", culture.get_item_text(culture.selected))
	_check("(c) reopened on the stored variant", vtexts.has("#1"))
	_check("(c) reopened layout is the regenerated one", _bytes(cv._layout) == _bytes(t3))

	# -- every override back -> the untouched town, byte for byte ------------
	_pick(rules, 0)
	await _frames(4)
	var orig := _button(cv, "Original")
	_check("Original button shows while a variant is set", orig != null)
	if orig != null:
		orig.pressed.emit()
	await _frames(6)
	_check("(a) all overrides back = byte-identical to the untouched town",
		_bytes(cv._layout) == _bytes(t0) and _layout_bytes(k) == before[k])

	# -- (b) nothing else moved ---------------------------------------------
	var moved := []
	for i in with_layout:
		if i != k and _layout_bytes(i) != before[i]:
			moved.append(i)
	_check("(b) every other settlement's layout is byte-identical", moved.is_empty(),
		"compared %d, moved %s" % [with_layout.size() - 1, moved])
	for key in world_before.keys():
		var now: PackedByteArray = _bytes({"settlements": app.bridge.settlements(), "roads": app.bridge.roads(),
			"trade": app.bridge.trade_balances(), "stale": app.bridge.stale_stages()}[key])
		_check("(b) %s unchanged" % key, now == world_before[key])
	var dmoved := []
	for i in details_before.keys():
		if _bytes(app.bridge.civ_settlement_details(i)) != details_before[i]:
			dmoved.append(i)
	_check("(b) no other settlement's stored overrides changed", dmoved.is_empty(), str(dmoved))
	var others_cached := true
	for i in with_layout:
		if i != k and not ov._urban_layouts.has(i):
			others_cached = false
	_check("(b) the map cache still holds every other town", others_cached)

	# -- final visual: back on the variant for the after shot -----------------
	_pick(_option(cv, "The culture profile"), venus_i)
	await _frames(6)
	await _shot(cv, "5_after_panel")

	print("TPL done: %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
