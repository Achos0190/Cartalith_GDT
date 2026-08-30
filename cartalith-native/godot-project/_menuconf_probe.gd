extends Node
## THROWAWAY menu-conformance sweep (2026-08-25). Not to be committed.
##
## §48 walked *screens*. This walks **menus** -- every program menu, every
## submenu, the rail's three domain menus, the right dock's contexts, the tool
## options rows, the Layers popover, and on the phone the drawer / panel picker
## / overflow / tool sheet -- and dumps, per menu, every row's text, state and
## the popup's own measured geometry, beside a screenshot.
##
## Hosted in a `SubViewport` for §47/§48's reason: `--resolution WxH` is clamped
## to the dev monitor's work area (1680x1002) and boots the shell into tablet
## mode. `gui_embed_subwindows = true` also puts every `PopupMenu` -- which is a
## `Window`, not a `Control` -- inside the captured texture.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _menuconf_probe.tscn -- \
##       --vp 1600x900 --tag desktop
##   ... -- --vp 2560x1600 --tag tablet --force-touch
##   ... -- --vp 1440x3168 --tag phone1440 --force-touch
##   ... -- --vp 1080x2400 --tag phone1080 --force-touch

const SEED := 483920

var app: Node
var _vp: SubViewport
var _tag := "run"
var _dir := ""
var _log: Array = []


func _l(s: String) -> void:
	print(s)
	_log.append(s)


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _find(n: Node, pred: Callable) -> Node:
	if pred.call(n):
		return n
	for c in n.get_children():
		var h := _find(c, pred)
		if h != null:
			return h
	return null


func _shot(name: String) -> void:
	await _frames(4)
	var img := _vp.get_texture().get_image()
	var out := "%s/%s__%s.png" % [_dir, _tag, name]
	img.save_png(out)
	_l("  shot -> %s" % out.get_file())


func _hex(c: Color) -> String:
	return "#%02x%02x%02x/%.2f" % [int(c.r * 255), int(c.g * 255), int(c.b * 255), c.a]


# ---------------------------------------------------------------------------
# PopupMenu dump
# ---------------------------------------------------------------------------

func _popup_theme(p: PopupMenu) -> String:
	var panel: StyleBox = p.get_theme_stylebox("panel")
	var bg := "?"
	if panel is StyleBoxFlat:
		bg = _hex((panel as StyleBoxFlat).bg_color) + " border=" \
			+ _hex((panel as StyleBoxFlat).border_color)
	var sepfs: int = p.get_theme_font_size("font_separator_size")
	var sepf: Font = p.get_theme_font("font_separator")
	var sepfam := "sans"
	if sepf != null and sepf is FontVariation:
		sepfam = "mono"
	var hov: StyleBox = p.get_theme_stylebox("hover")
	var hovs := "?"
	if hov is StyleBoxFlat:
		hovs = _hex((hov as StyleBoxFlat).bg_color)
	return ("panel=%s fs=%d vsep=%d hsep=%d startpad=%d endpad=%d indent=%d "
		+ "sep{%s %d %s} hover=%s ink=%s dim=%s accel=%s") % [
		bg, p.get_theme_font_size("font_size"),
		p.get_theme_constant("v_separation"), p.get_theme_constant("h_separation"),
		p.get_theme_constant("item_start_padding"), p.get_theme_constant("item_end_padding"),
		p.get_theme_constant("indent"),
		sepfam, sepfs, _hex(p.get_theme_color("font_separator_color")),
		hovs, _hex(p.get_theme_color("font_color")),
		_hex(p.get_theme_color("font_disabled_color")),
		_hex(p.get_theme_color("font_accelerator_color"))]


