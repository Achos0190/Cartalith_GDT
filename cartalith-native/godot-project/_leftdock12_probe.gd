extends Node
## The left dock's §3 blocks — the gate, and the proof that it stranded nothing.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _leftdock12_probe.tscn
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _leftdock12_probe.tscn -- --force-touch
##
## Two runs, because `_touch` is decided once per process from the command line
## (`dcc_shell.gd:584`) and the three densities cannot share one. Without the
## flag this runs the desktop leg; with it, the tablet and phone legs.
##
## **What this exists to catch.** Owner ruling, `LARGE_ITEM_RULINGS.md`
## 2026-09-05 item 2, restructures the dock to `04-left-dock.md` §3's blocks,
## and §3 gates exactly one of the ten rail nodes — `world/b`, which renders the
## Sculpt block and hides WORLD's other eight categories. A gate is the one
## change to this dock that can make a control unreachable, and this shell has
## already shipped the mirror-image failure once (a rail node that selected a
## mode and opened nothing, `dcc_shell.gd::_on_rail_node_pressed()`'s own
## header). So §2 below does not count what survives: it **names every one of
## the thirty-four categories** and, for each, prints the modes that render it
## and the rail node that reaches each of those modes. A count would pass a
## build where `Climate` vanished and `Climate ` appeared; a route census will
## not.
##
## `_railfold_probe.gd` is the sibling that asserts the 5→3 *fold*; this one
## asserts the *gate* laid over it. The two overlap deliberately on the category
## name list, which is the property both are protecting.

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

## Every L2 category the three docks build, by name. Independent of
## `panel.categories` on purpose — asserting a dock against itself proves
## nothing. Thirty-four: `_railfold_probe.gd`'s thirty-three plus `Religion`,
## which `civilization_workspace.gd::_build_religion()` has built since the
## belief layer landed and which that probe's own `EXPECTED` still omits (it
## reports it as `EXTRA`, and that row is a real gap in that file, not here).
const EXPECTED: Dictionary = {
	"world": [
		"Generate", "Terrain", "Geology", "Hydrology", "Climate",
		"Biomes", "Ecology", "Resources", "World data",
	],
	"civilization": [
		"Civilizations", "Factions", "Territories", "Settlements", "Landmarks",
		"Routes & ways", "Travel", "Trade", "Economy", "Culture", "Religion",
		"Politics", "Military", "Relationships", "Simulation",
	],
	"cartography": [
		"Map style", "Terrain appearance", "Colours", "Layers", "Roads & routes",
		"Labels", "Assets & landmarks", "Political display", "Visibility / zoom",
		"Map presets",
	],
}

## The gate, stated here independently of `DccShell.RAIL_NODES` so §1 compares
## two sources rather than one source with itself. `04-left-dock.md` §3 row 2:
## `ldSculpt = domain==='WORLD' && worldMode==='b'`, and row 1's `ldPipe` is its
## complement. Every other row's condition is a plain `domain===` with no mode
## in it, which is why nine of the ten nodes are ungated.
const DESIGN_GATES: Dictionary = {"world/b": ["Terrain"]}

func _boot(w: int, h: int) -> Node:
	var vp := SubViewport.new()
	vp.size = Vector2i(w, h)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(50)
	return app

## The header/body/divider wrapper `DccWidgets.category()` builds. Read through
## the body's parent rather than a stored handle, which is what the shell itself
## does — a probe that knows a private field breaks on a rename instead of on a
## regression.
func _wrap(panel: Control, title: String) -> Control:
	for e in (panel.get("categories") as Array):
		if String(e["title"]) == title:
			return (e["body"] as Control).get_parent() as Control
	return null

func _body(panel: Control, title: String) -> Control:
	for e in (panel.get("categories") as Array):
		if String(e["title"]) == title:
			return e["body"]
	return null

## The category headers this dock is rendering right now, in dock order.
func _rendered(panel: Control) -> Array:
	var out: Array = []
	for e in (panel.get("categories") as Array):
		var w := (e["body"] as Control).get_parent() as Control
		if w != null and w.visible:
			out.append(String(e["title"]))
	return out

func _modes(app: Node, dom: String) -> Array:
	var out: Array = []
	for n in app.call("domain_nodes", dom):
		out.append(String(n["mode"]))
	return out

