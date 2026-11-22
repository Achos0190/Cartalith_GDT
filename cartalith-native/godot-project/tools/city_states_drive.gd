extends RefCounted
## F26 / Ruling GL (owner 2026-11-22): city-states and their spheres of influence, driven through the REAL shell
## (`TOOLING_PLAN.md` TP-2). Run:
##   python -I cartalith-native/tools/fennara_drive.py GODOT --shell desktop res://tools/city_states_drive.gd \
##       --dll DLL --out DIR --root C:/cw/<own scratch>
## Steps: (1) past the welcome card with a real click; (2) a zero-faction world: generate seed 483920 (the engine
## call, as `_gi_probe.gd`'s `_generate`), delete every faction (`civ_remove_faction`, the button's call, which
## `_gi_probe.gd` walks by its button) and repaint the claims through `polity_moved`; capture the map; (3) the
## city-state with the most aligned places: Inspect shows `City-state · N aligned` and the action, an aligned place
## shows `Aligned to <city>`; capture both; (4) a real right-click on the city-state's marker opens the card, whose
## subtitle carries the sphere line and whose rows offer "Make this sphere a faction"; capture; (5) a real click on
## Inspect's "Make this sphere a faction": one faction exists, the city-state and its members are in it; capture;
## (6) Edit > Undo (the menu, real key events): zero factions again, the place is loose and a city-state again;
## capture; (7) a world with factions (the sixth deleted, so its places are loose beside five nations): spheres
## exist, no sphere cell is claimed land; capture.
## Every check prints `ok` or `FAIL`; the launcher exits 1 on any FAIL. Mutant: `city_states_mutant.py`.

var _fails := 0
var app: Node
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


func _click_entry(ctx, e: Dictionary) -> void:
	var r: Array = e["rect"]
	await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))