func _dump_popup(p: PopupMenu, path: String, depth: int) -> Array:
	## Returns the list of (submenu popup, path) discovered, so the caller can
	## recurse without re-walking.
	var subs: Array = []
	var pad := "  ".repeat(depth)
	_l("%sMENU %s  items=%d  size=%s" % [pad, path, p.item_count, p.size])
	_l("%s  THEME %s" % [pad, _popup_theme(p)])
	for i in p.item_count:
		var bits: Array = []
		if p.is_item_separator(i):
			var t := p.get_item_text(i)
			_l("%s  [%02d] ---- SEPARATOR %s" % [pad, i, ("'" + t + "'") if t != "" else "(unlabelled)"])
			continue
		if p.is_item_disabled(i):
			bits.append("disabled")
		if p.is_item_checkable(i):
			bits.append("check" + ("=on" if p.is_item_checked(i) else "=off"))
		if p.is_item_radio_checkable(i):
			bits.append("radio" + ("=on" if p.is_item_checked(i) else "=off"))
		var sm := p.get_item_submenu(i)
		if sm != "":
			bits.append("submenu:" + sm)
			var node := p.get_node_or_null(NodePath(sm))
			if node is PopupMenu:
				subs.append([node, path + " > " + p.get_item_text(i)])
		var ac: int = p.get_item_accelerator(i)
		if ac != 0:
			bits.append("accel=" + OS.get_keycode_string(ac))
		if p.get_item_icon(i) != null:
			bits.append("icon")
		_l("%s  [%02d] %s%s" % [pad, i, p.get_item_text(i),
			("   {" + ", ".join(bits) + "}") if not bits.is_empty() else ""])
	return subs


func _walk_menu(mb: MenuButton) -> void:
	var p := mb.get_popup()
	if p.about_to_popup.get_connections().size() > 0:
		p.about_to_popup.emit()
	mb.show_popup()
	await _frames(6)
	var subs := _dump_popup(p, mb.text, 0)
	await _shot("menu_%s" % mb.text.to_lower())
	var queue: Array = subs.duplicate()
	var depth := 1
	while not queue.is_empty():
		var next: Array = []
		for entry in queue:
			var sp: PopupMenu = entry[0]
			var label: String = entry[1]
			if sp.about_to_popup.get_connections().size() > 0:
				sp.about_to_popup.emit()
			## `popup(rect)` clamps to the desktop usable rect; assign size after.
			sp.position = Vector2i(mini(420, _vp.size.x / 3), 120)
			sp.popup()
			await _frames(6)
			next.append_array(_dump_popup(sp, label, depth))
			var slug := label.to_lower().replace(" > ", "-").replace(" ", "_").replace("…", "").replace("▸", "")
			await _shot("submenu_%s" % slug)
			sp.hide()
			await _frames(2)
		queue = next
		depth += 1
	p.hide()
	await _frames(2)


# ---------------------------------------------------------------------------
# Control-tree dump (rail, docks, tool bar, popover, phone sheets)
# ---------------------------------------------------------------------------

func _ctl_line(c: Control, depth: int) -> String:
	var t := ""
	if c.get("text") != null:
		t = String(c.get("text"))
	var extra := ""
	if c is Label:
		var f := (c as Label).get_theme_font("font")
		var fam := "mono" if (f != null and f is FontVariation) else "sans"
		extra = " {%s/%d %s}" % [fam, (c as Label).get_theme_font_size("font_size"),
			_hex((c as Label).get_theme_color("font_color"))]
	elif c is Button:
		var f2 := (c as Button).get_theme_font("font")
		var fam2 := "mono" if (f2 != null and f2 is FontVariation) else "sans"
		extra = " {%s/%d}" % [fam2, (c as Button).get_theme_font_size("font_size")]
	return "%s%s '%s' %.0fx%.0f%s" % ["  ".repeat(depth), c.get_class(),
		t.substr(0, 44).replace("\n", "\\n"), c.size.x, c.size.y, extra]


