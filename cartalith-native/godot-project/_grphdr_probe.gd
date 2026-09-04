extends Node
## Lane B / batch 25 -- BLAST RADIUS of `DccWidgets.group()`'s header width.
##
## The row (OUTSTANDING_WORK.md, 2026-09-04) was filed off ONE header
## (`› VESSEL REFERENCE · SPEED BY WATER`, 258 px) and deliberately not taken
## because the radius was unmeasured. This measures it before anything changes.
##
## Two halves, and the second exists to make the first trustworthy:
##
##   A. SYNTHETIC SWEEP -- every `group()`/`advanced()` title in the shell,
##      built through the real factory into the real containment chain
##      (`section()` body inside the dock's own `ScrollContainer`), so the
##      number reported is the DOCK WIDTH that title demands, chrome included,
##      not a bare label width that still needs a fudge added to it.
##   B. LIVE CROSS-CHECK -- boot the shell, commit a route, arm Journey, find
##      the real header in the real dock and read the same two numbers off it.
##      If A and B disagree on that title, A is modelling the chain wrong and
##      every other row in it is worthless.
##
## Three seeds, because MISTAKES.md's layout row says one sample has given the
## wrong answer twice in this area -- and 13 of the 83 call sites interpolate
## world data into their title.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _grphdr_probe.tscn

const SEEDS := [483920, 77021, 4242]