func _generate(ctx, seed_v: int) -> void:
	bridge.generate({"seed": seed_v, "width_km": 800.0, "grid_w": 512, "grid_h": 328, "archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await ctx.wait(0.2)
	while bool(bridge.generating) and Time.get_ticks_msec() - t0 < 300000:
		await ctx.wait(0.25)
	await ctx.wait(2.0)


## The claims moved under the shell: the same refresh a polity move makes. The territory layer is off by default,
## so it is turned on through the Layers popover's own writer (`set_layer_visible`), and the view reset to the whole map.
func _repaint(ctx) -> void:
	app.viewport.set_layer_visible("territory", true)
	app.viewport.reset_view()
	app.place_editor_window.polity_moved.emit()
	app.place_editor_window.place_changed.emit()
	await ctx.wait(1.0)


## The settlement's marker on screen, in root canvas pixels (the overlay's own transform, `_ctthru_probe.gd`'s rule).
func _screen_of(i: int) -> Vector2:
	var ov = app.viewport.overlay
	var s: Dictionary = bridge.settlements()[i]
	return (ov as Control).get_global_transform() * ov._cell_to_screen(Vector2(float(s.get("x", 0)), float(s.get("y", 0))), ov._displayed_rect())


func _right_click(ctx, at: Vector2) -> void:
	DriveCtx._motion(ctx, at)
	await ctx.wait(0.1)
	DriveCtx._button(ctx, at, true, MOUSE_BUTTON_RIGHT)
	await ctx.wait(0.1)
	DriveCtx._button(ctx, at, false, MOUSE_BUTTON_RIGHT)
	await ctx.wait(1.0)


## The seat with the most aligned members, or {}.
func _best_seat() -> Dictionary:
	var best := {}
	for s in (bridge.civ_spheres().get("seats", []) as Array):
		if best.is_empty() or (s["members"] as PackedInt64Array).size() > (best["members"] as PackedInt64Array).size():
			best = s
	return best


func run(ctx) -> void:
	app = ctx.get_scene_root()
	if not app.has_method("promote_sphere"):
		app = ctx.get_scene_root().find_child("App", true, false)
	bridge = app.bridge
	var cont := await _wait_for(ctx, "Continue without a world")
	_check(ctx, "welcome card shows Continue", not cont.is_empty())
	if not cont.is_empty():
		await _click_entry(ctx, cont)
	await ctx.wait(0.5)

	# (2) zero factions
	await _generate(ctx, 483920)
	_check(ctx, "a world with six factions", int(bridge.civ_faction_count()) == 6, str(bridge.civ_faction_count()))
	_check(ctx, "positive control: no city-state while every place is a faction's", (bridge.civ_spheres().get("seats", []) as Array).is_empty(), str(bridge.civ_spheres()))
	while int(bridge.civ_faction_count()) > 0:
		bridge.civ_remove_faction()
	await _repaint(ctx)
	app.select_domain("civilization")
	await ctx.wait(1.0)
	var sph: Dictionary = bridge.civ_spheres()
	var seats: Array = sph.get("seats", [])
	_check(ctx, "zero factions: city-states exist", seats.size() > 0, "%d seats, %d aligned" % [seats.size(), int(sph.get("aligned", 0))])
	_check(ctx, "zero factions: some places are aligned", int(sph.get("aligned", 0)) > 0)
	ctx.log("zero-faction seats: %s" % ", ".join(seats.map(func(s): return "%s (%d aligned, %d cells)" % [s["name"], (s["members"] as PackedInt64Array).size(), s["cells"]])))
	await ctx.capture("zero_faction_spheres")

	# (3) Inspect
	var seat := _best_seat()
	var si := int(seat.get("index", -1))
	var n := (seat.get("members", PackedInt64Array()) as PackedInt64Array).size()
	## The city-state in the middle of the map, so its sphere is in every frame and its marker can be right-clicked.
	var sd: Dictionary = bridge.settlements()[si]
	app.viewport.move_view_to(float(sd.get("x", 0)), float(sd.get("y", 0)))
	await ctx.wait(1.0)
	app.select("settlement", si)
	await ctx.wait(1.0)
	var row := await _wait_for(ctx, "City-state · %d aligned" % n, 5.0)
	_check(ctx, "Inspect: the city-state reads City-state · %d aligned" % n, not row.is_empty())
	var btn := await _wait_for(ctx, "Make this sphere a faction", 5.0, "Button")
	_check(ctx, "Inspect: the promotion action is there and enabled", not btn.is_empty() and bool(btn.get("enabled", false)))
	await ctx.capture("inspect_city_state")
	if n > 0:
		var mi := int((seat["members"] as PackedInt64Array)[0])
		app.select("settlement", mi)
		await ctx.wait(1.0)
		var al := await _wait_for(ctx, String(seat["name"]), 5.0)
		_check(ctx, "Inspect: an aligned place names its city", not al.is_empty() and not _find(ctx, "Aligned to").is_empty())
		await ctx.capture("inspect_aligned")

	# (4) the card, by a real right-click on the marker
	app.select("settlement", si)
	await ctx.wait(0.5)
	await _right_click(ctx, _screen_of(si))
	await ctx.wait(1.0)
	## Read INSIDE the open card (a PopupPanel running `context_card.gd`): Inspect shows the same words, so a
	## whole-screen search cannot tell the card's line from Inspect's.
	var card: Node = null
	for pp in ctx.get_scene_root().get_tree().root.find_children("*", "PopupPanel", true, false):
		if (pp as Window).visible and (pp as Node).get_script() != null and String((pp as Node).get_script().resource_path).ends_with("context_card.gd"):
			card = pp
	_check(ctx, "card: the right-click opened the context card", card != null)
	var card_text := ""
	if card != null:
		for c in card.find_children("*", "Control", true, false):
			if c is Label:
				card_text += (c as Label).text + " | "
			elif c is Button:
				card_text += (c as Button).text + " | "
	_check(ctx, "card: the subtitle carries the sphere line", card_text.contains("City-state · %d aligned" % n), card_text.left(200))
	_check(ctx, "card: it offers Make this sphere a faction", card_text.contains("Make this sphere a faction"))
	await ctx.capture("card_city_state")
	await DriveCtx.tap_key(ctx, KEY_ESCAPE)
	await ctx.wait(0.5)

	# (5) promote through Inspect's button, a real click
	app.select("settlement", si)
	await ctx.wait(1.0)
	btn = await _wait_for(ctx, "Make this sphere a faction", 5.0, "Button")
	if not btn.is_empty():
		await _click_entry(ctx, btn)
	await ctx.wait(1.5)
	_check(ctx, "promotion: one faction now", int(bridge.civ_faction_count()) == 1, str(bridge.civ_faction_count()))
	var ss: Array = bridge.settlements()
	var in_f := ss.filter(func(s): return int(s.get("faction", 0)) == 1).size()
	_check(ctx, "promotion: the city-state and its %d places are the faction" % n, in_f == n + 1, str(in_f))
	_check(ctx, "promotion: the city-state is its capital", bool(ss[si].get("capital", false)) and int(ss[si].get("faction", 0)) == 1)
	await ctx.capture("promoted")

	# (6) Edit > Undo
	var undone: bool = await DriveCtx.menu_open(ctx, "Edit", "Undo")
	await ctx.wait(1.5)
	_check(ctx, "Edit > Undo pressed", undone)
	_check(ctx, "undo: zero factions again", int(bridge.civ_faction_count()) == 0, str(bridge.civ_faction_count()))
	_check(ctx, "undo: the place is a city-state again", String(bridge.civ_settlement_sphere(si).get("role", "")) == "seat")
	await ctx.capture("undone")

	# (6b) Edit > Redo promotes the same sphere again (2026-11-22); undo and redo walk both ways
	_check(ctx, "redo offered after the undo, named for the promotion", bridge.redo_available() and bridge.redo_label() == "Make %s a faction" % String(seat["name"]), bridge.redo_label())
	var redone: bool = await DriveCtx.menu_open(ctx, "Edit", "Redo")
	await ctx.wait(1.5)
	_check(ctx, "Edit > Redo pressed", redone)
	_check(ctx, "redo: one faction again", int(bridge.civ_faction_count()) == 1, str(bridge.civ_faction_count()))
	ss = bridge.settlements()
	_check(ctx, "redo: the same city-state and its %d places are the faction" % n, ss.filter(func(s): return int(s.get("faction", 0)) == 1).size() == n + 1 and bool(ss[si].get("capital", false)))
	_check(ctx, "redo: nothing left to redo", not bridge.redo_available())
	await ctx.capture("redone")
	undone = await DriveCtx.menu_open(ctx, "Edit", "Undo")
	await ctx.wait(1.5)
	_check(ctx, "redo is undoable again: zero factions", undone and int(bridge.civ_faction_count()) == 0, str(bridge.civ_faction_count()))
	_check(ctx, "...and redoable again", bridge.redo_available())

	# (7) a world with factions: the sixth deleted, five nations remain
	await _generate(ctx, 912345)
	bridge.civ_remove_faction()
	await _repaint(ctx)
	var sp2: Dictionary = bridge.civ_spheres()
	_check(ctx, "beside nations: five factions and a city-state", int(bridge.civ_faction_count()) == 5 and (sp2.get("seats", []) as Array).size() > 0, str(sp2))
	for s in bridge.settlements():
		if int(s.get("faction", 0)) != 0:
			var r: Dictionary = bridge.civ_settlement_sphere(bridge.settlements().find(s))
			if not r.is_empty():
				_check(ctx, "beside nations: a faction's place is never aligned", false, String(s.get("name", "")))
	await ctx.capture("factions_spheres")
	ctx.log("CITY-STATES %s" % ("FAIL: %d check(s)" % _fails if _fails > 0 else "PASS"))
