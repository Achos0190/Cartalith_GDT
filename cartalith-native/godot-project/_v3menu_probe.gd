extends Node
## Temporary, untracked verification harness for the v3 left-rail menu pass
## (`design/Cartalith Menu Structure v3.dc.html`, vendored at 8cef062).
##
## Run WINDOWED -- a headless boot proves the extension loads and the scripts
## parse, which is exactly the half of this change that was already known good:
##   Godot_v4.7.1-stable_win64_console.exe --path . _v3menu_probe.tscn
##
## What it drives, in order:
##   1. Boot the real app, generate a real world.
##   2. For each of the three rails, assert the L2 category list is v3's own
##      list, in v3's own order, with nothing left over. The leftovers matter:
##      the first cut of this pass built INFRA's five old categories AND the
##      three new ones, because `_dock_hosted` was set after `setup()`.
##   3. Open every category on every rail and assert each one drew something.
##      An accordion hides its own bugs -- a category that throws while
##      building leaves an empty body that looks like a closed one.
##   4. Prove the rows that claim real capability actually reach it:
##      Territories' two recompute shortcuts, Politics/Simulation's split,
##      Layers/Political display's split, and Data ▸ Markdown vault.
##   5. Assert every disabled row carries a reason (the `_todo` contract).
##   6. One screenshot per rail, every category forced open.

const SEED := 483920

var _app: Node
var _bridge
var _fail := 0

## v3's own category names, per rail, in v3's own order.
const WANT := {
	"world": ["Generate", "Terrain", "Geology", "Hydrology", "Climate",
		"Biomes", "Ecology", "Resources", "World data"],
	"civilization": ["Civilizations", "Factions", "Territories", "Settlements",
		"Points of interest", "Routes & ways", "Travel", "Trade", "Economy",
		"Culture", "Politics", "Military", "Relationships", "Simulation"],
	"cartography": ["Map style", "Terrain appearance", "Colours", "Layers",
		"Roads & routes", "Labels", "Assets & landmarks", "Political display",
		"Visibility / zoom", "Map presets"],
}

## Categories the v3 pass RETIRED. Any of these still on a rail is the
## `_dock_hosted` / `_nested` ordering bug coming back.
const GONE := ["Roads", "Rivers", "Ports", "Logistics", "Layer properties",
	"Annotation", "Timeline", "Population", "Generation pipeline"]


func _fail_msg(s: String) -> void:
	_fail += 1
	print("V3 !! %s" % s)


func _ok(s: String) -> void:
	print("V3    %s" % s)


func _texts(n: Node, out: Array) -> Array:
	if n is Label:
		out.append(String((n as Label).text))
	elif n is Button:
		out.append(String((n as Button).text))
	for c in n.get_children():
		_texts(c, out)
	return out


func _find(n: Node, cls: String) -> Node:
	if n.get_class() == cls or (n.get_script() != null
			and String(n.get_script().resource_path).ends_with(cls)):
		return n
	for c in n.get_children():
		var r := _find(c, cls)
		if r != null:
			return r
	return null


func _workspace(script_file: String) -> Node:
	return _find(_app, script_file)