## Every `DccWidgets.group()` / `DccWidgets.advanced()` call site in
## `godot-project/shell`, extracted from the tree rather than typed from
## memory. `h` is the host the call passes:
##   "sec"   -- a `DccWidgets.section()` body      (the common case)
##   "body"  -- the dock/window body directly       (no section margins)
##   "grp"   -- another `group()`'s body            (+10 px of indent)
## `w` is the narrowest container it can appear in: 260 = right dock at
## `W_RIGHT_DOCK_MIN`, 300 = left dock at `W_LEFT_DOCK_MIN`, otherwise the
## window's own `min_size.x`.
const SITES := [
	{"f": "faction_roster_window.gd:709", "t": "Settlements (%d)", "h": "body", "w": 620, "dyn": true},
	{"f": "journey_planner_view.gd:3224", "t": "vessel reference · speed by water", "h": "body", "w": 260},
	{"f": "journey_planner_view.gd:3304", "t": "vessels on this route", "h": "body", "w": 260},
	{"f": "place_editor_window.gd:372", "t": "Imports", "h": "sec", "w": 340},
	{"f": "place_editor_window.gd:376", "t": "Exports", "h": "sec", "w": 340},
	{"f": "place_editor_window.gd:687", "t": "Backlinks (%d)", "h": "sec", "w": 340, "dyn": true},
	{"f": "place_editor_window.gd:705", "t": "Unlinked mentions (%d)", "h": "sec", "w": 340, "dyn": true},
	{"f": "right_dock.gd:1753", "t": "Actions", "h": "sec", "w": 260},
	{"f": "right_dock.gd:1864", "t": "Adherence", "h": "sec", "w": 260},
	{"f": "right_dock.gd:2160", "t": "Actions", "h": "sec", "w": 260},
	{"f": "right_dock.gd:2495", "t": "Segments", "h": "sec", "w": 260},
	{"f": "right_dock.gd:2557", "t": "Actions", "h": "body", "w": 260},
	{"f": "right_dock.gd:3071", "t": "Actions", "h": "sec", "w": 260},
	{"f": "right_dock.gd:3261", "t": "Fauna (population estimate)", "h": "sec", "w": 260},
	{"f": "right_dock.gd:3338", "t": "History", "h": "body", "w": 260},
	{"f": "right_dock.gd:3352", "t": "Commit", "h": "body", "w": 260},
	{"f": "right_dock.gd:3422", "t": "Actions", "h": "sec", "w": 260},
	{"f": "right_dock.gd:3529", "t": "Legend · painted counts", "h": "sec", "w": 260},
	{"f": "right_dock.gd:3539", "t": "Commit", "h": "sec", "w": 260},
	{"f": "right_dock.gd:3685", "t": "Stops", "h": "sec", "w": 260},
	{"f": "right_dock.gd:3689", "t": "Edit", "h": "sec", "w": 260},
	{"f": "right_dock.gd:3697", "t": "Actions", "h": "body", "w": 260},
	{"f": "vault_window.gd:398", "t": "what this note holds", "h": "body", "w": 380},
	{"f": "vault_window.gd:467", "t": "New note from a template", "h": "body", "w": 380},
	{"f": "vault_window.gd:545", "t": "what %s holds", "h": "sec", "w": 380, "dyn": true},
	{"f": "vault_window.gd:598", "t": "%s — %s", "h": "sec", "w": 380, "dyn": true},
	{"f": "vault_window.gd:640", "t": "what the notes say", "h": "sec", "w": 380},
	{"f": "vault_window.gd:912", "t": "what this note holds", "h": "sec", "w": 380},
	{"f": "vault_window.gd:966", "t": "Map snapshot", "h": "body", "w": 380},
	{"f": "vault_window.gd:1031", "t": "%s (export field group)", "h": "sec", "w": 380, "dyn": true},
	{"f": "vault_window.gd:1310", "t": "Write confirmations", "h": "body", "w": 380},
	{"f": "vault_window.gd:1436", "t": "Missing & orphan notes", "h": "body", "w": 380},
	{"f": "cartography_workspace.gd:394", "t": "Settlements · by class", "h": "sec", "w": 300},
	{"f": "cartography_workspace.gd:1601", "t": "Classes", "h": "sec", "w": 300},
	{"f": "cartography_workspace.gd:1653", "t": "Type", "h": "sec", "w": 300},
	{"f": "cartography_workspace.gd:1940", "t": "Family", "h": "sec", "w": 300},
	{"f": "cartography_workspace.gd:1980", "t": "Placement rules", "h": "sec", "w": 300},
	{"f": "civilization_workspace.gd:1039", "t": "By province count", "h": "sec", "w": 300},
	{"f": "civilization_workspace.gd:1222", "t": "Per faction", "h": "body", "w": 300},
	{"f": "civilization_workspace.gd:1231", "t": "Contested borders", "h": "body", "w": 300},
	{"f": "civilization_workspace.gd:1290", "t": "Largest, by population", "h": "sec", "w": 300},
	{"f": "civilization_workspace.gd:1711", "t": "%d of %d settlements", "h": "sec", "w": 300, "dyn": true},
	{"f": "civilization_workspace.gd:1965", "t": "By faction", "h": "sec", "w": 300},
	{"f": "civilization_workspace.gd:2045", "t": "Territory, food and resources", "h": "sec", "w": 300},
	{"f": "civilization_workspace.gd:2113", "t": "Provinces", "h": "sec", "w": 300},
	{"f": "civilization_workspace.gd:2130", "t": "Continents", "h": "sec", "w": 300},
	{"f": "civilization_workspace.gd:2448", "t": "%d settlements", "h": "sec", "w": 300, "dyn": true},
	{"f": "civilization_workspace.gd:2454", "t": "%d with a population", "h": "sec", "w": 300, "dyn": true},
	{"f": "civilization_workspace.gd:2464", "t": "%d with no population", "h": "sec", "w": 300, "dyn": true},
	{"f": "civilization_workspace.gd:2712", "t": "%d diverged", "h": "sec", "w": 300, "dyn": true},
	{"f": "civilization_workspace.gd:2860", "t": "%d factions", "h": "sec", "w": 300, "dyn": true},
	{"f": "civilization_workspace.gd:3679", "t": "advanced", "h": "sec", "w": 300, "sigil": "+"},
	{"f": "civilization_workspace.gd:3823", "t": "%s (landmark family)", "h": "sec", "w": 300, "dyn": true},
	{"f": "civilization_workspace.gd:4051", "t": "placed by hand", "h": "sec", "w": 300},
	{"f": "civilization_workspace.gd:4967", "t": "By military power", "h": "grp", "w": 300},
	{"f": "civilization_workspace.gd:5001", "t": "Strongest places", "h": "grp", "w": 300},
	{"f": "civilization_workspace.gd:5113", "t": "Standing · field · emergency", "h": "sec", "w": 300},
	{"f": "civilization_workspace.gd:5130", "t": "How long each can stay out", "h": "sec", "w": 300},
	{"f": "civilization_workspace.gd:5150", "t": "What drives it", "h": "sec", "w": 300},
	{"f": "civilization_workspace.gd:5180", "t": "Who the bands are measured against", "h": "sec", "w": 300},
	{"f": "civilization_workspace.gd:5286", "t": "Every pair", "h": "sec", "w": 300},
	{"f": "civilization_workspace.gd:5625", "t": "Simulate collapse / recovery", "h": "body", "w": 300},
	{"f": "infrastructure_workspace.gd:707", "t": "By good", "h": "body", "w": 300},
	{"f": "infrastructure_workspace.gd:731", "t": "Busiest partners", "h": "body", "w": 300},
	{"f": "infrastructure_workspace.gd:767", "t": "Needs nothing can reach", "h": "body", "w": 300},
	{"f": "infrastructure_workspace.gd:785", "t": "Way load", "h": "body", "w": 300},
	{"f": "infrastructure_workspace.gd:894", "t": "Longest, by point count", "h": "sec", "w": 300},
	{"f": "infrastructure_workspace.gd:1019", "t": "Committed this session", "h": "sec", "w": 300},
	{"f": "infrastructure_workspace.gd:1025", "t": "Routes committed this session", "h": "sec", "w": 300},
	{"f": "infrastructure_workspace.gd:1292", "t": "Coastal settlements", "h": "sec", "w": 300},
	{"f": "infrastructure_workspace.gd:1301", "t": "Sea lanes", "h": "sec", "w": 300},
	{"f": "infrastructure_workspace.gd:1397", "t": "Journey Planner", "h": "sec", "w": 300},
	{"f": "render_workspace.gd:536a", "t": "Relief & light", "h": "body", "w": 300},
	{"f": "render_workspace.gd:536b", "t": "The sheet", "h": "body", "w": 300},
	{"f": "render_workspace.gd:536c", "t": "Materials", "h": "body", "w": 300},
	{"f": "render_workspace.gd:536d", "t": "Atmosphere", "h": "body", "w": 300},
	{"f": "render_workspace.gd:1341", "t": "contour interval", "h": "body", "w": 300, "sigil": "+"},
	{"f": "world_workspace.gd:1081", "t": "advanced", "h": "grp", "w": 300, "sigil": "+"},
	{"f": "world_workspace.gd:1104", "t": "advanced", "h": "sec", "w": 300, "sigil": "+"},
	{"f": "world_workspace.gd:1124", "t": "Stream-power carve", "h": "body", "w": 300},
	{"f": "world_workspace.gd:1134", "t": "Droplet hydraulic", "h": "body", "w": 300},
	{"f": "world_workspace.gd:1137a", "t": "Hillslope diffuse", "h": "body", "w": 300},
	{"f": "world_workspace.gd:1137b", "t": "Velocity (momentum)", "h": "body", "w": 300},
	{"f": "world_workspace.gd:1137c", "t": "Glacial", "h": "body", "w": 300},
	{"f": "world_workspace.gd:1137d", "t": "Coastal", "h": "body", "w": 300},
	{"f": "world_workspace.gd:2049", "t": "Commit", "h": "sec", "w": 300},
	{"f": "world_workspace.gd:2089", "t": "Painted lakes", "h": "sec", "w": 300},
	{"f": "world_workspace.gd:2309", "t": "Legend · painted counts", "h": "sec", "w": 300},
	{"f": "world_workspace.gd:2326", "t": "Commit", "h": "sec", "w": 300},
]