func _dump_tree(root: Node, tag: String, max_depth: int = 7) -> void:
	_l("TREE %s" % tag)
	var stack: Array = [[root, 0]]
	var out: Array = []
	while not stack.is_empty():
		var e: Array = stack.pop_back()
		var n: Node = e[0]
		var d: int = e[1]
		if n is Control and (n as Control).is_visible_in_tree():
			out.append([n, d])
		if d < max_depth:
			var ch := n.get_children()
			ch.reverse()
			for c in ch:
				stack.append([c, d + 1])
	for e in out:
		_l("  " + _ctl_line(e[0], e[1]))


func _screen(name: String, setup: Callable, dump_root: Callable = Callable()) -> void:
	_l("=== %s / %s ===" % [_tag, name])
	await setup.call()
	await _frames(8)
	if dump_root.is_valid():
		var r = dump_root.call()
		if r != null:
			_dump_tree(r, name)
	await _shot(name)


func _ctx_map_setup() -> void:
	await _close()
	app.select_domain("civilization")
	await _frames(6)
	var ws: Node = _find(app, func(n: Node) -> bool:
		var sc: Variant = n.get_script()
		return sc != null and String(sc.resource_path).ends_with("civilization_workspace.gd"))
	if ws != null and ws.has_method("on_map_right_clicked"):
		ws.on_map_right_clicked(120.0, 90.0, -1, Vector2(400, 400))
		await _frames(6)
		var cm = ws.get("_ctx_menu")
		if cm is PopupMenu:
			_dump_popup(cm as PopupMenu, "Map right-click (CIVIL)", 0)


func _close() -> void:
	if app.is_phone():
		app._set_overflow_open(false)
		app._set_drawer_open(false)
		app._set_panel_picker_open(false)
	await _frames(2)


# ---------------------------------------------------------------------------

