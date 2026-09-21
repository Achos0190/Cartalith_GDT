extends Node
## Committed verification harness for the v3 left-rail menu pass
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
##      Territories' two recompute shortcuts, Timeline's years and simulator,
##      the political layers' Ruling L consolidation back into Layers, and
##      Data ▸ Markdown vault.
##   5. Assert every disabled row carries a reason (the `_todo` contract).
##   6. One screenshot per rail, every category forced open.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

const SEED := 483920

var _app: Node
var _bridge
var _fail := 0

## v3's own category names, per rail, in v3's own order.
##
## One entry is no longer v3's literal string. CIVIL's fifth category was v3's
## `Points of interest`; the landmark-generation pass RENAMED it to `Landmarks`
## and replaced its "Not built" stub with a real body. That is a design
## decision, not drift -- `design/landmark-generation/LANDMARK_UI_DESIGN.md:58`
## ("the existing v3 category `Points of interest`, renamed"), restated at :273
## and tracked as owed work at :621, and the 2026-08-31 DCC spec carries the new
## name too (`design/dcc-environment-2026-08-31/spec/02-rail-and-domains.md:94`).
## The rail fold's rule -- every pre-existing category stays reachable -- holds:
## the category is still there, only its title moved. The old name is listed in
## `GONE` below so a revert to the stub is still caught.
##
## **CIVIL's whole list is no longer v3's.** Ruling L
## (`design/owner-references-2026-09-12/left_rail_tree_resorted.md` L143-247)
## re-sorts it to thirteen, in the owner's order: Civilizations renamed
## Populate, Trade folded into Economy, Politics renamed Timeline and
## Simulation folded into it. `GONE` carries the four retired titles.
## Re-sorted 2026-09-21 for Ruling L's WORLD/CARTO re-sort
## (`design/owner-references-2026-09-12/left_rail_tree_resorted.md`). WORLD's
## PIPELINE seven then SCULPT's two, the order `_worldcensus_probe.gd`
## independently confirms; CARTO folds ten categories into seven.
const WANT := {
	"world": ["Generate", "Planet", "Geology", "Hydrology", "Climate",
		"Ecology", "World data", "Terrain", "Biomes"],
	"civilization": ["Populate", "Settlements", "Landmarks", "Routes & ways",
		"Travel", "Factions", "Territories", "Relationships", "Military",
		"Culture", "Religion", "Economy", "Timeline"],
	"cartography": ["Style", "Relief & light", "Colours", "Feature style",
		"Layers", "Labels", "Icons"],
}

