extends RefCounted
## Ruling HX (2026-11-24): CIVIL > Factions > City-states, driven through the REAL shell on each form (`TOOLING_PLAN.md` TP-2).
##   python -I cartalith-native/tools/fennara_drive.py GODOT --shell desktop|tablet|phone res://tools/city_states_section_drive.gd \
##       --dll DLL --out DIR --root ROOT --timeout 400
## Steps: past the start card; a 0-faction world (setting `civ.factions` 0 then the engine's generate) so Ruling HK's minor
## seats exist; open CIVIL > Factions; one `CityStateRow<k>` and one `CityStatePromote<k>` per seat of `civ_spheres()`;
## a real click on a row selects that seat and centres the view on it; a real click on "Make this sphere a faction" makes
## one faction; Edit > Undo returns zero factions and the same rows; `civ.city_states` off shows the note and no rows.
## Mutant: `city_states_section_mutant.py`. Every check prints `ok` or `FAIL`; the launcher exits 1 on any FAIL.

var _fails := 0
var app: Node
var get_phone := false
var bridge


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _find(ctx, t: String, cls: String = "") -> Dictionary:
	for e in DriveCtx.dump_ui(ctx, t):
		if cls == "" or e["class"] == cls:
			return e
	return {}


func _wait_for(ctx, t: String, secs: float = 20.0, cls: String = "") -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		var e := _find(ctx, t, cls)
		if not e.is_empty():
			return e
		await ctx.wait(0.25)
	return {}


## The Control named `n` under the shell, or null.
func _named(n: String) -> Control:
	return app.find_child(n, true, false) as Control


## Real click on the control named `n`, scrolled into view first when it sits in a ScrollContainer.
func _click_named(ctx, n: String) -> bool:
	var c := _named(n)
	if c == null:
		return false
	if get_phone:
		await _raise_sheet(ctx)
	var p := c.get_parent()
	while p != null and not (p is ScrollContainer):
		p = p.get_parent()
	if p != null:
		(p as ScrollContainer).ensure_control_visible(c)
		await ctx.wait(0.4)
	await DriveCtx.click(ctx, DriveCtx.rect_of(c).get_center())
	await ctx.wait(1.0)
	return true


## The phone's CIVIL sheet opens at its peek: raise it to full (the detent a drag on its handle ends at) so the dock's rows are
## on screen to be clicked. Does nothing on the desktop and tablet forms.
func _raise_sheet(ctx) -> void:
	for n in app.get_tree().root.find_children("*", "Control", true, false):
		if n.has_method("_set_phone_detent"):
			n.call("_set_phone_detent", "full", false)
			await ctx.wait(0.8)
			return


## Scroll the dock's body so `c` is on screen.
func _scroll_to(ctx, c: Control) -> void:
	var p := c.get_parent()
	while p != null and not (p is ScrollContainer):
		p = p.get_parent()
	if p != null:
		(p as ScrollContainer).ensure_control_visible(c)
		await ctx.wait(0.4)


func _count(prefix: String) -> int:
	var k := 0
	for c in app.find_children(prefix + "*", "Button", true, false):
		k += 1
	return k


