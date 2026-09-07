extends Node
## TABLET-SPEC lane, 2026-09-07. Measures the SHIPPED tablet composition's
## chrome so `TABLET_UI_SPEC.md`'s "what exists today" column is measured
## rather than read off a constant table.
##
##   godot --path . _tabspec_probe.tscn -- --force-touch --vp 1600x1000 --tag land
##   godot --path . _tabspec_probe.tscn -- --force-touch --vp 800x1280  --tag port
##
## Reports, per region, the laid-out pixel size against the tablet canvas's own
## figure (`Cartalith Tablet.dc.html`, `valsShell()`). Reads nothing; writes
## nothing; changes no behaviour.

var app: Node
var _vp: SubViewport
var _tag := "tab"

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _w(label: String, node: Node, canvas_want: float) -> void:
	if node == null:
		print("  %-22s ABSENT" % label)
		return
	var c := node as Control
	print("  %-22s w=%7.1f  canvas=%6.1f  vis=%s" % [label, c.size.x, canvas_want, str(c.visible)])

func _h(label: String, node: Node, canvas_want: float) -> void:
	if node == null:
		print("  %-22s ABSENT" % label)
		return
	var c := node as Control
	print("  %-22s h=%7.1f  canvas=%6.1f  vis=%s" % [label, c.size.y, canvas_want, str(c.visible)])

func _ready() -> void:
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
	_tag = _arg("--tag", "tab")
	var vw := int(parts[0])
	var vh := int(parts[1])
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(6)

	var portrait := vh > vw
	print("=== %s vp=%dx%d portrait=%s touch=%s tablet=%s phone=%s ===" % [
		_tag, vw, vh, str(portrait), str(DccTheme.is_touch()),
		str(DccTheme.is_tablet()), str(DccTheme.is_phone())])

	print("[heights]  (canvas: menuH 52, railH 56, sbH 28)")
	_h("menu_bar_row", app.menu_bar_row, 52.0)
	_h("tool_options_row", app.tool_options_row, 56.0)
	_h("timeline_bar", app.timeline_bar, 28.0)
	_h("status_row", app.status_row, 28.0)

	print("[widths]   (canvas: railW 52, ldW/rdW %d)" % (232 if portrait else 320))
	var want_dock := 232.0 if portrait else 320.0
	_w("rail_column", app.rail_column, 52.0)
	_w("left_dock", app.left_dock, want_dock)
	_w("right_dock", app.right_dock, want_dock)
	_w("viewport_area", app.viewport_area, 0.0)

	## **Which child is forcing the left dock past its own role?**
	##
	## Added 2026-09-07. The role resolves to the canvas figure and the RIGHT
	## dock draws it exactly; the left dock draws 331 with a minimum of 331. So
	## the question is not "is the role applied" but "what inside refuses to
	## fit", and a dock-level total cannot answer that -- the same reason
	## `_panemin_probe` walks DOWN rather than arguing with a window total.
	##
	## Prints the chain of descendants whose own combined minimum meets or
	## exceeds the dock's, which is what makes one of them the driver rather
	## than a passenger.
	## The dock's OWN floor, printed beside the role it should have taken.
	## `_build_left_dock()` does `left_dock.custom_minimum_size.x = _left_width`,
	## and `_left_width` is a cached var, not a live role read -- so it can hold
	## a figure from whichever band was current when it was last resolved.
	print("  [floor] _left_width=%s  _right_width=%s  role_left=%s  role_right=%s  ld_cms=%.1f" % [
		str(app.get("_left_width")), str(app.get("_right_width")),
		str(DccTheme.role_px("w_left_dock")), str(DccTheme.role_px("w_right_dock")),
		(app.left_dock as Control).custom_minimum_size.x])
	_drivers("left_dock", app.left_dock, want_dock)
	_drivers("right_dock", app.right_dock, want_dock)

	print("[fit]")
	var shell: Control = app.shell if "shell" in app else null
	if shell == null:
		for ch in app.get_children():
			if ch is Control and ch.get_class() == "Control":
				pass
	var vpa := app.viewport_area as Control
	print("  viewport_area  x=%.1f..%.1f   frame_w=%d" % [
		vpa.global_position.x, vpa.global_position.x + vpa.size.x, vw])
	var rd := app.right_dock as Control
	print("  right_dock     x=%.1f..%.1f   overflow=%.1f" % [
		rd.global_position.x, rd.global_position.x + rd.size.x,
		maxf(0.0, rd.global_position.x + rd.size.x - float(vw))])
	print("  sum rail+ld+vp+rd = %.1f" % [
		(app.rail_column as Control).size.x + (app.left_dock as Control).size.x
		+ vpa.size.x + rd.size.x])
	print("  viewport_area min.x=%.1f  left_dock min.x=%.1f  right_dock min.x=%.1f" % [
		vpa.get_combined_minimum_size().x,
		(app.left_dock as Control).get_combined_minimum_size().x,
		rd.get_combined_minimum_size().x])

	print("[menu titles]")
	var titles: Array[String] = []
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton:
			titles.append((ch as MenuButton).text)
		elif ch is MenuBar:
			for i in (ch as MenuBar).get_menu_count():
				titles.append((ch as MenuBar).get_menu_title(i))
	print("  count=%d  %s" % [titles.size(), ", ".join(titles)])
	print("  canvas draws 3 (File, World, Data) + one overflow square")

	print("[role table, live]")
	print("  h_menu_bar=%s h_status=%s w_rail=%s w_left_dock=%s w_right_dock=%s" % [
		str(DccTheme.role_px("h_menu_bar")), str(DccTheme.role_px("h_status")),
		str(DccTheme.role_px("w_rail")), str(DccTheme.role_px("w_left_dock")),
		str(DccTheme.role_px("w_right_dock"))])
	print("  TOUCH_SCALE=%.2f" % DccTheme.TOUCH_SCALE)
	print("=== end %s ===" % _tag)
	get_tree().quit()

