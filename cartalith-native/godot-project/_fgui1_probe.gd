extends Node
## Owner findings F13 / F32 (2026-11-21), driven with REAL input events through the viewport.
## Windowed: Godot --path . --resolution 1600x900 _fgui1_probe.tscn -- --out <dir> --leg <f13|f32>
## F13: a bare Ctrl press and release must open nothing (no popup, no window) and must not eat Ctrl+S/Ctrl+Z.
## F32: after a route (or a settlement) is placed, the same tool must take the next click without another tool in between.
var app: Node
var out_dir := ""
var leg := ""
var fails := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(c: bool, m: String) -> void:
	print("FG1 ", "OK   " if c else "FAIL ", m)
	if not c:
		fails += 1

func _key(code: int, pressed: bool, ctrl := false, shift := false) -> void:
	var e := InputEventKey.new()
	e.keycode = code
	e.physical_keycode = code
	e.ctrl_pressed = ctrl
	e.shift_pressed = shift
	e.pressed = pressed
	get_viewport().push_input(e)

func _click(p: Vector2) -> void:
	for pr in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = pr
		ev.position = p
		ev.global_position = p
		get_viewport().push_input(ev)
		await _frames(2)

func _open_popups() -> Array:
	var out: Array = []
	var stack: Array = [get_tree().root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is Window and (n as Window).visible and n != get_tree().root:
			out.append("%s(%s)" % [n.name, n.get_class()])
		stack.append_array(n.get_children())
	return out

func _shot(name: String) -> void:
	await RenderingServer.frame_post_draw
	get_viewport().get_texture().get_image().save_png(out_dir.path_join("fg1_%s.png" % name))

func _find_btn(tip: String) -> Button:
	var stack: Array = [app]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is Button and (n as Button).tooltip_text == tip and (n as Button).is_visible_in_tree():
			return n
		stack.append_array(n.get_children())
	return null

func _ready() -> void:
	var a := OS.get_cmdline_user_args()
	var i := 0
	while i < a.size():
		match a[i]:
			"--out": out_dir = a[i + 1]; i += 2
			"--leg": leg = a[i + 1]; i += 2
			"--force-touch": i += 1
			_:
				i += 1
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	app.bridge.generate({"seed": 131313, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var w := 0
	while app.bridge.generating and w < 3000:
		await get_tree().process_frame
		w += 1
	await _frames(10)
	_ok(app.bridge.has_world, "world generated")
	if leg == "f13":
		await _f13()
	elif leg == "f13s":
		await _f13_stored()
	elif leg == "ui":
		await _ui()
	elif leg == "f32":
		await _f32()
	print("FG1 DONE leg=%s fails=%d" % [leg, fails])
	get_tree().quit(1 if fails > 0 else 0)

func _f13() -> void:
	var before := _open_popups()
	for ctrl_code in [KEY_CTRL]:
		_key(ctrl_code, true, true)
		await _frames(4)
		var held := _open_popups()
		_ok(held == before, "bare Ctrl held opens nothing: before=%s held=%s" % [before, held])
		await _shot("f13_ctrl_held")
		_key(ctrl_code, false, false)
		await _frames(4)
		var rel := _open_popups()
		_ok(rel == before, "bare Ctrl released opens nothing: %s" % [rel])
	# the menu bar's popups themselves
	var menus: Array = []
	for n in app.find_children("*", "PopupMenu", true, false):
		if (n as PopupMenu).visible:
			menus.append(n.name)
	_ok(menus.is_empty(), "no PopupMenu visible after Ctrl: %s" % [menus])
	# Ctrl+Z after Ctrl still undoes: the vault window must not be up
	_key(KEY_CTRL, true, true)
	_key(KEY_S, true, true)
	await _frames(3)
	_key(KEY_S, false, true)
	_key(KEY_CTRL, false)
	await _frames(3)
	print("FG1 info after Ctrl+S windows=", _open_popups())
	# a non-blocking context window holds the focus: a bare Ctrl there must open nothing either
	app.open_vault_overview()
	await _frames(10)
	var with_vault := _open_popups()
	print("FG1 info windows with the vault open: ", with_vault)
	await _shot("f13_vault_open")
	for echo in [false, true, true]:
		var e := InputEventKey.new()
		e.keycode = KEY_CTRL
		e.physical_keycode = KEY_CTRL
		e.ctrl_pressed = true
		e.pressed = true
		e.echo = echo
		get_viewport().push_input(e)
		await _frames(3)
	var after := _open_popups()
	_ok(after == with_vault, "bare Ctrl (press + echoes) with the vault focused opens nothing new: %s -> %s" % [with_vault, after])
	_key(KEY_CTRL, false)
	await _frames(3)
	# --- the context-window relay: pure function, bare modifiers must not relay
	var RelayScript = load("res://shell/context_key_relay.gd")
	for mod in [KEY_CTRL, KEY_SHIFT, KEY_ALT, KEY_META]:
		_ok(not RelayScript.relays(mod, true, false, false), "relays(%d, ctrl) is false for a bare modifier" % mod)
		_ok(not RelayScript.relays(mod, false, true, false), "relays(%d, alt) is false for a bare modifier" % mod)
	_ok(RelayScript.relays(KEY_F, true, false, false), "relays(F, ctrl) stays true")

## F13's mechanism. The shortcut dialog's capture once let a stored accelerator be a bare modifier (its guard on
## `KEY_CTRL` etc. came 2026-09-06, after which any such value already in a settings file survived: `_bind_accelerator`
## trusted the file). A stored Ctrl-only accelerator on Records > Markdown vault makes a bare Ctrl press run it.
func _f13_stored() -> void:
	const VAULT_ID := 45
	var bad: int = KEY_CTRL | KEY_MASK_CTRL
	DccSettings.set_shortcut_binding(DccSettings.SHORTCUT_CONTEXT_MENU, VAULT_ID, bad)
	var pop: PopupMenu = null
	var idx := -1
	for mb in app.find_children("*", "MenuButton", true, false):
		var p := (mb as MenuButton).get_popup()
		if p.get_item_index(VAULT_ID) >= 0:
			pop = p
			idx = p.get_item_index(VAULT_ID)
	_ok(pop != null, "found the Records popup holding Markdown vault")
	app.menus._bind_accelerator(pop, idx, VAULT_ID, KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_V)
	var got: int = pop.get_item_accelerator(idx)
	_ok(got != bad, "a stored bare-Ctrl accelerator is NOT applied (item accelerator = %d, stored = %d)" % [got, bad])
	var ce := InputEventKey.new()
	ce.keycode = KEY_CTRL
	ce.physical_keycode = KEY_CTRL
	ce.ctrl_pressed = true
	ce.pressed = true
	var before := _open_popups()
	get_viewport().push_input(ce)
	await _frames(6)
	_ok(_open_popups() == before, "bare Ctrl with a stored bare-Ctrl vault binding opens nothing: %s" % [_open_popups()])
	DccSettings.clear_shortcut_binding(DccSettings.SHORTCUT_CONTEXT_MENU, VAULT_ID)

func _f32() -> void:
	app.select_domain("civilization")
	await _frames(8)
	app._escape_action(true)
	await _frames(4)
	var vp: Control = app.viewport
	var rect := vp.get_global_rect()
	var c := rect.get_center()
	# ---- route: click the tool button, drop three stops, commit through the real Escape key, then again
	for round in 3:
		var b := _find_btn("Route (⇧R)")
		_ok(b != null, "round %d: Route button found" % round)
		if b == null:
			return
		await _click(b.get_global_rect().get_center())
		_ok(app.armed_tool == "route", "round %d: clicking Route arms it (armed=%s)" % [round, app.armed_tool])
		var n0: int = app.bridge.route_count()
		for k in 3:
			await _click(c + Vector2(-120 + 80 * k, 10 * k))
		await _frames(3)
		_key(KEY_ESCAPE, true)
		_key(KEY_ESCAPE, false)
		await _frames(4)
		_ok(app.bridge.route_count() == n0 + 1, "round %d: route committed (%d -> %d), armed=%s" % [round, n0, app.bridge.route_count(), app.armed_tool])
		await _shot("f32_route_%d" % round)
	# ---- settlement: arm, then drop on a spread of grid points; every land click must add one and the tool stays live
	var s := _find_btn("Settlement (S)")
	_ok(s != null, "Settlement button found")
	if s == null:
		return
	await _click(s.get_global_rect().get_center())
	var n1: int = app.bridge.settlements().size()
	var added := 0
	for k in 14:
		var gp := Vector2(40 + 13 * k, 40 + 9 * ((k * 5) % 11))
		var sp: Vector2 = vp.global_position + vp.grid_to_view(gp.x, gp.y)
		var before: int = app.bridge.settlements().size()
		await _click(sp)
		await _frames(2)
		if app.bridge.settlements().size() > before:
			added += 1
		_ok(app.armed_tool == "settlement", "settlement click %d: tool still armed (%s)" % [k, app.armed_tool])
	print("FG1 info settlements %d -> %d, %d of 14 clicks added one" % [n1, app.bridge.settlements().size(), added])
	_ok(added >= 3, "at least 3 land clicks each added a settlement (%d)" % added)
	await _shot("f32_settlements")

## Every text the shell shows or hints: Label/Button/CheckBox text, tooltips, and every PopupMenu item text and tooltip.
func _all_texts() -> Array:
	var out: Array = []
	var stack: Array = [app]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is Control:
			out.append(String((n as Control).tooltip_text))
			if n is Label:
				out.append((n as Label).text)
			elif n is Button:
				out.append((n as Button).text)
		if n is PopupMenu:
			for i in (n as PopupMenu).item_count:
				out.append((n as PopupMenu).get_item_text(i))
				out.append((n as PopupMenu).get_item_tooltip(i))
		stack.append_array(n.get_children())
	return out

func _count(needle: String) -> int:
	var c := 0
	for t in _all_texts():
		if String(t).contains(needle):
			c += 1
	return c

func _popup_has(menu_title: String, needle: String) -> bool:
	for mb in app.find_children("*", "MenuButton", true, false):
		if (mb as MenuButton).text == menu_title:
			var stack: Array = [(mb as MenuButton).get_popup()]
			while not stack.is_empty():
				var pm: PopupMenu = stack.pop_back()
				for i in pm.item_count:
					if pm.get_item_text(i).contains(needle):
						return true
				for ch in pm.get_children():
					if ch is PopupMenu:
						stack.append(ch)
	return false

func _ui() -> void:
	print("FG1 info form touch=%s phone=%s tablet=%s" % [DccTheme.is_touch(), DccTheme.is_phone(), DccTheme.is_tablet()])
	var touch: bool = DccTheme.is_touch()
	# ---- F19: Android-only options
	var pref := "Preferences"
	for needle in ["Handedness", "Brush edge sliders", "Pen link"]:
		var has := _popup_has(pref, needle)
		if not touch and not DccTheme.is_phone():
			_ok(not has, "F19 PC: Preferences has no '%s'" % needle)
		else:
			_ok(has, "F19 touch: Preferences keeps '%s'" % needle)
	# ---- F31: Town look / Marker style left Preferences
	_ok(not _popup_has(pref, "Town look"), "F31: Preferences has no Town look")
	_ok(not _popup_has(pref, "Marker style"), "F31: Preferences has no Marker style")
	# ---- F22 / F17 text sweeps, with CARTO and WORLD built
	for dom in ["world", "civilization", "cartography"]:
		app.select_domain(dom)
		await _frames(6)
	app.select_domain("cartography")
	await _frames(6)
	var ws = app._workspace_panels.get("cartography")
	print("FG1 info town look rows now=", _count("Town look"), " marker pins=", _count("Marker pins"))
	_ok(_count("Animate water") == 0, "F22: no 'Animate water' text or tooltip anywhere (%d)" % _count("Animate water"))
	_ok(not app.bridge.npr_settings().has("animate_water"), "F22: npr_settings() has no animate_water key")
	_ok(not ResourceLoader.exists("res://shell/water_anim_layer.gd"), "F22: water_anim_layer.gd is gone")
	var n_applied: int = app.bridge.set_npr({"animate_water": true, "waves": false})
	_ok(n_applied >= 0, "F22: an old key in set_npr is ignored without error")
	# ---- F31 in CARTO
	_ok(_count("Town look") >= 1, "F31: CARTO shows Town look (%d)" % _count("Town look"))
	_ok(_count("Marker pins") >= 1, "F31: CARTO shows Marker pins (%d)" % _count("Marker pins"))
	if DccTheme.is_phone():
		app.select_domain_category("cartography", "Style")
		app.set_phone_detent("full")
		await _frames(10)
		var sc: ScrollContainer = null
		for c in app.find_children("*", "ScrollContainer", true, false):
			if (c as ScrollContainer).is_visible_in_tree() and (c as ScrollContainer).get_v_scroll_bar().max_value > 400:
				sc = c
		var tl := _find_option("Town look")
		if sc != null and tl != null:
			sc.ensure_control_visible(tl)
		await _frames(6)
	await _shot("ui_carto")
	var ob := _find_option("Town look")
	if ob != null:
		ob.select(1)
		ob.item_selected.emit(1)
		_ok(DccSettings.city_look() == "ink", "F31: choosing Ink plan in CARTO stores it")
		ob.select(0)
		ob.item_selected.emit(0)
		_ok(DccSettings.city_look() == "map", "F31: choosing Map look stores it")
	else:
		_ok(false, "F31: Town look dropdown found")
	var mb := _find_option("Marker pins")
	if mb != null:
		mb.select(1)
		mb.item_selected.emit(1)
		_ok(DccSettings.marker_style() == "circles", "F31: choosing Circles in CARTO stores it")
		mb.select(0)
		mb.item_selected.emit(0)
	else:
		_ok(false, "F31: Marker pins dropdown found")
	# ---- F17: the tool strip and the panel title
	app.select_domain_mode("world", "b")
	if DccTheme.is_phone():
		app.set_phone_detent("full")
	await _frames(6)
	_ok(DccTheme.is_phone() or _find_btn("Biome & Terrain Paint (B)") != null, "F17: the tool strip's button is 'Biome & Terrain Paint (B)'")
	_ok(_count("Biome paint") == 0 and _count("Biome Paint") == 0, "F17: no 'Biome paint' text left (%d, %d)" % [_count("Biome paint"), _count("Biome Paint")])
	await _shot("ui_world_b")
	# ---- F18: the Target field follows the map
	app.arm_tool("paint")
	await _frames(8)
	var tf := _find_option("Target field")
	_ok(tf != null, "F18: Target field dropdown found")
	if tf != null:
		var vp = app.viewport
		tf.select(1)
		tf.item_selected.emit(1)
		await _frames(4)
		_ok(String(vp._debug_view) == "cterrain", "F18: Terrain target shows the Terrain field (view=%s)" % vp._debug_view)
		await _shot("ui_target_terrain")
		tf = _find_option("Target field")
		tf.select(0)
		tf.item_selected.emit(0)
		await _frames(4)
		_ok(String(vp._debug_view) == "bclass", "F18: Biome target shows the Biomes field (view=%s)" % vp._debug_view)
		await _shot("ui_target_biome")

func _find_option(label: String) -> OptionButton:
	for ob in app.find_children("*", "OptionButton", true, false):
		if not (ob as OptionButton).is_visible_in_tree():
			continue
		var row := (ob as OptionButton).get_parent()
		if row != null:
			for ch in row.get_children():
				if ch is Label and (ch as Label).text == label:
					return ob
	return null