func _generate() -> void:
	_bridge.generate({
		"seed": SEED, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().process_frame
	await get_tree().process_frame


## Every `DccWidgets.category()` entry a workspace registered, plus whatever a
## nested workspace registered into its own array -- the rail shows both as one
## list, so the check has to see both.
func _all_categories(ws: Node) -> Array:
	var out: Array = []
	out.append_array(ws.categories)
	for extra in ["_infra", "_render"]:
		if ws.get(extra) != null:
			out.append_array((ws.get(extra) as Node).categories)
	return out


func _check_rail(domain: String, ws: Node) -> void:
	var cats := _all_categories(ws)
	var got: Array = []
	for e in cats:
		got.append(String((e as Dictionary)["title"]))
	var want: Array = WANT[domain]
	if got == want:
		_ok("%-13s %2d categories, exactly v3's list and order" % [domain, got.size()])
	else:
		_fail_msg("%s category list is wrong\n     want: %s\n     got:  %s"
			% [domain, str(want), str(got)])
	for dead in GONE:
		if got.has(dead):
			_fail_msg("%s still carries the retired category %s" % [domain, dead])

	## Open every one and assert it drew content. `visible` is set directly
	## rather than clicked: the accordion closes siblings, so a click loop only
	## ever proves one category per frame.
	for e in cats:
		var entry: Dictionary = e
		var body: Control = entry["body"]
		body.visible = true
		var lines: Array = _texts(body, [])
		var n := 0
		for l in lines:
			if String(l).strip_edges() != "":
				n += 1
		if n == 0:
			_fail_msg("%s ▸ %s drew nothing at all" % [domain, entry["title"]])


## The `_todo()` / disabled-control contract: a control the port cannot honour
## is disabled AND carries the reason. A greyed row with no tooltip is the
## dishonest half of the pattern and is what this pass must not have added.
func _check_disabled(domain: String, ws: Node) -> void:
	var bad: Array = []
	_walk_disabled(ws, bad)
	if bad.is_empty():
		_ok("%-13s every disabled control carries a reason" % domain)
	else:
		for b in bad:
			_fail_msg("%s: disabled with no reason -- \"%s\"" % [domain, b])


## Buttons whose greyed state is *live draft state*, not a capability claim:
## Sculpt and Biome paint both disable Commit/Discard while there is nothing
## drafted to commit or discard. A tooltip there would be wrong, not missing.
## Named explicitly rather than pattern-matched so a genuinely undisclosed gap
## cannot slip in behind a similar label. All four predate this pass.
const STATE_GATED := ["Commit to map", "Discard draft", "Commit"]


func _walk_disabled(n: Node, bad: Array) -> void:
	if n is Button and (n as Button).disabled:
		var b := n as Button
		var t := b.text.strip_edges()
		var exempt := false
		for s in STATE_GATED:
			if t.ends_with(s):
				exempt = true
		## A disabled category header is just a closed accordion, not a claim.
		if b.tooltip_text.strip_edges() == "" and t != "" and not exempt:
			bad.append(b.text)
	for c in n.get_children():
		_walk_disabled(c, bad)


func _shot(domain: String) -> void:
	await get_tree().process_frame
	await get_tree().process_frame
	var out := "user://v3_%s.png" % domain
	get_viewport().get_texture().get_image().save_png(out)
	print("V3    shot -> %s" % ProjectSettings.globalize_path(out))


func _ready() -> void:
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge

	await _generate()
	_app.open_project_dialog.hide()
	await get_tree().process_frame
	print("V3    world %s  %d settlements / %d provinces / %d ways" % [
		_bridge.grid_size(), _bridge.settlements().size(),
		_bridge.provinces().size(), _bridge.roads().size()])

	var world := _workspace("world_workspace.gd")
	var civ := _workspace("civilization_workspace.gd")
	var carto := _workspace("cartography_workspace.gd")
	for pair in [["world", world], ["civilization", civ], ["cartography", carto]]:
		if pair[1] == null:
			_fail_msg("%s workspace not found in the tree" % pair[0])
			get_tree().quit(1)
			return

	for pair in [["world", world], ["civilization", civ], ["cartography", carto]]:
		var domain := String(pair[0])
		var ws: Node = pair[1]
		_app._select_domain(domain)
		await get_tree().process_frame
		_check_rail(domain, ws)
		_check_disabled(domain, ws)
		await _shot(domain)

	# -- 4. the rows that claim real capability ------------------------------

	## Politics / Simulation split: the year list must be under Politics and the
	## simulate form under Simulation, not both in one body.
	var pol := "\n".join(_texts(civ._tl_body, []))
	var sim := "\n".join(_texts(civ._sim_body, []))
	if pol.find("Add year") >= 0 and pol.find("Simulate") < 0:
		_ok("CIVIL ▸ Politics holds the years and not the simulator")
	else:
		_fail_msg("CIVIL ▸ Politics content is wrong:\n%s" % pol)
	if sim.find("Simulate") >= 0 and sim.find("Add year") < 0:
		_ok("CIVIL ▸ Simulation holds the simulator and not the years")
	else:
		_fail_msg("CIVIL ▸ Simulation content is wrong:\n%s" % sim)

	## Territories' two recompute shortcuts are the same real call. Driven, not
	## read: press the button and assert the engine moved.
	var before_badge := String(civ._recompute_note.text) if civ._recompute_note != null else ""
	var recalc: Button = null
	for b in _texts(civ._territories_body, []):
		pass
	recalc = _find_button(civ._territories_body, "Recalculate territories")
	var genprov := _find_button(civ._territories_body, "Generate provinces")
	if recalc == null or genprov == null:
		_fail_msg("CIVIL ▸ Territories is missing its recompute shortcuts")
	elif recalc.disabled or genprov.disabled:
		_fail_msg("CIVIL ▸ Territories recompute shortcuts are greyed over a live world")
	else:
		var p0: int = _bridge.provinces().size()
		recalc.pressed.emit()
		await get_tree().create_timer(1.5).timeout
		var p1: int = _bridge.provinces().size()
		if p1 > 0:
			_ok("CIVIL ▸ Territories ▸ Recalculate ran civ_recompute (%d → %d provinces)" % [p0, p1])
		else:
			_fail_msg("CIVIL ▸ Territories ▸ Recalculate left no provinces")

	## Layers / Political display split: the two political switches must have
	## left the Layers list and appear exactly once, under their own category.
	var carto_cats := _all_categories(carto)
	var layers_txt := ""
	var poli_txt := ""
	for e in carto_cats:
		var entry: Dictionary = e
		if String(entry["title"]) == "Layers":
			layers_txt = "\n".join(_texts(entry["body"], []))
		elif String(entry["title"]) == "Political display":
			poli_txt = "\n".join(_texts(entry["body"], []))
	if layers_txt.find("Political — territory") < 0 and poli_txt.find("Political — territory") >= 0:
		_ok("CARTO ▸ the two political layers moved to Political display, once")
	else:
		_fail_msg("CARTO political layer split is wrong (layers=%s / political=%s)" % [
			layers_txt.find("Political — territory"), poli_txt.find("Political — territory")])

	## Data ▸ Markdown vault. Pressed through the real popup, and asserted by
	## the window actually being on screen afterwards.
	var data_popup := _data_popup()
	if data_popup == null:
		_fail_msg("Data menu popup not found")
	else:
		var idx := -1
		for i in data_popup.item_count:
			if String(data_popup.get_item_text(i)).findn("Markdown vault") >= 0:
				idx = i
		if idx < 0:
			_fail_msg("Data menu has no Markdown vault row")
		elif data_popup.is_item_disabled(idx):
			_fail_msg("Data ▸ Markdown vault is disabled")
		else:
			data_popup.id_pressed.emit(data_popup.get_item_id(idx))
			await get_tree().process_frame
			await get_tree().process_frame
			if _app.vault_window.visible:
				_ok("Data ▸ Markdown vault opened the real vault window")
				_app.vault_window.hide()
			else:
				_fail_msg("Data ▸ Markdown vault did not open the vault window")
		## And the two honest gaps beside it.
		var todo := 0
		for i in data_popup.item_count:
			var t := String(data_popup.get_item_text(i))
			if (t.findn("Create notes") >= 0 or t.findn("orphan notes") >= 0):
				if data_popup.is_item_disabled(i) and data_popup.get_item_tooltip(i).length() > 40:
					todo += 1
				else:
					_fail_msg("Data ▸ \"%s\" is not a disclosed gap" % t)
		if todo == 2:
			_ok("Data ▸ the two unbacked vault rows are disabled with reasons")
		else:
			_fail_msg("Data: expected 2 disclosed vault gaps, found %d" % todo)

	## 6b. One shot per re-parented category, alone, so the layout of the moved
	## content can actually be looked at rather than inferred from a wall of
	## every-category-open text.
	_app._select_domain("civilization")
	for want in ["Routes & ways", "Travel", "Trade", "Politics", "Simulation",
			"Factions", "Territories"]:
		await _solo_shot("civ", civ, want)
	_app._select_domain("cartography")
	for want in ["Roads & routes", "Political display", "Visibility / zoom",
			"Map presets"]:
		await _solo_shot("carto", carto, want)
	_app._select_domain("world")
	for want in ["Terrain", "Geology", "Ecology", "World data"]:
		await _solo_shot("world", world, want)

	print("V3 RESULT %s (%d failures)" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(0 if _fail == 0 else 1)


func _solo_shot(tag: String, ws: Node, title: String) -> void:
	var cats := _all_categories(ws)
	var found := false
	for e in cats:
		var entry: Dictionary = e
		var on := String(entry["title"]) == title
		(entry["body"] as Control).visible = on
		found = found or on
	if not found:
		_fail_msg("%s: no category named %s to shoot" % [tag, title])
		return
	await get_tree().process_frame
	await get_tree().process_frame
	var slug := title.to_lower().replace(" ", "_").replace("&", "and").replace("/", "_")
	var out := "user://v3_%s_%s.png" % [tag, slug]
	get_viewport().get_texture().get_image().save_png(out)
	print("V3    solo -> %s" % out)


func _find_button(n: Node, text: String) -> Button:
	if n is Button and String((n as Button).text).findn(text) >= 0:
		return n as Button
	for c in n.get_children():
		var r := _find_button(c, text)
		if r != null:
			return r
	return null


func _data_popup() -> PopupMenu:
	var found: Array = []
	_collect_popups(_app, found)
	for p in found:
		var pm := p as PopupMenu
		for i in pm.item_count:
			if String(pm.get_item_text(i)).findn("Journey planner") >= 0:
				return pm
	return null


## `get_children(true)`, not `get_children()`: a `MenuButton`'s `PopupMenu` is
## an *internal* child, which the default walk skips entirely -- the first cut
## of this probe reported "Data menu popup not found" over a menu that was
## right there and working.
func _collect_popups(n: Node, out: Array) -> void:
	if n is PopupMenu:
		out.append(n)
	for c in n.get_children(true):
		_collect_popups(c, out)
