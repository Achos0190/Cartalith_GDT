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
## So §3 below does not count categories. It **names all thirty-three**, one
## string at a time, and asserts each is still openable and still owned by a rail
## node. A count would pass a build that dropped `Trade` and gained `Trade ` with
## a trailing space; a name list will not. That is the same discipline
## `_cmdindex_probe` §2 uses when it asserts the engine's own labels rather than
## a row count, and for the same reason.
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
## `cartography_workspace.gd` — including the four CIVIL categories whose bodies
## are filled by `infrastructure_workspace.gd` (`Routes & ways`, `Travel`,
## `Trade`) and the four CARTO ones filled by `render_workspace.gd` (`Map style`,
## `Terrain appearance`, `Colours`, `Map presets`). Those eight are the fold's
## actual cargo and the reason the list is written out rather than walked.
##
## Hard-coded on purpose. Walking `panel.categories` and asserting each entry
## against itself would prove nothing; this list is an independent statement of
## what must exist, and it fails loudly when a category is renamed as well as
## when one is lost. If a category is deliberately renamed, this list changes in
## the same commit and the diff says so.
const EXPECTED: Dictionary = {
	"world": [
		"Generate", "Terrain", "Geology", "Hydrology", "Climate",
		"Biomes", "Ecology", "Resources", "World data",
	],
	"civilization": [
		"Civilizations", "Factions", "Territories", "Settlements", "Landmarks",
		"Routes & ways", "Travel", "Trade", "Economy", "Culture",
		"Politics", "Military", "Relationships", "Simulation",
	],
	"cartography": [
		"Map style", "Terrain appearance", "Colours", "Layers", "Roads & routes",
		"Labels", "Assets & landmarks", "Political display", "Visibility / zoom",
		"Map presets",
	],
}

## The design's own node tree (`ENV:1824`, transcribed in
## `spec/02-rail-and-domains.md` §3 and settled by BUILD_ANSWERS §2.1), stated
## here independently of `DccShell.RAIL_NODES` so that §1 compares two sources
## rather than one source with itself.
const DESIGN_NODES: Array = [
	["world", "a", "Generation pipeline"],
	["world", "b", "Sculpt"],
	["civilization", "landmarks", "Landmarks"],
	["civilization", "factions", "Factions & settlements"],
	["civilization", "infra", "Ways & routes"],
	["civilization", "planner", "Journey planner"],
	["cartography", "style", "Layers & style"],
	["cartography", "labels", "Labels"],
	["cartography", "icons", "Icons"],
	["cartography", "terrain", "Terrain appearance"],
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
	print("\n=== 3: nothing was stranded — all 33 categories, named ===")
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
	_ok("all thirty-three were asserted", total, 33)

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
		["civilization", "Simulation", "factions"],
		["civilization", "Routes & ways", "infra"],
		["civilization", "Territories", "factions"],
		["civilization", "Factions", "factions"],
		["civilization", "Trade", "infra"],
		["cartography", "Labels", "labels"],
		["cartography", "Political display", "style"],
		["cartography", "Assets & landmarks", "icons"],
		["cartography", "Roads & routes", "style"],
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
		"style": "Layers", "labels": "Labels",
		"icons": "Assets & landmarks", "terrain": "Terrain appearance",
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
	## BUILD_ANSWERS §2.1: "LABELS and ICONS are new and real". They are drawn
	## in full and disabled with their reason (`cartography_workspace.gd`'s own
	## header on the block) — so the assertion is that the controls EXIST and
	## that every one of them is inert. A panel that quietly omitted the
	## unbindable half would pass a "does it have controls" test; this fails it.
	var carto_body := _category_body(carto, "Labels")
	_ok("the Labels category has a body", carto_body != null, true)
	## Counted by inertness, not by total. The Labels category also hosts the
	## LIVE region-label edit form, whose Size/Arc/Angle sliders appear the
	## moment a label is selected — so "every slider under Labels is inert"
	## would be true at boot and false after one click, which is a probe that
	## reports the shell's state rather than its correctness. Exactly three
	## inert dials is the claim: size, halo, tracking.
	var lab_sliders := _sliders_in(carto_body)
	var lab_dead := _inert(lab_sliders)
	print("  info sliders under Labels: %d (%d inert)" % [lab_sliders.size(), lab_dead.size()])
	_ok("the design's three per-class dials are drawn, and inert", lab_dead.size(), 3)
	_ok("...each carrying its reason", _silent(lab_dead).size(), 0)

	var icons_body := _category_body(carto, "Assets & landmarks")
	var ico_sliders := _sliders_in(icons_body)
	var ico_dead := _inert(ico_sliders)
	print("  info sliders under Assets & landmarks: %d (%d inert)" % [ico_sliders.size(), ico_dead.size()])
	_ok("icon scale and min spacing are drawn, and inert", ico_dead.size(), 2)
	_ok("...each carrying its reason", _silent(ico_dead).size(), 0)
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
	print("  info toggles under Assets & landmarks: ", rule_names)
	_ok("the three placement rules are drawn",
		rule_names.has("avoid label boxes") and rule_names.has("enforce min spacing")
		and rule_names.has("snap sea marks to coast"), true)
	var enabled_rules := 0
	for c in checks:
		if not (c as CheckBox).disabled:
			enabled_rules += 1
	_ok("every placement rule is disabled", enabled_rules, 0)
	## Disabled is not enough — the house rule is "disabled WITH its reason".
	var silent := []
	for c in checks:
		if (c as CheckBox).disabled and String((c as CheckBox).tooltip_text).strip_edges().is_empty():
			silent.append(_row_label(c))
	if not silent.is_empty():
		print("  SILENT  disabled with no reason: ", silent)
	_ok("every disabled rule carries a reason", silent.size(), 0)

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

## The sliders among `all` that cannot be moved.
func _inert(all: Array) -> Array:
	var out := []
	for s in all:
		if not (s as HSlider).editable:
			out.append(s)
	return out

## Of those, the ones that do not say why — the house rule is "disabled WITH its
## reason", so an inert control with an empty tooltip is a defect, not a pass.
func _silent(controls: Array) -> Array:
	var out := []
	for c in controls:
		if String((c as Control).tooltip_text).strip_edges().is_empty():
			out.append(_row_label(c))
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
