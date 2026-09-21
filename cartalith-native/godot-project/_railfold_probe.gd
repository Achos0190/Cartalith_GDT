extends Node
## The rail fold — five domains to three, and the node tree that replaced them.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . \
##       --resolution 1600x900 --rendering-driver opengl3 _railfold_probe.tscn
##
## **Why this probe exists at all.** `spec/00-REPLACEMENT-PLAN.md` §4 lists five
## risks for the GUI replacement and names a guard for four of them —
## `_cmdindex_probe` for menu rows, `_tabletparity_probe` for tablet density,
## `_phonechrome_probe` for tap floors, `_landmark_probe` for the CIVIL dock —
## and then says of the fifth, "a category becomes unreachable in the 5→3 fold:
## **no existing probe**". This is that probe.
##
## The risk is specific and it is not hypothetical. INFRA and RENDER stopped
## being rail buttons on 2026-08-20; their content survived only because
## `civilization_workspace.gd` and `cartography_workspace.gd` compose those two
## classes into their own docks. A fold moves content, and content that moves
## can be dropped: `dcc_shell.gd`'s own `DOMAINS` comment records that INFRA's
## five categories were once built twice, under the wrong parent, before
## `_dock_hosted` was set early enough. Nothing at the time would have caught
## the opposite mistake — building them zero times.
##
## So §3 below does not count categories. It **names all twenty-nine**
## (re-derived 2026-09-21 for Ruling L's WORLD/CARTO re-sort: WORLD swaps
## `Resources` for `Planet`, CARTO folds ten categories into seven — the total
## moved 32 -> 29), one string at a time, and asserts each is still openable and
## still owned by a rail node. A count would pass a build that dropped `Trade`
## and gained `Trade ` with a trailing space; a name list will not. That is the
## same discipline `_cmdindex_probe` §2 uses when it asserts the engine's own
## labels rather than a row count, and for the same reason.
##
## Every assertion drives the shell's real entry points — `select_domain_mode`,
## `select_domain_category`, `Workspace.open_category` — rather than reading
## state back out of the objects it just wrote to. A probe that asserts what it
## set is a probe that passes over a broken build.

var _vp: SubViewport
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

## **The list this stage is measured against.** Every L2 accordion category the
## three docks build, transcribed from the `DccWidgets.category(self, "…")` calls
## in `world_workspace.gd`, `civilization_workspace.gd` and
## `cartography_workspace.gd` — including the CIVIL ones
## `infrastructure_workspace.gd` fills (`Routes & ways` and `Travel`, and since
## Ruling L parts of `Settlements` and `Economy`) and the CARTO ones filled
## by `render_workspace.gd` (`Style`, `Relief & light`, `Colours` — re-sorted
## 2026-09-21, `render_workspace.gd`'s composed categories renamed and its
## tenth, `Map presets`, folded into `Style ▸ Saved looks`). Those are the fold's actual cargo and the reason the list is
## written out rather than walked. CIVIL's thirteen are Ruling L's, in its order
## (`design/owner-references-2026-09-12/left_rail_tree_resorted.md` L143-247).
##
## Hard-coded on purpose. Walking `panel.categories` and asserting each entry
## against itself would prove nothing; this list is an independent statement of
## what must exist, and it fails loudly when a category is renamed as well as
## when one is lost. If a category is deliberately renamed, this list changes in
## the same commit and the diff says so.
const EXPECTED: Dictionary = {
	"world": [
		"Generate", "Planet", "Geology", "Hydrology", "Climate",
		"Ecology", "World data", "Terrain", "Biomes",
	],
	"civilization": [
		"Populate", "Settlements", "Landmarks", "Routes & ways", "Travel",
		"Factions", "Territories", "Relationships", "Military", "Culture",
		"Religion", "Economy", "Timeline",
	],
	"cartography": [
		"Style", "Relief & light", "Colours", "Feature style", "Layers",
		"Labels", "Icons",
	],
}

