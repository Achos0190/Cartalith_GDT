extends Node
## VERIFIER probe (untracked). Independent re-derivation of fixes 3, 4, 5.
## Written from the design + engine API, not from the lane's probe.
##
##   godot --headless --path . _vfy345_probe.tscn
##
## Deliberately different from `_confsix_probe.gd` in what it asserts:
##  - fix 3: scans EVERY settlement for distinct faction ids (not the first 5),
##    cross-checks the id->name mapping against the array ORDER of
##    `get_factions()` so an off-by-one in `_faction_roster` would show, and
##    requires >= 2 DISTINCT names rendered so "always returns row 0" fails.
##    Also asserts each dashed row carries a tooltip reason.
##  - fix 4: distinguishes "discards the draft" from "wipes the layer" by
##    committing first and asserting the committed total SURVIVES the discard;
##    and asserts the sculpt draft is untouched.
##  - fix 5: ACTUALLY collapses the dock in each domain, and switches domain
##    while collapsed -- the stale-value shape.

const SEEDS := [4021, 77123, 918237]

var app: Node
var _fail := 0
var _n := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _eq(label: String, got: Variant, want: Variant) -> void:
	_n += 1
	var p := str(got) == str(want)
	if not p:
		_fail += 1
	print("V %s %-52s got=%s want=%s" % ["ok  " if p else "FAIL", label, str(got), str(want)])

