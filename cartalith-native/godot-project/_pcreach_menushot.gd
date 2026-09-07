extends Node
## Pixel proof for the PC-REACH light-theme leg: the shell in LIGHT with a
## program menu open. `gui_embed_subwindows` puts the `PopupMenu` (a `Window`)
## inside the captured texture, so the same frame yields both the screenshot
## and a sampled ground colour.
##
## The ground is taken as the **modal colour of a grid sampled over the inner
## 60 % of the popup's rect**, not a single point: an embedded `Window`'s
## `position`/`size` include its drop-shadow margin, so a corner sample lands
## on the app behind the menu and reads `bg` no matter what the menu paints.
## That mis-sample is what this version exists to not repeat.
var app: Node
var _vp: SubViewport

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _hex(c: Color) -> String:
	return "#%02x%02x%02x" % [int(round(c.r * 255)), int(round(c.g * 255)),
		int(round(c.b * 255))]

func _ground(img: Image, r: Rect2i) -> String:
	var tally: Dictionary = {}
	var x0 := r.position.x + int(r.size.x * 0.2)
	var x1 := r.position.x + int(r.size.x * 0.8)
	var y0 := r.position.y + int(r.size.y * 0.2)
	var y1 := r.position.y + int(r.size.y * 0.8)
	var x := x0
	while x < x1:
		var y := y0
		while y < y1:
			if x >= 0 and y >= 0 and x < img.get_width() and y < img.get_height():
				var k := _hex(img.get_pixel(x, y))
				tally[k] = int(tally.get(k, 0)) + 1
			y += 3
		x += 3
	var best := ""
	var bestn := 0
	for k in tally:
		if int(tally[k]) > bestn:
			bestn = int(tally[k])
			best = String(k)
	var total := 0
	for k in tally:
		total += int(tally[k])
	return "%s (%d of %d samples, %d distinct)" % [best, bestn, total, tally.size()]

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var dir := "user://pcreach"
	var j := args.find("--out")
	if j >= 0 and j + 1 < args.size():
		dir = String(args[j + 1])
	DirAccess.make_dir_recursive_absolute(dir)
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
	print("[MODE] bg=%s panel=%s sunken=%s text=%s" % [_hex(DccTheme.c("bg")),
		_hex(DccTheme.c("panel")), _hex(DccTheme.c("sunken")), _hex(DccTheme.c("text"))])
	## Baseline: the shell with no menu open, sampled over the same band the
	## popup will occupy, so "the ground reads bg" cannot be an artefact of
	## sampling the app instead of the menu.
	var base := _vp.get_texture().get_image()
	print("  (no menu)   band ground=%s" % _ground(base, Rect2i(243, 15, 436, 886)))
	for want in ["Data", "Preferences"]:
		for ch in app.get("menu_bar_row").get_children():
			if ch is MenuButton and ch.text == want:
				var p := (ch as MenuButton).get_popup()
				if p.about_to_popup.get_connections().size() > 0:
					p.about_to_popup.emit()
				(ch as MenuButton).show_popup()
				await _frames(10)
				var sb: StyleBox = p.get_theme_stylebox("panel")
				var declared := "(not flat)"
				if sb is StyleBoxFlat:
					declared = _hex((sb as StyleBoxFlat).bg_color)
				var img := _vp.get_texture().get_image()
				if img != null:
					var r := Rect2i(p.position, p.size)
					print("  %-12s declared=%s  rect=%s  ground=%s" % [want,
						declared, r, _ground(img, r)])
					img.save_png("%s/light_menu_%s.png" % [dir, want.to_lower()])
				p.hide()
				await _frames(4)
	print("### DONE ###")
	get_tree().quit()
