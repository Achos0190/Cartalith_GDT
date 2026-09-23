extends Node
## Windowed measurement: every top menu's popup panel colour against the active
## palette's `panel` token (`OUTSTANDING_WORK.md` §2.10, "the Data menu draws as
## a dark popup on the light shell"). Also opens each popup and reads it again,
## since a popup can be restyled on open.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _menupop_probe.tscn

var _fails := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("MENUPOP %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _panel_color(p: PopupMenu) -> Color:
	var sb := p.get_theme_stylebox("panel")
	if sb is StyleBoxFlat:
		return (sb as StyleBoxFlat).bg_color
	return Color(-1, -1, -1)

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("MENUPOP REFUSED: headless")
		get_tree().quit(2)
		return
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(4)
	print("MENUPOP palette %s  panel token %s" % ["dark" if DccTheme.is_dark() else "light", DccTheme.c("panel")])
	var shell: Node = app.shell if "shell" in app else app
	var row: Node = null
	for n in [app, shell]:
		if n != null and "menu_bar_row" in n and n.menu_bar_row != null:
			row = n.menu_bar_row
	if row == null:
		print("MENUPOP ABORT: no menu_bar_row")
		get_tree().quit(2)
		return
	for c in row.get_children():
		if c is MenuButton:
			var mb := c as MenuButton
			var p := mb.get_popup()
			var built := _panel_color(p)
			mb.show_popup()
			await _frames(4)
			var opened := _panel_color(p)
			## The DRAWN pixel, against the light canvas's own `--pan` literal
			## (#fbfaf7), not against the token the code read -- a probe that
			## compares a token with itself cannot fail (`MISTAKES.md`).
			## A popup here is its own OS window, so its pixels are in ITS viewport,
			## not the main one. Sampled in the top padding strip: bare panel, no row.
			var img := p.get_texture().get_image()
			## The window carries a transparent shadow margin: walk down the centre
			## column to the first opaque pixel (the 1 px border), then 2 px further
			## into the top padding, which is bare panel.
			var cx := p.size.x / 2
			var top := 0
			while top < img.get_height() - 3 and img.get_pixel(cx, top).a < 0.99:
				top += 1
			var at := Vector2i(cx, top + 2)
			var px := img.get_pixelv(at) if Rect2i(Vector2i.ZERO, img.get_size()).has_point(at) else Color(-1, -1, -1)
			_check("%s popup DRAWS the light canvas panel #fbfaf7 (pixel %s at %s)" % [mb.text, px, at],
				px.is_equal_approx(Color("#fbfaf7")) or (absf(px.r - 0.984) < 0.02 and absf(px.g - 0.980) < 0.02 and absf(px.b - 0.969) < 0.02))
			p.hide()
			await _frames(2)
			_check("%s popup panel = palette panel (built %s, open %s)" % [mb.text, built, opened],
				opened.is_equal_approx(DccTheme.c("panel")))
	print("MENUPOP %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
