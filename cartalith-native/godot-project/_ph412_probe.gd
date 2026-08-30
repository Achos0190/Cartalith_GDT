extends Node
## The 412 dp phone migration pass. Measures every phone chrome region in
## physical pixels and in canvas dp, at both target sizes, and screenshots.
##
##   godot --path . _ph412_probe.tscn -- --force-touch --vp 1440x3168 --tag ph1440
##   godot --path . _ph412_probe.tscn -- --force-touch --vp 1080x2400 --tag ph1080
##
## Hosted in a `SubViewport`: `--resolution WxH` is clamped to the dev monitor's
## work area and boots the shell into *tablet* mode. `gui_embed_subwindows`
## keeps dialogs and `PopupMenu`s in the captured texture.
##
## Untracked, like every other probe in this folder.

var app: Node
var _vp: SubViewport
var _tag := "ph"
var _lines: Array[String] = []

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	_lines.append(s)
	print(s)

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

## One region, reported three ways: physical px, dp against the *live*
## `PHONE_REF_SHORT`, and the canvas figure it is supposed to be.
func _region(label: String, node: Node, want_dp: float) -> void:
	if node == null:
		_log("  %-24s ABSENT" % label)
		return
	var c := node as Control
	var s: float = app.phone_scale()
	var px: float = c.size.y
	_log("  %-24s px=%7.1f  dp=%6.2f  want=%5.1f  vis=%s" % [
		label, px, px / s, want_dp, str(c.visible)])

func _find_all(node: Node, pred: Callable, out: Array) -> Array:
	if pred.call(node):
		out.append(node)
	for ch in node.get_children():
		_find_all(ch, pred, out)
	return out

func _shot(name: String) -> void:
	await _frames(4)
	var out := "user://ph412/%s__%s.png" % [_tag, name]
	_vp.get_texture().get_image().save_png(out)
	_log("  shot %s" % ProjectSettings.globalize_path(out))

func _ready() -> void:
	DirAccess.make_dir_recursive_absolute("user://ph412")
	var parts: PackedStringArray = _arg("--vp", "1440x3168").split("x")
	_tag = _arg("--tag", "ph")
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout

	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	var s: float = app.phone_scale()
	_log("=== %s  vp=%s  phone=%s  scale=%.4f  REF_SHORT=%.1f ===" % [
		_tag, str(_vp.size), str(app.is_phone()), s, DccTheme.PHONE_REF_SHORT])

	_log("[chrome]")
	_region("status row", app._phone_top_safe, 28.0)
	_region("app bar", app._phone_app_bar if "_phone_app_bar" in app else null, 56.0)
	_region("bottom nav", app._phone_menu_bar, 64.0)
	_region("gesture inset", app._phone_gesture_inset, 20.0)
	_region("tool sheet", app._phone_tool_sheet, 0.0)
	if "_phone_scrim" in app and app._phone_scrim != null:
		_region("top scrim", app._phone_scrim, 0.0)
	else:
		_log("  top scrim                DELETED (canvas draws a solid ground)")
	if "_phone_drawer" in app and app._phone_drawer != null:
		_log("  drawer                   PRESENT (canvas draws none)")
	else:
		_log("  drawer                   DELETED (canvas draws none)")

	## Bottom-nav cells: caption, glyph presence, cell size.
	_log("[bottom nav cells]")
	var bar: Control = app._phone_menu_bar
	if bar != null:
		var buttons: Array = []
		_find_all(bar, func(n): return n is Button, buttons)
		for b in buttons:
			var btn := b as Button
			var caps := PackedStringArray()
			var glyphs := 0
			var inner: Array = []
			_find_all(btn, func(n): return n is Label or n is TextureRect, inner)
			for n in inner:
				if n is Label:
					caps.append((n as Label).text)
				else:
					glyphs += 1
			_log("  cell %-9s size=%s  labels=[%s] glyphs=%d" % [
				btn.tooltip_text.substr(0, 9), str(btn.size), " ".join(caps), glyphs])

	## App bar contents.
	_log("[app bar]")
	var chrome_buttons: Array = []
	if "_phone_app_bar" in app and app._phone_app_bar != null:
		_find_all(app._phone_app_bar, func(n): return n is Button, chrome_buttons)
		for b in chrome_buttons:
			_log("  btn '%s' size=%s tip=%s" % [(b as Button).text, str((b as Button).size),
				(b as Button).tooltip_text])

	await _shot("viewport")

	## The phone menu root (canvas "07 More").
	app._set_overflow_open(true)
	await _frames(6)
	var pm = app._phone_menu
	_log("[phone menu root]")
	_log("  title='%s' trail='%s'" % [pm._screen_head_title.text, pm._screen_head_trail.text])
	var head_buttons: Array = []
	_find_all(pm._screen, func(n): return n is Button, head_buttons)
	var crosses := 0
	for b in head_buttons:
		if (b as Button).text == DccIcons.SYMBOLS["cross"]:
			crosses += 1
	_log("  head buttons=%d  crosses=%d" % [head_buttons.size(), crosses])
	var bands: Array = []
	_find_all(pm._screen_body, func(n): return n is Label, bands)
	var band_txt := PackedStringArray()
	for b in bands:
		var t: String = (b as Label).text
		if t.begins_with("STATUS") or t.begins_with("PROJECT") or t.begins_with("CONTENT") \
				or t.begins_with("SYSTEM") or t.begins_with("OTHER"):
			band_txt.append(t)
	_log("  bands=[%s]" % " | ".join(band_txt))
	await _shot("more")
	app._phone_menu.close()
	await _frames(3)

	## The left dock as a full-screen sheet (canvas "02 Domain").
	app._set_sheet_open("left", true)
	await _frames(8)
	_log("[left sheet]")
	var pills: Array = []
	_find_all(app.left_dock, func(n): return n is Button and n.has_meta(DccWidgets.ACTION_META), pills)
	var shown := 0
	for b in pills:
		var btn := b as Button
		if not btn.is_visible_in_tree() or shown >= 6:
			continue
		shown += 1
		var sb := btn.get_theme_stylebox("normal")
		var rad := (sb as StyleBoxFlat).corner_radius_top_left if sb is StyleBoxFlat else -1
		var fill := (sb as StyleBoxFlat).bg_color if sb is StyleBoxFlat else Color(0, 0, 0, 0)
		_log("  pill '%-22s' h=%6.1f dp=%5.1f radius=%d fill=%s" % [
			btn.text.substr(0, 22), btn.size.y, btn.size.y / s, rad, str(fill)])
	var sliders: Array = []
	_find_all(app.left_dock, func(n): return n is HSlider, sliders)
	shown = 0
	for sl in sliders:
		var hs := sl as HSlider
		if not hs.is_visible_in_tree() or shown >= 3:
			continue
		shown += 1
		var g := hs.get_theme_icon("grabber")
		_log("  slider h=%6.1f dp=%5.1f thumb=%s" % [hs.size.y, hs.size.y / s,
			str(g.get_size()) if g != null else "none"])
	await _shot("left_sheet")
	app._close_all_phone_overlays()
	await _frames(3)

	var f := FileAccess.open("user://ph412/%s_log.txt" % _tag, FileAccess.WRITE)
	f.store_string("\n".join(_lines))
	f.close()
	_log("=== log %s ===" % ProjectSettings.globalize_path("user://ph412/%s_log.txt" % _tag))
	get_tree().quit()
