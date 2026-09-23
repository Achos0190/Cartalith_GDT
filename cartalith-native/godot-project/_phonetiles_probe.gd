extends Node
## Verifier for §6.6 `data-tiles` on the phone MORE stack (2026-09-23):
## `PhoneMenu._fill_data_tiles()` and the `leaflet-preview.html` page
## `cartalith_io::slippy::leaflet_preview_html` writes into the pyramid archive.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _phonetiles_probe.tscn \
##       -- --force-touch --nowelcome
##
## Every step goes through the rows a finger would touch -- a row is a
## `PanelContainer` driven by `gui_input`, a chip is a `Button` -- never by
## calling `_push_screen()` / `_dt_export()` directly:
##
##   1. no world: the `data` screen carries `Maps · tile pyramid`, and the
##      screen draws its Export row DISABLED with a reason (the control state);
##   2. with a world: MORE ▸ Data manager ▸ Maps · tile pyramid lands on
##      `data-tiles`; chips set XYZ / 256 / 0–2, retina stays on (§6.6's
##      default), and the Export row writes a real zip to the exports root;
##   3. the zip holds 42 tiles + tiles.json + leaflet-preview.html, and the
##      page's own tile-URL template -- read out of the HTML, not restated --
##      resolves every (z, x, y, @2x) address of every level to an entry that
##      IS in the archive; the page's tileSize is the decoded 0/0/0.png's size;
##   4. the same for TMS, whose rows are flipped: both previews show the same
##      tile bytes at one Leaflet coordinate, with a control that the XYZ
##      template pointed at the TMS archive shows a different one;
##   5. back returns to the `data` screen.

var app: Node
var pm
var _fail := 0
var _written: Array = []

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("PT %s  %s%s" % ["ok  " if cond else "FAIL", name,
		"" if cond else (("  -- " + detail) if detail != "" else "")])
	if not cond:
		_fail += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## The tappable row (a `PanelContainer` with a `gui_input` handler) whose first
## label reads `title`, or the non-tappable one when `tappable` is false.
func _row(title: String, tappable: bool = true) -> PanelContainer:
	return _find_row(pm._screen_body, title, tappable)

func _find_row(n: Node, title: String, tappable: bool) -> PanelContainer:
	for c in n.get_children():
		if c is PanelContainer:
			var stop: bool = (c as Control).mouse_filter == Control.MOUSE_FILTER_STOP
			if stop == tappable and _has_label(c, title):
				return c
		var r := _find_row(c, title, tappable)
		if r != null:
			return r
	return null

## A row's labels are [glyph], title, subtitle, trail -- so match any of them
## exactly rather than the first, which is the glyph on a root row.
func _has_label(n: Node, text: String) -> bool:
	for c in n.get_children():
		if c is Label and String((c as Label).text) == text:
			return true
		if _has_label(c, text):
			return true
	return false

func _tap(row: Control) -> void:
	for pressed in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = pressed
		ev.position = Vector2(10, 10)
		row.gui_input.emit(ev)
	await _frames(3)

func _chip(text: String) -> Button:
	return _find_button(pm._screen_body, text)

func _find_button(n: Node, text: String) -> Button:
	for c in n.get_children():
		if c is Button and String((c as Button).text) == text:
			return c
		var r := _find_button(c, text)
		if r != null:
			return r
	return null

func _top() -> String:
	return String(pm._stack[pm._stack.size() - 1].screen)

func _ready() -> void:
	get_tree().create_timer(600.0).timeout.connect(func():
		print("PT WATCHDOG TIMEOUT")
		get_tree().quit(3))
	DisplayServer.window_set_size(Vector2i(1080, 2400))
	get_window().size = Vector2i(1080, 2400)
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)
	pm = app.get("_phone_menu")
	if pm == null:
		print("PT  !! no PhoneMenu -- run with -- --force-touch")
		get_tree().quit(1)
		return

	## -- 1. No world: the row exists, the export is refused with a reason ------
	pm.open()
	await _tap(_row("Data manager"))
	_check("MORE ▸ Data manager lands on 'data'", _top() == "data", _top())
	var nav := _row("Maps · tile pyramid")
	_check("data screen draws 'Maps · tile pyramid'", nav != null)
	if nav == null:
		_finish()
		return
	await _tap(nav)
	_check("the row lands on 'data-tiles'", _top() == "data-tiles", _top())
	## §6.6 defaults: 0–5, retina on -> (1+4+16+64+256+1024) x 2.
	_check("no world: Export row is drawn DISABLED",
		_row("Export 2730 tiles", false) != null and _row("Export 2730 tiles", true) == null)

	## -- 2. With a world ------------------------------------------------------
	pm.close()
	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 3000:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	_check("a world generated", bool(app.bridge.has_world))
	if not app.bridge.has_world:
		_finish()
		return

	pm.open()
	await _tap(_row("Data manager"))
	await _tap(_row("Maps · tile pyramid"))
	_check("reached 'data-tiles' again by taps", _top() == "data-tiles", _top())
	for t in ["XYZ", "256", "0 – 2"]:
		var b := _chip(String(t))
		_check("chip '%s' exists" % t, b != null)
		if b != null:
			b.pressed.emit()
			await _frames(3)
	_check("state after chips: xyz / 256 / z2 / retina",
		pm._dt_scheme == "xyz" and pm._dt_tile == 256 and pm._dt_zmax == 2 and pm._dt_retina,
		"%s %d %d %s" % [pm._dt_scheme, pm._dt_tile, pm._dt_zmax, pm._dt_retina])
	_check("estimate: size/time dashed before any export",
		_row("Size · render time", false) != null)

	var names_xyz := await _export_and_read("xyz", 42)
	_check("estimate: Size row drawn from the measured run", _row("Size", false) != null)
	## Its fullest state (measured rows, a world): the body must fit the screen.
	var mw: float = pm._screen_body.get_combined_minimum_size().x
	_check("data-tiles fits 1080 px with every row drawn", mw <= 1080.0, "min_w=%.0f" % mw)
	print("PT   body min width %.0f px" % mw)

	## -- 4. TMS -----------------------------------------------------------------
	var tms := _chip("TMS")
	if tms != null:
		tms.pressed.emit()
		await _frames(3)
	var names_tms := await _export_and_read("tms", 42)
	if not names_xyz.is_empty() and not names_tms.is_empty():
		## The same Leaflet coordinate must show the same ground in both
		## previews -- and, as the control, the XYZ page pointed at the TMS
		## archive must NOT (a flipped level holds every name, so a names-only
		## check cannot tell the two templates apart; bytes can).
		var xt := String(names_xyz["__tpl"])
		var tt := String(names_tms["__tpl"])
		var xb := _read(String(names_xyz["__path"]), _fill(xt, 2, 1, 0, ""))
		var tb := _read(String(names_tms["__path"]), _fill(tt, 2, 1, 0, ""))
		var wrong := _read(String(names_tms["__path"]), _fill(xt, 2, 1, 0, ""))
		_check("xyz and tms previews show the same tile at Leaflet (2, 1, 0)",
			not xb.is_empty() and xb == tb, "%d vs %d bytes" % [xb.size(), tb.size()])
		_check("control: the XYZ template on the TMS archive shows a different tile",
			not wrong.is_empty() and wrong != xb)

	## -- 5. Back ----------------------------------------------------------------
	pm.go_back()
	await _frames(2)
	_check("back from data-tiles returns to 'data'", _top() == "data", _top())
	_finish()