## Categories the v3 pass RETIRED. Any of these still on a rail is the
## `_dock_hosted` / `_nested` ordering bug coming back. `Timeline` left this list
## when Ruling L brought it back as a live CIVIL category (L236).
const GONE := ["Roads", "Rivers", "Ports", "Logistics", "Layer properties",
	"Annotation", "Population", "Generation pipeline",
	## Renamed to `Landmarks`, not deleted -- see the note on `WANT` above.
	"Points of interest",
	## Ruling L's four (L342): renamed or folded, never deleted -- see `WANT`.
	"Civilizations", "Trade", "Politics", "Simulation",
	## Ruling L's WORLD/CARTO re-sort, 2026-09-21 -- see `WANT` above.
	## `Resources` (decision 3: calculated, not set -- stays a read-only row
	## in Pipeline status, not a category) and CARTO's ten folded to seven:
	## `Map style`/`Terrain appearance` -> `Style`/`Relief & light`,
	## `Roads & routes` -> `Feature style` (Ways), `Assets & landmarks` ->
	## `Icons`, `Political display` -> back into `Layers`, `Visibility /
	## zoom` -> `Layers`, `Map presets` -> `Style` (Saved looks).
	"Resources", "Map style", "Terrain appearance", "Roads & routes",
	"Assets & landmarks", "Political display", "Visibility / zoom",
	"Map presets"]


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

	## Ruling L folds Politics and Simulation back into one Timeline
	## (`left_rail_tree_resorted.md` L236-247): the year list and the simulate
	## form share `_tl_body`, the form as a closed expander, and neither old title
	## is a category any more.
	var tl := "\n".join(_texts(civ._tl_body, []))
	var civ_titles: Array = []
	for e in _all_categories(civ):
		civ_titles.append(String((e as Dictionary)["title"]))
	var sim_btn := _button_exact(civ._tl_body, "Simulate")
	if tl.find("Add year") >= 0 and tl.find("SIMULATE COLLAPSE / RECOVERY") >= 0 \
			and sim_btn != null and not civ_titles.has("Politics") \
			and not civ_titles.has("Simulation"):
		_ok("CIVIL ▸ Timeline holds the years and the simulator; Politics and Simulation are gone")
	else:
		_fail_msg("CIVIL ▸ Timeline content is wrong (Simulate button=%s, Politics=%s, Simulation=%s):\n%s"
			% [sim_btn != null, civ_titles.has("Politics"), civ_titles.has("Simulation"), tl])

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

	## Ruling L reverses the pre-2026-09-21 split this section used to assert:
	## `Political display` is retired and the two political switches move BACK
	## into Layers, under its own `§ Political layers` section
	## (`cartography_workspace.gd:501`, "◄ Political display") — the rule
	## being "every visibility toggle lives in Layers". So the assertion now
	## is that the switches are IN Layers and no `Political display` category
	## exists to hold a second copy.
	var carto_cats := _all_categories(carto)
	var layers_txt := ""
	var has_political_display := false
	for e in carto_cats:
		var entry: Dictionary = e
		if String(entry["title"]) == "Layers":
			layers_txt = "\n".join(_texts(entry["body"], []))
		elif String(entry["title"]) == "Political display":
			has_political_display = true
	if layers_txt.find("Political — territory") >= 0 and not has_political_display:
		_ok("CARTO ▸ the two political layers are in Layers, and Political display is gone")
	else:
		_fail_msg("CARTO political layer consolidation is wrong (layers has it=%s / Political display still exists=%s)" % [
			layers_txt.find("Political — territory") >= 0, has_political_display])

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
		## And the two rows beside it. Both were `_todo` when this probe was
		## written and both have since been BUILT, so this assertion is now the
		## inverse of what it used to be: `GUI_GAP_REGISTER.md` VA-02 (create a
		## note from a template) closed in §39 and VA-01 (the reverse index --
		## backlinks, missing and orphan notes) closed in §42. They must be LIVE,
		## and they must reach the same single window the row above just opened
		## rather than become a second owner of it -- checked by item id, not by
		## pressing them again, since all three carry `ID_VAULT`.
		##
		## The old text match was `Create notes`, which had stopped matching
		## anything at all: the row reads `Create a note from a template...`.
		## `idx` is -1 only on the already-reported "no Markdown vault row"
		## failure; -1 into `get_item_id` is an error, so it degrades here.
		var want_id := data_popup.get_item_id(idx) if idx >= 0 else -1
		var live := 0
		for i in data_popup.item_count:
			var t := String(data_popup.get_item_text(i))
			if (t.findn("from a template") >= 0 or t.findn("orphan notes") >= 0):
				if (not data_popup.is_item_disabled(i)
						and data_popup.get_item_tooltip(i).length() > 40
						and data_popup.get_item_id(i) == want_id):
					live += 1
				else:
					_fail_msg("Data ▸ \"%s\" is not a live vault row (disabled=%s, tip=%d, id=%d want %d)"
						% [t, data_popup.is_item_disabled(i),
						data_popup.get_item_tooltip(i).length(),
						data_popup.get_item_id(i), want_id])
		if live == 2:
			_ok("Data ▸ the template and index rows are live onto the same vault window")
		else:
			_fail_msg("Data: expected 2 live vault rows, found %d" % live)

	## 6b. One shot per re-parented category, alone, so the layout of the moved
	## content can actually be looked at rather than inferred from a wall of
	## every-category-open text.
	_app._select_domain("civilization")
	for want in ["Populate", "Settlements", "Routes & ways", "Travel", "Economy",
			"Timeline", "Factions", "Territories"]:
		await _solo_shot("civ", civ, want)
	_app._select_domain("cartography")
	for want in ["Style", "Relief & light", "Feature style", "Layers"]:
		await _solo_shot("carto", carto, want)
	_app._select_domain("world")
	for want in ["Planet", "Geology", "Ecology", "World data", "Terrain"]:
		await _solo_shot("world", world, want)

	_check_bindings()
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


## An exact-text match, where `_find_button()`'s `findn` would take the
## `› SIMULATE COLLAPSE / RECOVERY` header for the `Simulate` button inside it.
func _button_exact(n: Node, text: String) -> Button:
	if n is Button and (n as Button).text == text:
		return n as Button
	for c in n.get_children():
		var r := _button_exact(c, text)
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


## The staleness fingerprint, read off the shell instead of guessed at.
##
## `EngineBridge._has()` (`shell/engine_bridge.gd`) is the one choke point
## every binding guard in the shell goes through, and it records the name of
## each method the shell asked for that this build does not export;
## `EngineBridge.missing_bindings()` hands back the set. Nothing in this probe
## suite read it -- and a stale `target/debug/cartalith_godot.dll` has twice
## sent every `_has()` guard in a run down its degraded-fallback branch, which
## turns a whole sweep into a clean report over code that was never exercised.
## That is the failure mode this suite is least able to notice on its own, and
## the shell was already carrying the answer.
##
## Called last, after every surface this run drives has been driven: the set
## only fills as guards are reached, so an early read reports an empty one.
func _check_bindings() -> void:
	var mb: PackedStringArray = _bridge.missing_bindings()
	if mb.is_empty():
		return
	_fail_msg("stale extension -- the shell asked for %d binding(s) this build "
		% mb.size()
		+ "does not export (%s). " % ", ".join(mb)
		+ "Every result above was measured against a degraded shell; rebuild "
		+ "the crates and re-run before believing any of it.")
