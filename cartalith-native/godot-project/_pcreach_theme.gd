extends Node
## **Lane PC-REACH, light-theme leg.** `_menuconf_probe` at 1920x1080 on
## 2026-09-07 dumped the seven program menus with `panel=#121314 ink=#c8cbcd`
## (DARK) in the same run whose map right-click menu came back `panel=#fbfaf7
## ink=#23241f` (LIGHT). This measures which is right and when the divergence
## appears: the shell's own stored theme, then every MenuBar popup's panel
## stylebox and font colour beside `DccTheme.c()`.

var app: Node
var _vp: SubViewport

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _hex(c: Color) -> String:
	return "#%02x%02x%02x/%.2f" % [int(round(c.r * 255)), int(round(c.g * 255)),
		int(round(c.b * 255)), c.a]

func _dump(tag: String) -> void:
	print("[%s] DccTheme  bg=%s panel=%s text=%s accent=%s" % [tag,
		_hex(DccTheme.c("bg")), _hex(DccTheme.c("panel")),
		_hex(DccTheme.c("text")), _hex(DccTheme.c("accent"))])
	var mbr = app.get("menu_bar_row")
	for ch in mbr.get_children():
		if ch is MenuButton:
			var p := (ch as MenuButton).get_popup()
			var sb: StyleBox = p.get_theme_stylebox("panel")
			var bg := "(not StyleBoxFlat)"
			if sb is StyleBoxFlat:
				bg = _hex((sb as StyleBoxFlat).bg_color)
			print("   menu %-12s panel=%s  font=%s" % [ch.text, bg,
				_hex(p.get_theme_color("font_color"))])

func _ready() -> void:
	_vp = SubViewport.new()
	_vp.size = Vector2i(1920, 1080)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(40)
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(10)
	_dump("boot")
	## Open one, in case the styling is deferred to `about_to_popup`.
	for ch in app.get("menu_bar_row").get_children():
		if ch is MenuButton and ch.text == "File":
			var p := (ch as MenuButton).get_popup()
			if p.about_to_popup.get_connections().size() > 0:
				p.about_to_popup.emit()
			(ch as MenuButton).show_popup()
			await _frames(8)
			p.hide()
	await _frames(4)
	_dump("after opening File")
	## And after an explicit theme flip, both ways.
	for mode in ["dark", "light"]:
		DccTheme.apply_theme(mode == "dark")
		await _frames(20)
		_dump("after set " + mode)
	## Is the popup an internal child? `_recolor_subtree()` (dcc_shell.gd:1764)
	## iterates `node.get_children()` with `include_internal` left at false.
	for ch in app.get("menu_bar_row").get_children():
		if ch is MenuButton:
			print("   %s children=%d children(true)=%d popup_is_child=%s" % [
				ch.text, ch.get_children().size(), ch.get_children(true).size(),
				(ch.get_popup() in ch.get_children(true))])
			break
	var ob: OptionButton = null
	var stack: Array = [app]
	while not stack.is_empty() and ob == null:
		var n: Node = stack.pop_back()
		if n is OptionButton:
			ob = n
		for c in n.get_children():
			stack.append(c)
	if ob != null:
		var sb2: StyleBox = ob.get_popup().get_theme_stylebox("panel")
		print("   OptionButton popup panel=%s (children=%d children(true)=%d)" % [
			(_hex((sb2 as StyleBoxFlat).bg_color) if sb2 is StyleBoxFlat else "?"),
			ob.get_children().size(), ob.get_children(true).size()])
	print("### DONE ###")
	get_tree().quit()