var app: Node
var _rig_scroll: ScrollContainer
var _rig_body: VBoxContainer

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## The exact chain a dock header sits in: a `ScrollContainer` with the
## horizontal axis DISABLED (which is what folds a child's minimum into the
## container's own -- `MISTAKES.md`'s disabled-axis trap, and the reason a
## header width becomes a DOCK width at all), a body VBox, and whatever host
## the call site passes. Built once and reused so a measurement cannot inherit
## a previous row's leftovers.
func _build_rig() -> void:
	var panel := PanelContainer.new()
	panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_rig_scroll = ScrollContainer.new()
	_rig_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_rig_body = VBoxContainer.new()
	_rig_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_rig_scroll.add_child(_rig_body)
	panel.add_child(_rig_scroll)
	add_child(panel)

## Dock width `title` demands, through host shape `h`. Returns the whole
## container's combined minimum x -- chrome included -- not a bare label width.
func _demand(title: String, h: String, sigil: String) -> float:
	for c in _rig_body.get_children():
		_rig_body.remove_child(c)
		c.queue_free()
	var host: Control = _rig_body
	if h == "sec":
		host = DccWidgets.section(_rig_body, "S")
	elif h == "grp":
		host = DccWidgets.group(DccWidgets.section(_rig_body, "S"), "G")
	if sigil != "":
		DccWidgets.group(host, title, false, sigil)
	else:
		DccWidgets.group(host, title, false)
	## `queue_free()` above only detaches next frame; the removes are immediate,
	## so one frame is enough for the fresh subtree to resolve.
	await _frames(2)
	return (_rig_scroll.get_parent() as Control).get_combined_minimum_size().x