func run(ctx) -> void:
	var root: Node = ctx.get_scene_root()
	app = root if root.has_method("select_domain") else root.find_child("App", true, false)
	bridge = app.bridge
	var size: Vector2 = root.get_viewport().get_visible_rect().size
	var phone := size.x < 700.0
	get_phone = phone
	ctx.log("form: canvas %s %s" % [str(size), "phone" if phone else "desktop/tablet"])
	if phone:
		var nw := await _wait_for(ctx, "+ NEW WORLD")
		_check(ctx, "worlds list shows + NEW WORLD", not nw.is_empty())
		await DriveCtx.click(ctx, Vector2(nw["rect"][0] + nw["rect"][2] / 2.0, nw["rect"][1] + nw["rect"][3] / 2.0))
		var cancel := await _wait_for(ctx, "CANCEL")
		if not cancel.is_empty():
			await DriveCtx.click(ctx, Vector2(cancel["rect"][0] + cancel["rect"][2] / 2.0, cancel["rect"][1] + cancel["rect"][3] / 2.0))
	else:
		var cont := await _wait_for(ctx, "Continue without a world")
		_check(ctx, "start card shows Continue", not cont.is_empty())
		if not cont.is_empty():
			await DriveCtx.click(ctx, Vector2(cont["rect"][0] + cont["rect"][2] / 2.0, cont["rect"][1] + cont["rect"][3] / 2.0))
	await ctx.wait(1.0)

	# a 0-faction world: the setting 0, then the engine's generate (Ruling HK's minor seats)
	bridge.param_set("civ.factions", 0)
	bridge.generate({"seed": 483920, "width_km": 800.0, "grid_w": 512, "grid_h": 328, "archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await ctx.wait(0.5)
	while (bool(bridge.generating) or bridge.settlements().size() == 0) and Time.get_ticks_msec() - t0 < 300000:
		await ctx.wait(0.5)
	await ctx.wait(3.0)
	_check(ctx, "positive control: the world has 0 factions", int(bridge.civ_faction_count()) == 0, str(bridge.civ_faction_count()))
	var seats: Array = bridge.civ_spheres().get("seats", [])
	_check(ctx, "positive control: the engine reports city-state seats", seats.size() > 0, str(seats.size()))
	var minor := seats.filter(func(s): return int(s["population"]) < 5000).size()
	ctx.log("seats %d, minor (pop < 5000) %d" % [seats.size(), minor])
	app.viewport.set_layer_visible("territory", true)
	app.viewport.reset_view()
	app.select_domain_category("civilization", "Factions")
	await ctx.wait(2.0)

	# the section
	if get_phone:
		await _raise_sheet(ctx)
	_check(ctx, "the City-states section header is on screen or in the dock", _named("CityStateRow0") != null)
	_check(ctx, "one row per seat", _count("CityStateRow") == seats.size(), "%d rows, %d seats" % [_count("CityStateRow"), seats.size()])
	_check(ctx, "one Make-this-sphere button per seat", _count("CityStatePromote") == seats.size())
	var r0 := _named("CityStateRow0") as Button
	var sorted := seats.duplicate()
	sorted.sort_custom(func(a, b): return int(a["population"]) > int(b["population"]))
	var top: Dictionary = sorted[0]
	_check(ctx, "row 0 names the most populous seat with its population and aligned count",
		r0 != null and r0.text.to_lower().begins_with(String(top["name"]).to_lower()) and r0.text.to_lower().contains("pop %d" % int(top["population"])) and r0.text.to_lower().contains("%d aligned" % (top["members"] as PackedInt64Array).size()),
		r0.text if r0 != null else "no row")
	await _scroll_to(ctx, _named("CityStatePromote1"))
	await ctx.capture("section")

	# promote, then undo (before any selection: on the phone a selection opens Inspect over the dock)
	var si := int(top["index"])
	var st: Dictionary = bridge.settlements()[si]
	var n_members := (top["members"] as PackedInt64Array).size()
	_check(ctx, "a click lands on Make this sphere a faction", await _click_named(ctx, "CityStatePromote0"))
	_check(ctx, "one faction now", int(bridge.civ_faction_count()) == 1, str(bridge.civ_faction_count()))
	var ss: Array = bridge.settlements()
	_check(ctx, "the seat and its aligned places are in it", ss.filter(func(s): return int(s.get("faction", 0)) == 1).size() == n_members + 1)
	_check(ctx, "the section refilled without that seat", _count("CityStateRow") == seats.size() - 1, str(_count("CityStateRow")))
	await ctx.capture("promoted")

	# undo
	var undone: bool = await DriveCtx.menu_open(ctx, "Edit", "Undo")
	await ctx.wait(1.5)
	_check(ctx, "Edit > Undo pressed", undone)
	_check(ctx, "undo: zero factions", int(bridge.civ_faction_count()) == 0, str(bridge.civ_faction_count()))
	_check(ctx, "undo: the rows are back", _count("CityStateRow") == seats.size(), str(_count("CityStateRow")))
	await ctx.capture("undone")

	# select a row: the seat is selected and the view is centred on it
	## Zoomed in first: at the fit-the-world zoom the map already covers the view and cannot move (`_keep_map_covering`).
	app.viewport._zoom_at(app.viewport.size * 0.5, 6.0)
	await ctx.wait(1.0)
	app.viewport.move_view_to(0.0, 0.0)
	await ctx.wait(0.5)
	_check(ctx, "positive control: the seat is off screen before the click", not Rect2(Vector2.ZERO, app.viewport.size).has_point(app.viewport.grid_to_view(float(st["x"]), float(st["y"]))))
	_check(ctx, "a click lands on row 0", await _click_named(ctx, "CityStateRow0"))
	_check(ctx, "the seat is the selection", String(app.selection.get("kind", "")) == "settlement" and int(app.selection.get("id", -1)) == si, str(app.selection))
	## On screen, not centred: a seat near the map's edge cannot be centred (`_keep_map_covering` holds the map over the view).
	var on_screen := Rect2(Vector2.ZERO, app.viewport.size).has_point(app.viewport.grid_to_view(float(st["x"]), float(st["y"])))
	_check(ctx, "the seat is on screen after the click", on_screen, str(app.viewport.grid_to_view(float(st["x"]), float(st["y"]))))
	await ctx.capture("selected")

	# off: an older save
	app.clear_selection()
	if get_phone:
		await _raise_sheet(ctx)
	bridge.param_set("civ.city_states", false)
	app.place_editor_window.polity_moved.emit()
	await ctx.wait(1.0)
	_check(ctx, "off: no rows", _count("CityStateRow") == 0)
	var note := _find(ctx, "City-states are off")
	if note.is_empty():
		var c := _named("CityStateRow0")
		for l in app.find_children("*", "Label", true, false):
			if (l as Label).text.begins_with("City-states are off"):
				note = {"text": (l as Label).text}
	_check(ctx, "off: the note says so", not note.is_empty())
	var nl: Control = null
	for l in app.find_children("*", "Label", true, false):
		if (l as Label).text.begins_with("City-states are off"):
			nl = l
	if nl != null:
		var p := nl.get_parent()
		while p != null and not (p is ScrollContainer):
			p = p.get_parent()
		if p != null:
			(p as ScrollContainer).ensure_control_visible(nl)
			await ctx.wait(0.4)
	await ctx.capture("off")
	bridge.param_set("civ.city_states", true)
	ctx.log("CITY-STATES SECTION %s" % ("FAIL: %d check(s)" % _fails if _fails > 0 else "PASS"))