func _run(app: Node, label: String) -> void:
	print("\n########## ", label, " ##########")

	# =====================================================================
	print("\n=== 1: the gate table — one node gates, nine do not ===")
	var gated: Array = []
	for n in app.get("RAIL_NODES"):
		if String(n.get("kind", "")) != "node":
			continue
		var key := "%s/%s" % [String(n["domain"]), String(n["mode"])]
		var shows: Array = n.get("shows", [])
		if not shows.is_empty():
			gated.append("%s=%s" % [key, ",".join(shows)])
	_ok("exactly the design's gates, and no others",
		"; ".join(gated), "world/b=Terrain")
	## The accessor, not just the constant: `apply_mode()` reads through
	## `mode_shows()`, and a key spelt right in a table nothing reads is the
	## same defect as a key spelt wrong.
	for dom in EXPECTED:
		for mode in _modes(app, String(dom)):
			var key := "%s/%s" % [String(dom), String(mode)]
			var want: Array = DESIGN_GATES.get(key, [])
			_ok("mode_shows(%s)" % key,
				",".join(app.call("mode_shows", String(dom), String(mode))),
				",".join(want))
	_ok("domain_gates: WORLD", app.call("domain_gates", "world"), true)
	_ok("domain_gates: CIVIL", app.call("domain_gates", "civilization"), false)
	_ok("domain_gates: CARTO", app.call("domain_gates", "cartography"), false)

	# =====================================================================
	print("\n=== 2: the route census — all 34 categories, named, with their routes ===")
	## For each category: which modes render it, and which rail node reaches
	## each of those modes. A category rendered by no mode is unreachable; a
	## category rendered only by a mode with no node is reachable only by
	## accident. Both are failures, and both are stated per name.
	var total := 0
	for dom in EXPECTED:
		var panel: Control = app.call("workspace_panel", String(dom))
		_ok("the %s dock exists" % dom, panel != null, true)
		if panel == null:
			continue
		## Drive the real entry point for every mode and record what renders.
		var per_mode := {}
		for mode in _modes(app, String(dom)):
			app.call("select_domain_mode", String(dom), String(mode))
			await _frames(2)
			per_mode[mode] = _rendered(panel)
		var union := {}
		for mode in per_mode:
			for t in per_mode[mode]:
				if not union.has(t):
					union[t] = []
				(union[t] as Array).append(mode)
		for title in EXPECTED[dom]:
			total += 1
			var t := String(title)
			var routes: Array = union.get(t, [])
			var labels: Array = []
			for m in routes:
				labels.append("%s ▸ %s" % [String(dom),
					String(app.call("rail_node", String(dom), String(m)).get("label", m))])
			_ok("[%s] %s is rendered by at least one mode" % [dom, t],
				routes.is_empty(), false)
			print("        route(s): ", " · ".join(labels) if not labels.is_empty() else "NONE")
			## The rail must also *light* for it, or the shell knows where the
			## category is and the user cannot tell.
			_ok("[%s] %s is owned by a rail node" % [dom, t],
				String(app.call("mode_for_category", String(dom), t)).is_empty(), false)
		## The converse: the union over every mode must be the whole list, so a
		## category cannot be lost by being rendered nowhere *and* absent from
		## `categories`.
		var extra: Array = []
		for t in union:
			if not (EXPECTED[dom] as Array).has(String(t)):
				extra.append(String(t))
		_ok("[%s] nothing rendered that EXPECTED does not name" % dom,
			", ".join(extra), "")
		_ok("[%s] the union over all modes is the whole list" % dom,
			union.size(), (EXPECTED[dom] as Array).size())
	_ok("all thirty-four were asserted by name", total, 34)

	# =====================================================================
	print("\n=== 3: every rail node reaches its block ===")
	## The failure this section exists for: a node that selects a mode and
	## opens nothing, or opens a category the mode then hides. Driven through
	## the real button, not the handler.
	## The phone composition has no expansion column — its rail is the floating
	## one `_build_phone_menu_bar()` builds, with no per-node rows — so there the
	## same journey is driven through `select_domain_mode()`, which is what the
	## phone's own domain cells call. Stated rather than skipped: a section that
	## quietly does nothing on one density is a section that cannot fail there.
	var rows: Dictionary = app.get("_rail_node_rows")
	var by_button := not rows.is_empty()
	print("        driven by: ", "rail node buttons" if by_button else "select_domain_mode (no expansion column on this composition)")
	for dom in EXPECTED:
		for mode in _modes(app, String(dom)):
			var key := "%s/%s" % [String(dom), String(mode)]
			if by_button:
				(rows[key] as Button).pressed.emit()
			else:
				app.call("select_domain_mode", String(dom), String(mode))
			await _frames(2)
			var node: Dictionary = app.call("rail_node", String(dom), String(mode))
			var cat := String(node["category"])
			var panel: Control = app.call("workspace_panel", String(dom))
			var b := _body(panel, cat)
			var w := _wrap(panel, cat)
			_ok("%s → '%s' body open" % [key, cat], b != null and b.visible, true)
			_ok("%s → '%s' header rendered" % [key, cat], w != null and w.visible, true)
			_ok("%s → the domain is active" % key, app.call("active_domain"), String(dom))

	# =====================================================================
	print("\n=== 4: the gate actually gates, and only where §3 says ===")
	var world: Control = app.call("workspace_panel", "world")
	var civ: Control = app.call("workspace_panel", "civilization")
	var carto: Control = app.call("workspace_panel", "cartography")
	app.call("select_domain_mode", "world", "a")
	await _frames(2)
	_ok("WORLD ▸ a renders the whole pipeline", _rendered(world).size(), 9)
	app.call("select_domain_mode", "world", "b")
	await _frames(2)
	_ok("WORLD ▸ b renders the Sculpt block alone",
		", ".join(_rendered(world)), "Terrain")
	for mode in _modes(app, "civilization"):
		app.call("select_domain_mode", "civilization", String(mode))
		await _frames(2)
		_ok("CIVIL ▸ %s keeps every header (§3 point 3)" % mode,
			_rendered(civ).size(), 15)
	for mode in _modes(app, "cartography"):
		app.call("select_domain_mode", "cartography", String(mode))
		await _frames(2)
		_ok("CARTO ▸ %s keeps every header (§3 point 2)" % mode,
			_rendered(carto).size(), 10)

	# =====================================================================
	print("\n=== 5: every transition INTO the gated state ===")
	## Enumerated from the callers, not from the one that was easiest to reach:
	## the disarm path is never the one that breaks.

	## (a) rail node press — covered in §3; re-checked here as the baseline the
	## rest are compared against.
	app.call("select_domain_mode", "world", "b")
	await _frames(2)
	_ok("(a) rail node → b", app.call("active_mode", "world"), "b")

	## (b) the dock's own mode switch, pressed as a button.
	var segs: Dictionary = app.get("_mode_switch_buttons")
	(segs["a"] as Button).pressed.emit()
	await _frames(2)
	_ok("(b) mode switch → a", app.call("active_mode", "world"), "a")
	_ok("(b) ...and the pipeline is back", _rendered(world).size(), 9)
	(segs["b"] as Button).pressed.emit()
	await _frames(2)
	_ok("(b) mode switch → b", app.call("active_mode", "world"), "b")

	## (c) a cross-domain jump to a category the *current* mode hides. This is
	## the one that silently did nothing before the gate learned to un-gate.
	app.call("select_domain_category", "world", "Climate")
	await _frames(2)
	_ok("(c) jump to a gated category switches mode", app.call("active_mode", "world"), "a")
	_ok("(c) ...and Climate is rendered", _wrap(world, "Climate").visible, true)
	_ok("(c) ...and open", _body(world, "Climate").visible, true)

	## (d) `Workspace.open_category()` called directly, which is what every
	## in-dock "→ …" button reaches, while the mode hides the target.
	app.call("select_domain_mode", "world", "b")
	await _frames(2)
	_ok("(d) precondition: Geology is hidden", _wrap(world, "Geology").visible, false)
	_ok("(d) open_category returns true", world.call("open_category", "Geology"), true)
	await _frames(2)
	_ok("(d) ...and Geology is actually rendered", _wrap(world, "Geology").visible, true)
	_ok("(d) ...and the rail followed", app.call("active_mode", "world"), "a")

	## (e) arming Sculpt from another domain — `04-left-dock.md` §2.4's
	## `armTool`: "arming a sculpt tool from anywhere jumps the dock to WORLD·b".
	app.call("select_domain", "civilization")
	app.call("arm_tool", "inspect")
	await _frames(2)
	app.call("arm_tool", "sculpt")
	await _frames(3)
	_ok("(e) arming Sculpt from CIVIL selects WORLD", app.call("active_domain"), "world")
	_ok("(e) ...and mode b", app.call("active_mode", "world"), "b")
	_ok("(e) ...and Terrain is rendered and open",
		_wrap(world, "Terrain").visible and _body(world, "Terrain").visible, true)

	## (f) and Biome paint, whose controls are parented into `Biomes`, not
	## `Terrain` — so it must land in the other mode, not the same one.
	app.call("arm_tool", "paint")
	await _frames(3)
	_ok("(f) arming Biome paint selects mode a", app.call("active_mode", "world"), "a")
	_ok("(f) ...and Biomes is rendered and open",
		_wrap(world, "Biomes").visible and _body(world, "Biomes").visible, true)
	app.call("arm_tool", "inspect")
	await _frames(2)

	## (g) mode persistence across a domain round trip. `_domain_mode` is
	## per-domain state; leaving WORLD in Sculpt and coming back must restore
	## Sculpt's body, not repaint the pipeline over a Sculpt rail.
	app.call("select_domain_mode", "world", "b")
	await _frames(2)
	app.call("select_domain", "cartography")
	await _frames(2)
	app.call("select_domain", "world")
	await _frames(2)
	_ok("(g) returning to WORLD restores mode b", app.call("active_mode", "world"), "b")
	_ok("(g) ...and the Sculpt block, not the pipeline",
		", ".join(_rendered(world)), "Terrain")

	## (h) the floor. Re-clicking the one open header in a gated dock would
	## otherwise leave a dock with a header and nothing under it.
	var terrain_btn: Button = null
	for e in (world.get("categories") as Array):
		if String(e["title"]) == "Terrain":
			terrain_btn = e["button"]
	terrain_btn.pressed.emit()
	await _frames(2)
	_ok("(h) collapsing the only visible category re-opens it",
		_body(world, "Terrain").visible, true)
	## And CIVIL's named floor still wins over build order: `Civilizations` is
	## built first, `Landmarks` is §6's floor.
	app.call("select_domain_mode", "civilization", "factions")
	await _frames(2)
	var civ_open: Button = null
	for e in (civ.get("categories") as Array):
		if (e["body"] as Control).visible:
			civ_open = e["button"]
	if civ_open != null:
		civ_open.pressed.emit()
		await _frames(2)
	_ok("(h) CIVIL's floor is still Landmarks, not the first-built category",
		_body(civ, "Landmarks").visible, true)

	# =====================================================================
	print("\n=== 6: the mode switch (§2.1 band 2 / §2.3) ===")
	app.call("select_domain_mode", "world", "a")
	await _frames(2)
	var row: Control = app.get("_mode_switch_row")
	_ok("shown in WORLD", row.visible, true)
	app.call("select_domain", "civilization")
	await _frames(2)
	_ok("hidden in CIVIL (no gate, no switch)", row.visible, false)
	app.call("select_domain", "cartography")
	await _frames(2)
	_ok("hidden in CARTO", row.visible, false)
	app.call("select_domain_mode", "world", "b")
	await _frames(2)
	_ok("back in WORLD", row.visible, true)
	## The lit segment has to agree with the body. Two segments, one lit.
	var lit: Array = []
	for m in segs:
		var col: Color = (segs[m] as Button).get_theme_color("font_color")
		if col.is_equal_approx(DccTheme.c("accent")):
			lit.append(String(m))
	_ok("exactly one segment is accent", lit.size(), 1)
	_ok("...and it is the active mode", ", ".join(lit), app.call("active_mode", "world"))

	# =====================================================================
	print("\n=== 7: the collapsed dock, measured ===")
	## The phone has no collapsed dock: `_build_left_dock(true)` makes the dock a
	## full-height sheet with a close button where the collapse chevron is, and
	## `_toggle_dock()` is never reached. §6 above already proved the switch is
	## built and correct in the sheet, which is the half of this that applies.
	if bool(app.call("is_phone")):
		print("        phone composition — the dock is a sheet, there is no collapsed strip")
		var seg_a: Button = segs["a"]
		print("        sheet mode-switch combined minimum: ", row.get_combined_minimum_size())
		print("        segment 'a' custom_min=", seg_a.custom_minimum_size,
			"  combined_min=", seg_a.get_combined_minimum_size(),
			"  font_size=", seg_a.get_theme_font_size("font_size"))
		## `phone_fit()` floors every `BaseButton` at `PHONE_TAP_MIN * unit`.
		## Assert the segment landed on that floor and not on some multiple of
		## it: a pre-scaled figure written here is scaled a second time by the
		## fitter, which is how one segment measured 301 px.
		var unit := float(app.get("_phone_scale"))
		var want_tap := roundf(DccTheme.PHONE_TAP_MIN * unit)
		_ok("segment height is exactly the phone tap floor, not a multiple of it",
			seg_a.custom_minimum_size.y, want_tap)
		return
	## `_toggle_dock()` hides the scroll and the title. The switch is a third
	## pinned band and had to join them: two `SIZE_EXPAND_FILL` segments have a
	## combined minimum far past the 40 px collapsed strip, and a
	## `MarginContainer` propagates a child's minimum straight up — no
	## scrollbar anywhere to reveal it, which is this tree's recurring
	## overflow class.
	var dock: Control = app.get("left_dock")
	var open_min := dock.get_combined_minimum_size().x
	app.call("_toggle_dock", true)
	await _frames(3)
	_ok("collapsing hides the mode switch", row.visible, false)
	var shut_min := dock.get_combined_minimum_size().x
	var strip := float(DccTheme.W_RAIL_COLLAPSED)
	print("        left dock combined minimum x: open=", open_min,
		"  collapsed=", shut_min, "  strip=", strip)
	## **The assertion is about this change, not about the strip.** The collapsed
	## dock does not fit `W_RAIL_COLLAPSED` and did not before the mode switch
	## existed; the residue is the header band's own children, attributed below
	## so the number has a named owner rather than a suspicion. What the switch
	## has to prove is that it adds **nothing** to that residue — a hidden child
	## is skipped by a `Container`'s minimum-size pass, and this is the measured
	## form of that claim rather than the reasoned one.
	_attribute(dock, "        collapsed contributor: ")
	row.visible = true
	await _frames(2)
	var with_switch := dock.get_combined_minimum_size().x
	row.visible = false
	await _frames(2)
	_ok("the mode switch adds nothing while the dock is collapsed",
		dock.get_combined_minimum_size().x, shut_min)
	print("        (same dock with the switch forced visible: ", with_switch, ")")
	app.call("_toggle_dock", true)
	await _frames(3)
	_ok("re-opening restores the switch", row.visible, true)
	print("        mode-switch combined minimum x: ", row.get_combined_minimum_size().x,
		"  segment min y: ", (segs["a"] as Button).custom_minimum_size.y)