## Walk `node`, printing every descendant whose combined minimum width is at
## least the container's own -- the ones that could be setting it. A leaf with
## a minimum below the dock's cannot be the cause, so listing everything would
## bury the answer.
##
## Prints the class and the text where a control has one, because "which
## Label" is not an answer a person can act on and "the Label reading
## 'Established Caravan Route'" is.
static func _drivers(tag: String, root: Node, want: float) -> void:
	if root == null:
		return
	var have: float = (root as Control).get_combined_minimum_size().x
	print("[drivers] %s min=%.1f want=%.1f over=%.1f" % [tag, have, want, have - want])
	if have <= want + 0.5:
		print("  (within its role -- nothing to blame)")
		return
	## Threshold on WANT, not on `have`: the container is already over, so
	## filtering at its own minimum hides every contributor below it.
	_drivers_walk(root, want, 0)

## **Walk only the CONTRIBUTING subtree.** A container's minimum counts a child
## by that child's own `visible`, so a hidden child contributes nothing and
## neither does anything beneath it -- descending into one prints nodes that
## cannot be the cause, which is how a wrong answer was filed twice on
## 2026-09-07. Threshold on the TARGET width, not on the container's current
## minimum: filtering at the symptom hides every contributor smaller than it.
static func _drivers_walk(node: Node, floor_w: float, depth: int) -> void:
	for child in node.get_children():
		if not (child is Control):
			_drivers_walk(child, floor_w, depth)
			continue
		var ctl := child as Control
		if not ctl.visible:
			continue   ## contributes nothing, and nor does its subtree
		var m: float = ctl.get_combined_minimum_size().x
		if m > floor_w + 0.5:
			var txt := ""
			if "text" in ctl:
				txt = str(ctl.get("text")).strip_edges()
			var extra := ""
			if ctl is ScrollContainer:
				extra = "  [h_scroll=%d ← DISABLED folds the child's min outward]" % (ctl as ScrollContainer).horizontal_scroll_mode
			elif ctl is OptionButton:
				extra = "  [fit_to_longest_item=%s]" % str((ctl as OptionButton).fit_to_longest_item)
			## Name the row, not just the container. "An `HBoxContainer` is 290 px"
			## is not something a person can act on; "the row reading Sea level is
			## 290 px" is. Container classes carry no text, so pull the first text
			## from a descendant — which is what a reader would call the row.
			if txt == "":
				txt = _first_text(ctl)
			print("  %s%s  min=%.1f  %s%s" % [
				"  ".repeat(depth), ctl.get_class(), m,
				("\"" + txt.left(44) + "\"") if txt != "" else ctl.name, extra])
		_drivers_walk(ctl, floor_w, depth + 1)

## The first non-empty text under `node`, breadth-ish, for naming a row.
static func _first_text(node: Node) -> String:
	for child in node.get_children():
		if child is Control and "text" in child:
			var t := str(child.get("text")).strip_edges()
			if t != "":
				return t
	for child in node.get_children():
		var deeper := _first_text(child)
		if deeper != "":
			return deeper
	return ""