## Bare header width, the number the row itself quotes (258 px).
func _bare(title: String, sigil: String) -> float:
	var tmp := VBoxContainer.new()
	add_child(tmp)
	if sigil != "":
		DccWidgets.group(tmp, title, false, sigil)
	else:
		DccWidgets.group(tmp, title, false)
	await _frames(2)
	var w := (tmp.get_child(0) as Control).get_combined_minimum_size().x
	tmp.queue_free()
	return w

func _find_group_headers(root: Node, out: Array) -> void:
	var expand: String = DccIcons.SYMBOLS["expand"]
	for c in root.get_children():
		if c is Button:
			var t := String((c as Button).text)
			if t.begins_with(expand + " ") or t.begins_with("+ "):
				out.append(c)
		_find_group_headers(c, out)

func _ready() -> void:
	print("GRP === group() header blast radius ================================")
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	print("GRP density: is_tablet=%s is_laptop=%s  FS_HEADER=%d  w_right_dock=%d w_left_dock=%d"
		% [DccTheme.is_tablet(), DccTheme.is_laptop(), DccTheme.FS_HEADER,
			DccTheme.role_px("w_right_dock"), DccTheme.role_px("w_left_dock")])
	print("GRP dock floors: RIGHT_MIN=%d LEFT_MIN=%d  TABLET=%d"
		% [DccTheme.W_RIGHT_DOCK_MIN, DccTheme.W_LEFT_DOCK_MIN, DccTheme.W_DOCK_TABLET])

	_build_rig()
	await _frames(2)

	# == A. synthetic sweep ====================================================
	var rows: Array = []
	for s in SITES:
		var d: Dictionary = s
		var sig := String(d.get("sigil", ""))
		var title := String(d["t"])
		var bare: float = await _bare(title, sig)
		var demand: float = await _demand(title, String(d["h"]), sig)
		rows.append({"f": d["f"], "t": title, "h": d["h"], "w": int(d["w"]),
			"bare": bare, "demand": demand, "dyn": bool(d.get("dyn", false))})
	rows.sort_custom(func(a, b): return float(a["demand"]) > float(b["demand"]))

	print("GRP --- A. every call site, widest first ---------------------------")
	print("GRP    demand  bare  floor  over?  host  site / title")
	var over := 0
	var over_static := 0
	for r in rows:
		var e: Dictionary = r
		var ov: bool = float(e["demand"]) > float(e["w"])
		if ov:
			over += 1
			if not bool(e["dyn"]):
				over_static += 1
		print("GRP   %7.0f %5.0f %6d  %-5s  %-4s  %s  |  %s" % [
			float(e["demand"]), float(e["bare"]), int(e["w"]),
			"OVER" if ov else "fits", String(e["h"]), String(e["f"]), String(e["t"])])
	print("GRP  sites=%d  over-floor=%d (of which literal-title=%d)" % [rows.size(), over, over_static])

	## Distribution, because a count alone cannot say whether a shared-widget
	## change is the right instrument.
	var buckets := {"<=200": 0, "201-240": 0, "241-260": 0, "261-300": 0, ">300": 0}
	for r2 in rows:
		var dv := float((r2 as Dictionary)["demand"])
		if dv <= 200.0: buckets["<=200"] += 1
		elif dv <= 240.0: buckets["201-240"] += 1
		elif dv <= 260.0: buckets["241-260"] += 1
		elif dv <= 300.0: buckets["261-300"] += 1
		else: buckets[">300"] += 1
	print("GRP  demand distribution: %s" % [buckets])

	# == B. live cross-check on the row's own header ===========================
	var bridge = app.bridge
	var rd = app.right_dock_ctrl
	for seed_v in SEEDS:
		bridge.generate({"seed": seed_v, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
			"archetype": "", "villages": true, "sea_level": 0.45})
		var waited := 0
		while bridge.generating and waited < 3000:
			await get_tree().process_frame
			waited += 1
		await _frames(10)
		if not bridge.has_world:
			print("GRP seed %d: generate FAILED" % seed_v)
			continue
		var gs: Vector2i = bridge.grid_size()
		bridge.route_begin("mixed")
		bridge.route_append_stop(gs.x * 0.20, gs.y * 0.30)
		bridge.route_append_stop(gs.x * 0.55, gs.y * 0.50)
		bridge.route_append_stop(gs.x * 0.82, gs.y * 0.72)
		bridge.route_commit()
		app.select_domain("civilization")
		await _frames(3)
		app.arm_tool("journey")
		await _frames(16)

		var hdrs: Array = []
		_find_group_headers(app.right_dock_body, hdrs)
		var live_max := 0.0
		var live_name := ""
		for hb in hdrs:
			var w := (hb as Control).get_combined_minimum_size().x
			if w > live_max:
				live_max = w
				live_name = String((hb as Button).text)
		var dock_min: float = (app.right_dock as Control).get_combined_minimum_size().x
		print("GRP [seed %d] right dock: headers=%d  widest=%.0f  %s" % [seed_v, hdrs.size(), live_max, live_name])
		print("GRP [seed %d]   right_dock combined min.x=%.0f   (floor %d)" % [seed_v, dock_min, DccTheme.W_RIGHT_DOCK_MIN])
		for hb2 in hdrs:
			var w2 := (hb2 as Control).get_combined_minimum_size().x
			if w2 >= 200.0:
				print("GRP [seed %d]   live header %6.0f  %s" % [seed_v, w2, String((hb2 as Button).text)])

		## Left dock, same world, every workspace section its domain builds.
		for dom in ["world", "cartography", "civilization", "infrastructure", "render"]:
			app.select_domain(dom)
			await _frames(10)
			var lh: Array = []
			_find_group_headers(app.left_dock_body, lh)
			var lmax := 0.0
			var lname := ""
			for hb3 in lh:
				var w3 := (hb3 as Control).get_combined_minimum_size().x
				if w3 > lmax:
					lmax = w3
					lname = String((hb3 as Button).text)
			print("GRP [seed %d] left dock %-15s headers=%2d widest=%6.0f  dock min.x=%6.0f  %s" % [
				seed_v, dom, lh.size(), lmax,
				(app.left_dock as Control).get_combined_minimum_size().x, lname])

	print("GRP === done ======================================================")
	get_tree().quit(0)
