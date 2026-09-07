extends Node
## **Lane PC-REACH (2026-09-07).** Converts the previous diff lane's "NOT
## reached" regions into measurements: the left dock row by row for all ten
## rail nodes, the right dock row by row for every context it can be driven
## into, and the exact node chain that forces the left dock to 404 px under
## CIVIL > Journey planner (`_ds03fit_probe`'s two standing FAILs).
##
## Unlike `_menuconf_probe._dump_tree()` this walk has **no depth cap** -- that
## probe's `max_depth = 7` is why the dock bodies were never actually read: a
## stage's field rows sit at depth 9-12 and were silently absent from a dump
## that looked complete.
##
##   Godot_v4.7.1 --headless --path . _pcreach_probe.tscn -- --vp 1920x1080
##
## Read-only: instantiates `res://shell/app.tscn`, drives its own public
## methods, writes nothing but a log to stdout.

const SEED := 483920

var app: Node
var _vp: SubViewport
var _cap := 372.0


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _hex(c: Color) -> String:
	return "#%02x%02x%02x/%.2f" % [int(round(c.r * 255)), int(round(c.g * 255)),
		int(round(c.b * 255)), c.a]


func _line(c: Control, d: int) -> String:
	var t := ""
	var v: Variant = c.get("text")
	if v != null:
		t = String(v)
	var extra := ""
	if c is Label or c is Button or c is LineEdit:
		var f: Font = c.get_theme_font("font")
		var fam := "mono" if (f != null and f is FontVariation) else "sans"
		extra = " {%s/%d %s}" % [fam, c.get_theme_font_size("font_size"),
			_hex(c.get_theme_color("font_color"))]
	var mn := c.get_combined_minimum_size()
	return "%s%s '%s' %.0fx%.0f min=%.0fx%.0f%s" % ["  ".repeat(d), c.get_class(),
		t.substr(0, 52).replace("\n", "\n"), c.size.x, c.size.y, mn.x, mn.y, extra]


func _walk(n: Node, d: int, visible_only: bool = true) -> void:
	if n is Control:
		var c := n as Control
		if (not visible_only) or c.is_visible_in_tree():
			print(_line(c, d))
	for ch in n.get_children():
		_walk(ch, d + 1, visible_only)


## Every node on the path from `root` down whose own combined minimum width
## exceeds `cap`, printed as a chain -- the fix lane wants the *deepest* one.
func _over(n: Node, d: int, cap: float, path: String) -> void:
	if n is Control:
		var c := n as Control
		var mn := c.get_combined_minimum_size().x
		if mn > cap and c.is_visible_in_tree():
			var t: Variant = c.get("text")
			print("  OVER %5.0f  %s%s '%s'" % [mn, "  ".repeat(d), c.get_class(),
				String(t).substr(0, 44) if t != null else ""])
			path = path
	for ch in n.get_children():
		_over(ch, d + 1, cap, path)


func _sections(root: Node) -> Array:
	## Every `§ ` section header the dock body draws, in order.
	var out: Array = []
	var stack: Array = [root]
	var order: Array = []
	_collect(root, order)
	for c in order:
		var v: Variant = c.get("text")
		if v != null and String(v).begins_with("§ "):
			out.append(String(v).substr(2))
	return out


func _collect(n: Node, out: Array) -> void:
	if n is Control and (n as Control).is_visible_in_tree():
		out.append(n)
	for ch in n.get_children():
		_collect(ch, out)


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var w := 1920
	var h := 1080
	var i := args.find("--vp")
	if i >= 0 and i + 1 < args.size():
		var p: PackedStringArray = String(args[i + 1]).split("x")
		w = int(p[0]); h = int(p[1])

	_vp = SubViewport.new()
	_vp.size = Vector2i(w, h)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(30)
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(10)

	print("[MODE] vp=%s touch=%s tablet=%s phone=%s theme=%s docks=%s/%s" % [
		_vp.size, DccTheme.is_touch(), DccTheme.is_tablet(), DccTheme.is_phone(),
		("light" if DccTheme.c("bg").r > 0.5 else "dark"),
		app.get("_left_width"), app.get("_right_width")])
	_cap = float(app.get("_left_width"))

	var bridge = app.bridge
	bridge.generate({"seed": SEED, "width_km": 1200.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	print("[WORLD] generated")

	# --- 1. the left dock, row by row, every rail node ----------------------
	for n in DccShell.RAIL_NODES:
		if String(n.get("kind", "")) != "node":
			continue
		var id: String = String(n["domain"]) + "/" + String(n["mode"])
		app.call("_on_rail_node_pressed", String(n["domain"]), String(n["mode"]))
		await _frames(14)
		var ld := app.get("left_dock") as Control
		print("")
		print("##### LEFT %s  dock=%.0f cap=%.0f  header=%s" % [id, ld.size.x,
			_cap, "OVER" if ld.size.x > _cap + 0.5 else "fit"])
		print("  SECTIONS: %s" % [", ".join(_sections(app.get("left_dock_body")))])
		if ld.size.x > _cap + 0.5:
			print("  --- chain of over-wide nodes (min_x > %.0f) ---" % _cap)
			_over(app.get("left_dock_body"), 0, _cap, "")
		print("  --- full tree ---")
		_walk(ld, 1)

	# --- 2. the right dock, every context ----------------------------------
	var rd = app.get("right_dock_ctrl")
	var ctxs: Array = [
		["sample", func(): rd.show_sample_hint() if rd.has_method("show_sample_hint") else null],
		["measure", func(): rd.show_measure({"length_km": 12.0}, "distance")],
		["region", func(): rd.show_region({"x0": 1, "y0": 1, "x1": 40, "y1": 30})],
		["history", func(): rd.show_history()],
		["stamps", func(): rd.show_sculpt_stack()],
		["paint", func(): rd.show_paint("biome")],
		["stops", func(): rd.show_stops()],
		["anno", func(): rd.show_anno()],
		["territory", func(): rd.show_territory(0)],
		["faction", func(): rd.show_faction(0)],
	]
	for e in ctxs:
		var nm: String = e[0]
		var cb: Callable = e[1]
		cb.call()
		await _frames(14)
		var rdc := app.get("right_dock") as Control
		print("")
		print("##### RIGHT %s  dock=%.0f" % [nm, rdc.size.x])
		print("  SECTIONS: %s" % [", ".join(_sections(app.get("right_dock_body")))])
		print("  --- full tree ---")
		_walk(rdc, 1)

	print("### DONE ###")
	get_tree().quit()