## Tap Export, open what it wrote, and hold leaflet-preview.html to the archive.
## Returns the archive's names (plus `__tpl`, the page's URL template), or {}.
func _export_and_read(scheme: String, tiles: int) -> Dictionary:
	var go := _row("Export %d tiles" % tiles)
	_check("%s: Export %d tiles row is tappable" % [scheme, tiles], go != null)
	if go == null:
		return {}
	var path := DccSettings.storage_root("exports").path_join(pm._dt_file_name())
	if FileAccess.file_exists(path):
		DirAccess.remove_absolute(path)
	var t0 := Time.get_ticks_msec()
	await _tap(go)
	print("PT   %s export took %d ms -> %s" % [scheme, Time.get_ticks_msec() - t0, path])
	_written.append(path)
	var z := ZIPReader.new()
	_check("%s: the zip was written and opens" % scheme, z.open(path) == OK, path)
	var names := {}
	for n in z.get_files():
		names[n] = true
	var html := z.read_file("leaflet-preview.html").get_string_from_utf8()
	var root_png := z.read_file("0/0/0.png")
	z.close()
	_check("%s: %d tiles + tiles.json + leaflet-preview.html" % [scheme, tiles],
		names.size() == tiles + 2, "%d entries" % names.size())
	_check("%s: leaflet-preview.html is in the archive" % scheme, names.has("leaflet-preview.html"))
	_check("%s: the page is Leaflet on L.CRS.Simple" % scheme,
		html.contains("leaflet.js") and html.contains("L.CRS.Simple"))
	_check("%s: the page's MAXZ is 2" % scheme, html.contains("MAXZ = 2, RETINA = true;"))
	var img := Image.new()
	var dims := Vector2i(-1, -1)
	if img.load_png_from_buffer(root_png) == OK:
		dims = Vector2i(img.get_width(), img.get_height())
	_check("%s: the page's tile size is 0/0/0.png's %s" % [scheme, dims],
		html.contains("const TW = %d, TH = %d," % [dims.x, dims.y]),
		html.substr(html.find("const TW"), 60))
	var a := html.find("return `")
	var b := html.find("`;", a)
	var tpl := html.substr(a + 8, b - a - 8) if a >= 0 and b > a else ""
	print("PT   %s template: %s" % [scheme, tpl])
	_check("%s: the page carries a tile-URL template" % scheme, tpl != "")
	var miss := _unresolved(tpl, names, 2, true)
	_check("%s: every address the page asks for, 1x and @2x, is in the archive" % scheme,
		miss == 0, "%d unresolved" % miss)
	names["__tpl"] = tpl
	names["__path"] = path
	return names

func _read(zip_path: String, entry: String) -> PackedByteArray:
	var z := ZIPReader.new()
	if z.open(zip_path) != OK:
		return PackedByteArray()
	var b := z.read_file(entry)
	z.close()
	return b

## The page's template filled the way its `getTileUrl` fills it.
func _fill(tpl: String, zz: int, x: int, y: int, r: String) -> String:
	var n := 1 << zz
	return tpl.replace("${ty}", str(n - 1 - y)).replace("${z}", str(zz)) \
		.replace("${x}", str(x)).replace("${y}", str(y)).replace("${r}", r)

## How many (z, x, y, r) the template resolves to a name NOT in `names`.
func _unresolved(tpl: String, names: Dictionary, maxz: int, retina: bool) -> int:
	var miss := 0
	for r in (["", "@2x"] if retina else [""]):
		for zz in maxz + 1:
			var n := 1 << zz
			for x in n:
				for y in n:
					if not names.has(_fill(tpl, zz, x, y, r)):
						miss += 1
	return miss

func _finish() -> void:
	for p in _written:
		if FileAccess.file_exists(String(p)):
			DirAccess.remove_absolute(String(p))
	pm.close()
	print("PT done: %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
