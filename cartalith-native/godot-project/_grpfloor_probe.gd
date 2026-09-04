extends Node
## Lane B / batch 25, part 2 -- the DECIDING measurement: for every surface a
## `DccWidgets.group()` header can appear in, drag the dock to its documented
## floor and read the width it actually stops at, then name the node that held
## it there.
##
## Part 1 (`_grphdr_probe.gd`) swept the source strings through a *replica*
## chain. The live cross-check in that same probe showed the replica's chrome
## was 26 px fatter than the real dock's (a default-themed `PanelContainer`),
## which is exactly why the cross-check exists -- so nothing here models the
## chain. Every number below comes off the real `left_dock` / `right_dock`.
##
## It also found the thing a source sweep structurally cannot: the widest group
## header in the shell is not a literal at all. `civilization_workspace.gd`'s
## `_lm_refresh_group()` REWRITES the header text after `group()` built it,
## appending `"   %d of %d armed · %d placed"`, so the source string
## (`_lm_pretty(f)`, e.g. "historical") measures 161 px and the shipped header
## measures 280.
##
## The walk is every `RAIL_NODES` domain+mode pair (11 of them -- "render" and
## "infrastructure" are NOT domains, which a first pass got wrong and the
## body_min=0 rows exposed), and inside each, every L2 category in turn.
## Categories are an accordion (`DccWidgets._toggle_category` closes the
## siblings), so they have to be visited one at a time, not all opened at once.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _grpfloor_probe.tscn

const SEEDS := [483920, 77021, 4242]

## Every domain+mode pair `dcc_shell.gd::RAIL_NODES` defines.
const NODES := [
	["world", "a"], ["world", "b"],
	["civilization", "landmarks"], ["civilization", "factions"],
	["civilization", "infra"], ["civilization", "planner"],
	["cartography", "style"], ["cartography", "labels"],
	["cartography", "icons"], ["cartography", "terrain"],
]

var app: Node
var _worst: Array = []       ## Every group header that is the binding node somewhere.
var _seen: Dictionary = {}   ## header text -> widest observed min.x

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## A `DccWidgets.group()` header, and NOT merely a button whose label starts
## with the same glyph. This predicate took two corrections, both caught by
## widening the sweep rather than by reading it:
##
##   1. `begins_with(sigil + " ")` alone counted `+ Add faction` -- a chip in
##      an `HBoxContainer` -- as a group header 16 times per seed, which would
##      have shipped a false claim about no call site handing this factory a
##      distributing parent.
##   2. Adding "the text after the sigil is upper-case" fixed that and broke
##      the headers that matter most: `civilization_workspace.gd`'s
##      `_lm_refresh_group()` appends `"   %d of %d armed · %d placed"` in
##      LOWER case, so the widest header in the shell (280 px) was silently
##      excluded from every count.
##
## So the test is STRUCTURAL, off what `group()` actually builds: a flat,
## unfocusable `Button` at the group header's own font size, followed
## immediately by the `MarginContainer` holding its body `VBoxContainer`.
## Nothing else in the shell has that shape, and no amount of runtime text
## rewriting can move it.
func _is_group_header(c: Node) -> bool:
	if not (c is Button):
		return false
	var b := c as Button
	if not b.flat or b.focus_mode != Control.FOCUS_NONE:
		return false
	var t := String(b.text)
	if not (t.begins_with(DccIcons.SYMBOLS["expand"] + " ") or t.begins_with("+ ")):
		return false
	var want := DccTheme.role_px("fs_dock_header") if DccTheme.is_tablet() else DccTheme.FS_HEADER
	if b.get_theme_font_size("font_size") != want:
		return false
	var p := b.get_parent()
	if p == null or b.get_index() + 1 >= p.get_child_count():
		return false
	var pad := p.get_child(b.get_index() + 1)
	if not (pad is MarginContainer) or pad.get_child_count() == 0:
		return false
	return pad.get_child(0) is VBoxContainer

func _is_category(c: Node) -> bool:
	if not (c is Button):
		return false
	var t := String((c as Button).text)
	return t.begins_with(DccIcons.SYMBOLS["caret"] + "  ") or t.begins_with(DccIcons.SYMBOLS["submenu"] + "  ")