func _sweep() -> void:
	_l("### %s vp=%s phone=%s tablet=%s scale=%.3f ###" %
		[_tag, _vp.size, app.is_phone(), app.get("_touch"), app.phone_scale()])

	# -- 1. the menu bar itself -------------------------------------------
	var mbr = app.get("menu_bar_row")
	if mbr != null:
		_l("MENUBAR height=%.1f  children=%d" % [mbr.size.y, mbr.get_child_count()])
		for c in mbr.get_children():
			if c is Control:
				_l("  " + _ctl_line(c, 1) + " rect=%s" % [(c as Control).get_rect()])

	# -- 2. every program menu and every submenu --------------------------
	if not app.is_phone() and mbr != null:
		for c in mbr.get_children():
			if c is MenuButton:
				await _walk_menu(c as MenuButton)
	elif mbr != null:
		## Phone: the same PopupMenu objects, dumped without showing them, then
		## the phone's own re-presentation captured separately below.
		for c in mbr.get_children():
			if c is MenuButton:
				var p := (c as MenuButton).get_popup()
				if p.about_to_popup.get_connections().size() > 0:
					p.about_to_popup.emit()
				var subs := _dump_popup(p, c.text, 0)
				while not subs.is_empty():
					var nx: Array = []
					for e in subs:
						nx.append_array(_dump_popup(e[0], e[1], 1))
					subs = nx

	# -- 3. the rail's three domain menus ---------------------------------
	for dom in ["world", "civilization", "cartography"]:
		await _screen("rail_%s" % dom, func():
			await _close()
			app.select_domain(dom)
			if app.is_phone():
				app._set_sheet_open("left", true),
			func(): return app.get("left_dock"))

	# -- 4. the rail column itself ----------------------------------------
	var rail = app.get("rail_column")
	if rail != null:
		_dump_tree(rail, "rail_column")

	# -- 5. tool options rows ---------------------------------------------
	for dom in ["world", "civilization", "cartography"]:
		await _screen("toolopts_%s" % dom, func():
			await _close()
			app.arm_tool("inspect")
			app.select_domain(dom),
			func(): return app.get("tool_options_row"))
	for tool in ["measure", "paint", "sculpt"]:
		await _screen("toolopts_%s" % tool, func():
			await _close()
			app.select_domain("world")
			app.arm_tool(tool),
			func(): return app.get("tool_options_row"))

	# -- 5b. an OptionButton's own popup -----------------------------------
	await _screen("dropdown_paint_class", func():
		await _close()
		app.select_domain("world")
		app.arm_tool("paint")
		await _frames(6)
		var ob: Node = _find(app.get("tool_options_row"), func(n: Node) -> bool:
			return n is OptionButton)
		if ob is OptionButton:
			var pp := (ob as OptionButton).get_popup()
			_dump_popup(pp, "OptionButton (PAINT · Class)", 0)
			pp.position = Vector2i(200, 140)
			pp.popup())

	# -- 6. layers popover -------------------------------------------------
	await _screen("layers_popover", func():
		await _close()
		app.arm_tool("inspect")
		app.layers_popover.open(),
		func(): return app.layers_popover)

	# -- 7. right dock contexts -------------------------------------------
	var rd = app.get("right_dock_ctrl")
	await _screen("rdock_sample", func():
		await _close()
		app.layers_popover.hide()
		app.select_domain("world"),
		func(): return app.get("right_dock"))
	if rd != null:
		await _screen("rdock_history", func(): rd.show_history(),
			func(): return app.get("right_dock"))
		await _screen("rdock_sculpt", func(): rd.show_sculpt_stack(),
			func(): return app.get("right_dock"))
		await _screen("rdock_faction", func(): rd.show_faction(0),
			func(): return app.get("right_dock"))

	# -- 8. the map right-click context menu ------------------------------
	await _screen("ctx_map", _ctx_map_setup)

	# -- 9. phone chrome ---------------------------------------------------
	if app.is_phone():
		await _screen("phone_drawer", func():
			await _close()
			app._set_drawer_open(true), func(): return app)
		await _screen("phone_panels", func():
			await _close()
			app._set_panel_picker_open(true), func(): return app)
		await _screen("phone_overflow_root", func():
			await _close()
			app._set_overflow_open(true), func(): return app)
		## Drill one program menu deep, then one submenu deep -- L3 and L4.
		var pm = app.get("_phone_menu")
		if pm != null:
			await _screen("phone_overflow_file", func():
				var mb2 = null
				for c in app.get("menu_bar_row").get_children():
					if c is MenuButton and c.text == "File":
						mb2 = c
				if mb2 != null:
					pm._push(mb2.get_popup(), "File", 3)
					pm._render(), func(): return pm)
			await _screen("phone_overflow_prefs", func():
				var mb3 = null
				for c in app.get("menu_bar_row").get_children():
					if c is MenuButton and c.text == "Preferences":
						mb3 = c
				if mb3 != null:
					pm._push(mb3.get_popup(), "Preferences", 3)
					pm._render(), func(): return pm)
		await _screen("phone_tool_sheet", func():
			await _close()
			app.arm_tool("sculpt"), func(): return app.get("tool_options_row"))

	await _close()


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var w := 1600
	var h := 900
	var i := args.find("--vp")
	if i >= 0 and i + 1 < args.size():
		var p: PackedStringArray = String(args[i + 1]).split("x")
		w = int(p[0])
		h = int(p[1])
	var j := args.find("--tag")
	if j >= 0 and j + 1 < args.size():
		_tag = String(args[j + 1])
	_dir = "user://menuconf"
	DirAccess.make_dir_recursive_absolute(_dir)

	Input.set_emulate_touch_from_mouse(true)
	_vp = SubViewport.new()
	_vp.size = Vector2i(w, h)
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)

	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.0).timeout

	var bridge = app.bridge
	bridge.generate({
		"seed": SEED, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	await _sweep()

	var f := FileAccess.open(_dir + "/log_%s.txt" % _tag, FileAccess.WRITE)
	f.store_string("\n".join(_log))
	f.close()
	_l("### DONE %s -> %s ###" % [_tag, ProjectSettings.globalize_path(_dir)])
	get_tree().quit()
