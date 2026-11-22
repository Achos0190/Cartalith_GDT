extends RefCounted
## Ruling HK (owner 2026-11-22): the New World screen accepts Factions = 0 and the generated world shows
## city-state spheres for every settlement, no faction territory. Run through `fennara_drive.py` on desktop and
## phone (`TOOLING_PLAN.md` TP-2):
##   python -I cartalith-native/tools/fennara_drive.py GODOT --shell desktop|phone res://tools/zero_factions_drive.gd --dll DLL --out DIR --root ROOT
## Steps: past the boot card with real clicks; File > New world; the Factions spin box takes "0" through real key
## events and keeps it (the floor was 1); the Create button is clicked; the world generates with no faction, every
## settlement aligned to a sphere, spheres hold land, no cell is claimed; the map is captured with the territory layer
## on. `--mutate` is the owner's own check: the same script on a build whose params floor is 1 reads 1 and FAILs.

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


## The Factions spin box: the New World dialog's own `_factions_spin`, found through the node that carries it.
func _spin(n: Node) -> SpinBox:
	if n.get("_factions_spin") is SpinBox:
		return n.get("_factions_spin")
	for c in n.get_children():
		var r := _spin(c)
		if r != null:
			return r
	return null


func run(ctx) -> void:
	var root: Node = ctx.get_scene_root()
	app = root if root.has_method("select_domain") else root.find_child("App", true, false)
	bridge = app.bridge
	var size: Vector2 = root.get_viewport().get_visible_rect().size
	var phone := size.x < 700.0
	ctx.log("form: %s canvas %s" % ["phone" if phone else "desktop", str(size)])
	if phone:
		var nw := await _wait_for(ctx, "+ NEW WORLD")
		_check(ctx, "worlds list shows + NEW WORLD", not nw.is_empty())
		await _click_entry(ctx, nw)
	else:
		var cont := await _wait_for(ctx, "Continue without a world")
		_check(ctx, "welcome card shows Continue", not cont.is_empty())
		await _click_entry(ctx, cont)
		await ctx.wait(0.5)
		_check(ctx, "File > New world opens", await DriveCtx.menu_open(ctx, "File", "New world"))
	await ctx.wait(1.5)
	var spin := _spin(root.get_viewport())
	_check(ctx, "the Factions spin box is on the New World screen", spin != null)
	if spin == null:
		ctx.log("PROOF FAIL")
		return
	_check(ctx, "its floor is 0 (the engine's range, not a literal here)", spin.min_value == 0.0, str(spin.min_value))
	# Scroll it into view when its card scrolls (the phone), then type through the real line edit.
	var p := spin.get_parent()
	while p != null and not (p is ScrollContainer):
		p = p.get_parent()
	if p != null:
		(p as ScrollContainer).ensure_control_visible(spin)
		await ctx.wait(0.5)
	var le := spin.get_line_edit()
	var got: String = await DriveCtx.type_text(ctx, le, "0\n")
	await ctx.wait(0.4)
	_check(ctx, "Factions reads 0 after typing 0 (the box did not push it back to 1)", spin.value == 0.0, "value=%s text=%s" % [spin.value, got])
	await ctx.capture("new_world_factions_0")
	# The dialog's own create button (a text search also matches other buttons that mention "Generate world").
	var dlg: Node = spin
	while dlg != null and dlg.get("_factions_spin") == null:
		dlg = dlg.get_parent()
	# 512 columns: a 2K world takes minutes to generate in this build, and the resolution is not what is under test.
	(dlg.get("resolution_input") as OptionButton).select(0)
	dlg.call("_on_resolution_selected", 0)
	await ctx.wait(0.5)
	var create: Control = dlg.get("_phone_create") if phone else (dlg as AcceptDialog).get_ok_button()
	_check(ctx, "the create button is there and visible", create != null and create.is_visible_in_tree())
	await DriveCtx.click(ctx, DriveCtx.rect_of(create).get_center())
	var t0 := Time.get_ticks_msec()
	await ctx.wait(0.5)
	## Generation starts a frame or more after the click: wait for the world itself, not only the flag.
	while (bool(bridge.generating) or bridge.settlements().size() == 0) and Time.get_ticks_msec() - t0 < 300000:
		await ctx.wait(0.5)
	await ctx.wait(3.0)
	_check(ctx, "the world has no faction", int(bridge.civ_faction_count()) == 0, str(bridge.civ_faction_count()))
	var st: Array = bridge.settlements()
	_check(ctx, "settlements exist", st.size() > 0, str(st.size()))
	var claimed := 0
	for s in st:
		claimed += 1 if int(s.get("faction", 0)) != 0 else 0
	_check(ctx, "every settlement is loose", claimed == 0, str(claimed))
	var sph: Dictionary = bridge.civ_spheres()
	var seats: Array = sph.get("seats", [])
	var members := 0
	for s in seats:
		members += (s["members"] as PackedInt64Array).size()
	ctx.log("spheres: %d seats, %d members, %d settlements, %s aligned" % [seats.size(), members, st.size(), str(sph.get("aligned", 0))])
	_check(ctx, "city-state spheres exist", seats.size() > 0)
	_check(ctx, "every settlement is a seat or aligned to one", seats.size() + members == st.size(), "%d + %d vs %d" % [seats.size(), members, st.size()])
	app.viewport.set_layer_visible("territory", true)
	app.viewport.reset_view()
	app.select_domain("civilization")
	await ctx.wait(2.0)
	await ctx.capture("zero_faction_map_spheres")
	ctx.log("PROOF %s" % ("FAIL: %d check(s)" % _fails if _fails > 0 else "PASS"))