## Only what is on screen. A hidden `Control` reports a combined minimum and
## contributes none of it to its parent, so counting hidden headers would
## inflate every figure below -- and the left dock keeps every workspace panel
## built, with all but one hidden.
func _walk(root: Node, out: Array, want_group: bool) -> void:
	for c in root.get_children():
		if c is Control and not (c as Control).visible:
			continue
		if want_group and _is_group_header(c):
			out.append(c)
		elif not want_group and _is_category(c):
			out.append(c)
		_walk(c, out, want_group)

func _headers(root: Node) -> Array:
	var out: Array = []
	_walk(root, out, true)
	return out

func _categories(root: Node) -> Array:
	var out: Array = []
	_walk(root, out, false)
	return out

## Deepest single node accountable for `root`'s combined minimum x, and the
## chain that carried it up. A width-distributing container is named as such --
## its blame is the SUM of its children, and descending into one child of it
## understates the number (`_jpwidth_probe.gd`'s own lesson).
func _blame(root: Control) -> Array:
	var chain: Array = []
	var n: Control = root
	var guard := 0
	while n != null and guard < 40:
		guard += 1
		var distributes := ((n is BoxContainer) and not (n as BoxContainer).vertical) 			or (n is HFlowContainer) or (n is GridContainer)
		var sum := 0.0
		var next: Control = null
		var best := -1.0
		for c in n.get_children():
			if not (c is Control) or not (c as Control).visible:
				continue
			var w := (c as Control).get_combined_minimum_size().x
			sum += w
			if w > best:
				best = w
				next = c
		chain.append("%s%s=%.0f%s" % [n.get_class(), _tx(n), n.get_combined_minimum_size().x,
			("  [distributes across %d, sum=%.0f]" % [n.get_child_count(), sum]) if distributes and n.get_child_count() > 1 else ""])
		if next == null:
			break
		n = next
	return chain

## The single node the chain bottoms out on, so "which node held the dock open"
## has an answer rather than an impression.
func _blame_leaf(root: Control) -> Control:
	var n: Control = root
	var guard := 0
	while n != null and guard < 40:
		guard += 1
		var next: Control = null
		var best := -1.0
		for c in n.get_children():
			if not (c is Control) or not (c as Control).visible:
				continue
			var w := (c as Control).get_combined_minimum_size().x
			if w > best:
				best = w
				next = c
		if next == null:
			return n
		n = next
	return n

func _tx(n: Node) -> String:
	if n is Button:
		return "[%s]" % (n as Button).text
	if n is Label:
		return "[%s]" % (n as Label).text
	return ""

## Drag `dock` to `floor_px` the way `_on_dock_drag_input()` does -- by writing
## `custom_minimum_size.x` -- then read the width it actually settled at.
func _drag_to(dock: Control, floor_px: int) -> float:
	var was := dock.custom_minimum_size.x
	dock.custom_minimum_size.x = float(floor_px)
	await _frames(4)
	var got := dock.size.x
	dock.custom_minimum_size.x = was
	await _frames(2)
	return got