## The design's own node tree (`ENV:1824`, transcribed in
## `spec/02-rail-and-domains.md` §3 and settled by BUILD_ANSWERS §2.1), stated
## here independently of `DccShell.RAIL_NODES` so that §1 compares two sources
## rather than one source with itself. CIVIL's labels are Ruling L's
## (`left_rail_tree_resorted.md` L22-27), the newer source: `ENV:1824`'s
## `Factions & settlements` and `Ways & routes` are stale for those two nodes,
## and L23's Settlements node is not built (it would need a fifth CIVIL mode id).
const DESIGN_NODES: Array = [
	["world", "a", "Generate"],
	["world", "b", "Sculpt"],
	["civilization", "landmarks", "Landmarks"],
	["civilization", "factions", "Factions"],
	["civilization", "infra", "Routes & ways"],
	["civilization", "planner", "Journey planner"],
	["cartography", "style", "Style"],
	["cartography", "layers", "Layers"],
	["cartography", "labels", "Labels"],
	["cartography", "icons", "Icons"],
]

func _category_body(panel: Control, title: String) -> Control:
	for e in (panel.get("categories") as Array):
		if String(e["title"]) == title:
			return e["body"]
	return null

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

	# =====================================================================
	print("\n=== 1: three domains, in order, and ten nodes under them ===")
	var doms: Array = app.get("DOMAINS")
	_ok("exactly three domains", doms.size(), 3)
	var order := []
	for d in doms:
		order.append(String(d["id"]))
	_ok("in order world / civilization / cartography",
		"/".join(order), "world/civilization/cartography")
	## The rail caption is what the user reads, and `_build_rail()` upper-cases
	## `d.rail` rather than `d.label` — WORLD/CIVIL/CARTO, not
	## World/Civilization/Cartography (which are the *menu* labels, `ENV:2016`).
	var rails := []
	for d in doms:
		rails.append(String(d["rail"]))
	_ok("rail captions are the prototype's", " ".join(rails), "WORLD CIVIL CARTO")

	var nodes: Array = app.get("RAIL_NODES")
	var built := []
	var heads := []
	for n in nodes:
		if String(n["kind"]) == "node":
			built.append("%s/%s/%s" % [String(n["domain"]), String(n["mode"]), String(n["label"])])
		else:
			heads.append(String(n["label"]))
	var want := []
	for d in DESIGN_NODES:
		want.append("%s/%s/%s" % [String(d[0]), String(d[1]), String(d[2])])
	_ok("ten nodes, matching the design exactly (domain/mode/label)",
		"\n" + "\n".join(built), "\n" + "\n".join(want))
	## Three headers interleaved, one per domain — `ENV:1824`'s
	## `nodes.push({t:'h',label:h})`.
	_ok("three headers, one per domain", " ".join(heads), "WORLD CIVIL CARTO")

	# =====================================================================
	print("\n=== 2: every node is reachable, and each selects its own mode ===")
	## Driven through `select_domain_mode()` — the same call
	## `_on_rail_node_pressed()` makes — so this exercises the click path rather
	## than a private setter.
	var seen_modes := {}
	for d in DESIGN_NODES:
		var dom := String(d[0])
		var mode := String(d[1])
		app.call("select_domain_mode", dom, mode)
		await _frames(2)
		_ok("%s ▸ %s selects the domain" % [dom, mode], app.call("active_domain"), dom)
		_ok("...and the mode", app.call("active_mode", dom), mode)
		## Distinct within a domain: two nodes that wrote the same mode would
		## be two rail rows with one destination — the exact defect
		## BUILD_ANSWERS §2.1 was answering for CARTO.
		var key := "%s/%s" % [dom, mode]
		_ok("...and that mode is distinct within %s" % dom, seen_modes.has(key), false)
		seen_modes[key] = true
		## The node's own category is open. `RAIL_NODES` names it; the accordion
		## has to have actually opened it, which is the half that silently does
		## nothing if the string is misspelt.
		var node: Dictionary = app.call("rail_node", dom, mode)
		_ok("...and the node names a category", node.is_empty(), false)
		if not node.is_empty():
			var panel: Control = app.call("workspace_panel", dom)
			var body := _category_body(panel, String(node["category"]))
			_ok("...and category '%s' is open" % String(node["category"]),
				body != null and body.visible, true)

	# =====================================================================
	print("\n=== 3: nothing was stranded — all 29 categories, named ===")
	var total := 0
	for dom in EXPECTED:
		var panel: Control = app.call("workspace_panel", dom)
		_ok("the %s dock exists" % dom, panel != null, true)
		if panel == null:
			continue
		var titles := []
		for e in (panel.get("categories") as Array):
			titles.append(String(e["title"]))
		for title in EXPECTED[dom]:
			total += 1
			var t := String(title)
			## Reachable two ways, and both must hold. `open_category` is what
			## every cross-domain jump button in the shell calls; `mode_for_category`
			## is what tells the rail which node to light. A category the docks
			## still build but no node owns is half-stranded: you can get to it,
			## and the rail then lies about where you are.
			_ok("[%s] %s is still built" % [dom, t], titles.has(t), true)
			_ok("[%s] %s opens" % [dom, t], panel.call("open_category", t), true)
			_ok("[%s] %s is owned by a rail node" % [dom, t],
				String(app.call("mode_for_category", dom, t)).is_empty(), false)
		## The converse: a dock that grew a category no `EXPECTED` row names is
		## as much a drift as one that lost a category. Reported by name so the
		## next reader can decide whether to add it above or delete it below.
		for t in titles:
			if not (EXPECTED[dom] as Array).has(String(t)):
				print("  EXTRA   [%s] %s — built but not in EXPECTED" % [dom, String(t)])
		_ok("[%s] the dock builds no category EXPECTED does not name" % dom,
			titles.size(), (EXPECTED[dom] as Array).size())
	print("  info categories asserted by name: ", total)
	_ok("all twenty-nine were asserted", total, 29)

	## Every real `select_domain_category()` call site in the shell, by (domain,
	## category), each resolving to the node that will light. These are grepped
	## from `faction_roster_window.gd`, `menus.gd`, `phone_menu.gd`,
	## `cartography_workspace.gd` and `civilization_workspace.gd`; the signature
	## gained an optional `mode` this stage and none of them passes it, so this
	## is the assertion that the derivation covers them all.
	print("\n=== 3a: every live cross-domain jump resolves to a node ===")
	for jump in [
		["civilization", "Military", "factions"],
		["civilization", "Landmarks", "landmarks"],
		["civilization", "Timeline", "factions"],
		["civilization", "Routes & ways", "infra"],
		["civilization", "Territories", "factions"],
		["civilization", "Factions", "factions"],
		["civilization", "Economy", "factions"],
		["cartography", "Labels", "labels"],
		["cartography", "Feature style", "style"],
		["cartography", "Icons", "icons"],
		["world", "Ecology", "a"],
		["world", "World data", "a"],
	]:
		app.call("select_domain_category", String(jump[0]), String(jump[1]))
		await _frames(1)
		_ok("→ %s ▸ %s lights node '%s'" % [jump[0], jump[1], jump[2]],
			app.call("active_mode", String(jump[0])), String(jump[2]))

	# =====================================================================
	print("\n=== 4: the expansion column — click the domain, click a node ===")
	app.call("select_domain", "world")
	app.call("set_rail_expanded", false)
	await _frames(2)
	_ok("collapsed at rest (railExp:false, ENV:1199)", app.call("is_rail_expanded"), false)

	## BUILD_ANSWERS §2.5, first half: clicking the ALREADY-ACTIVE domain
	## toggles the expansion. Driven by pressing the real rail button, not by
	## calling the handler — a button wired to `_select_domain` instead of
	## `_on_domain_pressed` would pass a handler-level test and fail here.
	var rail_buttons: Dictionary = app.get("_domain_buttons")
	var world_btn: Button = rail_buttons["world"]
	world_btn.pressed.emit()
	await _frames(2)
	_ok("clicking the active domain opens the expansion", app.call("is_rail_expanded"), true)
	world_btn.pressed.emit()
	await _frames(2)
	_ok("...and clicking it again closes it", app.call("is_rail_expanded"), false)

	## Second half: clicking a NODE closes the expansion.
	world_btn.pressed.emit()
	await _frames(2)
	_ok("re-opened for the node test", app.call("is_rail_expanded"), true)
	var node_rows: Dictionary = app.get("_rail_node_rows")
	_ok("ten node rows were built", node_rows.size(), 10)
	(node_rows["world/b"] as Button).pressed.emit()
	await _frames(2)
	_ok("clicking a node closes the expansion", app.call("is_rail_expanded"), false)
	_ok("...and it selected that node's mode", app.call("active_mode", "world"), "b")

	## Switching domain closes it too (`setDomain`'s own `railExp:false`,
	## `ENV:2054`) — otherwise the column would survive a domain change showing
	## the previous domain's selection.
	world_btn.pressed.emit()
	await _frames(2)
	(rail_buttons["cartography"] as Button).pressed.emit()
	await _frames(2)
	_ok("switching domain closes the expansion", app.call("is_rail_expanded"), false)
	_ok("...and switched", app.call("active_domain"), "cartography")

	# =====================================================================
	print("\n=== 5: CARTO's four nodes are four destinations, not one ===")
	## The defect this section exists for: in the truncated prototype all four
	## CARTO nodes carried `mode:''`, so all four lit together and all four
	## opened the same dock (`spec/02-rail-and-domains.md` §3a). BUILD_ANSWERS
	## §2.1 gave them four real modes. This asserts the port did not inherit the
	## broken version — one node lit, one category open, three closed, four
	## times over.
	var carto: Control = app.call("workspace_panel", "cartography")
	var carto_nodes := {
		"style": "Style", "layers": "Layers",
		"labels": "Labels", "icons": "Icons",
	}
	var reached := {}
	for mode in carto_nodes:
		(node_rows["cartography/%s" % mode] as Button).pressed.emit()
		await _frames(2)
		var open_titles := []
		for e in (carto.get("categories") as Array):
			if (e["body"] as Control).visible:
				open_titles.append(String(e["title"]))
		_ok("CARTO ▸ %s opens exactly one category" % mode, open_titles.size(), 1)
		if open_titles.size() == 1:
			_ok("...and it is '%s'" % carto_nodes[mode], open_titles[0], String(carto_nodes[mode]))
			reached[String(open_titles[0])] = true
		## Exactly one node accent, and it is this one — the `!n.mode`
		## short-circuit that lit all four is gone.
		var lit := []
		for key in node_rows:
			if not String(key).begins_with("cartography/"):
				continue
			var b: Button = node_rows[key]
			if b.get_theme_color("font_color").is_equal_approx(DccTheme.c("accent")):
				lit.append(String(key))
		_ok("...and exactly one CARTO node is accent", lit.size(), 1)
		if lit.size() == 1:
			_ok("...and it is this one", lit[0], "cartography/%s" % mode)
	_ok("four distinct destinations, not one", reached.size(), 4)

	# =====================================================================
	print("\n=== 6: the two new panels are drawn, and drawn honestly ===")
	## BUILD_ANSWERS §2.1: "LABELS and ICONS are new and real". This section
	## used to assert the opposite of what it says below: `cartography_workspace.gd`
	## drew Labels' three per-class dials and Icons' two generation sliders and
	## three placement-rule checkboxes disabled, with a reason, because nothing
	## in the engine backed them yet. That was true when this section was
	## written (2026-08-31, `c03b43c`) and stopped being true within days: the
	## owner's 2026-09-02/09-03 rulings (`LARGE_ITEM_RULINGS.md`) built
	## `LabelTypography` (size/halo/tracking, `cartalith-civ/src/labels.rs`) and
	## the generated icon-placement pass (`icon_bridge/generate.rs`), and
	## `cartography_workspace.gd`'s own header block says so at length — "Both
	## dials are live... All three are live now". Two later rail-fold edits to
	## THIS file (`5f839d7`, `04b3b27`, 2026-09-20/21) touched this section only
	## to rename a category string ("Assets & landmarks" -> "Icons") and never
	## re-checked the assertion itself, so it kept asserting pre-ruling
	## behaviour for three weeks after it went false. Confirmed stale by
	## running this probe live, 2026-09-21: `got=0` inert dials against a
	## `want=3`, `got=0` against `want=2`, and `got=3` enabled rules against a
	## `want=0` — the exact three failures this reinvestigation started from.
	## Reasserted here as LIVE, and with a real effect when driven, not a bare
	## `disabled == false` (the house rule this file's own brief names: a
	## control's state is a claim about the live build, checked at the build,
	## not read off a comment).
	var carto_body := _category_body(carto, "Labels")
	_ok("the Labels category has a body", carto_body != null, true)
	## Counted by editability, not by total, for the reason the old comment
	## here gave for inertness: the Labels category also hosts the region-label
	## edit form, whose Size/Arc/Angle sliders exist only once a label is
	## selected. Exactly three live per-class dials is the claim: size, halo,
	## tracking.
	var lab_sliders := _sliders_in(carto_body)
	var lab_live := 0
	for s in lab_sliders:
		if (s as HSlider).editable:
			lab_live += 1
	print("  info sliders under Labels: %d (%d live)" % [lab_sliders.size(), lab_live])
	_ok("the design's three per-class dials are drawn, and live", lab_live, 3)

	## Real effect, not a flag read back: drag the halo dial to a value
	## nothing starts at, release it (`_regenerate_labels`'s own tooltip:
	## "Released, not dragged: letting go re-runs the labelling pass" --
	## exercised here so the release path is proven not to crash even with no
	## world generated), and read the change back out of the class spec the
	## dial's `on_change` writes into (`_label_class_dial()`'s own
	## `spec[field] = v`).
	##
	## **Not** read back through the engine's `label_class_table()`: this probe
	## never calls `generate()`, and `labels_generate()` discards `typography`
	## and returns `ok:false` before any world exists
	## (`label_bridge/generate.rs`'s own early return on `self.labels.is_none()`)
	## -- so `label_class_table()` would still report the shipped defaults no
	## matter what the dial did, and that absence would be this probe's, not
	## the control's.
	var halo_slider: HSlider = null
	for s in lab_sliders:
		if _row_label(s) == "halo":
			halo_slider = s
	_ok("the halo dial is one of the three", halo_slider != null, true)
	if halo_slider != null:
		var active_class := String(carto.get("_label_class"))
		var want_halo: float = halo_slider.min_value if is_equal_approx(halo_slider.value, halo_slider.max_value) else halo_slider.max_value
		halo_slider.value = want_halo
		halo_slider.drag_ended.emit(true)
		await _frames(2)
		var got_halo := -1.0
		for entry in (carto.get("_label_class_specs") as Array):
			var d: Dictionary = entry
			if String(d.get("key", "")) == active_class:
				got_halo = float(d.get("halo", -1.0))
		_ok("...and dragging it changes the class's own spec",
			is_equal_approx(got_halo, want_halo), true)

	var icons_body := _category_body(carto, "Icons")
	var ico_sliders := _sliders_in(icons_body)
	var ico_live := 0
	for s in ico_sliders:
		if (s as HSlider).editable:
			ico_live += 1
	print("  info sliders under Icons: %d (%d live)" % [ico_sliders.size(), ico_live])
	_ok("icon scale and min spacing are drawn, and live", ico_live, 2)

	## Real effect for both: drive each to a value distinct from its default
	## and confirm the GDScript state `_run_icon_placement()` actually sends
	## into `bridge.icon_generate()` moved with it — the same private vars
	## named at that call site, `_icon_gen_scale` / `_icon_gen_spacing`.
	var scale_slider: HSlider = null
	var spacing_slider: HSlider = null
	for s in ico_sliders:
		if _row_label(s) == "icon scale":
			scale_slider = s
		elif _row_label(s) == "min spacing":
			spacing_slider = s
	if scale_slider != null:
		scale_slider.value = scale_slider.max_value
		await _frames(1)
		_ok("...and dragging icon scale updates the value the engine call sends",
			is_equal_approx(float(carto.get("_icon_gen_scale")), scale_slider.max_value), true)
	if spacing_slider != null:
		spacing_slider.value = spacing_slider.max_value
		await _frames(1)
		_ok("...and dragging min spacing updates the value the engine call sends",
			is_equal_approx(float(carto.get("_icon_gen_spacing")), spacing_slider.max_value), true)

	## The three placement rules. `snap sea marks to coast` is the one whose
	## family has no engine counterpart at all, so it is named specifically.
	var checks := _checks_in(icons_body)
	## The row label, not `CheckBox.text`. `DccWidgets._row()` draws the caption
	## as a separate `Label` at the head of the row and leaves the box itself
	## textless (its own comment: "Only the *value* on the right is Plex"), so
	## reading `.text` off the box returns "" for every toggle in the shell.
	var rule_names := []
	for c in checks:
		rule_names.append(_row_label(c))
	print("  info toggles under Icons: ", rule_names)
	_ok("the three placement rules are drawn",
		rule_names.has("avoid label boxes") and rule_names.has("enforce min spacing")
		and rule_names.has("snap sea marks to coast"), true)
	var disabled_rules := 0
	for c in checks:
		if (c as CheckBox).disabled:
			disabled_rules += 1
	_ok("every placement rule is live, none disabled", disabled_rules, 0)
	## Each rule's own backing var, so a checkbox miswired to a NEIGHBOUR's
	## flag (live, but the wrong one) would still fail here even though
	## "disabled_rules == 0" above cannot see that.
	var rule_vars := {
		"avoid label boxes": "_icon_gen_avoid_labels",
		"enforce min spacing": "_icon_gen_enforce_spacing",
		"snap sea marks to coast": "_icon_gen_snap_coast",
	}
	for c in checks:
		var rlabel := _row_label(c)
		var field: String = rule_vars.get(rlabel, "")
		if field.is_empty():
			continue
		var before := bool(carto.get(field))
		(c as CheckBox).toggled.emit(not before)
		await _frames(1)
		_ok("...and toggling '%s' flips its own flag" % rlabel,
			bool(carto.get(field)), not before)
	## Live controls still owe the user a description, even though it is no
	## longer a disabled-reason.
	var no_tooltip := []
	for c in checks:
		if String((c as CheckBox).tooltip_text).strip_edges().is_empty():
			no_tooltip.append(_row_label(c))
	if not no_tooltip.is_empty():
		print("  MISSING  no tooltip: ", no_tooltip)
	_ok("every placement rule still carries an explanatory tooltip", no_tooltip.size(), 0)

	print("\n_railfold_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)

func _sliders_in(node: Node) -> Array:
	var out := []
	if node == null:
		return out
	for c in node.get_children():
		if c is HSlider:
			out.append(c)
		else:
			out.append_array(_sliders_in(c))
	return out

## The caption `DccWidgets._row()` drew for a control -- the first `Label` among
## the control's own siblings.
func _row_label(c: Control) -> String:
	var row := c.get_parent()
	if row == null:
		return ""
	for sib in row.get_children():
		if sib is Label:
			return (sib as Label).text
	return ""

func _checks_in(node: Node) -> Array:
	var out := []
	if node == null:
		return out
	for c in node.get_children():
		if c is CheckBox:
			out.append(c)
		else:
			out.append_array(_checks_in(c))
	return out
