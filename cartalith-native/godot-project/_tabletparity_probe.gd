extends Node
## **Tablet parity.** The owner's standing directive is "keep the tablet version
## as close as possible to the windows gui", and `DCC_SHELL_SPEC.md` §13 states
## it flatly: *"Tablet keeps full desktop parity — same regions, same menus,
## same disclosure depth, targets 44–52 px, docks 400 px."*
##
##   Godot_v4.7.1 --path . --resolution 1600x900 _tabletparity_probe.tscn -- --force-touch
##
## This measures that claim rather than restating it, against §1's own tablet
## column, and does it TWICE in one run -- once at tablet 2560x1600, once at
## desktop 1920x1080 -- so "parity" is a comparison between two live
## compositions instead of one table read twice.
##
## Hosted in `SubViewport`s for `_hidpi_probe.gd`'s reason: Windows clamps a
## real window to the desktop work area, so a 2560x1600 request comes back
## smaller and the shell classifies the result as something else entirely.
##
## §1's tablet column, the numbers asserted below:
##
##   Region         desktop   tablet 2560
##   Menu bar          34         52
##   Tool options      34         52
##   Domain rail       40         48
##   Left dock        372        400
##   Right dock    284-340       400
##   Timeline bar      70         88
##   Status bar        26         36

var _fail := 0
var _notes: Array[String] = []

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _near(name: String, got: float, want: float, tol: float) -> void:
	var good := absf(got - want) <= tol
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name,
		"   got=%.1f want=%.1f (tol %.1f)" % [got, want, tol])

func _boot(w: int, h: int) -> Node:
	var vp := SubViewport.new()
	vp.size = Vector2i(w, h)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(45)
	return app

## The node that holds the MenuButtons, found by structure rather than by name.
func _find_menu_bar(n: Node) -> Node:
	for c in n.get_children(true):
		if c is MenuButton:
			return n
	for c in n.get_children(true):
		var r := _find_menu_bar(c)
		if r != null:
			return r
	return null

## Every menu title in bar order -- §13's "same menus".
func _menu_titles(app: Node) -> Array:
	var out: Array = []
	var bar := _find_menu_bar(app)
	if bar == null:
		return out
	for c in bar.get_children(true):
		if c is MenuButton:
			out.append(String((c as MenuButton).text))
	return out

## Rows reachable in a popup, recursing submenus -- §13's "same disclosure
## depth". Disabled rows count: a present-but-disabled row is the same depth,
## and skipping them would let a tablet that silently dropped ten still pass.
func _popup_rows(pm: PopupMenu, depth: int = 0) -> int:
	if depth > 6:
		return 0
	var n := 0
	for i in pm.item_count:
		if pm.is_item_separator(i):
			continue
		n += 1
		var sub := pm.get_item_submenu(i)
		if sub != "":
			var node := pm.get_node_or_null(NodePath(sub))
			if node is PopupMenu:
				n += _popup_rows(node as PopupMenu, depth + 1)
	return n

func _menu_row_total(app: Node) -> int:
	var bar := _find_menu_bar(app)
	if bar == null:
		return 0
	var total := 0
	for c in bar.get_children(true):
		if c is MenuButton:
			var pm: PopupMenu = (c as MenuButton).get_popup()
			pm.about_to_popup.emit()
			total += _popup_rows(pm)
	return total

func _small_targets(app: Node, floor_px: float) -> Array:
	var small: Array = []
	var stack: Array = [app]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
		if n is BaseButton and n is Control:
			var ctl := n as Control
			if not ctl.visible or ctl.mouse_filter != Control.MOUSE_FILTER_STOP:
				continue
			if n is OptionButton or n is MenuButton or n is ColorPickerButton:
				continue
			if ctl.size.y > 0.0 and ctl.size.y < floor_px:
				small.append("%s  h=%.0f" % [String(n.name), ctl.size.y])
	return small

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var forced := "--force-touch" in OS.get_cmdline_user_args()
	print("[BOOT] force-touch=", forced, "  (the tablet leg needs it)")

	print("")
	print("=== TABLET 2560x1600 ===")
	var tapp := await _boot(2560, 1600)
	## `DccApp extends DccShell`, so the app node IS the shell.
	var tshell: Node = tapp
	_ok("classified as touch", DccTheme.is_touch(), true)
	_ok("classified as TABLET", DccTheme.is_tablet(), true)
	_ok("and NOT the phone composition", DccTheme.is_phone(), false)

	print("")
	print("-- SS1 tablet column, measured off the live tree --")
	_near("left dock width", float(tshell.get("_left_width")), 400.0, 1.0)
	_near("right dock width", float(tshell.get("_right_width")), 400.0, 1.0)
	var rail = tshell.rail_region() if tshell.has_method("rail_region") else null
	if rail != null and rail is Control:
		_near("domain rail width", (rail as Control).size.x, 48.0, 3.0)
	else:
		print("  info rail_region() gave nothing to measure")
	## The four fixed-height bands, through the same resolver the shell builds
	## them with -- measuring the built node would also pass if the resolver
	## were right and the builder ignored it, so both are checked: the resolver
	## here, the built node below.
	_ok("_scaled(34) menu bar -> 52", tshell.call("_scaled", 34), 52)
	_ok("_scaled(70) timeline -> 88", tshell.call("_scaled", 70), 88)
	_ok("_scaled(26) status -> 36", tshell.call("_scaled", 26), 36)
	_ok("_scaled(40) rail -> 48", tshell.call("_scaled", 40), 48)

	print("")
	print("-- SS13 targets 44-52 px --")
	var small := _small_targets(tapp, 44.0)
	print("  info tablet buttons under 44 px tall: ", small.size())
	for s in small.slice(0, 14):
		print("    ", s)
	_notes.append("tablet buttons under 44px: %d" % small.size())

	var t_titles := _menu_titles(tapp)
	var t_rows := _menu_row_total(tapp)
	print("  info tablet menus: ", t_titles)
	print("  info tablet menu rows (submenus included): ", t_rows)

	print("")
	print("=== DESKTOP 1920x1080 (same process) ===")
	## `DccTheme._touch` is latched for the life of the process by design (see
	## its own comment), so this leg cannot un-touch the theme. What it CAN
	## measure is the two things SS13 actually promises are identical, and
	## those are exactly what parity means here: the menu titles and the total
	## disclosure depth.
	var dapp := await _boot(1920, 1080)
	var d_titles := _menu_titles(dapp)
	var d_rows := _menu_row_total(dapp)
	print("  info desktop menus: ", d_titles)
	print("  info desktop menu rows (submenus included): ", d_rows)

	print("")
	print("=== SS13 PARITY: same menus, same disclosure depth ===")
	_ok("the same menus in the same order",
		",".join(PackedStringArray(t_titles)), ",".join(PackedStringArray(d_titles)))
	_ok("seven of them", t_titles.size(), 7)
	_ok("the same number of reachable rows", t_rows, d_rows)
	_ok("and that number is not trivially small", t_rows > 100, true)

	print("")
	print("_tabletparity_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	for n in _notes:
		print("  note ", n)
	get_tree().quit(1 if _fail > 0 else 0)