## Print every visible descendant that carries a horizontal minimum, so a dock
## that will not shrink names the child holding it open instead of being a
## number with a story attached to it.
func _attribute(root: Control, prefix: String, depth: int = 0) -> void:
	for c in root.get_children():
		if not (c is Control) or not (c as Control).visible:
			continue
		var ctl := c as Control
		var mx := ctl.get_combined_minimum_size().x
		if mx > 0.0:
			print(prefix, "  ".repeat(depth), ctl.get_class(), " '", ctl.name, "' min.x=", mx)
		if depth < 3:
			_attribute(ctl, prefix, depth + 1)

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var forced := "--force-touch" in OS.get_cmdline_user_args()
	print("[BOOT] force-touch=", forced)
	if not forced:
		var app := await _boot(1920, 1080)
		_ok("classified as desktop", DccTheme.is_touch(), false)
		await _run(app, "DESKTOP 1920x1080")
	else:
		var tapp := await _boot(2560, 1600)
		_ok("classified as TABLET", DccTheme.is_tablet(), true)
		await _run(tapp, "TABLET 2560x1600")
		var papp := await _boot(1080, 2340)
		_ok("classified as PHONE", DccTheme.is_phone(), true)
		await _run(papp, "PHONE 1080x2340 (sheet)")
	print("\n_leftdock12_probe: ", _fail, " FAILURE(S)" if _fail != 1 else " FAILURE")
	get_tree().quit(1 if _fail > 0 else 0)