func _report(tag: String, dock: Control, body: Control, floor_px: int) -> void:
	var hdrs: Array = _headers(body)
	var widest := 0.0
	var widest_t := ""
	for h in hdrs:
		var w := (h as Control).get_combined_minimum_size().x
		if w > widest:
			widest = w
			widest_t = String((h as Button).text)
		var key := String((h as Button).text)
		if w > float(_seen.get(key, 0.0)):
			_seen[key] = w
	var content := body.get_combined_minimum_size().x
	var stops: float = await _drag_to(dock, floor_px)
	var leaf := _blame_leaf(dock)
	var leaf_is_group := _is_group_header(leaf)
	if leaf_is_group:
		_worst.append({"tag": tag, "t": String((leaf as Button).text),
			"w": leaf.get_combined_minimum_size().x, "stops": stops, "floor": floor_px})
	print("GF %-40s floor=%d stops=%.0f over=%+.0f body=%.0f hdrs=%2d widest_hdr=%6.0f binder=%s%s | %s"
		% [tag, floor_px, stops, stops - float(floor_px), content, hdrs.size(), widest,
			leaf.get_class(), "  <-- A GROUP HEADER" if leaf_is_group else "", _tx(leaf)])
	if stops > float(floor_px) + 0.5 and leaf_is_group:
		for line in _blame(dock):
			print("GF     | %s" % line)

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	print("GF density is_tablet=%s is_laptop=%s  right floor=%d  left floor=%d"
		% [DccTheme.is_tablet(), DccTheme.is_laptop(),
			DccTheme.W_RIGHT_DOCK_MIN, DccTheme.W_LEFT_DOCK_MIN])

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
			print("GF seed %d: generate FAILED" % seed_v)
			continue
		print("GF ================= seed %d =================" % seed_v)

		# -- left dock: every rail node, and inside it every L2 category ------
		for pair in NODES:
			app.select_domain_mode(String(pair[0]), String(pair[1]))
			await _frames(14)
			var tag0 := "LEFT %s/%s" % [pair[0], pair[1]]
			await _report(tag0 + " (default)", app.left_dock, app.left_dock_body,
				DccTheme.W_LEFT_DOCK_MIN)
			var cats: Array = _categories(app.left_dock_body)
			for ci in cats.size():
				var cb: Button = cats[ci]
				if not is_instance_valid(cb):
					continue
				var label := String(cb.text).substr(2).strip_edges()
				cb.emit_signal("pressed")
				await _frames(10)
				await _report("%s > %s" % [tag0, label], app.left_dock,
					app.left_dock_body, DccTheme.W_LEFT_DOCK_MIN)

		# -- right dock: selection, then Journey armed on top -----------------
		var gs: Vector2i = bridge.grid_size()
		bridge.route_begin("mixed")
		bridge.route_append_stop(gs.x * 0.20, gs.y * 0.30)
		bridge.route_append_stop(gs.x * 0.55, gs.y * 0.50)
		bridge.route_append_stop(gs.x * 0.82, gs.y * 0.72)
		bridge.route_commit()
		app.select_domain("civilization")
		await _frames(4)
		var settlements: Array = bridge.settlements()
		if not settlements.is_empty():
			rd.on_settlement_selected(settlements[0], 0)
		await _frames(8)
		await _report("RIGHT settlement", app.right_dock, app.right_dock_body,
			DccTheme.W_RIGHT_DOCK_MIN)
		app.arm_tool("journey")
		await _frames(16)
		await _report("RIGHT settlement+journey", app.right_dock, app.right_dock_body,
			DccTheme.W_RIGHT_DOCK_MIN)
		for b in _headers(app.right_dock_body):
			(b as Button).emit_signal("pressed")
		await _frames(10)
		await _report("RIGHT journey, groups toggled", app.right_dock, app.right_dock_body,
			DccTheme.W_RIGHT_DOCK_MIN)
		## Every arm of `RightDock._tool_section()`, each in the domain that arm
		## requires -- `paint` answers `TOOL_PAINT` only in WORLD, `inspect`
		## answers `TOOL_STOPS` only in CARTO, and the sculpt clause is
		## world-gated. Enumerated from that function's own `match`, not from
		## the tool ids that happen to appear elsewhere in the tree.
		for combo in [["world", "paint"], ["cartography", "inspect"],
				["civilization", "label"], ["civilization", "icon"],
				["civilization", "territory"], ["world", "sculpt"],
				["world", "way"], ["world", "route"], ["civilization", "measure"],
				["world", "inspect"]]:
			app.select_domain(String(combo[0]))
			await _frames(4)
			app.arm_tool(String(combo[1]))
			await _frames(12)
			await _report("RIGHT %s/%s  sect=%s" % [combo[0], combo[1], rd._tool_section()],
				app.right_dock, app.right_dock_body, DccTheme.W_RIGHT_DOCK_MIN)

	print("GF --- every group header seen, widest first ---")
	var ks: Array = _seen.keys()
	ks.sort_custom(func(a, b): return float(_seen[a]) > float(_seen[b]))
	for k in ks:
		print("GF   %6.0f  %s" % [float(_seen[k]), String(k)])
	print("GF --- group headers that BOUND a dock above its floor: %d ---" % _worst.size())
	for w in _worst:
		var d: Dictionary = w
		print("GF   %s  stops=%.0f floor=%d  %s" % [d["tag"], d["stops"], d["floor"], d["t"]])
	print("GF === done ===")
	get_tree().quit(0)