func _yes(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if not cond:
		_fail += 1
	print("V %s %s%s" % ["ok  " if cond else "FAIL", label, ("  [" + detail + "]") if detail != "" else ""])

## label -> [value, tooltip] for every `_field()` row on screen.
func _rows(n: Node, out: Dictionary) -> void:
	for c in n.get_children():
		if c is HBoxContainer:
			var ls: Array = []
			for g in c.get_children():
				if g is Label:
					ls.append(g)
			if ls.size() == 2:
				out[String((ls[0] as Label).text)] = [
					String((ls[1] as Label).text), String((c as Control).tooltip_text)]
		_rows(c, out)

func _dock_rows() -> Dictionary:
	var o := {}
	_rows(app.right_dock_body, o)
	return o

func _bar_buttons() -> Array:
	var out: Array = []
	_walk_buttons(app.tool_options_row, out)
	return out

func _walk_buttons(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Button:
			out.append(c)
		_walk_buttons(c, out)

func _chip(t: String) -> Button:
	for b in _bar_buttons():
		if String((b as Button).text) == t:
			return b
	return null

func _readout() -> String:
	return String((app._dock_readouts["left"] as Label).text)

func _readout_visible() -> bool:
	return (app._dock_readouts["left"] as Label).get_parent().visible

func _gen(s: int) -> bool:
	app.bridge.generate({"seed": s, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var w := 0
	while app.bridge.generating and w < 6000:
		await get_tree().process_frame
		w += 1
	await _frames(10)
	return app.bridge.has_world

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	var bridge = app.bridge
	var rd = app.right_dock_ctrl

	# ---- FIX 5, part 1: no world yet, collapse in each domain --------------
	for pair in [["world", "no world"], ["civilization", "CIVIL"], ["cartography", "CARTO"]]:
		app.select_domain(String(pair[0]))
		await _frames(6)
		if not app.is_dock_collapsed("left"):
			app._toggle_dock(true)
		await _frames(4)
		_yes("collapsed strip visible in %s" % pair[0], _readout_visible())
		_eq("no-world readout, collapsed, %s" % pair[0], _readout(), String(pair[1]))
	# leave it collapsed on purpose for the switch-while-collapsed test below

	for sv in SEEDS:
		if not await _gen(sv):
			_yes("seed %d generated" % sv, false)
			continue
		print("V ---------------- seed %d ----------------" % sv)

		# ---- FIX 3 ---------------------------------------------------------
		var facs: Array = bridge.get_factions()
		var by_id := {}
		var order_ok := true
		for k in facs.size():
			var fd: Dictionary = facs[k]
			by_id[int(fd.get("id", -1))] = String(fd.get("name", ""))
			## Independent of `_faction_roster`'s own `id` match: the k-th row
			## must be id k+1. If that ever stopped holding, an index-based
			## lookup and an id-based lookup would disagree and this says so.
			if int(fd.get("id", -1)) != k + 1:
				order_ok = false
		_yes("seed %d roster ids are 1..n in array order" % sv, order_ok,
			"%d rows" % facs.size())
		_yes("seed %d roster has no id 0" % sv, not by_id.has(0))

		var sets: Array = bridge.settlements()
		## One settlement per DISTINCT faction id, scanning all of them.
		var rep := {}
		for i in sets.size():
			var fid := int((sets[i] as Dictionary).get("faction", -999))
			if not rep.has(fid):
				rep[fid] = i
		_yes("seed %d distinct faction ids among %d settlements" % [sv, sets.size()],
			rep.size() >= 2, str(rep.keys()))

		app.select_domain("civilization")
		await _frames(4)
		var seen_names := {}
		for fid in rep.keys():
			var idx: int = rep[fid]
			rd.on_settlement_selected(sets[idx] as Dictionary, idx)
			await _frames(6)
			var row: Array = _dock_rows().get("Faction", ["<missing>", ""])
			var shown := String(row[0])
			var tip := String(row[1])
			var want: String = String(by_id.get(fid, "—")) if fid > 0 else "—"
			_eq("seed %d faction id %d -> name" % [sv, fid], shown, want)
			_yes("  seed %d id %d value is not a bare integer" % [sv, fid],
				not shown.is_valid_int(), shown)
			if shown == "—":
				_yes("  seed %d id %d dash carries a reason" % [sv, fid],
					tip.length() > 20, tip)
			else:
				seen_names[shown] = true
		_yes("seed %d rendered >= 2 DISTINCT faction names" % sv,
			seen_names.size() >= 2, str(seen_names.keys()))

	# ---- FIX 3: the two absent arms, with their reasons --------------------
	app.select_domain("civilization")
	await _frames(4)
	for c in [[0, "unclaimed sentinel"], [99, "no roster row"]]:
		rd.on_settlement_selected({"name": "Vfy", "kind": "town", "population": 10,
			"faction": int(c[0]), "coastal": false, "capital": false, "tid": 987654}, 0)
		await _frames(6)
		var row: Array = _dock_rows().get("Faction", ["<missing>", ""])
		_eq("synthetic faction %d (%s) -> dash" % [int(c[0]), String(c[1])], String(row[0]), "—")
		_yes("  reason given for faction %d" % int(c[0]), String(row[1]).length() > 20,
			String(row[1]))

	# ---- FIX 5, part 2: stale value across a domain switch, while collapsed
	app.select_domain("world")
	await _frames(8)
	if not app.is_dock_collapsed("left"):
		app._toggle_dock(true)
	await _frames(4)
	var world_word := _readout()
	_eq("WORLD readout with a world, collapsed", world_word, "resolved")
	for d in ["civilization", "cartography", "world", "cartography", "civilization"]:
		app.select_domain(d)
		await _frames(6)
		var want := "resolved" if d == "world" else ("CIVIL" if d == "civilization" else "CARTO")
		_eq("switch-while-collapsed -> %s" % d, _readout(), want)
	## A `generation_stage` tick arriving while the reader is NOT in WORLD.
	app.select_domain("cartography")
	await _frames(6)
	app._workspace_panels["world"].push_dock_readout()
	await _frames(3)
	_eq("WORLD push while in CARTO does not overwrite", _readout(), "CARTO")
	app._workspace_panels["civilization"].push_dock_readout()
	await _frames(3)
	_eq("CIVIL push while in CARTO does not overwrite", _readout(), "CARTO")
	app._toggle_dock(true)   # uncollapse

	# ---- FIX 4: Discard on the tool options bar ----------------------------
	app.select_domain("world")
	await _frames(6)
	var layers: PackedStringArray = bridge.get_paint_layers()
	_yes("paint layers exist", layers.size() > 0, str(layers))
	bridge.paint_set_layer(String(layers[0]))
	bridge.paint_set_brush(1, 6.0, 1.0, 0.0, false, true)
	app.arm_tool("paint")
	await _frames(8)
	_yes("Commit chip present", _chip("Commit") != null)
	_yes("Discard chip present", _chip("Discard") != null)
	if _chip("Discard") == null:
		_finish()
		return
	_eq("Discard dead with an empty draft", _chip("Discard").disabled, true)

	var gs: Vector2i = bridge.grid_size()
	## Leg 1 -- COMMIT some paint, so there is a committed layer to protect.
	for k in 5:
		bridge.paint_stroke_at(float(gs.x) * (0.30 + 0.03 * k), float(gs.y) * 0.5)
	bridge.paint_commit("vfy")
	await _frames(4)
	var committed := int(bridge.paint_painted_counts().get("total", 0))
	_yes("a committed paint layer exists", committed > 0, "%d cells" % committed)
	_eq("draft empty after commit", bridge.paint_draft_count(), 0)

	## Leg 2 -- a NEW draft on top of it, then Discard from the BAR.
	var sculpt_before: int = bridge.sculpt_stroke_point_count()
	for k in 4:
		bridge.paint_stroke_at(float(gs.x) * (0.60 + 0.03 * k), float(gs.y) * 0.4)
	app.tool_bar.rebuild()
	await _frames(6)
	var pending: int = bridge.paint_draft_count()
	_yes("draft pending after 4 dabs", pending > 0, "%d cells" % pending)
	_eq("Discard live with a pending draft", _chip("Discard").disabled, false)
	_chip("Discard").emit_signal("pressed")
	await _frames(8)
	_eq("Discard emptied the PAINT draft", bridge.paint_draft_count(), 0)
	## The discriminating pair: a Discard wired to the wrong call (or to a
	## layer wipe) would move one of these two.
	_eq("committed layer SURVIVED the discard",
		int(bridge.paint_painted_counts().get("total", 0)), committed)
	_eq("sculpt draft untouched by paint Discard",
		bridge.sculpt_stroke_point_count(), sculpt_before)
	_yes("Discard redrawn after the press", _chip("Discard") != null)
	if _chip("Discard") != null:
		_eq("Discard dead again", _chip("Discard").disabled, true)
	## Fix 4's claimed third refresh: the right dock's paint context.
	var ws = app._world_workspace()
	_yes("_refresh_right_dock_paint exists on WorldWorkspace",
		ws != null and ws.has_method("_refresh_right_dock_paint"))

	_finish()

func _finish() -> void:
	print("V ==== %s  %d/%d assertions, %d failed ====" % [
		"PASS" if _fail == 0 else "FAIL", _n - _fail, _n, _fail])
	get_tree().quit(_fail)
